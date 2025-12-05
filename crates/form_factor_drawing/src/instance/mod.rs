//! Form instance implementation module
//!
//! This module provides concrete implementations of the FormInstance trait,
//! multi-page support via FormPage, and instance management.

mod error;
#[cfg(feature = "ocr")]
mod extraction;
mod implementation;
pub mod migration;

// Re-export public types
pub use error::{InstanceError, InstanceErrorKind};
pub use implementation::{DrawingInstance, FormPage};
pub use migration::{LEGACY_TEMPLATE_ID, ProjectFormat, migrate_canvas_to_instance};
