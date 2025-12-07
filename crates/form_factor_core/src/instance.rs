//! Form instance definitions and traits
//!
//! This module provides the core abstractions for form instances - concrete
//! filled forms based on templates. Instances contain actual data, detected
//! shapes, OCR results, and validation state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::template::{FieldBounds, FieldType};

/// Trait for form instances (concrete filled forms)
///
/// A form instance references a template and contains actual data,
/// detected shapes, and OCR results. Instances can override template
/// positions to handle form variations.
pub trait FormInstance: Send + Sync {
    /// ID of the template this instance is based on
    ///
    /// This is a strong reference - the template should exist in the registry.
    fn template_id(&self) -> &str;

    /// Optional human-readable instance name
    ///
    /// Examples: "John Doe W-2 2024", "Acme Corp Invoice #1234"
    fn instance_name(&self) -> Option<&str>;

    /// Number of pages in this instance
    ///
    /// Should typically match the template's page count, but may differ
    /// for partial forms or scanned pages.
    fn page_count(&self) -> usize;

    /// Get all field values across all pages
    fn field_values(&self) -> Vec<FieldValue>;

    /// Get field values for a specific page (0-indexed)
    fn field_values_for_page(&self, page_index: usize) -> Vec<FieldValue>;

    /// Get value for a specific field by ID
    fn field_value(&self, field_id: &str) -> Option<&FieldValue>;

    /// Set value for a specific field
    ///
    /// Creates or updates the field value. Field ID must match a field
    /// definition in the template for validation to pass.
    fn set_field_value(
        &mut self,
        field_id: &str,
        value: FieldValue,
    ) -> Result<(), Box<dyn std::error::Error>>;

    // Note: Shape access methods are implementation-specific and not part of the trait.
    // Concrete implementations (e.g., DrawingInstance) provide their own shape access methods.

    /// Check if this instance has been validated against its template
    fn is_validated(&self) -> bool;

    /// Get validation results (if validated)
    fn validation_results(&self) -> Option<&ValidationResult>;

    /// Set validation results
    ///
    /// Called after template validation. Stores the validation state
    /// with the instance.
    fn set_validation_results(&mut self, results: ValidationResult);

    /// Get metadata for this instance
    ///
    /// Can store application-specific data like:
    /// - "created_date": "2024-12-04"
    /// - "scanned_by": "user@example.com"
    /// - "confidence_threshold": "0.85"
    fn metadata(&self) -> &HashMap<String, String>;

    /// Serialize to JSON for project file storage
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>>;
}

/// A field value in a form instance
///
/// Links a field definition (by ID) to its actual content and location.
/// The bounds may differ from the template's expected bounds to handle
/// form variations and detection results.
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into))]
pub struct FieldValue {
    /// ID of the field definition this value corresponds to
    ///
    /// Must match a FieldDefinition.id in the template.
    field_id: String,

    /// The actual value content
    content: FieldContent,

    /// Actual bounds where this field was found/placed
    ///
    /// May differ from template bounds due to:
    /// - Form variations (different printing, scaling)
    /// - Detection results
    /// - Manual adjustment
    bounds: FieldBounds,

    /// Page index where this value appears (0-indexed)
    page_index: usize,

    /// Confidence score (0.0-1.0) if detected via OCR/CV
    ///
    /// None if manually entered or not applicable.
    confidence: Option<f32>,

    /// Whether this value has been manually verified by a user
    ///
    /// Used to track human review of automated extractions.
    verified: bool,
}

impl FieldValue {
    /// Create a new field value with text content
    pub fn new_text(
        field_id: impl Into<String>,
        text: impl Into<String>,
        bounds: FieldBounds,
        page_index: usize,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            content: FieldContent::Text(text.into()),
            bounds,
            page_index,
            confidence: None,
            verified: false,
        }
    }

    /// Create a new field value with boolean content (checkbox/radio)
    pub fn new_boolean(
        field_id: impl Into<String>,
        value: bool,
        bounds: FieldBounds,
        page_index: usize,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            content: FieldContent::Boolean(value),
            bounds,
            page_index,
            confidence: None,
            verified: false,
        }
    }

    /// Create a new empty field value (placeholder)
    pub fn new_empty(field_id: impl Into<String>, bounds: FieldBounds, page_index: usize) -> Self {
        Self {
            field_id: field_id.into(),
            content: FieldContent::Empty,
            bounds,
            page_index,
            confidence: None,
            verified: false,
        }
    }

    /// Create a new field value with signature content
    pub fn new_signature(
        field_id: impl Into<String>,
        present: bool,
        shape_index: Option<usize>,
        bounds: FieldBounds,
        page_index: usize,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            content: FieldContent::Signature {
                present,
                shape_index,
            },
            bounds,
            page_index,
            confidence: None,
            verified: false,
        }
    }

    /// Set the confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }

    /// Mark as verified
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    /// Check if this field value is empty
    pub fn is_empty(&self) -> bool {
        matches!(self.content, FieldContent::Empty)
    }

    /// Get text content if present
    pub fn as_text(&self) -> Option<&str> {
        match &self.content {
            FieldContent::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get boolean content if present
    pub fn as_boolean(&self) -> Option<bool> {
        match self.content {
            FieldContent::Boolean(b) => Some(b),
            _ => None,
        }
    }

    /// Get numeric content if present
    pub fn as_number(&self) -> Option<f64> {
        match self.content {
            FieldContent::Number(n) => Some(n),
            _ => None,
        }
    }
}

/// Content of a field value
///
/// Represents different types of data that can be extracted from or
/// entered into form fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum FieldContent {
    /// Text content (from OCR or manual entry)
    Text(String),

    /// Boolean value (for checkboxes, radio buttons)
    Boolean(bool),

    /// Signature present
    ///
    /// Tracks whether a signature was detected and optionally which
    /// shape represents it.
    Signature {
        /// Whether a signature is present
        present: bool,
        /// Index of the shape in the shapes vector (if applicable)
        shape_index: Option<usize>,
    },

    /// Logo detected
    ///
    /// Tracks logo detection results and matching against known logos.
    Logo {
        /// Whether a logo was detected
        detected: bool,
        /// Name of matched logo template (if any)
        match_name: Option<String>,
        /// Detection confidence (0.0-1.0)
        confidence: Option<f32>,
    },

    /// Numeric value
    Number(f64),

    /// No content detected or entered yet
    Empty,
}

