//! Detection properties panel for editing detection metadata.
//!
//! This module provides UI for viewing and editing metadata associated
//! with detected regions (logos, text, OCR results).

use derive_getters::Getters;
use derive_setters::Setters;
use form_factor_drawing::{
    DetectionMetadata, FieldPropertiesPanel, FieldTypeSelector, FormFieldType,
    MetadataDetectionType,
};
use tracing::{debug, instrument};

/// Detection properties editor panel
#[derive(Debug, Getters, Setters)]
#[setters(prefix = "with_", borrow_self)]
pub struct DetectionPropertiesPanel {
    /// Current detection metadata being edited
    metadata: Option<DetectionMetadata>,
    /// Whether the panel is expanded
    expanded: bool,
    /// Field type selector widget
    field_type_selector: Option<FieldTypeSelector>,
    /// Field properties editor
    field_properties_panel: Option<FieldPropertiesPanel>,
    /// Whether to show field type selector
    show_field_selector: bool,
}

impl DetectionPropertiesPanel {
    /// Creates a new detection properties panel.
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating detection properties panel");
        Self {
            metadata: None,
            expanded: true,
            field_type_selector: None,
            field_properties_panel: None,
            show_field_selector: false,
        }
    }

    /// Sets the detection metadata to edit.
    pub fn set_metadata(&mut self, metadata: Option<DetectionMetadata>) {
        self.metadata = metadata;
    }

    /// Renders the properties panel UI.
    #[instrument(skip(self, ui))]
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<DetectionMetadata> {
        let mut updated_metadata = None;

        if let Some(metadata) = &mut self.metadata {
            ui.group(|ui| {
                ui.heading("Detection Properties");

                ui.separator();

                // Read-only fields
                ui.label(format!("ID: {}", metadata.id()));
                ui.label(format!("Type: {}", metadata.detection_type()));
                ui.label(format!("Confidence: {:.2}%", metadata.confidence() * 100.0));

                ui.separator();

                // Editable label
                ui.horizontal(|ui| {
                    ui.label("Label:");
                    let mut label = metadata.label().clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut label).changed() {
                        metadata.with_label(Some(label));
                        debug!("Updated detection label");
                    }
                });

                // For OCR: Editable extracted text
                if *metadata.detection_type() == MetadataDetectionType::OcrText {
                    ui.separator();
                    ui.label("Extracted Text:");
                    let mut text = metadata.extracted_text().clone().unwrap_or_default();
                    if ui.text_edit_multiline(&mut text).changed() {
                        metadata.with_extracted_text(Some(text));
                        debug!("Updated OCR text");
                    }
                }

                ui.separator();

                // Form field association
                ui.collapsing("Form Field Association", |ui| {
                    // Field type selection button
                    ui.horizontal(|ui| {
                        ui.label("Field Type:");
                        let current_text = metadata
                            .form_field_type()
                            .as_ref()
                            .map(|ft| format!("{:?}", ft))
                            .unwrap_or_else(|| "None".to_string());

                        if ui.button(current_text).clicked() {
                            self.show_field_selector = !self.show_field_selector;
                            if self.show_field_selector && self.field_type_selector.is_none() {
                                self.field_type_selector = Some(FieldTypeSelector::new());
                            }
                        }
                    });

                    // Show field type selector popup when enabled
                    if self.show_field_selector {
                        if self.field_type_selector.is_some() {
                            ui.separator();
                            // TODO: Convert between form_factor_core::FieldType and FormFieldType
                            // For now, use basic combo box until types are unified
                            egui::ComboBox::from_label("Select Field Type")
                                .selected_text(
                                    metadata
                                        .form_field_type()
                                        .as_ref()
                                        .map(|ft| format!("{:?}", ft))
                                        .unwrap_or_else(|| "None".to_string()),
                                )
                                .show_ui(ui, |ui| {
                                    let mut selected = metadata.form_field_type().clone();
                                    let current = selected.clone();

                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Text),
                                            "Text",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::TextArea),
                                            "Text Area",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Date),
                                            "Date",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Number),
                                            "Number",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Checkbox),
                                            "Checkbox",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Radio),
                                            "Radio",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Dropdown),
                                            "Dropdown",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui
                                        .selectable_value(
                                            &mut selected,
                                            Some(FormFieldType::Signature),
                                            "Signature",
                                        )
                                        .clicked()
                                    {
                                        metadata.with_form_field_type(selected.clone());
                                    }
                                    if ui.selectable_value(&mut selected, None, "None").clicked() {
                                        metadata.with_form_field_type(None);
                                    }

                                    if selected != current {
                                        debug!(?selected, "Updated form field type");
                                        self.show_field_selector = false;
                                    }
                                });
                        }
                    }

                    // Field configuration when type is selected
                    if metadata.form_field_type().is_some() {
                        ui.separator();

                        // Field name
                        ui.horizontal(|ui| {
                            ui.label("Field Name:");
                            let mut field_name =
                                metadata.form_field_name().clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut field_name).changed() {
                                metadata.with_form_field_name(Some(field_name));
                                debug!("Updated form field name");
                            }
                        });

                        // Required checkbox
                        ui.horizontal(|ui| {
                            let mut required = metadata.form_field_required().unwrap_or(false);
                            if ui.checkbox(&mut required, "Required field").changed() {
                                metadata.with_form_field_required(Some(required));
                                debug!(required, "Updated required status");
                            }
                        });

                        // Default value
                        ui.horizontal(|ui| {
                            ui.label("Default Value:");
                            let mut default_value = metadata
                                .form_field_default_value()
                                .clone()
                                .unwrap_or_default();
                            if ui.text_edit_singleline(&mut default_value).changed() {
                                metadata.with_form_field_default_value(Some(default_value));
                                debug!("Updated default value");
                            }
                        });

                        // Help text
                        ui.horizontal(|ui| {
                            ui.label("Help Text:");
                            let mut help_text =
                                metadata.form_field_help_text().clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut help_text).changed() {
                                metadata.with_form_field_help_text(Some(help_text));
                                debug!("Updated help text");
                            }
                        });

                        // Type-specific configuration
                        match metadata.form_field_type() {
                            Some(FormFieldType::Dropdown) | Some(FormFieldType::Radio) => {
                                ui.separator();
                                ui.label("Options (one per line):");
                                let mut options_text = metadata
                                    .form_field_options()
                                    .as_ref()
                                    .map(|opts| opts.join("\n"))
                                    .unwrap_or_default();
                                if ui.text_edit_multiline(&mut options_text).changed() {
                                    let options: Vec<String> = options_text
                                        .lines()
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    metadata.with_form_field_options(Some(options));
                                    debug!("Updated field options");
                                }
                            }
                            Some(FormFieldType::Number) => {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Min:");
                                    let mut min_str = metadata
                                        .form_field_min()
                                        .map(|v| v.to_string())
                                        .unwrap_or_default();
                                    if ui.text_edit_singleline(&mut min_str).changed() {
                                        if let Ok(min) = min_str.parse::<f64>() {
                                            metadata.with_form_field_min(Some(min));
                                            debug!(min, "Updated min value");
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Max:");
                                    let mut max_str = metadata
                                        .form_field_max()
                                        .map(|v| v.to_string())
                                        .unwrap_or_default();
                                    if ui.text_edit_singleline(&mut max_str).changed() {
                                        if let Ok(max) = max_str.parse::<f64>() {
                                            metadata.with_form_field_max(Some(max));
                                            debug!(max, "Updated max value");
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                });

                ui.separator();

                // Notes
                ui.label("Notes:");
                let mut notes = metadata.notes().clone().unwrap_or_default();
                if ui.text_edit_multiline(&mut notes).changed() {
                    metadata.with_notes(Some(notes));
                    debug!("Updated notes");
                }

                ui.separator();

                // Validation toggle
                let mut validated = *metadata.validated();
                if ui.checkbox(&mut validated, "Validated by user").changed() {
                    metadata.with_validated(validated);
                    debug!(validated, "Updated validation status");
                }

                updated_metadata = Some(metadata.clone());
            });
        } else {
            ui.label("No detection selected");
        }

        updated_metadata
    }
}

impl Default for DetectionPropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_panel() {
        let panel = DetectionPropertiesPanel::new();
        assert!(panel.metadata().is_none());
        assert!(*panel.expanded());
    }

    #[test]
    fn test_set_metadata() {
        let mut panel = DetectionPropertiesPanel::new();
        let metadata =
            DetectionMetadata::new("det_001".to_string(), MetadataDetectionType::Logo, 0.95);

        panel.set_metadata(Some(metadata.clone()));
        assert!(panel.metadata().is_some());
        assert_eq!(panel.metadata().as_ref().unwrap().id(), "det_001");
    }
}
