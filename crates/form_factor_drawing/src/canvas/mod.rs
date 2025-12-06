//! Drawing canvas with interactive annotation tools
//!
//! This module is organized into submodules:
//! - `core`: Core canvas state, error types, and initialization
//! - `io`: File I/O, serialization, and image loading
//! - `tools`: Tool interaction and state management
//! - `rendering`: UI rendering and painting logic
//! - `field_creator`: Convert shapes to template fields

mod core;
mod field_creator;
mod io;
mod rendering;
mod tools;

// Re-export public types
pub use core::{CanvasError, CanvasErrorKind, CanvasState, DetectionSubtype, DrawingCanvas};
