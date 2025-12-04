//! Canvas plugin for drawing tools and canvas manipulation.
//!
//! This plugin provides UI for:
//! - Tool selection (Select, Rectangle, Circle, Freehand, Edit, Rotate)
//! - Canvas pan and zoom controls
//! - Drawing state display

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use form_factor_drawing::ToolMode;
use strum::IntoEnumIterator;
use tracing::{debug, instrument};

/// Plugin for canvas drawing tools and manipulation.
///
/// Provides a toolbar with:
/// - Tool mode selection buttons
/// - Zoom level display and controls
/// - Pan offset display
pub struct CanvasPlugin {
    /// Current selected tool mode
    current_tool: ToolMode,
    /// Current zoom level (1.0 = 100%)
    zoom: f32,
    /// Pan offset X
    pan_x: f32,
    /// Pan offset Y
    pan_y: f32,
}

impl CanvasPlugin {
    /// Creates a new canvas plugin with default settings.
    pub fn new() -> Self {
        Self {
            current_tool: ToolMode::default(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    /// Renders the tool selection buttons.
    fn render_tool_buttons(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.horizontal(|ui| {
            ui.label("Tools:");

            for tool in ToolMode::iter() {
                let label = format!("{:?}", tool);
                let selected = self.current_tool == tool;

                if ui.selectable_label(selected, label).clicked() {
                    debug!(?tool, "Tool selected");
                    self.current_tool = tool;
                    ctx.events.emit(AppEvent::ToolSelected {
                        tool_name: format!("{:?}", tool),
                    });
                }
            }
        });
    }

    /// Renders zoom controls.
    fn render_zoom_controls(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.horizontal(|ui| {
            ui.label("Zoom:");

            if ui.button("-").clicked() {
                self.zoom *= 0.8;
                ctx.events
                    .emit(AppEvent::CanvasZoomChanged { zoom: self.zoom });
            }

            ui.label(format!("{:.0}%", self.zoom * 100.0));

            if ui.button("+").clicked() {
                self.zoom *= 1.25;
                ctx.events
                    .emit(AppEvent::CanvasZoomChanged { zoom: self.zoom });
            }

            if ui.button("Reset").clicked() {
                self.zoom = 1.0;
                ctx.events
                    .emit(AppEvent::CanvasZoomChanged { zoom: self.zoom });
            }
        });
    }

    /// Renders pan offset display.
    fn render_pan_display(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Pan:");
            ui.label(format!("X: {:.1}, Y: {:.1}", self.pan_x, self.pan_y));
        });
    }
}

impl Default for CanvasPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CanvasPlugin {
    fn name(&self) -> &str {
        "canvas"
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("Canvas Tools");
            self.render_tool_buttons(ui, ctx);
            self.render_zoom_controls(ui, ctx);
            self.render_pan_display(ui);
        });
    }

    #[instrument(skip(self, _ctx), fields(plugin = "canvas"))]
    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::CanvasZoomChanged { zoom } => {
                debug!(new_zoom = zoom, old_zoom = self.zoom, "Zoom changed");
                self.zoom = *zoom;
                None
            }
            AppEvent::CanvasPanChanged { x, y } => {
                debug!(x, y, "Pan changed");
                self.pan_x = *x;
                self.pan_y = *y;
                None
            }
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "Canvas drawing tools and manipulation controls"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_plugin_creation() {
        let plugin = CanvasPlugin::new();
        assert_eq!(plugin.name(), "canvas");
        assert_eq!(plugin.zoom, 1.0);
        assert_eq!(plugin.pan_x, 0.0);
        assert_eq!(plugin.pan_y, 0.0);
    }

    #[test]
    fn test_zoom_event_handling() {
        let mut plugin = CanvasPlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let event = AppEvent::CanvasZoomChanged { zoom: 2.0 };
        plugin.on_event(&event, &ctx);

        assert_eq!(plugin.zoom, 2.0);
    }

    #[test]
    fn test_pan_event_handling() {
        let mut plugin = CanvasPlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let event = AppEvent::CanvasPanChanged { x: 10.0, y: 20.0 };
        plugin.on_event(&event, &ctx);

        assert_eq!(plugin.pan_x, 10.0);
        assert_eq!(plugin.pan_y, 20.0);
    }
}
