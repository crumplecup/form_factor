//! Crate-level error types
//!
//! This module provides a unified error type that wraps all specific error types
//! from different modules in the crate.

use crate::{CanvasError, InstanceError, LayerError, ShapeError, TemplateError};
use derive_more::{Display, From};

/// Unified error kind for all form_factor_drawing operations
///
/// This enum wraps specific error types from different modules, allowing
/// functions that may encounter multiple error types to return a single
/// unified error type.
#[derive(Debug, From)]
pub enum FormErrorKind {
    /// Canvas operation error
    Canvas(CanvasError),
    /// Shape operation error
    Shape(ShapeError),
    /// Layer operation error
    Layer(LayerError),
    /// Template operation error
    Template(TemplateError),
    /// Instance operation error
    Instance(InstanceError),
}

impl std::fmt::Display for FormErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormErrorKind::Canvas(e) => write!(f, "{}", e),
            FormErrorKind::Shape(e) => write!(f, "{}", e),
            FormErrorKind::Layer(e) => write!(f, "{}", e),
            FormErrorKind::Template(e) => write!(f, "{}", e),
            FormErrorKind::Instance(e) => write!(f, "{}", e),
        }
    }
}

/// Crate-level error wrapper
#[derive(Debug, Display)]
#[display("Form Error: {}", kind)]
pub struct FormError {
    /// The specific error that occurred
    pub kind: FormErrorKind,
}

impl FormError {
    /// Create a new form error from a kind
    pub fn new(kind: FormErrorKind) -> Self {
        Self { kind }
    }
}

impl std::error::Error for FormError {}

impl From<CanvasError> for FormError {
    fn from(err: CanvasError) -> Self {
        FormError::new(FormErrorKind::Canvas(err))
    }
}

impl From<ShapeError> for FormError {
    fn from(err: ShapeError) -> Self {
        FormError::new(FormErrorKind::Shape(err))
    }
}

impl From<LayerError> for FormError {
    fn from(err: LayerError) -> Self {
        FormError::new(FormErrorKind::Layer(err))
    }
}

impl From<TemplateError> for FormError {
    fn from(err: TemplateError) -> Self {
        FormError::new(FormErrorKind::Template(err))
    }
}

impl From<InstanceError> for FormError {
    fn from(err: InstanceError) -> Self {
        FormError::new(FormErrorKind::Instance(err))
    }
}
