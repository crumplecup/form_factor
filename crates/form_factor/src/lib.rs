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
//!         egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
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

// UI helper modules (from main binary refactoring)
mod detection_results;
mod detection_tasks;
mod event_handlers;
mod file_dialogs;
mod overlays;
#[cfg(feature = "plugins")]
mod plugin_setup;
mod type_conversions;
mod ui_properties;

// ============================================================================
// Core Application Types
// ============================================================================

/// Core application trait - implement this to define your GUI logic
pub use form_factor_core::App;

/// Context provided to your app each frame (egui context, timing, etc.)
pub use form_factor_core::AppContext;

// ============================================================================
// Backend System
// ============================================================================

/// Trait for backend implementations (eframe, miniquad, etc.)
pub use form_factor_core::Backend;

/// Configuration for backend initialization (window size, vsync, etc.)
pub use form_factor_core::BackendConfig;

// Backend implementations (conditional compilation)
#[cfg(feature = "backend-eframe")]
pub use form_factor_backends::{EframeBackend, EframeError};

// ============================================================================
// Error Handling
// ============================================================================

/// Workspace-wide umbrella error aggregating all crate errors
pub use form_factor_error::{FormFactorError, FormFactorErrorKind, FormFactorResult};

/// Drawing crate error (temporary - will be renamed to DrawingError in future refactor)
pub use form_factor_drawing::FormError;

/// I/O error types (re-exported from core)
pub use form_factor_core::{IoError, IoOperation};

// ============================================================================
// Drawing Tools
// ============================================================================

/// Application mode management
pub use form_factor_drawing::{AppMode, AppState};

/// Mode switcher UI component
pub use form_factor_drawing::ModeSwitcher;

/// Drawing canvas for form annotations
pub use form_factor_drawing::{CanvasError, CanvasErrorKind, DetectionSubtype, DrawingCanvas};

/// Shape types (rectangles, circles, polygons)
pub use form_factor_drawing::{
    Circle, CircleBuilder, PolygonShape, Rectangle, Shape, ShapeError, ShapeErrorKind,
};

/// Drawing tool mode (rectangle, circle, freehand, select)
pub use form_factor_drawing::ToolMode;

/// Layer management types
pub use form_factor_drawing::{Layer, LayerError, LayerManager, LayerType};

/// Recent projects tracking
pub use form_factor_drawing::RecentProjects;

/// Instance data entry panel and actions
pub use form_factor_drawing::{DataEntryAction, DataEntryPanel};

/// Instance management panel and actions
pub use form_factor_drawing::{InstanceManagerAction, InstanceManagerPanel};

/// Template and instance types
pub use form_factor_drawing::{DrawingInstance, DrawingTemplate};

// ============================================================================
// Text Detection
// ============================================================================

#[cfg(feature = "text-detection")]
/// Text detector using OpenCV DB model
pub use form_factor_cv::TextDetector;

#[cfg(feature = "text-detection")]
/// Detected text region
pub use form_factor_cv::TextRegion;

#[cfg(feature = "text-detection")]
/// Text detection error
pub use form_factor_cv::TextDetectionError;

#[cfg(feature = "text-detection")]
/// Text detection error kind
pub use form_factor_cv::TextDetectionErrorKind;

// ============================================================================
// Logo Detection
// ============================================================================

#[cfg(feature = "logo-detection")]
/// Logo detector using OpenCV template and feature matching
pub use form_factor_cv::LogoDetector;

#[cfg(feature = "logo-detection")]
/// Logo detection method (template matching or feature matching)
pub use form_factor_cv::LogoDetectionMethod;

#[cfg(feature = "logo-detection")]
/// Logo template for detection
pub use form_factor_cv::Logo;

#[cfg(feature = "logo-detection")]
/// Logo detection result
pub use form_factor_cv::LogoDetectionResult;

#[cfg(feature = "logo-detection")]
/// Logo location in image
pub use form_factor_cv::LogoLocation;

