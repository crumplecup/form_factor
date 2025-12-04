//! Error types for template operations

use derive_more::{Display, Error};

/// Kinds of errors that can occur during template operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateErrorKind {
    /// Failed to find config directory
    ConfigDirNotFound,

    /// Template not found in registry
    NotFound(String),

    /// Template deserialization failed
    Deserialization(String),

    /// Template serialization failed
    Serialization(String),

    /// I/O error
    IoError(String),

    /// Invalid template structure
    InvalidTemplate(String),

    /// Version mismatch
    VersionMismatch {
        /// Expected version
        expected: String,
        /// Found version
        found: String,
    },

    /// Invalid field definition
    InvalidField(String),

    /// Duplicate field ID
    DuplicateFieldId(String),
}

impl std::fmt::Display for TemplateErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateErrorKind::ConfigDirNotFound => {
                write!(f, "Could not determine config directory")
            }
            TemplateErrorKind::NotFound(id) => write!(f, "Template not found: {}", id),
            TemplateErrorKind::Deserialization(msg) => {
                write!(f, "Failed to deserialize template: {}", msg)
            }
            TemplateErrorKind::Serialization(msg) => {
                write!(f, "Failed to serialize template: {}", msg)
            }
            TemplateErrorKind::IoError(msg) => write!(f, "I/O error: {}", msg),
            TemplateErrorKind::InvalidTemplate(msg) => write!(f, "Invalid template: {}", msg),
            TemplateErrorKind::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Version mismatch: expected {}, found {}",
                    expected, found
                )
            }
            TemplateErrorKind::InvalidField(msg) => write!(f, "Invalid field: {}", msg),
            TemplateErrorKind::DuplicateFieldId(id) => write!(f, "Duplicate field ID: {}", id),
        }
    }
}

/// Template error with location information
#[derive(Debug, Clone, Display, Error)]
#[display("Template Error: {} at line {} in {}", kind, line, file)]
pub struct TemplateError {
    /// The kind of error
    pub kind: TemplateErrorKind,
    /// Line number where error occurred
    pub line: u32,
    /// Source file where error occurred
    pub file: &'static str,
}

impl TemplateError {
    /// Create a new template error
    pub fn new(kind: TemplateErrorKind, line: u32, file: &'static str) -> Self {
        Self { kind, line, file }
    }
}
