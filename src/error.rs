//! Error handling for form_factor
//!
//! This module provides a comprehensive error system with a boxed enum pattern
//! to keep error types lightweight (pointer-sized) while allowing detailed
//! error information.
//!
//! # Architecture
//!
//! - `FormError`: Top-level error type (wraps `Box<FormErrorKind>`)
//! - `FormErrorKind`: Enum of error categories
//! - Specific error structs: Detailed error information for each category
//!
//! # Example
//!
//! ```
//! use form_factor::{FormError, AccessKitError};
//!
//! fn do_something() -> Result<(), FormError> {
//!     Err(AccessKitError::new(
//!         "Failed to initialize accessibility tree",
//!         line!(),
//!         file!(),
//!     ).into())
//! }
//! ```

use std::fmt;

/// Top-level error type for form_factor
///
/// This type wraps a boxed `FormErrorKind` to keep the error size small
/// (pointer-sized) regardless of the underlying error variant size.
#[derive(Debug)]
pub struct FormError(Box<FormErrorKind>);

impl FormError {
    /// Create a new FormError from a FormErrorKind
    pub fn new(kind: FormErrorKind) -> Self {
        Self(Box::new(kind))
    }

    /// Get a reference to the underlying error kind
    pub fn kind(&self) -> &FormErrorKind {
        &self.0
    }

    /// Consume the error and return the underlying kind
    pub fn into_kind(self) -> FormErrorKind {
        *self.0
    }
}

/// Categories of errors that can occur in form_factor
#[derive(Debug)]
pub enum FormErrorKind {
    /// AccessKit-related errors (accessibility)
    AccessKit(AccessKitError),

    /// Egui-related errors (GUI rendering)
    Egui(EguiError),

    /// Backend initialization or runtime errors
    Backend(BackendError),

    /// File I/O errors (form scanning, OCR metadata)
    Io(IoError),

    /// Configuration errors
    Config(ConfigError),

    /// Application-level errors
    App(AppError),
}

// ============================================================================
// Specific Error Structs
// ============================================================================

/// AccessKit accessibility errors
#[derive(Debug, Clone)]
pub struct AccessKitError {
    /// Description of what went wrong
    pub desc: String,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,

    /// Optional additional context
    pub context: Option<String>,
}

impl AccessKitError {
    /// Create a new AccessKitError
    pub fn new(desc: impl Into<String>, line: u32, file: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            line,
            file: file.into(),
            context: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Egui GUI rendering errors
#[derive(Debug, Clone)]
pub struct EguiError {
    /// Description of what went wrong
    pub desc: String,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,

    /// Widget or component that caused the error
    pub component: Option<String>,
}

impl EguiError {
    /// Create a new EguiError
    pub fn new(desc: impl Into<String>, line: u32, file: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            line,
            file: file.into(),
            component: None,
        }
    }

    /// Specify the component that caused the error
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }
}

/// Backend initialization and runtime errors
#[derive(Debug, Clone)]
pub struct BackendError {
    /// Description of what went wrong
    pub desc: String,

    /// Backend type (eframe, miniquad, etc.)
    pub backend_type: String,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,

    /// Underlying error from the backend
    pub source_error: Option<String>,
}

impl BackendError {
    /// Create a new BackendError
    pub fn new(
        desc: impl Into<String>,
        backend_type: impl Into<String>,
        line: u32,
        file: impl Into<String>,
    ) -> Self {
        Self {
            desc: desc.into(),
            backend_type: backend_type.into(),
            line,
            file: file.into(),
            source_error: None,
        }
    }

    /// Add the source error message
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_error = Some(source.into());
        self
    }
}

/// File I/O errors for form processing
#[derive(Debug, Clone)]
pub struct IoError {
    /// Description of what went wrong
    pub desc: String,

    /// Path to the file that caused the error
    pub path: String,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,

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
        file: impl Into<String>,
    ) -> Self {
        Self {
            desc: desc.into(),
            path: path.into(),
            line,
            file: file.into(),
            operation,
        }
    }
}

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

/// Configuration errors
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Description of what went wrong
    pub desc: String,

    /// Configuration key that caused the error
    pub key: Option<String>,

    /// Expected value or format
    pub expected: Option<String>,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,
}

impl ConfigError {
    /// Create a new ConfigError
    pub fn new(desc: impl Into<String>, line: u32, file: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            key: None,
            expected: None,
            line,
            file: file.into(),
        }
    }

    /// Specify the configuration key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Specify what was expected
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }
}

/// Application-level errors
#[derive(Debug, Clone)]
pub struct AppError {
    /// Description of what went wrong
    pub desc: String,

    /// Application state when error occurred
    pub state: Option<String>,

    /// Line number where the error occurred
    pub line: u32,

    /// File where the error occurred
    pub file: String,

    /// Whether the error is recoverable
    pub recoverable: bool,
}

impl AppError {
    /// Create a new AppError
    pub fn new(desc: impl Into<String>, line: u32, file: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            state: None,
            line,
            file: file.into(),
            recoverable: true,
        }
    }

    /// Mark the error as unrecoverable
    pub fn unrecoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }

    /// Add application state information
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }
}

// ============================================================================
// Display Implementations
// ============================================================================

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FormErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormErrorKind::AccessKit(e) => write!(f, "AccessKit error: {}", e),
            FormErrorKind::Egui(e) => write!(f, "Egui error: {}", e),
            FormErrorKind::Backend(e) => write!(f, "Backend error: {}", e),
            FormErrorKind::Io(e) => write!(f, "I/O error: {}", e),
            FormErrorKind::Config(e) => write!(f, "Configuration error: {}", e),
            FormErrorKind::App(e) => write!(f, "Application error: {}", e),
        }
    }
}

