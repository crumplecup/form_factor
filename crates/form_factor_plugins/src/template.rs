//! Template management plugin for visual template creation and editing.
//!
//! This plugin provides UI for:
//! - Template browser/list
//! - Template creation
//! - Template editing with visual field placement
//! - Template validation and save

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use form_factor_core::FormTemplate;
use form_factor_drawing::{
    EditorAction, ManagerAction, TemplateEditorPanel, TemplateManagerPanel, TemplateRegistry,
};
use tracing::{debug, info, instrument};

/// Plugin for template management UI.
///
/// Provides a panel with two modes:
/// - **Manager Mode**: Browse and manage templates
/// - **Editor Mode**: Create and edit templates visually
pub struct TemplatePlugin {
    /// Template registry for storage
    registry: TemplateRegistry,
    /// Template manager panel
    manager: TemplateManagerPanel,
    /// Template editor panel (if currently editing)
    editor: Option<TemplateEditorPanel>,
    /// Current mode
    mode: TemplatePluginMode,
}

/// Operating mode for the template plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemplatePluginMode {
    /// Showing template list/manager
    Manager,
    /// Editing a template
    Editor,
}

impl TemplatePlugin {
    /// Creates a new template plugin.
    ///
    /// Attempts to load the template registry. If loading fails, creates a new registry.
    pub fn new() -> Self {
        let registry = TemplateRegistry::new()
            .unwrap_or_else(|e| {
                debug!(error = ?e, "Failed to create template registry, using empty registry");
                // Create a minimal registry without global dir
                TemplateRegistry::new().expect("Failed to create fallback registry")
            });

        Self {
            registry,
            manager: TemplateManagerPanel::new(),
            editor: None,
            mode: TemplatePluginMode::Manager,
        }
    }

    /// Loads all templates from disk.
    #[instrument(skip(self))]
    fn load_templates(&mut self) {
        match self.registry.load_all() {
            Ok(_count) => {
                info!("Loaded templates from disk");
            }
            Err(e) => {
                debug!(error = ?e, "Failed to load templates");
            }
        }
    }
}

impl Default for TemplatePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TemplatePlugin {
    fn name(&self) -> &str {
        "template"
    }

    #[instrument(skip(self, ui, _ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PluginContext) {
        ui.heading("📄 Templates");
        ui.separator();

        match self.mode {
            TemplatePluginMode::Manager => {
                // Show template manager
                let action = self.manager.show(ui, &mut self.registry);

                match action {
                    ManagerAction::New => {
                        debug!("Creating new template");
                        let mut editor = TemplateEditorPanel::new();
                        editor.new_template("", "New Template");
                        self.editor = Some(editor);
                        self.mode = TemplatePluginMode::Editor;
                    }
                    ManagerAction::Edit(template_id) => {
                        debug!(template_id = %template_id, "Opening template for editing");
                        let mut editor = TemplateEditorPanel::new();
                        if editor.load_template(&template_id, &self.registry) {
                            self.editor = Some(editor);
                            self.mode = TemplatePluginMode::Editor;
                        } else {
                            debug!(template_id = %template_id, "Failed to load template");
                        }
                    }
                    ManagerAction::Delete(template_id) => {
                        debug!(template_id = %template_id, "Deleting template");
                        if let Err(e) = self.registry.delete_from_global(&template_id) {
                            debug!(error = ?e, template_id = %template_id, "Failed to delete template");
                        }
                    }
                    ManagerAction::Duplicate(template_id) => {
                        debug!(template_id = %template_id, "Duplicating template (not implemented)");
                        // TODO: Implement template duplication
                    }
                    ManagerAction::Export(template_id) => {
                        debug!(template_id = %template_id, "Exporting template (not implemented)");
                        // TODO: Implement template export
                    }
                    ManagerAction::Import => {
                        debug!("Importing template (not implemented)");
                        // TODO: Implement template import
                    }
                    ManagerAction::None => {}
                }
            }
            TemplatePluginMode::Editor => {
                // Show template editor
                if let Some(editor) = &mut self.editor {
                    let action = editor.show(ui, &self.registry);

                    match action {
                        EditorAction::Save { is_new } => {
                            debug!(is_new = is_new, "Saving template");
                            match editor.save_template(&mut self.registry) {
                                Ok(template) => {
                                    info!(template_id = %template.id(), "Template saved successfully");
                                    // Note: Registry save is handled by on_save() lifecycle method
                                    // Stay in editor mode for continued editing
                                }
                                Err(errors) => {
                                    debug!(error_count = errors.len(), "Template validation failed");
                                    // Errors are displayed in editor UI
                                }
                            }
                        }
                        EditorAction::Cancel => {
                            debug!("Cancelling template editing");
                            self.editor = None;
                            self.mode = TemplatePluginMode::Manager;
                        }
                        EditorAction::None => {}
                    }
                } else {
                    // Editor was removed, go back to manager
                    self.mode = TemplatePluginMode::Manager;
                }

                // Add "Back to Manager" button at bottom
                ui.separator();
                if ui.button("⬅ Back to Templates").clicked() {
                    self.editor = None;
                    self.mode = TemplatePluginMode::Manager;
                }
            }
        }
    }

    #[instrument(skip(self, _event, _ctx))]
    fn on_event(&mut self, _event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        // Template plugin doesn't currently respond to external events
        // but could be extended to handle TemplateSelected events, etc.
        None
    }

    #[instrument(skip(self, _ctx))]
    fn on_load(&mut self, _ctx: &PluginContext) {
        debug!("Template plugin loading");
        self.load_templates();
    }

    #[instrument(skip(self, _ctx))]
    fn on_save(&mut self, _ctx: &PluginContext) {
        debug!("Template plugin saving");
        // Templates are already saved to registry in memory via save_template()
        // Physical persistence happens when templates are registered
    }

    #[instrument(skip(self, _ctx))]
    fn on_shutdown(&mut self, _ctx: &PluginContext) {
        debug!("Template plugin shutting down");
        // Templates are persisted in registry files automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plugin() {
        let plugin = TemplatePlugin::new();
        assert_eq!(plugin.name(), "template");
        assert_eq!(plugin.mode, TemplatePluginMode::Manager);
        assert!(plugin.editor.is_none());
    }

    #[test]
    fn test_default() {
        let plugin = TemplatePlugin::default();
        assert_eq!(plugin.name(), "template");
    }
}
