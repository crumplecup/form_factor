//! Form instance implementation module
//!
//! This module provides concrete implementations of the FormInstance trait,
//! multi-page support via FormPage, and instance management.

mod error;
mod implementation;

// Re-export public types
pub use error::{InstanceError, InstanceErrorKind};
pub use implementation::{DrawingInstance, FormPage};