impl FieldContent {
    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, FieldContent::Empty)
    }

    /// Check if content matches the expected field type
    pub fn matches_field_type(&self, field_type: &FieldType) -> bool {
        match (self, field_type) {
            (FieldContent::Empty, _) => true, // Empty is valid for any type
            (FieldContent::Boolean(_), FieldType::Checkbox | FieldType::RadioButton) => true,
            (FieldContent::Signature { .. }, FieldType::Signature | FieldType::Initials) => true,
            (FieldContent::Logo { .. }, FieldType::Logo) => true,
            (FieldContent::Number(_), FieldType::NumericField | FieldType::Amount) => true,
            (FieldContent::Text(_), ft) if ft.expects_text() => true,
            _ => false,
        }
    }
}

/// Result of validating an instance against a template
///
/// Captures all validation errors, missing required fields, and
/// metadata about the validation process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_getters::Getters)]
pub struct ValidationResult {
    /// Overall validation passed
    ///
    /// True only if no errors and all required fields present.
    valid: bool,

    /// Field-specific validation issues
    field_errors: Vec<FieldValidationError>,

    /// Required fields that are missing or empty
    missing_required: Vec<String>,

    /// Template version this was validated against
    template_version: String,

    /// Timestamp of validation (ISO 8601 format)
    ///
    /// Example: "2024-12-04T10:30:00Z"
    timestamp: String,
}

impl ValidationResult {
    /// Get current timestamp in ISO 8601 format
    fn current_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        // Simple ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
        // For simplicity, we just use Unix timestamp for now
        // Implementations can override with proper formatting
        format!("{}", duration.as_secs())
    }

    /// Create a new successful validation result
    pub fn success(template_version: impl Into<String>) -> Self {
        Self {
            valid: true,
            field_errors: Vec::new(),
            missing_required: Vec::new(),
            template_version: template_version.into(),
            timestamp: Self::current_timestamp(),
        }
    }

    /// Create a new failed validation result
    pub fn failure(
        template_version: impl Into<String>,
        field_errors: Vec<FieldValidationError>,
        missing_required: Vec<String>,
    ) -> Self {
        Self {
            valid: field_errors.is_empty() && missing_required.is_empty(),
            field_errors,
            missing_required,
            template_version: template_version.into(),
            timestamp: Self::current_timestamp(),
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get total error count
    pub fn error_count(&self) -> usize {
        self.field_errors.len() + self.missing_required.len()
    }

    /// Add a field error
    pub fn add_field_error(&mut self, error: FieldValidationError) {
        self.field_errors.push(error);
        self.valid = false;
    }

    /// Add a missing required field
    pub fn add_missing_required(&mut self, field_id: impl Into<String>) {
        self.missing_required.push(field_id.into());
        self.valid = false;
    }
}

/// A field-specific validation error
///
/// Describes a validation failure for a particular field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, derive_getters::Getters)]
pub struct FieldValidationError {
    /// Field ID that failed validation
    field_id: String,

    /// Human-readable error message
    message: String,

    /// Error category for programmatic handling
    error_type: ValidationErrorType,
}

impl FieldValidationError {
    /// Create a new field validation error
    pub fn new(
        field_id: impl Into<String>,
        message: impl Into<String>,
        error_type: ValidationErrorType,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            message: message.into(),
            error_type,
        }
    }
}

impl std::fmt::Display for FieldValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Field '{}': {} ({:?})",
            self.field_id, self.message, self.error_type
        )
    }
}

/// Category of validation error
///
/// Allows programmatic handling of different error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorType {
    /// Required field is missing or empty
    Required,

    /// Value doesn't match validation pattern (regex)
    PatternMismatch,

    /// Value type doesn't match field type
    TypeMismatch,

    /// Field position is outside expected bounds
    OutOfBounds,

    /// Custom validation error
    Custom,
}

impl std::fmt::Display for ValidationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationErrorType::Required => write!(f, "Required field"),
            ValidationErrorType::PatternMismatch => write!(f, "Pattern mismatch"),
            ValidationErrorType::TypeMismatch => write!(f, "Type mismatch"),
            ValidationErrorType::OutOfBounds => write!(f, "Out of bounds"),
            ValidationErrorType::Custom => write!(f, "Custom validation error"),
        }
    }
}
