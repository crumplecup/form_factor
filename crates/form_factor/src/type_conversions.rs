//! Type-safe conversions from string names

use form_factor::{LayerType, ToolMode};
use tracing::instrument;

/// Tool name parser
pub struct ToolParser;

impl ToolParser {
    /// Parse tool name to ToolMode
    #[instrument(fields(name))]
    pub fn from_name(name: &str) -> Option<ToolMode> {
        let result = match name {
            "Select" => Some(ToolMode::Select),
            "Rectangle" => Some(ToolMode::Rectangle),
            "Circle" => Some(ToolMode::Circle),
            "Freehand" => Some(ToolMode::Freehand),
            "Edit" => Some(ToolMode::Edit),
            "Rotate" => Some(ToolMode::Rotate),
            _ => None,
        };

        if result.is_none() {
            tracing::warn!(name, "Unknown tool name");
        } else {
            tracing::debug!(name, tool = ?result, "Tool parsed");
        }

        result
    }
}

/// Layer name parser
pub struct LayerParser;

impl LayerParser {
    /// Parse layer name to LayerType
    #[instrument(fields(name))]
    pub fn from_name(name: &str) -> Option<LayerType> {
        let result = match name {
            "Canvas" => Some(LayerType::Canvas),
            "Detections" => Some(LayerType::Detections),
            "Shapes" => Some(LayerType::Shapes),
            "Grid" => Some(LayerType::Grid),
            "Template" => Some(LayerType::Template),
            "Instance" => Some(LayerType::Instance),
            _ => None,
        };

        if result.is_none() {
            tracing::warn!(name, "Unknown layer name");
        } else {
            tracing::debug!(name, layer = ?result, "Layer parsed");
        }

        result
    }
}
