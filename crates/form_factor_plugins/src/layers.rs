//! Layers plugin for layer management UI.
//!
//! This plugin provides UI for:
//! - Layer visibility toggles
//! - Layer lock status
//! - Layer selection
//! - Layer z-order display

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use form_factor_drawing::LayerType;
use strum::IntoEnumIterator;
use tracing::{debug, instrument};

/// Information about a single layer.
#[derive(Debug, Clone)]
struct LayerInfo {
    /// Type of the layer
    layer_type: LayerType,
    /// Display name
    name: String,
    /// Whether the layer is visible
    visible: bool,
    /// Whether the layer is locked
    locked: bool,
}

/// Plugin for layer management UI.
///
/// Provides a panel showing:
/// - All available layers
/// - Visibility toggle buttons
/// - Lock status indicators
/// - Selection highlighting
/// - Expandable object lists for Shapes and Detections layers
pub struct LayersPlugin {
    /// Information about each layer
    layers: Vec<LayerInfo>,
    /// Currently selected layer
    selected_layer: Option<LayerType>,
    /// Whether the Shapes layer is expanded to show individual shapes
    shapes_expanded: bool,
    /// Whether the Detections layer is expanded to show individual detections
    detections_expanded: bool,
}

impl LayersPlugin {
    /// Creates a new layers plugin with default layer configuration.
    pub fn new() -> Self {
        let mut layers = Vec::new();

        // Initialize layers in z-order (bottom to top)
        for layer_type in LayerType::iter() {
            layers.push(LayerInfo {
                layer_type,
                name: format!("{:?}", layer_type),
                visible: true,
                locked: false,
            });
        }

        Self {
            layers,
            selected_layer: None,
            shapes_expanded: false,
            detections_expanded: false,
        }
    }

    /// Renders the layer list.
    fn render_layer_list(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.vertical(|ui| {
            // Render layers from top to bottom (reverse z-order for UI)
            // Clone the layers to avoid borrow checker issues
            let layer_count = self.layers.len();
            for i in (0..layer_count).rev() {
                self.render_layer_row(ui, i, ctx);
            }
        });
    }

    /// Renders a single layer row.
    fn render_layer_row(&mut self, ui: &mut egui::Ui, index: usize, ctx: &PluginContext) {
        // Extract layer type first to avoid borrow checker issues
        let layer_type = self.layers[index].layer_type;

        // Check if this layer supports expansion
        let is_expandable = matches!(layer_type, LayerType::Shapes | LayerType::Detections);
        let is_expanded = match layer_type {
            LayerType::Shapes => self.shapes_expanded,
            LayerType::Detections => self.detections_expanded,
            _ => false,
        };

        // Render layer row UI
        {
            let layer = &mut self.layers[index];

            ui.horizontal(|ui| {
                // Expansion caret (for Shapes and Detections layers)
                if is_expandable {
                    let caret = if is_expanded { "▼" } else { "▶" };
                    if ui.button(caret).on_hover_text("Expand/collapse").clicked() {
                        match layer.layer_type {
                            LayerType::Shapes => self.shapes_expanded = !self.shapes_expanded,
                            LayerType::Detections => self.detections_expanded = !self.detections_expanded,
                            _ => {}
                        }
                    }
                } else {
                    // Spacer for alignment
                    ui.add_space(20.0);
                }

                // Selection indicator
                let is_selected = self.selected_layer == Some(layer.layer_type);
                if ui
                    .selectable_label(is_selected, "")
                    .on_hover_text("Select layer")
                    .clicked()
                {
                    self.selected_layer = Some(layer.layer_type);
                    debug!(layer = ?layer.layer_type, "Layer selected");
                    ctx.events.emit(AppEvent::LayerSelected {
                        layer_name: layer.name.clone(),
                    });
                }

                // Layer name
                ui.label(&layer.name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Clear layer button (skip for Grid layer)
                    if layer.layer_type != LayerType::Grid
                        && ui.button("🗑").on_hover_text("Clear layer").clicked()
                    {
                        debug!(layer = ?layer.layer_type, "Layer clear requested");
                        ctx.events.emit(AppEvent::LayerClearRequested {
                            layer_name: layer.name.clone(),
                        });
                    }

                    // Visibility toggle
                    let eye_icon = if layer.visible { "👁" } else { "⚫" };
                    if ui
                        .button(eye_icon)
                        .on_hover_text("Toggle visibility")
                        .clicked()
                    {
                        layer.visible = !layer.visible;
                        debug!(
                            layer = ?layer.layer_type,
                            visible = layer.visible,
                            "Layer visibility toggled"
                        );
                        ctx.events.emit(AppEvent::LayerVisibilityChanged {
                            layer_name: layer.name.clone(),
                            visible: layer.visible,
                        });
                    }

                    // Lock indicator
                    if layer.locked {
                        ui.label("🔒").on_hover_text("Layer is locked");
                    } else {
                        ui.label("🔓").on_hover_text("Layer is unlocked");
                    }
                });
            });
        } // layer borrow ends here

