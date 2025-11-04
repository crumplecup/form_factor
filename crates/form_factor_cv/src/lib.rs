//! Computer vision capabilities for form_factor
//!
//! This crate provides text detection and logo detection using OpenCV.
//! Heavy dependencies (opencv) are isolated here.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "text-detection")]
mod text_detection;

#[cfg(feature = "logo-detection")]
mod logo_detection;

#[cfg(feature = "text-detection")]
pub use text_detection::{TextDetectionError, TextDetectionErrorKind, TextDetector, TextRegion};

#[cfg(feature = "logo-detection")]
pub use logo_detection::{Logo, LogoDetectionMethod, LogoDetectionResult, LogoDetector, LogoLocation, LogoSize};
