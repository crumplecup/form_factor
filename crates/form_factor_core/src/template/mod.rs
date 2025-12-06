//! Form template definitions and traits
//!
//! This module provides the core abstractions for defining form templates.
//! Templates describe the structure and expected fields of a form type,
//! independent of any specific instance or filled data.

mod page_navigation;
mod types;

pub use page_navigation::PageNavigation;
pub use types::{FieldBounds, FieldDefinition, FieldDefinitionBuilder, FieldType, FormTemplate};
