//! Error types for form instance operations

use derive_more::{Display, Error};

/// Kinds of errors that can occur during instance operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceErrorKind {
    /// Instance deserialization failed
    Deserialization(String),

    /// Instance serialization failed
    Serialization(String),

    /// Referenced template not found
    TemplateNotFound(String),

    /// Page index out of bounds
    InvalidPageIndex {
        /// Requested index
        index: usize,
        /// Maximum valid index
        max: usize,
    },

    /// Field not found
    FieldNotFound(String),

    /// Validation failed
    ValidationFailed(String),

    /// I/O error
    IoError(String),

    /// Invalid instance structure
    InvalidInstance(String),

    /// OCR extraction failed
    #[cfg(feature = "ocr")]
    OCRFailed(String),
}

impl std::fmt::Display for InstanceErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceErrorKind::Deserialization(msg) => {
                write!(f, "Failed to deserialize instance: {}", msg)
            }
            InstanceErrorKind::Serialization(msg) => {
                write!(f, "Failed to serialize instance: {}", msg)
            }
            InstanceErrorKind::TemplateNotFound(id) => write!(f, "Template not found: {}", id),
            InstanceErrorKind::InvalidPageIndex { index, max } => {
                write!(f, "Invalid page index {} (max: {})", index, max)
            }
            InstanceErrorKind::FieldNotFound(id) => write!(f, "Field not found: {}", id),
            InstanceErrorKind::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            InstanceErrorKind::IoError(msg) => write!(f, "I/O error: {}", msg),
            InstanceErrorKind::InvalidInstance(msg) => write!(f, "Invalid instance: {}", msg),
            #[cfg(feature = "ocr")]
            InstanceErrorKind::OCRFailed(msg) => write!(f, "OCR extraction failed: {}", msg),
        }
    }
}

/// Instance error with location information
#[derive(Debug, Clone, Display, Error)]
#[display("Instance Error: {} at line {} in {}", kind, line, file)]
pub struct InstanceError {
    /// The kind of error
    pub kind: InstanceErrorKind,
    /// Line number where error occurred
    pub line: u32,
    /// Source file where error occurred
    pub file: &'static str,
}

impl InstanceError {
    /// Create a new instance error
    pub fn new(kind: InstanceErrorKind, line: u32, file: &'static str) -> Self {
        Self { kind, line, file }
    }
}
