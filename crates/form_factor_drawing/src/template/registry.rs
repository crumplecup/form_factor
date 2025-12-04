//! Template registry for loading and storing form templates

use super::{
    error::{TemplateError, TemplateErrorKind},
    implementation::DrawingTemplate,
};
use form_factor_core::FormTemplate;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument, warn};

/// Registry for managing form templates
///
/// Templates are stored in:
/// 1. Global config directory: `~/.config/form_factor/templates/`
/// 2. Project-local directory: `<project_dir>/templates/`
///
/// Project-local templates override global templates with the same ID.
pub struct TemplateRegistry {
    /// Loaded templates indexed by ID
    templates: HashMap<String, DrawingTemplate>,

    /// Path to global templates directory
    global_dir: PathBuf,

    /// Optional path to project-local templates directory
    project_dir: Option<PathBuf>,
}

impl TemplateRegistry {
    /// Create a new registry with global config directory
    pub fn new() -> Result<Self, TemplateError> {
        let global_dir = Self::global_templates_dir()?;

        Ok(Self {
            templates: HashMap::new(),
            global_dir,
            project_dir: None,
        })
    }

    /// Set project-local templates directory
    pub fn with_project_dir(mut self, project_dir: PathBuf) -> Self {
        self.project_dir = Some(project_dir);
        self
    }

    /// Get global templates directory path
    ///
    /// Platform-specific locations:
    /// - Linux: `~/.config/form_factor/templates/`
    /// - macOS: `~/Library/Application Support/form_factor/templates/`
    /// - Windows: `%APPDATA%\form_factor\templates\`
    fn global_templates_dir() -> Result<PathBuf, TemplateError> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            TemplateError::new(TemplateErrorKind::ConfigDirNotFound, line!(), file!())
        })?;

        let templates_dir = config_dir.join("form_factor").join("templates");
        std::fs::create_dir_all(&templates_dir).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        Ok(templates_dir)
    }

    /// Load all templates from directories
    ///
    /// Loads global templates first, then project-local templates.
    /// Project templates override global templates with the same ID.
    #[instrument(skip(self))]
    pub fn load_all(&mut self) -> Result<(), TemplateError> {
        // Load global templates first
        self.load_from_directory(&self.global_dir.clone())?;

        // Load project templates (override globals)
        if let Some(ref project_dir) = self.project_dir.clone() {
            let templates_dir = project_dir.join("templates");
            if templates_dir.exists() {
                self.load_from_directory(&templates_dir)?;
            }
        }

        info!("Loaded {} templates", self.templates.len());
        Ok(())
    }

    /// Load templates from a specific directory
    fn load_from_directory(&mut self, dir: &Path) -> Result<(), TemplateError> {
        if !dir.exists() {
            return Ok(());
        }

        debug!("Loading templates from {:?}", dir);

        let entries = std::fs::read_dir(dir).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_template_file(&path) {
                    Ok(template) => {
                        debug!(
                            "Loaded template: {} ({}) from {:?}",
                            template.name(),
                            template.id(),
                            path
                        );
                        self.templates.insert(template.id().to_string(), template);
                    }
                    Err(e) => {
                        warn!("Failed to load template from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single template file
    fn load_template_file(&self, path: &Path) -> Result<DrawingTemplate, TemplateError> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        DrawingTemplate::from_json(&json)
    }

    /// Get a template by ID
    ///
    /// Returns None if template not found.
    pub fn get(&self, template_id: &str) -> Option<&DrawingTemplate> {
        self.templates.get(template_id)
    }

    /// Get a mutable reference to a template by ID
    pub fn get_mut(&mut self, template_id: &str) -> Option<&mut DrawingTemplate> {
        self.templates.get_mut(template_id)
    }

    /// Check if a template exists
    pub fn contains(&self, template_id: &str) -> bool {
        self.templates.contains_key(template_id)
    }

    /// Register a new template
    ///
    /// If a template with the same ID already exists, it will be replaced.
    pub fn register(&mut self, template: DrawingTemplate) {
        let id = template.id().to_string();
        self.templates.insert(id, template);
    }

    /// Save a template to disk (global directory)
    ///
    /// Saves to `~/.config/form_factor/templates/<template_id>.json`
    pub fn save(&self, template: &DrawingTemplate) -> Result<(), TemplateError> {
        self.save_to_global(template)
    }

    /// Save a template to the global templates directory
    pub fn save_to_global(&self, template: &DrawingTemplate) -> Result<(), TemplateError> {
        let filename = format!("{}.json", template.id());
        let path = self.global_dir.join(filename);

        let json = template.to_json().map_err(|e| {
            TemplateError::new(
                TemplateErrorKind::Serialization(e.to_string()),
                line!(),
                file!(),
            )
        })?;

        std::fs::write(&path, json).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        info!("Saved template {} to {:?}", template.id(), path);
        Ok(())
    }

    /// Save a template to the project-local templates directory
    ///
    /// Returns an error if no project directory is set.
    pub fn save_to_project(&self, template: &DrawingTemplate) -> Result<(), TemplateError> {
        let project_dir = self.project_dir.as_ref().ok_or_else(|| {
            TemplateError::new(
                TemplateErrorKind::InvalidTemplate(
                    "No project directory set for project-local save".into(),
                ),
                line!(),
                file!(),
            )
        })?;

        let templates_dir = project_dir.join("templates");
        std::fs::create_dir_all(&templates_dir).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        let filename = format!("{}.json", template.id());
        let path = templates_dir.join(filename);

        let json = template.to_json().map_err(|e| {
            TemplateError::new(
                TemplateErrorKind::Serialization(e.to_string()),
                line!(),
                file!(),
            )
        })?;

        std::fs::write(&path, json).map_err(|e| {
            TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
        })?;

        info!("Saved template {} to project at {:?}", template.id(), path);
        Ok(())
    }

    /// Delete a template from the global directory
    pub fn delete_from_global(&mut self, template_id: &str) -> Result<(), TemplateError> {
        let filename = format!("{}.json", template_id);
        let path = self.global_dir.join(filename);

        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                TemplateError::new(TemplateErrorKind::IoError(e.to_string()), line!(), file!())
            })?;
        }

        self.templates.remove(template_id);
        info!("Deleted template {} from global directory", template_id);
        Ok(())
    }

    /// List all available template IDs
    pub fn list_ids(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// List all templates
    pub fn list(&self) -> Vec<&DrawingTemplate> {
        self.templates.values().collect()
    }

    /// Get number of loaded templates
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// Clear all loaded templates from memory (doesn't delete files)
    pub fn clear(&mut self) {
        self.templates.clear();
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create template registry")
    }
}
