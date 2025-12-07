//! File dialog utilities for project and image operations

use std::path::PathBuf;
use tracing::instrument;

/// File dialog utilities
pub struct FileDialogs;

impl FileDialogs {
    /// Show open project dialog
    #[instrument]
    pub fn open_project() -> Option<PathBuf> {
        tracing::debug!("Opening project file dialog");

        let result = rfd::FileDialog::new()
            .add_filter("Form Factor Project", &["ffp"])
            .pick_file();

        match &result {
            Some(path) => tracing::info!(path = ?path, "Project file selected"),
            None => tracing::debug!("Project file dialog cancelled"),
        }

        result
    }

    /// Show save project dialog
    #[instrument(fields(default_name))]
    pub fn save_project(default_name: &str) -> Option<PathBuf> {
        tracing::debug!(default_name, "Opening save project dialog");

        let result = rfd::FileDialog::new()
            .add_filter("Form Factor Project", &["ffp"])
            .set_file_name(format!("{}.ffp", default_name))
            .save_file();

        match &result {
            Some(path) => tracing::info!(path = ?path, "Save location selected"),
            None => tracing::debug!("Save dialog cancelled"),
        }

        result
    }

    /// Show load image dialog
    #[instrument]
    pub fn load_image() -> Option<PathBuf> {
        tracing::debug!("Opening image file dialog");

        let result = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file();

        match &result {
            Some(path) => tracing::info!(path = ?path, "Image file selected"),
            None => tracing::debug!("Image dialog cancelled"),
        }

        result
    }
}
