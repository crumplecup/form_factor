//! Form Factor - Accessible GUI framework for tagging scanned forms
//!
//! This library provides a backend-agnostic architecture for building
//! accessible GUI applications using egui, with support for multiple
//! rendering backends (eframe, wgpu, etc.)
//!
//! # Quick Start
//!
//! ```no_run
//! use form_factor::{App, AppContext, Backend, BackendConfig, EframeBackend};
//!
//! struct MyApp;
//!
//! impl App for MyApp {
//!     fn update(&mut self, ctx: &AppContext) {
//!         egui::CentralPanel::default().show(ctx.egui_ctx, |ui| {
//!             ui.heading("Hello Form Factor!");
//!         });
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let app = Box::new(MyApp);
//!     let config = BackendConfig::default();
//!     EframeBackend::run(app, config)?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

// Module declarations (private - users import from root)
mod app;
mod backend;
mod drawing;
mod error;

#[cfg(feature = "text-detection")]
mod text_detection;

#[cfg(feature = "logo-detection")]
mod logo_detection;

#[cfg(feature = "ocr")]
mod ocr;

// ============================================================================
// Core Application Types
// ============================================================================

/// Core application trait - implement this to define your GUI logic
pub use app::App;

/// Context provided to your app each frame (egui context, timing, etc.)
pub use app::AppContext;

// ============================================================================
// Backend System
// ============================================================================

/// Trait for backend implementations (eframe, miniquad, etc.)
pub use backend::Backend;

/// Configuration for backend initialization (window size, vsync, etc.)
pub use backend::BackendConfig;

// Backend implementations (conditional compilation)
#[cfg(feature = "backend-eframe")]
pub use backend::eframe_backend::{EframeBackend, EframeError};

// Note: MiniquadBackend is not yet available - waiting for egui 0.33 support
// #[cfg(feature = "backend-miniquad")]
// pub use backend::miniquad_backend::{MiniquadBackend, MiniquadError};

// ============================================================================
// Error Types
// ============================================================================

/// Top-level error type (wraps Box<FormErrorKind>)
pub use error::FormError;

/// Error category enum
pub use error::FormErrorKind;

/// Specific error types for each category
pub use error::{
    AccessKitError, AppError, BackendError, ConfigError, EguiError, IoError, IoOperation,
};

// ============================================================================
// Drawing Tools
// ============================================================================

/// Drawing canvas for form annotations
pub use drawing::{CanvasError, CanvasErrorKind, DrawingCanvas};

/// Shape types (rectangles, circles, polygons)
pub use drawing::{Circle, CircleBuilder, PolygonShape, Rectangle, Shape, ShapeError, ShapeErrorKind};

/// Drawing tool mode (rectangle, circle, freehand, select)
pub use drawing::ToolMode;

/// Layer management types
pub use drawing::{Layer, LayerError, LayerManager, LayerType};

/// Recent projects tracking
pub use drawing::RecentProjects;

// ============================================================================
// Text Detection
// ============================================================================

#[cfg(feature = "text-detection")]
/// Text detector using OpenCV DB model
pub use text_detection::TextDetector;

#[cfg(feature = "text-detection")]
/// Detected text region
pub use text_detection::TextRegion;

#[cfg(feature = "text-detection")]
/// Text detection error
pub use text_detection::TextDetectionError;

#[cfg(feature = "text-detection")]
/// Text detection error kind
pub use text_detection::TextDetectionErrorKind;

// ============================================================================
// Logo Detection
// ============================================================================

#[cfg(feature = "logo-detection")]
/// Logo detector using OpenCV template and feature matching
pub use logo_detection::LogoDetector;

#[cfg(feature = "logo-detection")]
/// Logo detection method (template matching or feature matching)
pub use logo_detection::LogoDetectionMethod;

#[cfg(feature = "logo-detection")]
/// Logo template for detection
pub use logo_detection::Logo;

#[cfg(feature = "logo-detection")]
/// Logo detection result
pub use logo_detection::LogoDetectionResult;

#[cfg(feature = "logo-detection")]
/// Logo location in image
pub use logo_detection::LogoLocation;

#[cfg(feature = "logo-detection")]
/// Logo size
pub use logo_detection::LogoSize;

// ============================================================================
// OCR (Optical Character Recognition)
// ============================================================================

#[cfg(feature = "ocr")]
/// OCR engine for text extraction using Tesseract
pub use ocr::OCREngine;

#[cfg(feature = "ocr")]
/// OCR configuration options
pub use ocr::OCRConfig;

#[cfg(feature = "ocr")]
/// Page segmentation mode for OCR
pub use ocr::PageSegmentationMode;

#[cfg(feature = "ocr")]
/// OCR engine mode (LSTM, Legacy, or both)
pub use ocr::EngineMode;

#[cfg(feature = "ocr")]
/// Result of OCR text extraction
pub use ocr::OCRResult;

#[cfg(feature = "ocr")]
/// Word-level OCR result with bounding box
pub use ocr::WordResult;

#[cfg(feature = "ocr")]
/// Bounding box for text regions
pub use ocr::BoundingBox;

#[cfg(feature = "ocr")]
/// OCR error
pub use ocr::OCRError;

#[cfg(feature = "ocr")]
/// OCR error kind
pub use ocr::OCRErrorKind;

// ============================================================================
// Advanced: Direct module access for backend implementations
// ============================================================================

/// Backend implementations and utilities
///
/// Most users should use the re-exported types at the crate root.
/// This module is provided for advanced use cases.
pub mod backends {
    #[cfg(feature = "backend-eframe")]
    pub use crate::backend::eframe_backend;

    pub use crate::backend::miniquad_backend;
}
