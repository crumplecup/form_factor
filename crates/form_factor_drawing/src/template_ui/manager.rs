//! Template manager panel for listing and managing templates.

use super::TemplateManagerState;
use crate::TemplateRegistry;
use egui::{ScrollArea, Ui};
use form_factor_core::FormTemplate;
use tracing::{debug, info, instrument};

/// Template manager panel.
#[derive(Debug)]
pub struct TemplateManagerPanel {
    state: TemplateManagerState,
}

impl Default for TemplateManagerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateManagerPanel {
    /// Creates a new template manager panel.
    pub fn new() -> Self {
        Self {
            state: TemplateManagerState::new(),
        }
    }

    /// Gets the panel state.
    pub fn state(&self) -> &TemplateManagerState {
        &self.state
    }

    /// Gets the panel state mutably.
    pub fn state_mut(&mut self) -> &mut TemplateManagerState {
        &mut self.state
    }

    /// Shows the template manager panel.
    #[instrument(skip(self, ui, registry))]
    pub fn show(&mut self, ui: &mut Ui, registry: &mut TemplateRegistry) -> ManagerAction {
        let mut action = ManagerAction::None;

        ui.heading("Template Manager");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("New Template").clicked() {
                debug!("New template button clicked");
                action = ManagerAction::New;
            }

            if ui.button("Import").clicked() {
                debug!("Import button clicked");
                action = ManagerAction::Import;
            }

            ui.add_space(10.0);

            // Search box
            ui.label("Search:");
            let search_response = ui.text_edit_singleline(self.state.search_query_mut());
            if search_response.changed() {
                debug!(query = %self.state.search_query(), "Search query changed");
            }
        });

        ui.separator();

        // Template list
        ScrollArea::vertical().show(ui, |ui| {
            let templates = registry.list();

            // Filter templates by search query
            let filtered: Vec<_> = if self.state.search_query().is_empty() {
                templates
            } else {
                let query = self.state.search_query().to_lowercase();
                templates
                    .into_iter()
                    .filter(|t| {
                        t.id().to_lowercase().contains(&query)
                            || t.name().to_lowercase().contains(&query)
                    })
                    .collect()
            };

            if filtered.is_empty() {
                ui.label("No templates found");
            } else {
                for template in filtered {
                    ui.group(|ui| {
                        // Template info
                        ui.horizontal(|ui| {
                            let is_selected = self.state.selected_template() == Some(template.id());
                            let radio = ui.selectable_label(is_selected, template.name());

                            if radio.clicked() {
                                debug!(template_id = %template.id(), "Template selected");
                                self.state.set_selected_template(Some(template.id().to_string()));
                            }
                        });

                        ui.label(format!("ID: {}", template.id()));
                        ui.label(format!("Pages: {}", template.page_count()));
                        ui.label(format!("Fields: {}", template.fields().len()));

                        // Actions
                        ui.horizontal(|ui| {
                            if ui.button("Edit").clicked() {
                                debug!(template_id = %template.id(), "Edit button clicked");
                                action = ManagerAction::Edit(template.id().to_string());
                            }

                            if ui.button("Duplicate").clicked() {
                                debug!(template_id = %template.id(), "Duplicate button clicked");
                                action = ManagerAction::Duplicate(template.id().to_string());
                            }

                            if ui.button("Export").clicked() {
                                debug!(template_id = %template.id(), "Export button clicked");
                                action = ManagerAction::Export(template.id().to_string());
                            }

                            if ui.button("Delete").clicked() {
                                debug!(template_id = %template.id(), "Delete button clicked");
                                self.state.show_delete_confirm(template.id().to_string());
                            }
                        });
                    });

                    ui.add_space(5.0);
                }
            }
        });

        // Delete confirmation dialog
        if self.state.is_showing_delete_confirm() {
            if let Some(template_id) = self.state.pending_delete() {
                let template_id = template_id.to_string();

                egui::Window::new("Delete Template")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("Are you sure you want to delete template '{}'?", template_id));
                        ui.label("This action cannot be undone.");

                        ui.horizontal(|ui| {
                            if ui.button("Delete").clicked() {
                                info!(template_id = %template_id, "Deleting template");
                                action = ManagerAction::Delete(template_id.clone());
                                self.state.hide_delete_confirm();
                            }

                            if ui.button("Cancel").clicked() {
                                debug!("Delete cancelled");
                                self.state.hide_delete_confirm();
                            }
                        });
                    });
            }
        }

        action
    }
}

impl TemplateManagerState {
    /// Gets the search query mutably (for UI binding).
    pub fn search_query_mut(&mut self) -> &mut String {
        &mut self.search_query
    }
}

/// Action to perform based on user interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAction {
    /// No action
    None,
    /// Create new template
    New,
    /// Edit template
    Edit(String),
    /// Duplicate template
    Duplicate(String),
    /// Delete template
    Delete(String),
    /// Import template from file
    Import,
    /// Export template to file
    Export(String),
}
