//! Drawing tools module for form annotation
//!
//! Provides a canvas with rectangle, circle, and freehand (polygon) drawing capabilities.

mod canvas;
mod shape;
mod tool;

pub use canvas::DrawingCanvas;
pub use shape::{Circle, PolygonShape, Rectangle, Shape};
pub use tool::ToolMode;
