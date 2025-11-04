//! Common error types for form_factor
//!
//! This module provides shared error types that can be used across
//! all form_factor crates.

use std::fmt;

/// File I/O errors
#[derive(Debug, Clone)]
pub struct IoError {
    /// Description of what went wrong
    pub desc: String,

    /// Path to the file that caused the error
    pub path: String,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: &'static str,

    /// The I/O operation that failed
    pub operation: IoOperation,
}

impl IoError {
    /// Create a new IoError
    pub fn new(
        desc: impl Into<String>,
        path: impl Into<String>,
        operation: IoOperation,
        line: u32,
        file: &'static str,
    ) -> Self {
        Self {
            desc: desc.into(),
            path: path.into(),
            line,
            file,
            operation,
        }
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "I/O error during {} of '{}': {} at line {} in {}",
            self.operation, self.path, self.desc, self.line, self.file
        )
    }
}

impl std::error::Error for IoError {}

/// Type of I/O operation that failed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    /// File read operation
    Read,
    /// File write operation
    Write,
    /// File creation operation
    Create,
    /// File deletion operation
    Delete,
    /// File open operation
    Open,
    /// File close operation
    Close,
}

impl fmt::Display for IoOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoOperation::Read => write!(f, "read"),
            IoOperation::Write => write!(f, "write"),
            IoOperation::Create => write!(f, "create"),
            IoOperation::Delete => write!(f, "delete"),
            IoOperation::Open => write!(f, "open"),
            IoOperation::Close => write!(f, "close"),
        }
    }
}
