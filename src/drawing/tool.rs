//! Drawing tool modes for canvas interaction
//!
//! Tools are displayed in UI menus in enum discriminant order.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The active drawing tool mode
///
/// Determines how the user interacts with the canvas. Tools are ordered
/// for UI display from most common (Select) to specialized (Rotate).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Default,
    strum::EnumIter,
)]
pub enum ToolMode {
    /// Select and manipulate existing shapes
    ///
    /// Default tool mode. Allows clicking to select shapes, dragging to move them,
    /// and accessing shape properties.
    #[default]
    Select,

    /// Draw rectangles
    ///
    /// Click and drag to create rectangular shapes.
    Rectangle,

    /// Draw circles
    ///
    /// Click and drag to create circular shapes. The first click sets the center,
    /// dragging sets the radius.
    Circle,

    /// Draw freehand closed polygons
    ///
    /// Click to add vertices, double-click or close to finish the polygon.
    Freehand,

    /// Edit shape vertices
    ///
    /// Select a shape and drag individual vertices to modify its geometry.
    /// Different from Select mode which moves the entire shape.
    Edit,

    /// Rotate shapes
    ///
    /// Select a shape and drag to rotate it around its center or a pivot point.
    Rotate,
}

impl fmt::Display for ToolMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolMode::Select => write!(f, "Select"),
            ToolMode::Rectangle => write!(f, "Rectangle"),
            ToolMode::Circle => write!(f, "Circle"),
            ToolMode::Freehand => write!(f, "Freehand"),
            ToolMode::Edit => write!(f, "Edit"),
            ToolMode::Rotate => write!(f, "Rotate"),
        }
    }
}
