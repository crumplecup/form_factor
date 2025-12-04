//! Concrete implementation of FormInstance trait

use super::error::{InstanceError, InstanceErrorKind};
use crate::{CanvasError, DrawingCanvas, Shape};
use form_factor_core::{FieldValue, FormInstance, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Concrete implementation of FormInstance
///
/// Represents a filled form based on a template, with support for multiple pages.
/// Each page wraps a DrawingCanvas containing shapes, detections, and image data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingInstance {
    /// Template ID this instance is based on
    template_id: String,

    /// Optional instance name for display
    instance_name: Option<String>,

    /// Pages in this instance
    pages: Vec<FormPage>,

    /// Field values across all pages (indexed by field ID)
    field_values: HashMap<String, FieldValue>,

    /// Validation results (if validated)
    #[serde(skip)]
    validation_results: Option<ValidationResult>,

    /// Instance metadata
    metadata: HashMap<String, String>,
}

impl DrawingInstance {
    /// Create a new instance from a template ID
    ///
    /// Creates an instance with the specified number of pages,
    /// each containing an empty DrawingCanvas.
    pub fn from_template(template_id: impl Into<String>, page_count: usize) -> Self {
        let pages = (0..page_count).map(FormPage::new).collect();

        Self {
            template_id: template_id.into(),
            instance_name: None,
            pages,
            field_values: HashMap::new(),
            validation_results: None,
            metadata: HashMap::new(),
        }
    }

    /// Create from JSON
    pub fn from_json(json: &str) -> Result<Self, InstanceError> {
        serde_json::from_str(json).map_err(|e| {
            InstanceError::new(
                InstanceErrorKind::Deserialization(e.to_string()),
                line!(),
                file!(),
            )
        })
    }

    /// Set the instance name
    pub fn set_instance_name(&mut self, name: impl Into<String>) {
        self.instance_name = Some(name.into());
    }

    /// Get a specific page
    pub fn page(&self, index: usize) -> Option<&FormPage> {
        self.pages.get(index)
    }

    /// Get a mutable reference to a specific page
    pub fn page_mut(&mut self, index: usize) -> Option<&mut FormPage> {
        self.pages.get_mut(index)
    }

    /// Get all pages
    pub fn pages(&self) -> &[FormPage] {
        &self.pages
    }

    /// Get shapes for a specific page
    pub fn shapes_for_page(&self, page_index: usize) -> Vec<&Shape> {
        self.pages
            .get(page_index)
            .map(|page| page.canvas.shapes().iter().collect())
            .unwrap_or_default()
    }

    /// Get detections for a specific page
    pub fn detections_for_page(&self, page_index: usize) -> Vec<&Shape> {
        self.pages
            .get(page_index)
            .map(|page| page.canvas.detections().iter().collect())
            .unwrap_or_default()
    }

    /// Add a metadata entry
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

impl FormInstance for DrawingInstance {
    fn template_id(&self) -> &str {
        &self.template_id
    }

    fn instance_name(&self) -> Option<&str> {
        self.instance_name.as_deref()
    }

    fn page_count(&self) -> usize {
        self.pages.len()
    }

    fn field_values(&self) -> Vec<FieldValue> {
        self.field_values.values().cloned().collect()
    }

    fn field_values_for_page(&self, page_index: usize) -> Vec<FieldValue> {
        self.field_values
            .values()
            .filter(|v| v.page_index == page_index)
            .cloned()
            .collect()
    }

    fn field_value(&self, field_id: &str) -> Option<&FieldValue> {
        self.field_values.get(field_id)
    }

    fn set_field_value(
        &mut self,
        field_id: &str,
        value: FieldValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.field_values.insert(field_id.to_string(), value);
        Ok(())
    }

    fn is_validated(&self) -> bool {
        self.validation_results.is_some()
    }

    fn validation_results(&self) -> Option<&ValidationResult> {
        self.validation_results.as_ref()
    }

    fn set_validation_results(&mut self, results: ValidationResult) {
        self.validation_results = Some(results);
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// A single page in a form instance
///
/// Wraps a DrawingCanvas to provide multi-page support while maintaining
/// compatibility with existing canvas functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormPage {
    /// Page index (0-indexed)
    pub page_index: usize,

    /// The drawing canvas for this page
    ///
    /// Contains shapes, detections, image path, zoom/pan state, etc.
    pub canvas: DrawingCanvas,

    /// Path to the image file for this page
    ///
    /// Duplicated from canvas for convenience during serialization.
    pub image_path: Option<String>,
}

impl FormPage {
    /// Create a new form page
    pub fn new(page_index: usize) -> Self {
        Self {
            page_index,
            canvas: DrawingCanvas::default(),
            image_path: None,
        }
    }

    /// Create a page from an existing canvas
    pub fn from_canvas(page_index: usize, canvas: DrawingCanvas) -> Self {
        let image_path = canvas.form_image_path().clone();
        Self {
            page_index,
            image_path,
            canvas,
        }
    }

    /// Get a reference to the canvas
    pub fn canvas(&self) -> &DrawingCanvas {
        &self.canvas
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut DrawingCanvas {
        &mut self.canvas
    }

    /// Load an image for this page
    ///
    /// Loads the image into the canvas and stores the path.
    pub fn load_image(
        &mut self,
        path: &str,
        ctx: &egui::Context,
    ) -> Result<(), CanvasError> {
        self.canvas.load_form_image(path, ctx)?;
        self.image_path = Some(path.to_string());
        Ok(())
    }

    /// Get the image path for this page
    pub fn image_path(&self) -> Option<&str> {
        self.image_path.as_deref()
    }

    /// Get all shapes on this page
    pub fn shapes(&self) -> &[Shape] {
        self.canvas.shapes()
    }

    /// Get all detections on this page
    pub fn detections(&self) -> &[Shape] {
        self.canvas.detections()
    }
}
