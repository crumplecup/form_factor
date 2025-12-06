//! Field properties editing panel.

use super::field_type_selector::FieldTypeSelector;
use super::state::TemplateEditorState;
use egui::{DragValue, TextEdit, Ui};
use form_factor_core::{FieldBounds, FieldType};
use tracing::{debug, instrument};

/// Field properties editing panel.
///
/// Displays when a field is selected and allows editing all field metadata.
#[derive(Debug)]
pub struct FieldPropertiesPanel {
    /// Validation error messages
    validation_errors: Vec<String>,

    /// Temporary field state for editing
    temp_id: String,
    temp_label: String,
    temp_field_type: FieldType,
    temp_required: bool,
    temp_validation_pattern: String,
    temp_help_text: String,
    temp_bounds: FieldBounds,

    /// Whether temporary state has been initialized
    temp_initialized: bool,

    /// Field type selector widget
    field_type_selector: Option<FieldTypeSelector>,
    
    /// Whether the field type selector is open
    show_field_type_selector: bool,
}

impl Default for FieldPropertiesPanel {
    fn default() -> Self {
        Self {
            validation_errors: Vec::new(),
            temp_id: String::new(),
            temp_label: String::new(),
            temp_field_type: FieldType::FreeText,
            temp_required: false,
            temp_validation_pattern: String::new(),
            temp_help_text: String::new(),
            temp_bounds: FieldBounds {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 30.0,
            },
            temp_initialized: false,
            field_type_selector: None,
            show_field_type_selector: false,
        }
    }
}

