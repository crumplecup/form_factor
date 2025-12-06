//! Concrete implementation of FormTemplate trait

use super::error::{TemplateError, TemplateErrorKind};
use crate::Shape;
use form_factor_core::{FieldDefinition, FormInstance, FormTemplate, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Concrete implementation of FormTemplate
///
/// Represents a form template with multiple pages, field definitions,
/// and optional reference shapes for visual guidance.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters)]
pub struct DrawingTemplate {
    /// Unique template identifier
    id: String,

    /// Human-readable template name
    name: String,

    /// Version string (semantic versioning recommended)
    version: String,

    /// Optional description
    description: Option<String>,

    /// Pages in this template
    pages: Vec<TemplatePage>,

    /// Template metadata
    metadata: HashMap<String, String>,
}

impl DrawingTemplate {
    /// Create a new template builder
    pub fn builder() -> DrawingTemplateBuilder {
        DrawingTemplateBuilder::default()
    }

    /// Load a template from JSON
    pub fn from_json(json: &str) -> Result<Self, TemplateError> {
        serde_json::from_str(json).map_err(|e| {
            TemplateError::new(
                TemplateErrorKind::Deserialization(e.to_string()),
                line!(),
                file!(),
            )
        })
    }

    /// Returns the number of pages in this template.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns the total number of fields across all pages.
    #[must_use]
    pub fn field_count(&self) -> usize {
        self.pages.iter().map(|page| page.fields.len()).sum()
    }

    /// Returns all field definitions across all pages.
    #[must_use]
    pub fn fields(&self) -> Vec<&FieldDefinition> {
        self.pages
            .iter()
            .flat_map(|page| page.fields.iter())
            .collect()
    }

    /// Validate the template structure
    ///
    /// Checks for:
    /// - At least one page
    /// - No duplicate field IDs
    /// - Valid page indices in field definitions
    pub fn validate(&self) -> Result<(), TemplateError> {
        if self.pages.is_empty() {
            return Err(TemplateError::new(
                TemplateErrorKind::InvalidTemplate("Template must have at least one page".into()),
                line!(),
                file!(),
            ));
        }

        // Check for duplicate field IDs
        let mut seen_ids = std::collections::HashSet::new();
        for field in self.fields() {
            if !seen_ids.insert(&field.id) {
                return Err(TemplateError::new(
                    TemplateErrorKind::DuplicateFieldId(field.id().clone()),
                    line!(),
                    file!(),
                ));
            }

            // Check that field's page_index is valid
            if field.page_index >= self.pages.len() {
                return Err(TemplateError::new(
                    TemplateErrorKind::InvalidField(format!(
                        "Field '{}' references invalid page index {}",
                        field.id, field.page_index
                    )),
                    line!(),
                    file!(),
                ));
            }
        }

        Ok(())
    }
}

impl FormTemplate for DrawingTemplate {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn page_count(&self) -> usize {
        self.pages.len()
    }

    fn fields(&self) -> Vec<&FieldDefinition> {
        self.pages
            .iter()
            .flat_map(|page| page.fields.iter())
            .collect()
    }

    fn fields_for_page(&self, page_index: usize) -> Vec<&FieldDefinition> {
        self.pages
            .get(page_index)
            .map(|page| page.fields.iter().collect())
            .unwrap_or_default()
    }

    fn field_by_id(&self, field_id: &str) -> Option<&FieldDefinition> {
        self.pages
            .iter()
            .flat_map(|page| page.fields.iter())
            .find(|f| f.id == field_id)
    }

    fn validate_instance(&self, instance: &dyn FormInstance) -> ValidationResult {
        // Validation implementation will be in a separate module
        // For now, return a simple result
        use form_factor_core::{FieldValidationError, ValidationErrorType};

        let mut result = ValidationResult::success(self.version());

        // Check template ID matches
        if instance.template_id() != self.id() {
            result.add_field_error(FieldValidationError::new(
                "_template",
                format!(
                    "Instance template ID '{}' does not match '{}'",
                    instance.template_id(),
                    self.id()
                ),
                ValidationErrorType::TypeMismatch,
            ));
            return result;
        }

        // Check all required fields are present
        for field_def in self.fields() {
            if field_def.required {
                if let Some(field_value) = instance.field_value(&field_def.id) {
                    if field_value.is_empty() {
                        result.add_missing_required(field_def.id.clone());
                    } else {
                        // Check type compatibility
                        if !field_value
                            .content
                            .matches_field_type(&field_def.field_type)
                        {
                            result.add_field_error(FieldValidationError::new(
                                &field_def.id,
                                format!(
                                    "Value type does not match expected type {}",
                                    field_def.field_type
                                ),
                                ValidationErrorType::TypeMismatch,
                            ));
                        }

                        // Check validation pattern if applicable
                        if let Some(pattern) = field_def.effective_validation_pattern()
                            && let Some(text) = field_value.as_text()
                            && let Ok(regex) = regex::Regex::new(pattern)
                            && !regex.is_match(text)
                        {
                            result.add_field_error(FieldValidationError::new(
                                &field_def.id,
                                format!("Value '{}' does not match pattern {}", text, pattern),
                                ValidationErrorType::PatternMismatch,
                            ));
                        }
                    }
                } else {
                    result.add_missing_required(field_def.id.clone());
                }
            }
        }

        result
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    fn page_dimensions(&self, page_index: usize) -> Option<(u32, u32)> {
        self.pages.get(page_index).and_then(|p| p.dimensions)
    }
}

/// A single page in a form template
///
/// Contains field definitions and optional reference shapes for this page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePage {
    /// Page number (0-indexed)
    pub page_index: usize,

    /// Expected dimensions (width, height in pixels)
    pub dimensions: Option<(u32, u32)>,

    /// Fields on this page
    pub fields: Vec<FieldDefinition>,

    /// Optional reference shapes for visual guidance
    ///
    /// These are not actual form fields but visual markers to help
    /// with template alignment and field placement.
    pub reference_shapes: Vec<Shape>,
}

impl TemplatePage {
    /// Create a new template page
    pub fn new(page_index: usize) -> Self {
        Self {
            page_index,
            dimensions: None,
            fields: Vec::new(),
            reference_shapes: Vec::new(),
        }
    }

    /// Create a new page builder
    pub fn builder(page_index: usize) -> TemplatePageBuilder {
        TemplatePageBuilder::new(page_index)
    }

    /// Add a field to this page
    pub fn add_field(&mut self, field: FieldDefinition) {
        self.fields.push(field);
    }

    /// Add a reference shape
    pub fn add_reference_shape(&mut self, shape: Shape) {
        self.reference_shapes.push(shape);
    }
}

/// Builder for DrawingTemplate
///
/// Provides a fluent API for constructing templates.
#[derive(Debug, Default, Clone)]
pub struct DrawingTemplateBuilder {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    pub(crate) pages: Vec<TemplatePage>,
    metadata: HashMap<String, String>,
}

impl DrawingTemplateBuilder {
    /// Set the template ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the template name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the template version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the template description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a page to the template
    pub fn add_page(mut self, page: TemplatePage) -> Self {
        self.pages.push(page);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the template ID (for validation)
    pub fn get_id(&self) -> &str {
        self.id.as_deref().unwrap_or("")
    }

    /// Get the template name (for validation)
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    /// Get the number of pages in the template
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Get all field definitions across all pages
    pub fn fields(&self) -> Vec<&FieldDefinition> {
        self.pages
            .iter()
            .flat_map(|page| page.fields.iter())
            .collect()
    }

    /// Get fields for a specific page (0-indexed)
    pub fn fields_for_page(&self, page_index: usize) -> Vec<&FieldDefinition> {
        self.pages
            .get(page_index)
            .map(|page| page.fields.iter().collect())
            .unwrap_or_default()
    }

    /// Build the template
    pub fn build(self) -> Result<DrawingTemplate, TemplateError> {
        let template = DrawingTemplate {
            id: self.id.ok_or_else(|| {
                TemplateError::new(
                    TemplateErrorKind::InvalidTemplate("id is required".into()),
                    line!(),
                    file!(),
                )
            })?,
            name: self.name.ok_or_else(|| {
                TemplateError::new(
                    TemplateErrorKind::InvalidTemplate("name is required".into()),
                    line!(),
                    file!(),
                )
            })?,
            version: self.version.unwrap_or_else(|| "1.0.0".to_string()),
            description: self.description,
            pages: self.pages,
            metadata: self.metadata,
        };

        // Validate before returning
        template.validate()?;

        Ok(template)
    }
}

/// Builder for TemplatePage
#[derive(Debug)]
pub struct TemplatePageBuilder {
    page_index: usize,
    dimensions: Option<(u32, u32)>,
    fields: Vec<FieldDefinition>,
    reference_shapes: Vec<Shape>,
}

impl TemplatePageBuilder {
    /// Create a new page builder
    pub fn new(page_index: usize) -> Self {
        Self {
            page_index,
            dimensions: None,
            fields: Vec::new(),
            reference_shapes: Vec::new(),
        }
    }

    /// Set page dimensions
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.dimensions = Some((width, height));
        self
    }

    /// Add a field definition
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }

    /// Add a reference shape
    pub fn add_reference_shape(mut self, shape: Shape) -> Self {
        self.reference_shapes.push(shape);
        self
    }

    /// Build the page
    pub fn build(self) -> TemplatePage {
        TemplatePage {
            page_index: self.page_index,
            dimensions: self.dimensions,
            fields: self.fields,
            reference_shapes: self.reference_shapes,
        }
    }
}
