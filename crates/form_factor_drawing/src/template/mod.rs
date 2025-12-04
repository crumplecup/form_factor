//! Form template implementation module
//!
//! This module provides concrete implementations of the FormTemplate trait,
//! template storage/registry, and builder patterns for creating templates.

mod error;
mod implementation;
mod registry;

// Re-export public types
pub use error::{TemplateError, TemplateErrorKind};
pub use implementation::{
    DrawingTemplate, DrawingTemplateBuilder, TemplatePage, TemplatePageBuilder,
};
pub use registry::TemplateRegistry;