impl FieldPropertiesPanel {
    /// Creates a new field properties panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the properties panel for the selected field.
    #[instrument(skip(self, ui, state, page_index))]
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut TemplateEditorState,
        page_index: usize,
    ) -> PropertiesAction {
        let mut action = PropertiesAction::None;

        if let Some(selected_idx) = state.selected_field() {
            // Get the current field from the template
            let field = if let Some(template) = state.current_template() {
                template
                    .fields_for_page(page_index)
                    .get(selected_idx)
                    .map(|f| (*f).clone())
            } else {
                None
            };

            if let Some(field) = field {
                // Initialize temp state if needed
                if !self.temp_initialized {
                    self.temp_id = field.id.clone();
                    self.temp_label = field.label.clone();
                    self.temp_field_type = field.field_type;
                    self.temp_required = field.required;
                    self.temp_validation_pattern =
                        field.validation_pattern.clone().unwrap_or_default();
                    self.temp_help_text = field.help_text.clone().unwrap_or_default();
                    self.temp_bounds = field.bounds;
                    self.temp_initialized = true;
                }

                ui.heading("Field Properties");
                ui.separator();

                // Show validation errors if any
                if !self.validation_errors.is_empty() {
                    ui.colored_label(egui::Color32::RED, "Validation Errors:");
                    for error in &self.validation_errors {
                        ui.colored_label(egui::Color32::RED, format!("• {}", error));
                    }
                    ui.separator();
                }

                // Basic properties
                ui.label("Basic Properties");
                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.add(TextEdit::singleline(&mut self.temp_id).hint_text("field_1"));
                });

                ui.horizontal(|ui| {
                    ui.label("Label:");
                    ui.add(TextEdit::singleline(&mut self.temp_label).hint_text("Field 1"));
                });

                // Field type selection
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(self.temp_field_type.display_name());
                    if ui.button("Change...").clicked() {
                        self.show_field_type_selector = !self.show_field_type_selector;
                        if self.show_field_type_selector && self.field_type_selector.is_none() {
                            self.field_type_selector = Some(
                                FieldTypeSelector::new().with_selected(self.temp_field_type.clone())
                            );
                        }
                    }
                });

                // Show field type selector if open
                if self.show_field_type_selector {
                    ui.separator();
                    ui.group(|ui| {
                        if let Some(ref mut selector) = self.field_type_selector {
                            if selector.show(ui) {
                                if let Some(selected) = selector.selected() {
                                    self.temp_field_type = selected.clone();
                                    self.show_field_type_selector = false;
                                    debug!(field_type = ?self.temp_field_type, "Field type changed");
                                }
                            }
                        }
                        
                        if ui.button("Close").clicked() {
                            self.show_field_type_selector = false;
                        }
                    });
                    ui.separator();
                }

                ui.separator();

                // Validation settings
                ui.label("Validation");
                ui.checkbox(&mut self.temp_required, "Required");

                ui.horizontal(|ui| {
                    ui.label("Pattern:");
                    ui.add(
                        TextEdit::singleline(&mut self.temp_validation_pattern)
                            .hint_text("^[A-Za-z]+$"),
                    );
                });

                // Pattern presets
                ui.horizontal(|ui| {
                    ui.label("Presets:");
                    if ui.small_button("Email").clicked() {
                        self.temp_validation_pattern =
                            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string();
                    }
                    if ui.small_button("Phone").clicked() {
                        self.temp_validation_pattern = r"^\d{3}-\d{3}-\d{4}$".to_string();
                    }
                    if ui.small_button("ZIP").clicked() {
                        self.temp_validation_pattern = r"^\d{5}(-\d{4})?$".to_string();
                    }
                });

                ui.separator();

                // Help text
                ui.label("Help Text");
                ui.add(
                    TextEdit::multiline(&mut self.temp_help_text)
                        .hint_text("Optional help text for this field")
                        .desired_rows(2),
                );

                ui.separator();

                // Bounds adjustment
                ui.label("Position & Size");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(DragValue::new(&mut self.temp_bounds.x).speed(1.0));
                    ui.label("Y:");
                    ui.add(DragValue::new(&mut self.temp_bounds.y).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(
                        DragValue::new(&mut self.temp_bounds.width)
                            .speed(1.0)
                            .range(20.0..=f32::INFINITY),
                    );
                    ui.label("Height:");
                    ui.add(
                        DragValue::new(&mut self.temp_bounds.height)
                            .speed(1.0)
                            .range(20.0..=f32::INFINITY),
                    );
                });

                ui.separator();

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        // Validate before applying
                        self.validation_errors.clear();

                        if self.temp_id.is_empty() {
                            self.validation_errors
                                .push("ID cannot be empty".to_string());
                        }

                        if self.temp_label.is_empty() {
                            self.validation_errors
                                .push("Label cannot be empty".to_string());
                        }

                        // Validate regex pattern if provided
                        if !self.temp_validation_pattern.is_empty()
                            && let Err(e) = regex::Regex::new(&self.temp_validation_pattern)
                        {
                            self.validation_errors
                                .push(format!("Invalid regex pattern: {}", e));
                        }

                        if self.validation_errors.is_empty() {
                            // Apply changes to the field
                            let mut applied = false;
                            if let Some(template) = state.current_template_mut()
                                && let Some(page) = template.pages.get_mut(page_index)
                                && let Some(field) = page.fields.get_mut(selected_idx)
                            {
                                field.id = self.temp_id.clone();
                                field.label = self.temp_label.clone();
                                field.field_type = self.temp_field_type.clone();
                                field.required = self.temp_required;
                                field.validation_pattern =
                                    if self.temp_validation_pattern.is_empty() {
                                        None
                                    } else {
                                        Some(self.temp_validation_pattern.clone())
                                    };
                                field.help_text = if self.temp_help_text.is_empty() {
                                    None
                                } else {
                                    Some(self.temp_help_text.clone())
                                };
                                field.bounds = self.temp_bounds;
                                applied = true;
                                debug!(field_id = %field.id, "Applied field property changes");
                            }

                            if applied {
                                state.push_snapshot(format!(
                                    "Edit properties of '{}'",
                                    self.temp_id
                                ));
                                action = PropertiesAction::Applied;
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        // Reset temp state to original field values
                        self.temp_initialized = false;
                        action = PropertiesAction::Cancelled;
                        debug!("Cancelled field property changes");
                    }

                    if ui.button("Delete Field").clicked() {
                        action = PropertiesAction::Delete(selected_idx);
                        debug!(field_index = selected_idx, "Delete field requested");
                    }
                });
            } else {
                ui.label("Selected field not found");
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No field selected.\nSelect a field to edit its properties.");
            });
        }

        action
    }

    /// Resets the temporary state when a new field is selected.
    pub fn reset(&mut self) {
        self.temp_initialized = false;
        self.validation_errors.clear();
        self.show_field_type_selector = false;
        self.field_type_selector = None;
    }
}

/// Action to perform based on properties panel interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertiesAction {
    /// No action
    None,
    /// Properties were applied
    Applied,
    /// Properties were cancelled
    Cancelled,
    /// Field should be deleted
    Delete(usize),
}
