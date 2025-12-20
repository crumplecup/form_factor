//! Templates plugin for template management UI.
//!
//! Provides a sidebar button to open the template browser overlay.

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use tracing::{debug, instrument};

/// Plugin for template management UI.
///
/// Provides a simple button to open the template browser overlay.
pub struct TemplatesPlugin {
    /// Plugin name
    name: String,
}

impl TemplatesPlugin {
    /// Creates a new templates plugin.
    pub fn new() -> Self {
        Self {
            name: "templates".to_string(),
        }
    }
}

impl Default for TemplatesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TemplatesPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    #[cfg(feature = "plugin-canvas")]
    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("📚 Templates");
            ui.separator();

            ui.label("Manage form templates");
            ui.add_space(5.0);

            if ui.button("Browse Templates").clicked() {
                debug!("Browse Templates button clicked");
                ctx.events.emit(AppEvent::OpenTemplateBrowserRequested);
            }

            ui.add_space(5.0);
            ui.label("Use Ctrl+T to open");
        });
    }

    #[cfg(not(feature = "plugin-canvas"))]
    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("📚 Templates");
            ui.separator();

            ui.label("Manage form templates");
            ui.add_space(5.0);

            if ui.button("Browse Templates").clicked() {
                debug!("Browse Templates button clicked");
                ctx.events.emit(AppEvent::OpenTemplateBrowserRequested);
            }

            ui.add_space(5.0);
            ui.label("Use Ctrl+T to open");
        });
    }

    fn description(&self) -> &str {
        "Template management and browser"
    }
}
