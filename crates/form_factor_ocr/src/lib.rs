//! OCR (Optical Character Recognition) for form_factor
//!
//! This crate provides OCR functionality using Tesseract via leptess.
//! Heavy dependencies (leptess) are isolated here.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod ocr;

pub use ocr::{
    BoundingBox, EngineMode, OCRConfig, OCREngine, OCRError, OCRErrorKind, OCRResult,
    PageSegmentationMode, WordResult,
};
