//! Error types for template browser operations.

/// Error kinds for template browser operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum TemplateBrowserErrorKind {
    /// Template index out of bounds.
    #[display("Template index {} out of bounds (max: {})", index, max)]
    IndexOutOfBounds {
        /// The requested index.
        index: usize,
        /// Maximum valid index.
        max: usize,
    },

    /// Invalid template ID.
    #[display("Invalid template ID: {}", _0)]
    InvalidTemplateId(String),

    /// Template not found.
    #[display("Template not found: {}", _0)]
    TemplateNotFound(String),
}

/// Error wrapper with location tracking for template browser operations.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("Template browser error: {} at {}:{}", kind, file, line)]
pub struct TemplateBrowserError {
    /// The specific error kind.
    pub kind: TemplateBrowserErrorKind,
    /// Line number where error occurred.
    pub line: u32,
    /// Source file where error occurred.
    pub file: &'static str,
}

impl TemplateBrowserError {
    /// Creates a new template browser error with caller location.
    #[track_caller]
    pub fn new(kind: TemplateBrowserErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

/// Result type for template browser operations.
pub type TemplateBrowserResult<T> = Result<T, TemplateBrowserError>;
