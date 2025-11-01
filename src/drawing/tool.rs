//! Drawing tool modes

use serde::{Deserialize, Serialize};

/// The active drawing tool mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolMode {
    /// Select and manipulate existing shapes (not yet implemented)
    #[default]
    Select,
    /// Draw rectangles
    Rectangle,
    /// Draw circles
    Circle,
    /// Draw freehand strokes
    Freehand,
}

impl ToolMode {
    /// Get the display name for this tool
    pub fn name(&self) -> &str {
        match self {
            ToolMode::Select => "Select",
            ToolMode::Rectangle => "Rectangle",
            ToolMode::Circle => "Circle",
            ToolMode::Freehand => "Freehand",
        }
    }
}
