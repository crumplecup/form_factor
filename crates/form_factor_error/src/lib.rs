#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! Workspace-wide error types for form_factor
//!
//! This crate provides the umbrella error type that aggregates all errors
//! from individual workspace crates, following Pattern 4 from CLAUDE.md.

use derive_more::{Display, Error, From};

/// Workspace-wide error kind aggregating ONLY crate-level umbrellas
///
/// This enum automatically converts from any crate umbrella error type
/// via the `From` trait, enabling seamless error propagation with `?`.
///
/// **Important**: This only aggregates crate umbrellas (CoreError, DrawingError, etc.),
/// not individual module errors. Module errors must first convert to their crate umbrella.
#[derive(Debug, From, Display, Error)]
pub enum FormFactorErrorKind {
    // Crate-level umbrellas only
    /// Core domain errors (I/O, templates, instances)
    #[display("Core: {}", _0)]
    Core(form_factor_core::CoreError),

    /// Drawing errors (canvas, shapes, layers, templates, instances)
    #[display("Drawing: {}", _0)]
    Drawing(form_factor_drawing::FormError),

    // TODO: Add when crate umbrellas are created:
    // Cv(form_factor_cv::CvError),
    // Ocr(form_factor_ocr::OcrError),
}

/// Workspace-wide umbrella error
///
/// This is the top-level error type for operations that span multiple crates.
/// It automatically converts from any crate-specific error via `From<T>`.
///
/// # Examples
///
/// ```ignore
/// use form_factor_error::FormFactorResult;
///
/// fn process_template() -> FormFactorResult<()> {
///     // Automatically converts from any crate error
///     let template = load_template()?;
///     draw_template(template)?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Display, Error)]
#[display("FormFactor: {}", _0)]
pub struct FormFactorError(Box<FormFactorErrorKind>);

impl<T> From<T> for FormFactorError
where
    T: Into<FormFactorErrorKind>,
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

/// Workspace-wide result type
///
/// Convenience alias for `Result<T, FormFactorError>`.
pub type FormFactorResult<T> = Result<T, FormFactorError>;
