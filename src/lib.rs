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

// Module declarations (private - users import from root)
mod app;
mod backend;
mod drawing;
mod error;

#[cfg(feature = "text-detection")]
mod text_detection;

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
pub use drawing::DrawingCanvas;

/// Shape types (rectangles, circles, polygons)
pub use drawing::{Circle, PolygonShape, Rectangle, Shape};

/// Drawing tool mode (rectangle, circle, freehand, select)
pub use drawing::ToolMode;

/// Layer management types
pub use drawing::{Layer, LayerManager, LayerType};

/// Recent projects tracking
pub use drawing::RecentProjects;

// ============================================================================
// Text Detection
// ============================================================================

#[cfg(feature = "text-detection")]
/// Text detector using OpenCV EAST model
pub use text_detection::TextDetector;

#[cfg(feature = "text-detection")]
/// Detected text region
pub use text_detection::TextRegion;

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
