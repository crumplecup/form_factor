//! Canvas event handlers

use crate::type_conversions::ToolParser;
use form_factor::DrawingCanvas;
use tracing::instrument;

/// Canvas event handler
pub struct CanvasEventHandler;

impl CanvasEventHandler {
    /// Handle zoom changed
    #[instrument(skip(canvas), fields(zoom))]
    pub fn handle_zoom_changed(canvas: &mut DrawingCanvas, zoom: f32) {
        tracing::debug!(zoom, "Canvas zoom changed");
        canvas.set_zoom(zoom);
    }

    /// Handle pan changed
    #[instrument(skip(canvas), fields(x, y))]
    pub fn handle_pan_changed(canvas: &mut DrawingCanvas, x: f32, y: f32) {
        tracing::debug!(x, y, "Canvas pan changed");
        canvas.set_pan_offset(x, y);
    }

    /// Handle tool selected
    #[instrument(skip(canvas), fields(tool_name))]
    pub fn handle_tool_selected(canvas: &mut DrawingCanvas, tool_name: &str) {
        tracing::debug!(tool_name, "Tool selected");

        if let Some(tool) = ToolParser::from_name(tool_name) {
            canvas.set_tool(tool);
        }
    }

    /// Handle canvas image visibility changed
    #[instrument(skip(canvas), fields(visible))]
    pub fn handle_image_visibility_changed(canvas: &mut DrawingCanvas, visible: bool) {
        tracing::debug!(visible, "Canvas image visibility changed");
        canvas.with_form_image_visible(visible);
    }

    /// Handle canvas image lock changed
    #[instrument(skip(canvas), fields(locked))]
    pub fn handle_image_lock_changed(canvas: &mut DrawingCanvas, locked: bool) {
        tracing::debug!(locked, "Canvas image lock state changed");
        canvas.with_form_image_locked(locked);
    }

    /// Handle canvas image clear requested
    #[instrument(skip(canvas))]
    pub fn handle_image_clear_requested(canvas: &mut DrawingCanvas) {
        tracing::info!("Canvas image cleared");
        canvas.with_form_image_path(None);
    }
}
