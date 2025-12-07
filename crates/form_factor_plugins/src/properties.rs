//! Property inspector plugin for editing shape and field properties.

use crate::{detection_properties::DetectionPropertiesPanel, AppEvent, Plugin, PluginContext};
use egui::{Color32, Ui};
use form_factor_drawing::DetectionMetadata;
use tracing::instrument;

/// Property inspector plugin for editing selected shape/field properties.
#[derive(Debug)]
pub struct PropertiesPlugin {
    name: String,
    enabled: bool,
    selected_item: Option<SelectedItem>,
    detection_panel: DetectionPropertiesPanel,
}

/// Type of item currently selected for property editing.
#[derive(Debug, Clone)]
enum SelectedItem {
    /// A shape on the canvas
    Shape {
        /// Shape ID
        id: usize,
        /// Shape type name
        shape_type: String,
        /// Position (x, y)
        position: (f32, f32),
        /// Size (width, height) if applicable
        size: Option<(f32, f32)>,
        /// Color
        color: Color32,
        /// Label text
        label: String,
    },
    /// A detection (logo, text, OCR)
    Detection {
        /// Detection metadata
        metadata: DetectionMetadata,
    },
}

impl PropertiesPlugin {
    /// Creates a new property inspector plugin.
    #[instrument]
    pub fn new() -> Self {
        tracing::debug!("Creating properties plugin");
        Self {
            name: "Properties".to_string(),
            enabled: true,
            selected_item: None,
            detection_panel: DetectionPropertiesPanel::new(),
        }
    }

    /// Updates the currently selected item based on app events.
    fn update_selection(&mut self, _ctx: &PluginContext) {
        // Listen for selection events from other plugins
        // For now, we'll poll for selection state from events
        // TODO: Implement proper selection tracking
    }
    
    /// Fetches detection metadata from application state.
    fn fetch_detection_metadata(
        &self,
        _detection_id: &str,
        _ctx: &PluginContext,
    ) -> Option<DetectionMetadata> {
        // TODO: Implement proper metadata fetching from app state
        // For now, return None - this needs to be wired to actual detection storage
        tracing::warn!("Detection metadata fetching not yet implemented");
        None
    }

    /// Renders property editor for a shape.
    fn render_shape_properties(
        ui: &mut Ui,
        id: usize,
        shape_type: &str,
        position: &mut (f32, f32),
        size: &mut Option<(f32, f32)>,
        color: &mut Color32,
        label: &mut String,
    ) {
        {
            ui.heading("Shape Properties");
            ui.separator();

            ui.label(format!("Type: {}", shape_type));
            ui.label(format!("ID: {}", id));
            ui.separator();

            // Position editor
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut position.0).speed(1.0));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut position.1).speed(1.0));
            });

            // Size editor (if applicable)
            if let Some((width, height)) = size {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(width).speed(1.0).range(1.0..=10000.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.add(egui::DragValue::new(height).speed(1.0).range(1.0..=10000.0));
                });
            }

            // Color editor
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Color:");
                ui.color_edit_button_srgba(color);
            });

            // Label editor
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Label:");
                ui.text_edit_singleline(label);
            });
        }
    }
}

impl Default for PropertiesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PropertiesPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut Ui, ctx: &PluginContext) {
        ui.heading(&self.name);
        ui.separator();

        // Update selection state
        self.update_selection(ctx);

        // Render property editor based on selection
        match &mut self.selected_item {
            Some(SelectedItem::Shape {
                id,
                shape_type,
                position,
                size,
                color,
                label,
            }) => {
                Self::render_shape_properties(ui, *id, shape_type, position, size, color, label);
            }
            Some(SelectedItem::Detection { metadata }) => {
                // Show detection properties panel
                self.detection_panel.set_metadata(Some(metadata.clone()));
                if let Some(_updated) = self.detection_panel.ui(ui) {
                    // TODO: Emit event with updated metadata
                    // For now, just log that an update happened
                    tracing::debug!("Detection metadata updated");
                }
            }
            None => {
                ui.label("No selection");
                ui.separator();
                ui.label("Select a shape or detection to edit its properties.");
            }
        }
    }

    fn on_event(&mut self, event: &AppEvent, ctx: &PluginContext) -> Option<AppEvent> {
        // Handle selection events
        match event {
            AppEvent::ShapeSelected { index } => {
                tracing::debug!(index, "Shape selected, updating properties panel");
                // TODO: Fetch actual shape data and populate selected_item
                self.selected_item = Some(SelectedItem::Shape {
                    id: *index,
                    shape_type: "Rectangle".to_string(),
                    position: (100.0, 100.0),
                    size: Some((200.0, 150.0)),
                    color: Color32::from_rgb(0, 120, 215),
                    label: String::new(),
                });
            }
            AppEvent::DetectionSelected { detection_id } => {
                tracing::debug!(detection_id, "Detection selected, fetching metadata");
                // Fetch detection metadata from state
                if let Some(metadata) = self.fetch_detection_metadata(detection_id, ctx) {
                    self.selected_item = Some(SelectedItem::Detection { metadata });
                }
            }
            AppEvent::SelectionCleared => {
                tracing::debug!("Selection cleared");
                self.selected_item = None;
                self.detection_panel.set_metadata(None);
            }
            AppEvent::Custom {
                plugin, event_type, ..
            } => {
                // Listen for selection events from canvas or layers plugin
                if plugin == "canvas" && event_type == "shape_selected" {
                    tracing::debug!("Shape selected event received from custom event");
                }
            }
            _ => {}
        }
        None
    }
    


    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn description(&self) -> &str {
        "Edit properties of selected shapes and template fields"
    }
}
