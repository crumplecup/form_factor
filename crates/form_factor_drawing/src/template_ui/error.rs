//! Error types for template management operations.

/// Errors that can occur during template management operations.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("Template manager error: {} at {}:{}", message, file, line)]
pub struct TemplateManagerError {
    /// Error message
    pub message: String,
    /// Line number where error occurred
    pub line: u32,
    /// Source file where error occurred
    pub file: &'static str,
}

impl TemplateManagerError {
    /// Create a new template manager error.
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            message: message.into(),
            line: loc.line(),
            file: loc.file(),
        }
    }
}

/// Result type for template management operations.
pub type TemplateManagerResult<T> = Result<T, TemplateManagerError>;
