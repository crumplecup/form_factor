//! Property inspector plugin for editing shape and field properties.

use crate::{AppEvent, Plugin, PluginContext};
use egui::{Color32, Ui};
use tracing::instrument;

/// Property inspector plugin for editing selected shape/field properties.
#[derive(Debug)]
pub struct PropertiesPlugin {
    name: String,
    enabled: bool,
    selected_item: Option<SelectedItem>,
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
    /// A template field
    TemplateField {
        /// Field ID
        id: usize,
        /// Field name
        name: String,
        /// Field type (text, checkbox, etc.)
        field_type: String,
        /// Position (x, y)
        position: (f32, f32),
        /// Size (width, height)
        size: (f32, f32),
        /// Required field flag
        required: bool,
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
        }
    }

    /// Updates the currently selected item based on app events.
    fn update_selection(&mut self, _ctx: &PluginContext) {
        // Listen for selection events from other plugins
        // For now, we'll poll for selection state from events
        // TODO: Implement proper selection tracking
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

    /// Renders property editor for a template field.
    fn render_field_properties(
        ui: &mut Ui,
        id: usize,
        name: &mut String,
        field_type: &mut String,
        position: &mut (f32, f32),
        size: &mut (f32, f32),
        required: &mut bool,
    ) {
        {
            ui.heading("Field Properties");
            ui.separator();

            ui.label(format!("ID: {}", id));
            ui.separator();

            // Name editor
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(name);
            });

            // Type selector
            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("field_type")
                    .selected_text(field_type.as_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(field_type, "text".to_string(), "Text");
                        ui.selectable_value(field_type, "checkbox".to_string(), "Checkbox");
                        ui.selectable_value(field_type, "date".to_string(), "Date");
                        ui.selectable_value(field_type, "signature".to_string(), "Signature");
                    });
            });

            // Position editor
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut position.0).speed(1.0));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut position.1).speed(1.0));
            });

            // Size editor
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(
                    egui::DragValue::new(&mut size.0)
                        .speed(1.0)
                        .range(10.0..=10000.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Height:");
                ui.add(
                    egui::DragValue::new(&mut size.1)
                        .speed(1.0)
                        .range(10.0..=10000.0),
                );
            });

            // Required checkbox
            ui.separator();
            ui.checkbox(required, "Required field");
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
            Some(SelectedItem::TemplateField {
                id,
                name,
                field_type,
                position,
                size,
                required,
            }) => {
                Self::render_field_properties(ui, *id, name, field_type, position, size, required);
            }
            None => {
                ui.label("No selection");
                ui.separator();
                ui.label("Select a shape or field to edit its properties.");
            }
        }
    }

    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
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
            AppEvent::SelectionCleared => {
                tracing::debug!("Selection cleared");
                self.selected_item = None;
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
