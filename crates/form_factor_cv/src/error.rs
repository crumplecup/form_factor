//! Crate-level error aggregation following CLAUDE.md Pattern 4

use derive_more::{Display, Error};

#[cfg(feature = "text-detection")]
use crate::TextDetectionError;

#[cfg(feature = "logo-detection")]
use crate::LogoDetectionError;

/// CV crate error kind aggregating all module errors
#[derive(Debug, Display, Error)]
pub enum CvErrorKind {
    /// Text detection error
    #[cfg(feature = "text-detection")]
    #[display("Text Detection: {}", _0)]
    TextDetection(TextDetectionError),

    /// Logo detection error
    #[cfg(feature = "logo-detection")]
    #[display("Logo Detection: {}", _0)]
    LogoDetection(LogoDetectionError),
}

#[cfg(feature = "text-detection")]
impl From<TextDetectionError> for CvErrorKind {
    fn from(err: TextDetectionError) -> Self {
        CvErrorKind::TextDetection(err)
    }
}

#[cfg(feature = "logo-detection")]
impl From<LogoDetectionError> for CvErrorKind {
    fn from(err: LogoDetectionError) -> Self {
        CvErrorKind::LogoDetection(err)
    }
}

/// CV crate umbrella error (Pattern 4 from CLAUDE.md)
#[derive(Debug, Display, Error)]
#[display("CV: {}", _0)]
pub struct CvError(Box<CvErrorKind>);

impl<T> From<T> for CvError
where
    T: Into<CvErrorKind>,
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

/// CV crate result type
pub type CvResult<T> = Result<T, CvError>;
