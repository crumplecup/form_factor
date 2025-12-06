//! Instance data entry panel
//!
//! Provides UI for filling out form instances with field-specific input widgets,
//! validation, and multi-page navigation.

use crate::{DrawingInstance, DrawingTemplate, TemplateError};
use egui::{Color32, RichText, ScrollArea, Ui};
use form_factor_core::FormInstance;
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

/// Action returned from data entry panel UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataEntryAction {
    /// Save instance as draft (may be incomplete)
    SaveDraft,
    /// Submit instance (requires validation)
    Submit,
    /// Cancel data entry
    Cancel,
    /// No action
    None,
}

/// Instance data entry panel state
#[derive(Debug, Clone)]
pub struct DataEntryPanel {
    /// Template being filled
    template: DrawingTemplate,
    /// Instance being edited
    instance: DrawingInstance,
    /// Current page index
    current_page: usize,
    /// Validation errors by field ID
    validation_errors: HashMap<String, String>,
    /// Whether instance has unsaved changes
    dirty: bool,
}

impl DataEntryPanel {
    /// Create a new data entry panel
    #[instrument(skip(template, instance))]
    pub fn new(template: DrawingTemplate, instance: DrawingInstance) -> Self {
        debug!(
            template_id = %template.id(),
            page_count = template.page_count(),
            "Creating data entry panel"
        );

        Self {
            template,
            instance,
            current_page: 0,
            validation_errors: HashMap::new(),
            dirty: false,
        }
    }

    /// Check if panel has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the current template
    pub fn template(&self) -> &DrawingTemplate {
        &self.template
    }

    /// Get the current instance
    pub fn instance(&self) -> &DrawingInstance {
        &self.instance
    }

    /// Get mutable reference to instance
    pub fn instance_mut(&mut self) -> &mut DrawingInstance {
        &mut self.instance
    }

    /// Set current page
    #[instrument(skip(self))]
    pub fn set_page(&mut self, page: usize) {
        if page < self.template.page_count() {
            debug!(old_page = self.current_page, new_page = page, "Changing page");
            self.current_page = page;
        } else {
            warn!(
                page,
                page_count = self.template.page_count(),
                "Attempted to set invalid page"
            );
        }
    }

    /// Get current page index
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Validate all fields
    #[instrument(skip(self))]
    pub fn validate(&mut self) -> Result<(), TemplateError> {
        self.validation_errors.clear();

        // TODO: Implement field validation based on template
        // For now, just check required fields
        debug!(field_count = self.template.field_count(), "Validating fields");

        Ok(())
    }

    /// Calculate completion percentage
    #[instrument(skip(self))]
    pub fn completion_percentage(&self) -> f32 {
        let total_fields = self.template.field_count();
        if total_fields == 0 {
            return 100.0;
        }

        let filled_fields = self.instance.field_values().len();
        (filled_fields as f32 / total_fields as f32) * 100.0
    }

    /// Render the data entry panel
    #[instrument(skip(self, ui))]
    pub fn ui(&mut self, ui: &mut Ui) -> DataEntryAction {
        let mut action = DataEntryAction::None;

        // Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new(self.template.name()).strong());
        });

        ui.separator();

        // Instance name input
        ui.horizontal(|ui| {
            ui.label("Instance Name:");
            let mut name = self.instance.instance_name().as_ref().map(|s| s.as_str()).unwrap_or_default().to_string();
            if ui.text_edit_singleline(&mut name).changed() {
                self.instance.set_instance_name(name);
                self.dirty = true;
            }
        });

        ui.separator();

        // Progress and page navigation
        ui.horizontal(|ui| {
            ui.label(format!(
                "Page {} of {}",
                self.current_page + 1,
                self.template.page_count()
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{:.0}% complete",
                    self.completion_percentage()
                ));
            });
        });

        ui.separator();

        // Field list (scrollable)
        ScrollArea::vertical()
            .id_salt("data_entry_fields")
            .show(ui, |ui| {
                action = self.render_fields(ui);
            });

        ui.separator();

        // Footer with navigation and actions
        ui.horizontal(|ui| {
            // Page navigation
            if ui
                .add_enabled(self.current_page > 0, egui::Button::new("◀ Previous"))
                .clicked()
            {
                self.set_page(self.current_page.saturating_sub(1));
            }

            if ui
                .add_enabled(
                    self.current_page < self.template.page_count() - 1,
                    egui::Button::new("Next ▶"),
                )
                .clicked()
            {
                self.set_page(self.current_page + 1);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Cancel
                if ui.button("Cancel").clicked() {
                    action = DataEntryAction::Cancel;
                }

                // Submit
                let can_submit = self.validation_errors.is_empty();
                if ui
                    .add_enabled(can_submit, egui::Button::new("Submit"))
                    .clicked()
                {
                    action = DataEntryAction::Submit;
                }

                // Save draft
                if ui.button("Save Draft").clicked() {
                    action = DataEntryAction::SaveDraft;
                }

                // Validation status
                if !self.validation_errors.is_empty() {
                    ui.label(
                        RichText::new(format!("{} errors", self.validation_errors.len()))
                            .color(Color32::RED),
                    );
                }
            });
        });

        action
    }

    /// Render fields for current page
    #[instrument(skip(self, ui))]
    fn render_fields(&mut self, ui: &mut Ui) -> DataEntryAction {
        // TODO: Get fields for current page from template
        // For now, render placeholder
        ui.label("Field rendering not yet implemented");

        DataEntryAction::None
    }

}
