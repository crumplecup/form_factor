//! Core traits and types for the form_factor framework
//!
//! This crate defines the foundational traits (App, Backend, AppContext)
//! that other crates build upon. It has minimal dependencies.
//!
//! # Form Templates and Instances
//!
//! The `template` and `instance` modules provide core abstractions for
//! working with structured forms:
//!
//! - **Templates**: Define form structure, field types, and validation rules
//! - **Instances**: Represent filled forms with actual data and detection results
//!
//! See the respective module documentation for details.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod app;
mod backend;
mod error;
pub mod instance;
pub mod template;
mod template_browser;
mod template_browser_error;

pub use app::{App, AppContext};
pub use backend::{Backend, BackendConfig};
pub use error::{IoError, IoOperation};

// Re-export template types at crate root
pub use template::{
    FieldBounds, FieldDefinition, FieldDefinitionBuilder, FieldType, FormTemplate, PageNavigation,
};

// Re-export instance types at crate root
pub use instance::{
    FieldContent, FieldValidationError, FieldValue, FormInstance, ValidationErrorType,
    ValidationResult,
};

// Re-export template browser types at crate root
pub use template_browser::{SortOrder, TemplateBrowser, TemplateEntry, TemplateMetadata};
pub use template_browser_error::{
    TemplateBrowserError, TemplateBrowserErrorKind, TemplateBrowserResult,
};
