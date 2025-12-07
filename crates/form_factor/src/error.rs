//! Error handling for form_factor binary crate

/// Simple error type for form_factor binary.
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("{}", message)]
pub struct FormFactorError {
    message: String,
    line: u32,
    file: &'static str,
}

impl FormFactorError {
    /// Creates a new error with location tracking.
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

/// Result type alias for form_factor binary.
pub type FormFactorResult<T> = Result<T, FormFactorError>;
