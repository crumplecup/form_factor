//! Error handling for form_factor binary crate (main.rs)
//!
//! Note: The library exports workspace-wide FormFactorError from form_factor_error crate.
//! This module provides binary-specific error handling.

/// Simple error type for form_factor binary (not part of library API).
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("{}", message)]
pub struct BinaryError {
    message: String,
    line: u32,
    file: &'static str,
}

impl BinaryError {
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

/// Result type alias for form_factor binary (not part of library API).
pub type BinaryResult<T> = Result<T, BinaryError>;
