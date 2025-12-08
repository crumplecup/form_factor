//! Error types for form_factor_core crate
//!
//! Follows CLAUDE.md error handling patterns with derive_more.

use derive_more::{Display, Error};

/// File I/O errors with location tracking
#[derive(Debug, Clone, Display, Error)]
#[display("I/O {} '{}': {} at {}:{}", operation, path, desc, file, line)]
pub struct IoError {
    /// Description of what went wrong
    pub desc: String,
    /// Path to the file that caused the error
    pub path: String,
    /// The I/O operation that failed
    pub operation: IoOperation,
    /// Line number where error occurred
    pub line: u32,
    /// File where error occurred
    pub file: &'static str,
}

impl IoError {
    /// Create a new IoError with location tracking
    #[track_caller]
    pub fn new(
        desc: impl Into<String>,
        path: impl Into<String>,
        operation: IoOperation,
    ) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            desc: desc.into(),
            path: path.into(),
            operation,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

/// Type of I/O operation that failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum IoOperation {
    /// File read operation
    #[display("read")]
    Read,
    /// File write operation
    #[display("write")]
    Write,
    /// File creation operation
    #[display("create")]
    Create,
    /// File deletion operation
    #[display("delete")]
    Delete,
    /// File open operation
    #[display("open")]
    Open,
    /// File close operation
    #[display("close")]
    Close,
}

// ============================================================================
// Crate-Level Umbrella (Pattern 4 from CLAUDE.md)
// ============================================================================

/// Core crate error kind aggregating all module errors
#[derive(Debug, Display, Error, derive_more::From)]
pub enum CoreErrorKind {
    /// I/O error
    #[display("I/O: {}", _0)]
    Io(IoError),
    
    /// Template browser error
    #[display("Template Browser: {}", _0)]
    TemplateBrowser(crate::TemplateBrowserError),
}

/// Core crate umbrella error
#[derive(Debug, Display, Error)]
#[display("Core: {}", _0)]
pub struct CoreError(Box<CoreErrorKind>);

impl<T> From<T> for CoreError
where
    T: Into<CoreErrorKind>,
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

/// Core crate result type
pub type CoreResult<T> = Result<T, CoreError>;
