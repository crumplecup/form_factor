//! Template validation for the template editor.

use crate::{DrawingTemplateBuilder, TemplateRegistry};
use regex::Regex;
use std::collections::HashSet;
use tracing::{debug, instrument};

/// Validates a template before saving.
pub struct TemplateValidator;

impl TemplateValidator {
    /// Validates a template builder against the registry.
    ///
    /// Returns a list of validation errors, or an empty vector if valid.
    #[instrument(skip(template, registry))]
    pub fn validate(
        template: &DrawingTemplateBuilder,
        registry: &TemplateRegistry,
        is_new: bool,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Get template metadata
        let id = template.get_id();
        let name = template.get_name();

        // Check template ID
        if id.is_empty() {
            errors.push(ValidationError::EmptyTemplateId);
            debug!("Validation error: Empty template ID");
        } else if is_new && registry.contains(id) {
            errors.push(ValidationError::DuplicateTemplateId(id.to_string()));
            debug!(template_id = %id, "Validation error: Duplicate template ID");
        }

        // Check template name
        if name.is_empty() {
            errors.push(ValidationError::EmptyTemplateName);
            debug!("Validation error: Empty template name");
        }

        // Check fields exist
        let all_fields = template.fields();
        if all_fields.is_empty() {
            errors.push(ValidationError::NoFields);
            debug!("Validation error: No fields defined");
        }

        // Check field ID uniqueness
        let mut seen_field_ids = HashSet::new();
        for field in &all_fields {
            if !seen_field_ids.insert(&field.id) {
                errors.push(ValidationError::DuplicateFieldId(field.id.clone()));
                debug!(field_id = %field.id, "Validation error: Duplicate field ID");
            }
        }

        // Check field bounds validity
        for field in &all_fields {
            if field.bounds.width <= 0.0 || field.bounds.height <= 0.0 {
                errors.push(ValidationError::InvalidFieldBounds {
                    field_id: field.id.clone(),
                    width: field.bounds.width,
                    height: field.bounds.height,
                });
                debug!(
                    field_id = %field.id,
                    width = field.bounds.width,
                    height = field.bounds.height,
                    "Validation error: Invalid field bounds"
                );
            }
        }

        // Check regex patterns
        for field in &all_fields {
            if let Some(pattern) = &field.validation_pattern
                && Regex::new(pattern).is_err()
            {
                errors.push(ValidationError::InvalidRegexPattern {
                    field_id: field.id.clone(),
                    pattern: pattern.clone(),
                });
                debug!(
                    field_id = %field.id,
                    pattern = %pattern,
                    "Validation error: Invalid regex pattern"
                );
            }
        }

        // Check that each field's page_index is valid
        let page_count = template.page_count();
        for field in &all_fields {
            if field.page_index >= page_count {
                errors.push(ValidationError::InvalidPageIndex {
                    field_id: field.id.clone(),
                    page_index: field.page_index,
                    page_count,
                });
                debug!(
                    field_id = %field.id,
                    page_index = field.page_index,
                    page_count = page_count,
                    "Validation error: Invalid page index"
                );
            }
        }

        if errors.is_empty() {
            debug!("Template validation passed");
        } else {
            debug!(error_count = errors.len(), "Template validation failed");
        }

        errors
    }
}

/// Validation errors that can occur during template validation.
#[derive(Debug, Clone, PartialEq, derive_more::Display)]
pub enum ValidationError {
    /// Template ID is empty
    #[display("Template ID cannot be empty")]
    EmptyTemplateId,

    /// Template ID already exists in registry (for new templates)
    #[display("Template ID '{}' already exists in registry", _0)]
    DuplicateTemplateId(String),

    /// Template name is empty
    #[display("Template name cannot be empty")]
    EmptyTemplateName,

    /// Template has no fields
    #[display("Template must have at least one field")]
    NoFields,

    /// Duplicate field ID within template
    #[display("Duplicate field ID: '{}'", _0)]
    DuplicateFieldId(String),

    /// Field has invalid bounds (non-positive width or height)
    #[display(
        "Field '{}' has invalid bounds: width={}, height={}",
        field_id,
        width,
        height
    )]
    InvalidFieldBounds {
        /// Field ID with invalid bounds
        field_id: String,
        /// Field width
        width: f32,
        /// Field height
        height: f32,
    },

    /// Field has invalid regex pattern
    #[display("Field '{}' has invalid regex pattern: '{}'", field_id, pattern)]
    InvalidRegexPattern {
        /// Field ID with invalid pattern
        field_id: String,
        /// Invalid pattern
        pattern: String,
    },

    /// Field references invalid page index
    #[display(
        "Field '{}' references invalid page {} (template has {} pages)",
        field_id,
        page_index,
        page_count
    )]
    InvalidPageIndex {
        /// Field ID with invalid page reference
        field_id: String,
        /// Referenced page index
        page_index: usize,
        /// Actual page count
        page_count: usize,
    },
}

