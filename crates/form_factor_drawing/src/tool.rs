//! Drawing tool modes for canvas interaction
//!
//! Tools are displayed in UI menus in enum discriminant order.

use serde::{Deserialize, Serialize};

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
    derive_more::Display,
)]
pub enum ToolMode {
    /// Select and manipulate existing shapes
    ///
    /// Default tool mode. Allows clicking to select shapes, dragging to move them,
    /// and accessing shape properties.
    #[default]
    #[display("Select")]
    Select,

    /// Draw rectangles
    ///
    /// Click and drag to create rectangular shapes.
    #[display("Rectangle")]
    Rectangle,

    /// Draw circles
    ///
    /// Click and drag to create circular shapes. The first click sets the center,
    /// dragging sets the radius.
    #[display("Circle")]
    Circle,

    /// Draw freehand closed polygons
    ///
    /// Click to add vertices, double-click or close to finish the polygon.
    #[display("Freehand")]
    Freehand,

    /// Edit shape vertices
    ///
    /// Select a shape and drag individual vertices to modify its geometry.
    /// Different from Select mode which moves the entire shape.
    #[display("Edit")]
    Edit,

    /// Rotate shapes
    ///
    /// Select a shape and drag to rotate it around its center or a pivot point.
    #[display("Rotate")]
    Rotate,
}