impl fmt::Display for AccessKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}", self.desc, self.file, self.line)?;
        if let Some(ctx) = &self.context {
            write!(f, " (context: {})", ctx)?;
        }
        Ok(())
    }
}

impl fmt::Display for EguiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}", self.desc, self.file, self.line)?;
        if let Some(comp) = &self.component {
            write!(f, " (component: {})", comp)?;
        }
        Ok(())
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] at {}:{}",
            self.desc, self.backend_type, self.file, self.line
        )?;
        if let Some(src) = &self.source_error {
            write!(f, " (source: {})", src)?;
        }
        Ok(())
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to {} '{}': {} at {}:{}",
            self.operation, self.path, self.desc, self.file, self.line
        )
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}", self.desc, self.file, self.line)?;
        if let Some(key) = &self.key {
            write!(f, " (key: {})", key)?;
        }
        if let Some(exp) = &self.expected {
            write!(f, " (expected: {})", exp)?;
        }
        Ok(())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}:{}", self.desc, self.file, self.line)?;
        if let Some(state) = &self.state {
            write!(f, " (state: {})", state)?;
        }
        if !self.recoverable {
            write!(f, " [FATAL]")?;
        }
        Ok(())
    }
}

// ============================================================================
// Error Trait Implementations
// ============================================================================

impl std::error::Error for FormError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind() {
            FormErrorKind::AccessKit(e) => Some(e),
            FormErrorKind::Egui(e) => Some(e),
            FormErrorKind::Backend(e) => Some(e),
            FormErrorKind::Io(e) => Some(e),
            FormErrorKind::Config(e) => Some(e),
            FormErrorKind::App(e) => Some(e),
        }
    }
}

impl std::error::Error for AccessKitError {}
impl std::error::Error for EguiError {}
impl std::error::Error for BackendError {}
impl std::error::Error for IoError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for AppError {}

// ============================================================================
// Conversion Implementations (From trait)
// ============================================================================

impl From<AccessKitError> for FormError {
    fn from(err: AccessKitError) -> Self {
        FormError::new(FormErrorKind::AccessKit(err))
    }
}

impl From<EguiError> for FormError {
    fn from(err: EguiError) -> Self {
        FormError::new(FormErrorKind::Egui(err))
    }
}

impl From<BackendError> for FormError {
    fn from(err: BackendError) -> Self {
        FormError::new(FormErrorKind::Backend(err))
    }
}

impl From<IoError> for FormError {
    fn from(err: IoError) -> Self {
        FormError::new(FormErrorKind::Io(err))
    }
}

impl From<ConfigError> for FormError {
    fn from(err: ConfigError) -> Self {
        FormError::new(FormErrorKind::Config(err))
    }
}

impl From<AppError> for FormError {
    fn from(err: AppError) -> Self {
        FormError::new(FormErrorKind::App(err))
    }
}

// Convert from std::io::Error
impl From<std::io::Error> for FormError {
    fn from(err: std::io::Error) -> Self {
        IoError::new(
            err.to_string(),
            "<unknown>",
            IoOperation::Read,
            line!(),
            file!(),
        )
        .into()
    }
}

// ============================================================================
// Convenience Macros
// ============================================================================

/// Create an AccessKitError with automatic file/line information
#[macro_export]
macro_rules! accesskit_error {
    ($desc:expr) => {
        $crate::error::AccessKitError::new($desc, line!(), file!())
    };
    ($desc:expr, $ctx:expr) => {
        $crate::error::AccessKitError::new($desc, line!(), file!()).with_context($ctx)
    };
}

/// Create an EguiError with automatic file/line information
#[macro_export]
macro_rules! egui_error {
    ($desc:expr) => {
        $crate::error::EguiError::new($desc, line!(), file!())
    };
    ($desc:expr, $component:expr) => {
        $crate::error::EguiError::new($desc, line!(), file!()).with_component($component)
    };
}

/// Create a BackendError with automatic file/line information
#[macro_export]
macro_rules! backend_error {
    ($desc:expr, $backend:expr) => {
        $crate::error::BackendError::new($desc, $backend, line!(), file!())
    };
    ($desc:expr, $backend:expr, $source:expr) => {
        $crate::error::BackendError::new($desc, $backend, line!(), file!()).with_source($source)
    };
}

/// Create an IoError with automatic file/line information
#[macro_export]
macro_rules! io_error {
    ($desc:expr, $path:expr, $op:expr) => {
        $crate::error::IoError::new($desc, $path, $op, line!(), file!())
    };
}

/// Create a ConfigError with automatic file/line information
#[macro_export]
macro_rules! config_error {
    ($desc:expr) => {
        $crate::error::ConfigError::new($desc, line!(), file!())
    };
    ($desc:expr, key: $key:expr) => {
        $crate::error::ConfigError::new($desc, line!(), file!()).with_key($key)
    };
    ($desc:expr, key: $key:expr, expected: $expected:expr) => {
        $crate::error::ConfigError::new($desc, line!(), file!())
            .with_key($key)
            .with_expected($expected)
    };
}

/// Create an AppError with automatic file/line information
#[macro_export]
macro_rules! app_error {
    ($desc:expr) => {
        $crate::error::AppError::new($desc, line!(), file!())
    };
    ($desc:expr, state: $state:expr) => {
        $crate::error::AppError::new($desc, line!(), file!()).with_state($state)
    };
    ($desc:expr, fatal) => {
        $crate::error::AppError::new($desc, line!(), file!()).unrecoverable()
    };
}