#[cfg(feature = "logo-detection")]
/// Logo size
pub use form_factor_cv::LogoSize;

// ============================================================================
// OCR (Optical Character Recognition)
// ============================================================================

#[cfg(feature = "ocr")]
/// OCR engine for text extraction using Tesseract
pub use form_factor_ocr::OCREngine;

#[cfg(feature = "ocr")]
/// OCR configuration options
pub use form_factor_ocr::OCRConfig;

#[cfg(feature = "ocr")]
/// Page segmentation mode for OCR
pub use form_factor_ocr::PageSegmentationMode;

#[cfg(feature = "ocr")]
/// OCR engine mode (LSTM, Legacy, or both)
pub use form_factor_ocr::EngineMode;

#[cfg(feature = "ocr")]
/// Result of OCR text extraction
pub use form_factor_ocr::OCRResult;

#[cfg(feature = "ocr")]
/// Word-level OCR result with bounding box
pub use form_factor_ocr::WordResult;

#[cfg(feature = "ocr")]
/// Bounding box for text regions
pub use form_factor_ocr::BoundingBox;

#[cfg(feature = "ocr")]
/// OCR error
pub use form_factor_ocr::OCRError;

#[cfg(feature = "ocr")]
/// OCR error kind
pub use form_factor_ocr::OCRErrorKind;

// ============================================================================
// Plugin System
// ============================================================================

#[cfg(feature = "plugins")]
/// Plugin trait for implementing modular UI components
pub use form_factor_plugins::Plugin;

#[cfg(feature = "plugins")]
/// Plugin manager for coordinating plugins
pub use form_factor_plugins::PluginManager;

#[cfg(feature = "plugins")]
/// Context provided to plugins during rendering and event handling
pub use form_factor_plugins::PluginContext;

#[cfg(feature = "plugins")]
/// Event bus for plugin communication
pub use form_factor_plugins::EventBus;

#[cfg(feature = "plugins")]
/// Event sender for publishing events
pub use form_factor_plugins::EventSender;

#[cfg(feature = "plugins")]
/// Application event types for inter-plugin communication
pub use form_factor_plugins::AppEvent;

#[cfg(feature = "plugins")]
/// Plugin builder trait
pub use form_factor_plugins::PluginBuilder;

// ============================================================================
// UI Helpers (from main binary refactoring)
// ============================================================================

/// Property rendering for shapes and detections
pub use ui_properties::PropertyRenderer;

/// File dialog utilities
pub use file_dialogs::FileDialogs;

/// Plugin setup and registration
#[cfg(feature = "plugins")]
pub use plugin_setup::PluginSetup;

/// Type conversion utilities
pub use type_conversions::{LayerParser, ToolParser};

/// Detection task spawning
#[cfg(all(feature = "plugins", feature = "text-detection"))]
pub use detection_tasks::TextDetectionTask;

#[cfg(all(feature = "plugins", feature = "logo-detection"))]
pub use detection_tasks::LogoDetectionTask;

#[cfg(all(feature = "plugins", feature = "ocr"))]
pub use detection_tasks::OcrExtractionTask;

/// Detection result processing
pub use detection_results::DetectionResultHandler;

/// Event handlers
pub use event_handlers::{
    CanvasEventHandler, DetectionEventHandler, FileEventHandler, LayerEventHandler,
    ObjectEventHandler, SelectionEventHandler, TemplateEventHandler,
};

// ============================================================================
// Overlay System
// ============================================================================

/// Overlay system exports
pub use overlays::{Overlay, OverlayManager, OverlayResponse, TemplateBrowserOverlay};

// ============================================================================
// Advanced: Direct module access for backend implementations
// ============================================================================

/// Backend implementations and utilities
///
/// Most users should use the re-exported types at the crate root.
/// This module is provided for advanced use cases.
pub mod backends {
    #[cfg(feature = "backend-eframe")]
    pub use form_factor_backends::eframe_backend;
}
