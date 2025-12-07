//! Instance management panel
//!
//! Provides UI for creating, listing, loading, and deleting form instances.

use crate::{DrawingInstance, DrawingTemplate};
use egui::{Color32, RichText, ScrollArea, Ui};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

/// Actions that can be triggered from the instance manager panel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceManagerAction {
    /// Create a new instance from a template
    CreateInstance {
        /// Template to base the instance on
        template_id: String,
    },
    /// Load an instance for editing
    LoadInstance {
        /// Instance ID to load
        instance_id: String,
    },
    /// Delete an instance
    DeleteInstance {
        /// Instance ID to delete
        instance_id: String,
    },
    /// No action
    None,
}

/// Instance management panel for creating and managing form instances
#[derive(Debug, Clone)]
pub struct InstanceManagerPanel {
    /// Available templates (template_id -> template)
    templates: HashMap<String, DrawingTemplate>,
    /// Available instances (instance_id -> instance)
    instances: HashMap<String, DrawingInstance>,
    /// Currently selected template for new instance creation
    selected_template_id: Option<String>,
    /// Search filter for instances
    search_filter: String,
}

impl InstanceManagerPanel {
    /// Create a new instance manager panel
    #[instrument(skip(templates, instances))]
    pub fn new(
        templates: HashMap<String, DrawingTemplate>,
        instances: HashMap<String, DrawingInstance>,
    ) -> Self {
        debug!(
            template_count = templates.len(),
            instance_count = instances.len(),
            "Creating instance manager panel"
        );

        Self {
            templates,
            instances,
            selected_template_id: None,
            search_filter: String::new(),
        }
    }

    /// Render the instance manager UI
    #[instrument(skip(self, ui))]
    pub fn ui(&mut self, ui: &mut Ui) -> InstanceManagerAction {
        let mut action = InstanceManagerAction::None;

        ui.heading("Instance Manager");
        ui.add_space(8.0);

        // Template selection section
        ui.group(|ui| {
            ui.label(RichText::new("Create New Instance").strong());
            ui.add_space(4.0);

            if self.templates.is_empty() {
                ui.label(
                    RichText::new("No templates available. Create a template first.")
                        .color(Color32::GRAY)
                        .italics(),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.label("Template:");

                    egui::ComboBox::from_id_salt("template_selector")
                        .selected_text(
                            self.selected_template_id
                                .as_ref()
                                .and_then(|id| self.templates.get(id))
                                .map(|t| t.name().as_str())
                                .unwrap_or("Select a template"),
                        )
                        .show_ui(ui, |ui| {
                            for (id, template) in &self.templates {
                                let label = format!("{} (v{})", template.name(), template.version());
                                if ui.selectable_label(
                                    self.selected_template_id.as_ref() == Some(id),
                                    label,
                                ).clicked() {
                                    self.selected_template_id = Some(id.clone());
                                    debug!(template_id = id, "Selected template");
                                }
                            }
                        });
                });

                ui.add_space(4.0);

                if ui
                    .add_enabled(
                        self.selected_template_id.is_some(),
                        egui::Button::new("Create Instance"),
                    )
                    .clicked()
                    && let Some(template_id) = &self.selected_template_id {
                        info!(template_id, "Creating new instance");
                        action = InstanceManagerAction::CreateInstance {
                            template_id: template_id.clone(),
                        };
                    }
            }
        });

        ui.add_space(12.0);

        // Instance list section
        ui.group(|ui| {
            ui.label(RichText::new("Existing Instances").strong());
            ui.add_space(4.0);

            // Search filter
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_filter);
                if ui.small_button("✖").clicked() {
                    self.search_filter.clear();
                }
            });

            ui.add_space(4.0);

            if self.instances.is_empty() {
                ui.label(
                    RichText::new("No instances created yet")
                        .color(Color32::GRAY)
                        .italics(),
                );
            } else {
                // Filter instances based on search
                let filtered_instances: Vec<_> = self
                    .instances
                    .iter()
                    .filter(|(id, instance)| {
                        if self.search_filter.is_empty() {
                            return true;
                        }
                        let search_lower = self.search_filter.to_lowercase();
                        id.to_lowercase().contains(&search_lower)
                            || instance
                                .instance_name()
                                .as_ref()
                                .map(|n| n.to_lowercase().contains(&search_lower))
                                .unwrap_or(false)
                            || instance.template_id().to_lowercase().contains(&search_lower)
                    })
                    .collect();

                ui.label(format!(
                    "Showing {} of {} instances",
                    filtered_instances.len(),
                    self.instances.len()
                ));

                ui.add_space(4.0);

                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for (instance_id, instance) in filtered_instances {
                            let instance_action = self.render_instance_item(ui, instance_id, instance);
                            if instance_action != InstanceManagerAction::None {
                                action = instance_action;
                            }
                        }
                    });
            }
        });

        action
    }

    /// Render a single instance item
    #[instrument(skip(self, ui, instance))]
    fn render_instance_item(
        &self,
        ui: &mut Ui,
        instance_id: &str,
        instance: &DrawingInstance,
    ) -> InstanceManagerAction {
        use form_factor_core::FormInstance;

        let mut action = InstanceManagerAction::None;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Instance name
                    let name = FormInstance::instance_name(instance)
                        .unwrap_or("Unnamed Instance");
                    ui.label(RichText::new(name).strong());

                    // Template and metadata
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Template: {}", instance.template_id()))
                                .small()
                                .color(Color32::GRAY),
                        );
                        ui.label(
                            RichText::new(format!("ID: {}", instance_id))
                                .small()
                                .color(Color32::GRAY),
                        );
                    });

                    // Validation status
                    if instance.is_validated()
                        && let Some(results) = instance.validation_results() {
                            let status_text = if *results.valid() {
                                RichText::new("✓ Valid").color(Color32::GREEN)
                            } else {
                                RichText::new(format!("✗ {} errors", results.error_count()))
                                    .color(Color32::RED)
                            };
                            ui.label(status_text.small());
                        }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Delete button
                    if ui.small_button("🗑").on_hover_text("Delete instance").clicked() {
                        warn!(instance_id, "Delete instance requested");
                        action = InstanceManagerAction::DeleteInstance {
                            instance_id: instance_id.to_string(),
                        };
                    }

                    // Load button
                    if ui.small_button("✏").on_hover_text("Edit instance").clicked() {
                        info!(instance_id, "Load instance for editing");
                        action = InstanceManagerAction::LoadInstance {
                            instance_id: instance_id.to_string(),
                        };
                    }
                });
            });
        });

        ui.add_space(4.0);

        action
    }

    /// Add a template to the available templates
    pub fn add_template(&mut self, template: DrawingTemplate) {
        let id = template.id().to_string();
        debug!(template_id = id, "Adding template to instance manager");
        self.templates.insert(id, template);
    }

    /// Add an instance to the list
    pub fn add_instance(&mut self, instance_id: String, instance: DrawingInstance) {
        debug!(instance_id, "Adding instance to manager");
        self.instances.insert(instance_id, instance);
    }

    /// Remove an instance from the list
    pub fn remove_instance(&mut self, instance_id: &str) -> Option<DrawingInstance> {
        debug!(instance_id, "Removing instance from manager");
        self.instances.remove(instance_id)
    }

    /// Get an instance by ID
    pub fn get_instance(&self, instance_id: &str) -> Option<&DrawingInstance> {
        self.instances.get(instance_id)
    }

    /// Get a template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&DrawingTemplate> {
        self.templates.get(template_id)
    }

    /// Get the number of instances
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Get the number of templates
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}
