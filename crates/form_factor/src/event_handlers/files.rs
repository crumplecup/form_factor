//! File event handlers


#[cfg(feature = "plugins")]
use crate::file_dialogs::FileDialogs;
#[cfg(feature = "plugins")]
use form_factor_drawing::DrawingCanvas;

#[cfg(feature = "plugins")]
use form_factor_plugins::{AppEvent, EventSender};

/// File event handler
pub struct FileEventHandler;

#[cfg(feature = "plugins")]
impl FileEventHandler {
    /// Handle open file request
    #[tracing::instrument(skip(canvas, sender, egui_ctx))]
    pub fn handle_open_requested(
        canvas: &mut DrawingCanvas,
        sender: &EventSender,
        egui_ctx: &egui::Context,
    ) {
        tracing::debug!("Handling open file request");

        if let Some(path) = FileDialogs::open_project() && let Some(path_str) = path.to_str() {
            match canvas.load_from_file(path_str, egui_ctx) {
                Ok(()) => {
                    tracing::info!("Loaded project from {}", path_str);
                    sender.emit(AppEvent::FileOpened {
                        path: path.clone(),
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, path = path_str, "Failed to load project");
                }
            }
        }
    }

    /// Handle save file request
    #[tracing::instrument(skip(canvas, sender), fields(project_name))]
    pub fn handle_save_requested(
        canvas: &mut DrawingCanvas,
        sender: &EventSender,
        project_name: &str,
    ) {
        tracing::debug!("Handling save file request");

        if let Some(path) = FileDialogs::save_project(project_name)
            && let Some(path_str) = path.to_str()
        {
            match canvas.save_to_file(path_str) {
                Ok(()) => {
                    tracing::info!("Saved project to {}", path_str);
                    sender.emit(AppEvent::FileSaved {
                        path: path.clone(),
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, path = path_str, "Failed to save project");
                }
            }
        }
    }

    /// Handle save as request
    #[tracing::instrument(skip(canvas, sender), fields(project_name))]
    pub fn handle_save_as_requested(
        canvas: &mut DrawingCanvas,
        sender: &EventSender,
        project_name: &str,
    ) {
        tracing::debug!("Handling save as request");

        if let Some(path) = FileDialogs::save_project(project_name)
            && let Some(path_str) = path.to_str()
        {
            match canvas.save_to_file(path_str) {
                Ok(()) => {
                    tracing::info!("Saved project to {}", path_str);
                    sender.emit(AppEvent::FileSaved {
                        path: path.clone(),
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, path = path_str, "Failed to save project");
                }
            }
        }
    }

    /// Handle load image request
    #[tracing::instrument(skip(canvas, egui_ctx))]
    pub fn handle_load_image_requested(canvas: &mut DrawingCanvas, egui_ctx: &egui::Context) {
        tracing::debug!("Handling load image request");

        if let Some(path) = FileDialogs::load_image() && let Some(path_str) = path.to_str() {
            match canvas.load_form_image(path_str, egui_ctx) {
                Ok(()) => {
                    tracing::info!("Loaded image from {}", path_str);
                }
                Err(e) => {
                    tracing::error!(error = %e, path = path_str, "Failed to load image");
                }
            }
        }
    }
}
