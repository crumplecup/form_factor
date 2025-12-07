//! Detection metadata for storing editable information about detected regions.
//!
//! This module provides structures for storing metadata about detections
//! (logos, text regions, OCR results) that users can edit through the UI.

use derive_getters::Getters;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of detection for metadata categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Display)]
pub enum MetadataDetectionType {
    /// Logo detection
    #[display("Logo")]
    Logo,
    /// Text region detection
    #[display("Text Region")]
    TextRegion,
    /// OCR extracted text
    #[display("OCR Text")]
    OcrText,
}

/// Form field type that a detection can be associated with
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Display)]
pub enum FormFieldType {
    /// Single-line text input
    #[display("Text")]
    Text,
    /// Multi-line text area
    #[display("Text Area")]
    TextArea,
    /// Date picker
    #[display("Date")]
    Date,
    /// Number input
    #[display("Number")]
    Number,
    /// Checkbox
    #[display("Checkbox")]
    Checkbox,
    /// Radio button
    #[display("Radio")]
    Radio,
    /// Dropdown selection
    #[display("Dropdown")]
    Dropdown,
    /// Signature field
    #[display("Signature")]
    Signature,
}

/// Metadata associated with a detection
#[derive(Debug, Clone, Getters, Setters, Serialize, Deserialize)]
#[setters(prefix = "with_", borrow_self)]
pub struct DetectionMetadata {
    /// Unique ID for this detection
    #[setters(doc = "Sets the detection ID")]
    id: String,

    /// Type of detection
    #[setters(doc = "Sets the detection type")]
    detection_type: MetadataDetectionType,

    /// User-assigned label/name
    #[setters(doc = "Sets the label")]
    label: Option<String>,

    /// Confidence score from detection (0.0-1.0)
    #[setters(doc = "Sets the confidence score")]
    confidence: f64,

    /// For OCR: the extracted text (user-editable)
    #[setters(doc = "Sets the extracted text")]
    extracted_text: Option<String>,

    /// Form field type association
    #[setters(doc = "Sets the form field type")]
    form_field_type: Option<FormFieldType>,

    /// Form field name (for data binding)
    #[setters(doc = "Sets the form field name")]
    form_field_name: Option<String>,

    /// Whether the form field is required
    #[setters(doc = "Sets required status")]
    form_field_required: Option<bool>,

    /// Default value for the form field
    #[setters(doc = "Sets default value")]
    form_field_default_value: Option<String>,

    /// Help text for the form field
    #[setters(doc = "Sets help text")]
    form_field_help_text: Option<String>,

    /// Options for dropdown/radio fields
    #[setters(doc = "Sets field options")]
    form_field_options: Option<Vec<String>>,

    /// Minimum value for number fields
    #[setters(doc = "Sets minimum value")]
    form_field_min: Option<f64>,

    /// Maximum value for number fields
    #[setters(doc = "Sets maximum value")]
    form_field_max: Option<f64>,

    /// User notes
    #[setters(doc = "Sets notes")]
    notes: Option<String>,

    /// Custom key-value metadata
    #[setters(doc = "Sets custom metadata")]
    custom_metadata: HashMap<String, String>,

    /// Whether this detection is validated by user
    #[setters(doc = "Sets validation status")]
    validated: bool,
}

impl DetectionMetadata {
    /// Creates new detection metadata.
    #[track_caller]
    pub fn new(id: String, detection_type: MetadataDetectionType, confidence: f64) -> Self {
        Self {
            id,
            detection_type,
            label: None,
            confidence,
            extracted_text: None,
            form_field_type: None,
            form_field_name: None,
            form_field_required: None,
            form_field_default_value: None,
            form_field_help_text: None,
            form_field_options: None,
            form_field_min: None,
            form_field_max: None,
            notes: None,
            custom_metadata: HashMap::new(),
            validated: false,
        }
    }

    /// Sets a custom metadata field.
    pub fn set_custom(&mut self, key: String, value: String) {
        self.custom_metadata.insert(key, value);
    }

    /// Gets a custom metadata field.
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom_metadata.get(key)
    }

    /// Removes a custom metadata field.
    pub fn remove_custom(&mut self, key: &str) -> Option<String> {
        self.custom_metadata.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_detection_metadata() {
        let metadata =
            DetectionMetadata::new("det_001".to_string(), MetadataDetectionType::Logo, 0.95);

        assert_eq!(metadata.id(), "det_001");
        assert_eq!(*metadata.detection_type(), MetadataDetectionType::Logo);
        assert_eq!(*metadata.confidence(), 0.95);
        assert!(!metadata.validated());
    }

    #[test]
    fn test_set_label() {
        let mut metadata =
            DetectionMetadata::new("det_001".to_string(), MetadataDetectionType::Logo, 0.95);

        metadata.with_label(Some("Company Logo".to_string()));
        assert_eq!(metadata.label(), &Some("Company Logo".to_string()));
    }

    #[test]
    fn test_custom_metadata() {
        let mut metadata =
            DetectionMetadata::new("det_001".to_string(), MetadataDetectionType::OcrText, 0.89);

        metadata.set_custom("language".to_string(), "en".to_string());
        metadata.set_custom("font_size".to_string(), "12".to_string());

        assert_eq!(metadata.get_custom("language"), Some(&"en".to_string()));
        assert_eq!(metadata.get_custom("font_size"), Some(&"12".to_string()));

        let removed = metadata.remove_custom("language");
        assert_eq!(removed, Some("en".to_string()));
        assert_eq!(metadata.get_custom("language"), None);
    }

    #[test]
    fn test_form_field_association() {
        let mut metadata =
            DetectionMetadata::new("det_001".to_string(), MetadataDetectionType::OcrText, 0.92);

        metadata
            .with_form_field_type(Some(FormFieldType::Date))
            .with_form_field_name(Some("birth_date".to_string()));

        assert_eq!(metadata.form_field_type(), &Some(FormFieldType::Date));
        assert_eq!(metadata.form_field_name(), &Some("birth_date".to_string()));
    }
}
