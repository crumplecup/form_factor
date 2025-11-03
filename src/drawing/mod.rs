//! Drawing tools module for form annotation
//!
//! Provides a canvas with rectangle, circle, and freehand (polygon) drawing capabilities.

mod canvas;
mod layer;
mod recent_projects;
mod shape;
mod tool;

pub use canvas::DrawingCanvas;
pub use layer::{Layer, LayerManager, LayerType};
pub use recent_projects::RecentProjects;
pub use shape::{Circle, CircleBuilder, PolygonShape, Rectangle, Shape, ShapeError, ShapeErrorKind};
pub use tool::ToolMode;
