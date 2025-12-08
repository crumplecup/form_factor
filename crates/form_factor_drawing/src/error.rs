//! Crate-level error aggregation following CLAUDE.md Pattern 4

use crate::{CanvasError, InstanceError, LayerError, ShapeError, TemplateError};
use derive_more::{Display, Error, From};

/// Drawing crate error kind aggregating all module errors
///
/// This enum wraps specific error types from different modules, allowing
/// functions that may encounter multiple error types to return a single
/// unified error type.
#[derive(Debug, Display, Error, From)]
pub enum FormErrorKind {
    /// Canvas operation error
    #[display("Canvas: {}", _0)]
    Canvas(CanvasError),
    
    /// Shape operation error
    #[display("Shape: {}", _0)]
    Shape(ShapeError),
    
    /// Layer operation error
    #[display("Layer: {}", _0)]
    Layer(LayerError),
    
    /// Template operation error
    #[display("Template: {}", _0)]
    Template(TemplateError),
    
    /// Instance operation error
    #[display("Instance: {}", _0)]
    Instance(InstanceError),
}

/// Drawing crate umbrella error (Pattern 4 from CLAUDE.md)
#[derive(Debug, Display, Error)]
#[display("Drawing: {}", _0)]
pub struct FormError(Box<FormErrorKind>);

impl<T> From<T> for FormError
where
    T: Into<FormErrorKind>,
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

/// Drawing crate result type
pub type DrawingResult<T> = Result<T, FormError>;
