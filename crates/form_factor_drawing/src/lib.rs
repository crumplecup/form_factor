//! Drawing canvas with interactive annotation tools
//!
//! This crate provides the DrawingCanvas, shapes, layers, and tool management.
//! It depends on form_factor_core for the core traits.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod canvas;
mod layer;
mod recent_projects;
mod shape;
mod tool;

pub use canvas::{CanvasError, CanvasErrorKind, DetectionSubtype, DrawingCanvas};
pub use layer::{Layer, LayerError, LayerManager, LayerType};
pub use recent_projects::RecentProjects;
pub use shape::{Circle, CircleBuilder, PolygonShape, Rectangle, Shape, ShapeError, ShapeErrorKind};
pub use tool::ToolMode;
