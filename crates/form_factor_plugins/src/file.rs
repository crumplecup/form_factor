//! File operations plugin for open/save/export functionality.
//!
//! This plugin provides UI for:
//! - Opening files
//! - Saving the current project
//! - Save-as functionality
//! - Recent files list
//! - Current file path display

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use std::path::PathBuf;
use tracing::{debug, instrument};

/// Plugin for file operations.
///
/// Provides a panel with:
/// - File operation buttons (Open, Save, Save As)
/// - Current file path display
/// - Recent files list
pub struct FilePlugin {
    /// Currently open file path
    current_file: Option<PathBuf>,
    /// List of recently opened files
    recent_files: Vec<PathBuf>,
    /// Maximum number of recent files to track
    max_recent: usize,
}

impl FilePlugin {
    /// Creates a new file operations plugin.
    pub fn new() -> Self {
        Self {
            current_file: None,
            recent_files: Vec::new(),
            max_recent: 10,
        }
    }

    /// Creates a new file plugin with a specified maximum number of recent files.
    pub fn with_max_recent(max_recent: usize) -> Self {
        Self {
            current_file: None,
            recent_files: Vec::new(),
            max_recent,
        }
    }

    /// Renders the file operation buttons.
    fn render_file_buttons(&self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.horizontal(|ui| {
            if ui.button("Open...").clicked() {
                debug!("Open file requested");
                ctx.events.emit(AppEvent::OpenFileRequested);
            }

            let save_enabled = self.current_file.is_some();
            if ui
                .add_enabled(save_enabled, egui::Button::new("Save"))
                .clicked()
            {
                debug!("Save file requested");
                ctx.events.emit(AppEvent::SaveFileRequested);
            }

            if ui.button("Save As...").clicked() {
                debug!("Save as requested");
                ctx.events.emit(AppEvent::SaveAsRequested);
            }
        });
    }

    /// Renders the current file path display.
    fn render_current_file(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Current:");
            if let Some(path) = &self.current_file {
                ui.label(path.display().to_string());
            } else {
                ui.label("(No file open)");
            }
        });
    }

    /// Renders the recent files list.
    fn render_recent_files(&self, ui: &mut egui::Ui, ctx: &PluginContext) {
        if !self.recent_files.is_empty() {
            ui.separator();
            ui.label("Recent files:");

            for path in &self.recent_files {
                if ui.button(path.display().to_string()).clicked() {
                    debug!(?path, "Recent file clicked");
                    ctx.events.emit(AppEvent::FileOpened { path: path.clone() });
                }
            }
        }
    }

    /// Adds a file to the recent files list.
    fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already in list
        self.recent_files.retain(|p| p != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Trim to max size
        if self.recent_files.len() > self.max_recent {
            self.recent_files.truncate(self.max_recent);
        }
    }
}

impl Default for FilePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for FilePlugin {
    fn name(&self) -> &str {
        "file"
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("File");
            self.render_file_buttons(ui, ctx);
            ui.separator();
            self.render_current_file(ui);
            self.render_recent_files(ui, ctx);
        });
    }

    #[instrument(skip(self, _ctx), fields(plugin = "file"))]
    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::FileOpened { path } => {
                debug!(?path, "File opened");
                self.current_file = Some(path.clone());
                self.add_recent_file(path.clone());
                None
            }
            AppEvent::FileSaved { path } => {
                debug!(?path, "File saved");
                self.current_file = Some(path.clone());
                self.add_recent_file(path.clone());
                None
            }
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "File open, save, and recent files management"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_plugin_creation() {
        let plugin = FilePlugin::new();
        assert_eq!(plugin.name(), "file");
        assert!(plugin.current_file.is_none());
        assert!(plugin.recent_files.is_empty());
        assert_eq!(plugin.max_recent, 10);
    }

    #[test]
    fn test_file_opened_event() {
        let mut plugin = FilePlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let test_path = PathBuf::from("/test/file.txt");
        let event = AppEvent::FileOpened {
            path: test_path.clone(),
        };
        plugin.on_event(&event, &ctx);

        assert_eq!(plugin.current_file, Some(test_path.clone()));
        assert_eq!(plugin.recent_files.len(), 1);
        assert_eq!(plugin.recent_files[0], test_path);
    }

    #[test]
    fn test_recent_files_limit() {
        let mut plugin = FilePlugin::with_max_recent(3);
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        // Add 5 files
        for i in 1..=5 {
            let path = PathBuf::from(format!("/test/file{}.txt", i));
            let event = AppEvent::FileOpened { path };
            plugin.on_event(&event, &ctx);
        }

        // Should only keep the 3 most recent
        assert_eq!(plugin.recent_files.len(), 3);
        assert_eq!(plugin.recent_files[0], PathBuf::from("/test/file5.txt"));
        assert_eq!(plugin.recent_files[1], PathBuf::from("/test/file4.txt"));
        assert_eq!(plugin.recent_files[2], PathBuf::from("/test/file3.txt"));
    }

    #[test]
    fn test_recent_files_deduplication() {
        let mut plugin = FilePlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let test_path = PathBuf::from("/test/file.txt");

        // Open the same file twice
        plugin.on_event(
            &AppEvent::FileOpened {
                path: test_path.clone(),
            },
            &ctx,
        );
        plugin.on_event(
            &AppEvent::FileOpened {
                path: test_path.clone(),
            },
            &ctx,
        );

        // Should only appear once
        assert_eq!(plugin.recent_files.len(), 1);
        assert_eq!(plugin.recent_files[0], test_path);
    }
}
