//! Detection properties panel for editing detection metadata.
//!
//! This module provides UI for viewing and editing metadata associated
//! with detected regions (logos, text, OCR results).

use derive_getters::Getters;
use derive_setters::Setters;
use form_factor_drawing::{DetectionMetadata, FormFieldType, MetadataDetectionType};
use tracing::{debug, instrument};

/// Detection properties editor panel
#[derive(Debug, Clone, Getters, Setters)]
#[setters(prefix = "with_", borrow_self)]
pub struct DetectionPropertiesPanel {
    /// Current detection metadata being edited
    metadata: Option<DetectionMetadata>,
    /// Whether the panel is expanded
    expanded: bool,
}

impl DetectionPropertiesPanel {
    /// Creates a new detection properties panel.
    pub fn new() -> Self {
        Self {
            metadata: None,
            expanded: true,
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
                    ui.horizontal(|ui| {
                        ui.label("Field Type:");
                        let current_type = metadata.form_field_type().clone();
                        let mut selected = current_type.clone();

                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", selected.as_ref().unwrap_or(&FormFieldType::Text)))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected, Some(FormFieldType::Text), "Text");
                                ui.selectable_value(&mut selected, Some(FormFieldType::TextArea), "Text Area");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Date), "Date");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Number), "Number");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Checkbox), "Checkbox");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Radio), "Radio");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Dropdown), "Dropdown");
                                ui.selectable_value(&mut selected, Some(FormFieldType::Signature), "Signature");
                                ui.selectable_value(&mut selected, None, "None");
                            });

                        if selected != current_type {
                            metadata.with_form_field_type(selected.clone());
                            debug!(?selected, "Updated form field type");
                        }
                    });

                    if metadata.form_field_type().is_some() {
                        ui.horizontal(|ui| {
                            ui.label("Field Name:");
                            let mut field_name = metadata.form_field_name().clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut field_name).changed() {
                                metadata.with_form_field_name(Some(field_name));
                                debug!("Updated form field name");
                            }
                        });
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
        let metadata = DetectionMetadata::new(
            "det_001".to_string(),
            MetadataDetectionType::Logo,
            0.95,
        );

        panel.set_metadata(Some(metadata.clone()));
        assert!(panel.metadata().is_some());
        assert_eq!(panel.metadata().as_ref().unwrap().id(), "det_001");
    }
}
