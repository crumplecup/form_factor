//! Legacy project migration utilities
//!
//! This module provides functionality to migrate legacy single-page projects
//! (DrawingCanvas) to the new multi-page instance format (DrawingInstance).
//!
//! # Migration Strategy
//!
//! - **Version 0 or missing**: Legacy single-page DrawingCanvas format
//! - **Version 1**: Legacy single-page DrawingCanvas format (explicit)
//! - **Version 2**: Multi-page DrawingInstance format
//!
//! When loading a project file:
//! 1. Try to parse as DrawingInstance (version 2)
//! 2. If that fails or version < 2, parse as DrawingCanvas (legacy)
//! 3. Convert legacy canvas to DrawingInstance with single page
//! 4. Wrap in instance with default template ID

use super::error::{InstanceError, InstanceErrorKind};
use super::implementation::{DrawingInstance, FormPage};
use crate::DrawingCanvas;
use form_factor_core::FormInstance;
use serde_json::Value;
use tracing::{debug, info, instrument};

/// Default template ID for migrated legacy projects
pub const LEGACY_TEMPLATE_ID: &str = "legacy";

/// Project file format with version information
#[derive(Debug)]
pub enum ProjectFormat {
    /// Legacy single-page format (DrawingCanvas)
    Legacy(Box<DrawingCanvas>),
    /// Multi-page instance format (DrawingInstance)
    Instance(Box<DrawingInstance>),
}

impl ProjectFormat {
    /// Detect the format version from JSON
    ///
    /// Returns the detected version:
    /// - 0 or missing: Legacy format (no version field)
    /// - 1: Legacy format (explicit version)
    /// - 2: Multi-page instance format
    #[instrument(skip(json), fields(json_length = json.len()))]
    pub fn detect_version(json: &str) -> Result<u32, InstanceError> {
        // Parse as generic JSON value to inspect structure
        let value: Value = serde_json::from_str(json).map_err(|e| {
            InstanceError::new(
                InstanceErrorKind::Deserialization(format!("Invalid JSON: {}", e)),
                line!(),
                file!(),
            )
        })?;

        // Check for version field
        let version = value
            .get("version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0);

        debug!(version, "Detected project version");
        Ok(version)
    }

    /// Load a project file with automatic format detection
    ///
    /// Attempts to load the file as either a legacy DrawingCanvas or
    /// a multi-page DrawingInstance, automatically detecting the format.
    #[instrument(skip(json), fields(json_length = json.len()))]
    pub fn from_json(json: &str) -> Result<Self, InstanceError> {
        let version = Self::detect_version(json)?;

        match version {
            0 | 1 => {
                // Legacy format - parse as DrawingCanvas
                info!(version, "Loading as legacy DrawingCanvas format");
                let canvas: DrawingCanvas = serde_json::from_str(json).map_err(|e| {
                    InstanceError::new(
                        InstanceErrorKind::Deserialization(format!(
                            "Failed to parse legacy canvas: {}",
                            e
                        )),
                        line!(),
                        file!(),
                    )
                })?;

                debug!(
                    shapes = canvas.shapes().len(),
                    detections = canvas.detections().len(),
                    "Loaded legacy canvas"
                );

                Ok(ProjectFormat::Legacy(Box::new(canvas)))
            }
            2 => {
                // New format - parse as DrawingInstance
                info!(version, "Loading as multi-page DrawingInstance format");
                let instance: DrawingInstance = serde_json::from_str(json).map_err(|e| {
                    InstanceError::new(
                        InstanceErrorKind::Deserialization(format!(
                            "Failed to parse instance: {}",
                            e
                        )),
                        line!(),
                        file!(),
                    )
                })?;

                debug!(
                    page_count = instance.page_count(),
                    template_id = instance.template_id(),
                    "Loaded DrawingInstance"
                );

                Ok(ProjectFormat::Instance(Box::new(instance)))
            }
            _ => {
                // Unknown version
                Err(InstanceError::new(
                    InstanceErrorKind::InvalidInstance(format!("Unsupported version: {}", version)),
                    line!(),
                    file!(),
                ))
            }
        }
    }

    /// Convert to DrawingInstance
    ///
    /// If already an instance, returns it unchanged.
    /// If legacy format, converts the canvas to a single-page instance.
    #[instrument(skip(self))]
    pub fn into_instance(self) -> DrawingInstance {
        match self {
            ProjectFormat::Instance(instance) => {
                debug!("Already DrawingInstance format");
                *instance
            }
            ProjectFormat::Legacy(canvas) => {
                info!("Migrating legacy canvas to DrawingInstance");
                migrate_canvas_to_instance(*canvas)
            }
        }
    }
}

/// Migrate a legacy DrawingCanvas to a multi-page DrawingInstance
///
/// Creates a new DrawingInstance with:
/// - A single page containing the legacy canvas
/// - Default template ID ("legacy")
/// - Preserved project name as instance name
/// - All shapes, detections, and settings preserved
#[instrument(skip(canvas), fields(
    project_name = %canvas.project_name(),
    shapes = canvas.shapes().len(),
    detections = canvas.detections().len(),
))]
pub fn migrate_canvas_to_instance(canvas: DrawingCanvas) -> DrawingInstance {
    debug!("Starting legacy canvas migration");

    // Extract project name for instance name
    let project_name = canvas.project_name().to_string();

    // Create a single page from the canvas
    let page = FormPage::from_canvas(0, canvas);

    // Create instance with legacy template ID and single page
    let mut instance = DrawingInstance::from_single_page(LEGACY_TEMPLATE_ID, page);

    // Set instance name from original project name
    instance.set_instance_name(&project_name);

    // Add metadata to track migration
    instance.add_metadata("migrated_from", "legacy_canvas");
    instance.add_metadata(
        "migration_date",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    );
    instance.add_metadata("original_project_name", &project_name);

    info!(
        project_name,
        page_count = instance.page_count(),
        "Migrated legacy canvas to single-page instance"
    );

    instance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_legacy_no_version() {
        let json = r#"{"project_name": "Test", "shapes": []}"#;
        let version = ProjectFormat::detect_version(json).unwrap();
        assert_eq!(version, 0);
    }

    #[test]
    fn test_detect_version_legacy_explicit() {
        let json = r#"{"version": 1, "project_name": "Test", "shapes": []}"#;
        let version = ProjectFormat::detect_version(json).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_detect_version_instance() {
        let json = r#"{"version": 2, "template_id": "test", "pages": []}"#;
        let version = ProjectFormat::detect_version(json).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_detect_version_invalid_json() {
        let json = "not json";
        assert!(ProjectFormat::detect_version(json).is_err());
    }
}
