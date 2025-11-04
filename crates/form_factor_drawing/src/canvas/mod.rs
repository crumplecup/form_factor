//! Drawing canvas with interactive annotation tools
//!
//! This module is organized into submodules:
//! - `core`: Core canvas state, error types, and initialization
//! - `io`: File I/O, serialization, and image loading
//! - `tools`: Tool interaction and state management
//! - `rendering`: UI rendering and painting logic

mod core;
mod io;
mod rendering;
mod tools;

// Re-export public types
pub use core::{CanvasError, CanvasErrorKind, DetectionSubtype, DrawingCanvas};