        // Render individual shapes/detections if expanded
        if is_expanded {
            self.render_layer_objects(ui, layer_type, ctx);
        }
    }

    /// Renders individual objects within an expanded layer.
    fn render_layer_objects(&self, ui: &mut egui::Ui, layer_type: LayerType, ctx: &PluginContext) {
        if let Some(canvas) = ctx.canvas {
            let shapes = match layer_type {
                LayerType::Shapes => canvas.shapes(),
                LayerType::Detections => canvas.detections(),
                _ => return,
            };

            if shapes.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Indent
                    ui.label(egui::RichText::new("(empty)").weak());
                });
                return;
            }

            for (i, shape) in shapes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Indent to show hierarchy

                    // Get shape name and visibility
                    let (shape_name, is_visible) = match shape {
                        form_factor_drawing::Shape::Rectangle(r) => (r.name(), r.visible()),
                        form_factor_drawing::Shape::Circle(c) => (c.name(), c.visible()),
                        form_factor_drawing::Shape::Polygon(p) => (p.name(), p.visible()),
                    };

                    // Display shape with index and name
                    ui.label(format!("{}. {}", i + 1, shape_name));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button
                        if ui.button("🗑").on_hover_text("Delete object").clicked() {
                            debug!(
                                layer = ?layer_type,
                                index = i,
                                name = shape_name,
                                "Object delete requested"
                            );
                            ctx.events.emit(AppEvent::ObjectDeleteRequested {
                                layer_type,
                                object_index: i,
                            });
                        }

                        // Visibility toggle
                        let eye_icon = if *is_visible { "👁" } else { "⚫" };
                        if ui
                            .button(eye_icon)
                            .on_hover_text("Toggle visibility")
                            .clicked()
                        {
                            debug!(
                                layer = ?layer_type,
                                index = i,
                                name = shape_name,
                                visible = !is_visible,
                                "Object visibility toggled"
                            );
                            ctx.events.emit(AppEvent::ObjectVisibilityChanged {
                                layer_type,
                                object_index: i,
                                visible: !is_visible,
                            });
                        }
                    });
                });
            }
        }
    }
}

impl Default for LayersPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LayersPlugin {
    fn name(&self) -> &str {
        "layers"
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("Layers");
            ui.separator();
            self.render_layer_list(ui, ctx);
        });
    }

    #[instrument(skip(self, _ctx), fields(plugin = "layers"))]
    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::LayerVisibilityChanged {
                layer_name,
                visible,
            } => {
                debug!(layer_name, visible, "Received visibility change event");
                // Update our layer state
                if let Some(layer) = self.layers.iter_mut().find(|l| l.name == *layer_name) {
                    layer.visible = *visible;
                }
                None
            }
            AppEvent::LayerSelected { layer_name } => {
                debug!(layer_name, "Received layer selection event");
                // Update our selection state
                if let Some(layer) = self.layers.iter().find(|l| l.name == *layer_name) {
                    self.selected_layer = Some(layer.layer_type);
                }
                None
            }
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "Layer management and visibility controls"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layers_plugin_creation() {
        let plugin = LayersPlugin::new();
        assert_eq!(plugin.name(), "layers");
        assert_eq!(plugin.layers.len(), 6); // Canvas, Detections, Shapes, Template, Instance, Grid
        assert!(plugin.selected_layer.is_none());
    }

    #[test]
    fn test_visibility_event_handling() {
        let mut plugin = LayersPlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        // Initially all layers are visible
        assert!(plugin.layers.iter().all(|l| l.visible));

        // Send visibility change event
        let event = AppEvent::LayerVisibilityChanged {
            layer_name: "Canvas".to_string(),
            visible: false,
        };
        plugin.on_event(&event, &ctx);

        // Check that Canvas layer is now hidden
        let canvas_layer = plugin.layers.iter().find(|l| l.name == "Canvas").unwrap();
        assert!(!canvas_layer.visible);
    }

    #[test]
    fn test_layer_selection_event() {
        let mut plugin = LayersPlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let event = AppEvent::LayerSelected {
            layer_name: "Shapes".to_string(),
        };
        plugin.on_event(&event, &ctx);

        assert_eq!(plugin.selected_layer, Some(LayerType::Shapes));
    }
}
