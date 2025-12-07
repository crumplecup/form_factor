//! Layers plugin for layer management UI.
//!
//! This plugin provides UI for:
//! - Layer visibility toggles
//! - Layer lock status
//! - Layer selection
//! - Layer z-order display
//!
//! Available with the `plugin-layers` feature.

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
    /// Whether the Detections layer is expanded to show detection subtypes
    detections_expanded: bool,
    /// Whether the Logos subtype is expanded
    logos_expanded: bool,
    /// Whether the Text subtype is expanded
    text_expanded: bool,
    /// Whether the OCR subtype is expanded
    ocr_expanded: bool,
    /// Whether the Images layer is expanded to show individual images
    images_expanded: bool,
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
            logos_expanded: false,
            text_expanded: false,
            ocr_expanded: false,
            images_expanded: false,
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
        let is_expandable = matches!(layer_type, LayerType::Shapes | LayerType::Detections | LayerType::Canvas);
        let is_expanded = match layer_type {
            LayerType::Shapes => self.shapes_expanded,
            LayerType::Detections => self.detections_expanded,
            LayerType::Canvas => self.images_expanded,
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
                            LayerType::Detections => {
                                self.detections_expanded = !self.detections_expanded
                            }
                            LayerType::Canvas => self.images_expanded = !self.images_expanded,
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
    fn render_layer_objects(
        &mut self,
        ui: &mut egui::Ui,
        layer_type: LayerType,
        ctx: &PluginContext,
    ) {
        match layer_type {
            LayerType::Shapes => self.render_shapes_list(ui, ctx),
            LayerType::Detections => self.render_detections_groups(ui, ctx),
            LayerType::Canvas => self.render_canvas_image(ui, ctx),
            _ => {}
        }
    }

    /// Renders the shapes list with individual shape entries.
    fn render_shapes_list(&self, ui: &mut egui::Ui, ctx: &PluginContext) {
        if let Some(canvas) = ctx.canvas {
            let shapes = canvas.shapes();

            if shapes.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Indent
                    ui.label(egui::RichText::new("(empty)").weak());
                });
                return;
            }

            for (i, shape) in shapes.iter().enumerate() {
                Self::render_shape_entry_static(ui, i, shape, ctx);
            }
        }
    }

    /// Renders detection groups (Logos, Text, and OCR) with expandable subtypes.
    fn render_detections_groups(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        if let Some(canvas) = ctx.canvas {
            let detections = canvas.detections();

            // Separate detections by type
            let logos: Vec<_> = detections
                .iter()
                .enumerate()
                .filter(|(_, shape)| {
                    matches!(shape, form_factor_drawing::Shape::Rectangle(r) if r.name().starts_with("Logo:"))
                })
                .collect();

            let text: Vec<_> = detections
                .iter()
                .enumerate()
                .filter(|(_, shape)| {
                    matches!(shape, form_factor_drawing::Shape::Rectangle(r) if r.name().starts_with("Text Region"))
                })
                .collect();

            // Get OCR detections (stored separately with text)
            let ocr_detections = canvas.ocr_detections();

            // Render Logos group
            Self::render_detection_subtype_static(
                ui,
                "Logos",
                &logos,
                &mut self.logos_expanded,
                ctx,
            );

            // Render Text group
            Self::render_detection_subtype_static(ui, "Text", &text, &mut self.text_expanded, ctx);

            // Render OCR group
            Self::render_ocr_subtype(ui, "OCR", ocr_detections, &mut self.ocr_expanded, ctx);
        }
    }

    /// Renders a detection subtype group (e.g., "Logos" or "Text").
    fn render_detection_subtype_static(
        ui: &mut egui::Ui,
        label: &str,
        items: &[(usize, &form_factor_drawing::Shape)],
        is_expanded: &mut bool,
        ctx: &PluginContext,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(40.0); // Indent to show hierarchy under Detections

            // Expansion caret
            let caret = if *is_expanded { "▼" } else { "▶" };
            if ui.button(caret).on_hover_text("Expand/collapse").clicked() {
                *is_expanded = !*is_expanded;
            }

            // Group label with count
            ui.label(format!("{} ({})", label, items.len()));
        });

        // Render individual items if expanded
        if *is_expanded && !items.is_empty() {
            for &(index, shape) in items {
                LayersPlugin::render_shape_entry_static(ui, index, shape, ctx);
            }
        }
    }

    /// Renders OCR detection subtype group with text.
    fn render_ocr_subtype(
        ui: &mut egui::Ui,
        label: &str,
        items: &[(form_factor_drawing::Shape, String)],
        is_expanded: &mut bool,
        ctx: &PluginContext,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(40.0); // Indent to show hierarchy under Detections

            // Expansion caret
            let caret = if *is_expanded { "▼" } else { "▶" };
            if ui.button(caret).on_hover_text("Expand/collapse").clicked() {
                *is_expanded = !*is_expanded;
            }

            // Group label with count
            ui.label(format!("{} ({})", label, items.len()));
        });

        // Render individual OCR items if expanded
        if *is_expanded && !items.is_empty() {
            for (index, (shape, text)) in items.iter().enumerate() {
                Self::render_ocr_entry_static(ui, index, shape, text, ctx);
            }
        }
    }

    /// Renders a single OCR entry with its extracted text.
    fn render_ocr_entry_static(
        ui: &mut egui::Ui,
        index: usize,
        shape: &form_factor_drawing::Shape,
        text: &str,
        ctx: &PluginContext,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(60.0); // Deeper indent for individual items

            // Get shape visibility
            let is_visible = match shape {
                form_factor_drawing::Shape::Rectangle(r) => r.visible(),
                form_factor_drawing::Shape::Circle(c) => c.visible(),
                form_factor_drawing::Shape::Polygon(p) => p.visible(),
            };

            // Display OCR result: index and first 30 chars of text
            let display_text = if text.len() > 30 {
                format!("{}...", &text[..30])
            } else {
                text.to_string()
            };
            ui.label(format!("{}. {}", index + 1, display_text));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button
                if ui.button("🗑").on_hover_text("Delete OCR result").clicked() {
                    debug!(index = index, text = text, "OCR delete requested");
                    ctx.events
                        .emit(AppEvent::OcrObjectDeleteRequested { index });
                }

                // Visibility toggle
                let eye_icon = if *is_visible { "👁" } else { "⚫" };
                if ui
                    .button(eye_icon)
                    .on_hover_text("Toggle visibility")
                    .clicked()
                {
                    debug!(
                        index = index,
                        visible = !is_visible,
                        "OCR visibility toggled"
                    );
                    ctx.events.emit(AppEvent::OcrObjectVisibilityChanged {
                        index,
                        visible: !is_visible,
                    });
                }
            });
        });
    }

    /// Renders a single shape entry (used for both Shapes and Detections).
    fn render_shape_entry_static(
        ui: &mut egui::Ui,
        index: usize,
        shape: &form_factor_drawing::Shape,
        ctx: &PluginContext,
    ) {
        ui.horizontal(|ui| {
            ui.add_space(60.0); // Deeper indent for individual items

            // Get shape name and visibility
            let (shape_name, is_visible) = match shape {
                form_factor_drawing::Shape::Rectangle(r) => (r.name(), r.visible()),
                form_factor_drawing::Shape::Circle(c) => (c.name(), c.visible()),
                form_factor_drawing::Shape::Polygon(p) => (p.name(), p.visible()),
            };

            // Display shape with index and name
            ui.label(format!("{}. {}", index + 1, shape_name));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button
                if ui.button("🗑").on_hover_text("Delete object").clicked() {
                    debug!(index = index, name = shape_name, "Object delete requested");
                    ctx.events.emit(AppEvent::ObjectDeleteRequested {
                        layer_type: LayerType::Detections,
                        object_index: index,
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
                        index = index,
                        name = shape_name,
                        visible = !is_visible,
                        "Object visibility toggled"
                    );
                    ctx.events.emit(AppEvent::ObjectVisibilityChanged {
                        layer_type: LayerType::Detections,
                        object_index: index,
                        visible: !is_visible,
                    });
                }
            });
        });
    }

    /// Renders the canvas image (if loaded).
    fn render_canvas_image(&self, ui: &mut egui::Ui, ctx: &PluginContext) {
        if let Some(canvas) = ctx.canvas {
            // Check if a form image is loaded
            if let Some(path) = canvas.form_image_path() {
                // Extract just the filename from the path
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);

                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Indent

                    // Display image filename
                    ui.label(filename);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete/clear button
                        if ui.button("🗑").on_hover_text("Clear canvas image").clicked() {
                            debug!(path = path, "Canvas image clear requested");
                            ctx.events.emit(AppEvent::CanvasImageClearRequested);
                        }

                        // Lock toggle
                        let is_locked = canvas.form_image_locked();
                        let lock_icon = if *is_locked { "🔒" } else { "🔓" };
                        if ui
                            .button(lock_icon)
                            .on_hover_text("Toggle image lock")
                            .clicked()
                        {
                            debug!(locked = !is_locked, "Canvas image lock toggled");
                            ctx.events.emit(AppEvent::CanvasImageLockChanged {
                                locked: !is_locked,
                            });
                        }

                        // Visibility toggle
                        let is_visible = canvas.form_image_visible();
                        let eye_icon = if *is_visible { "👁" } else { "⚫" };
                        if ui
                            .button(eye_icon)
                            .on_hover_text("Toggle image visibility")
                            .clicked()
                        {
                            debug!(visible = !is_visible, "Canvas image visibility toggled");
                            ctx.events.emit(AppEvent::CanvasImageVisibilityChanged {
                                visible: !is_visible,
                            });
                        }
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(40.0); // Indent
                    ui.label(egui::RichText::new("(no image loaded)").weak());
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