impl std::error::Error for ValidationError {}

/// Removed redundant ValidationError implementation below
#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
enum _OldValidationError {}
impl ValidationError {
    /// Returns the field ID associated with this error, if any.
    pub fn field_id(&self) -> Option<&str> {
        match self {
            Self::DuplicateFieldId(id)
            | Self::InvalidFieldBounds { field_id: id, .. }
            | Self::InvalidRegexPattern { field_id: id, .. }
            | Self::InvalidPageIndex { field_id: id, .. } => Some(id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DrawingTemplateBuilder;
    use form_factor_core::{FieldBounds, FieldDefinition, FieldType};

    #[test]
    fn test_valid_template() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let mut builder = DrawingTemplateBuilder::default()
            .id("test_template")
            .name("Test Template")
            .version("1.0.0");

        // Add a page and field
        builder.pages.push(crate::TemplatePage::new(0));
        builder.pages[0].add_field(FieldDefinition {
            id: "field1".to_string(),
            label: "Field 1".to_string(),
            field_type: FieldType::FreeText,
            page_index: 0,
            bounds: FieldBounds {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 30.0,
            },
            required: false,
            validation_pattern: None,
            help_text: None,
            metadata: std::collections::HashMap::new(),
        });

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(errors.is_empty(), "Expected no validation errors");
    }

    #[test]
    fn test_empty_template_id() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let builder = DrawingTemplateBuilder::default().name("Test Template");

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(errors.contains(&ValidationError::EmptyTemplateId));
    }

    #[test]
    fn test_empty_template_name() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let builder = DrawingTemplateBuilder::default().id("test_template");

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(errors.contains(&ValidationError::EmptyTemplateName));
    }

    #[test]
    fn test_no_fields() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let builder = DrawingTemplateBuilder::default()
            .id("test_template")
            .name("Test Template");

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(errors.contains(&ValidationError::NoFields));
    }

    #[test]
    fn test_duplicate_field_id() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let mut builder = DrawingTemplateBuilder::default()
            .id("test_template")
            .name("Test Template");

        builder.pages.push(crate::TemplatePage::new(0));

        // Add two fields with the same ID
        let field = FieldDefinition {
            id: "field1".to_string(),
            label: "Field 1".to_string(),
            field_type: FieldType::FreeText,
            page_index: 0,
            bounds: FieldBounds {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 30.0,
            },
            required: false,
            validation_pattern: None,
            help_text: None,
            metadata: std::collections::HashMap::new(),
        };

        builder.pages[0].add_field(field.clone());
        builder.pages[0].add_field(field);

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(errors.contains(&ValidationError::DuplicateFieldId("field1".to_string())));
    }

    #[test]
    fn test_invalid_field_bounds() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let mut builder = DrawingTemplateBuilder::default()
            .id("test_template")
            .name("Test Template");

        builder.pages.push(crate::TemplatePage::new(0));
        builder.pages[0].add_field(FieldDefinition {
            id: "field1".to_string(),
            label: "Field 1".to_string(),
            field_type: FieldType::FreeText,
            page_index: 0,
            bounds: FieldBounds {
                x: 10.0,
                y: 10.0,
                width: 0.0, // Invalid
                height: 30.0,
            },
            required: false,
            validation_pattern: None,
            help_text: None,
            metadata: std::collections::HashMap::new(),
        });

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::InvalidFieldBounds { .. }))
        );
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let registry = TemplateRegistry::new().expect("Failed to create registry");
        let mut builder = DrawingTemplateBuilder::default()
            .id("test_template")
            .name("Test Template");

        builder.pages.push(crate::TemplatePage::new(0));
        builder.pages[0].add_field(FieldDefinition {
            id: "field1".to_string(),
            label: "Field 1".to_string(),
            field_type: FieldType::FreeText,
            page_index: 0,
            bounds: FieldBounds {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 30.0,
            },
            required: false,
            validation_pattern: Some("[invalid(regex".to_string()), // Invalid regex
            help_text: None,
            metadata: std::collections::HashMap::new(),
        });

        let errors = TemplateValidator::validate(&builder, &registry, true);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::InvalidRegexPattern { .. }))
        );
    }
}
