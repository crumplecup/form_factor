//! Recent projects tracking
//!
//! Maintains a list of recently opened project files with automatic
//! persistence to platform-specific config directories.

use form_factor_core::{IoError, IoOperation};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, instrument, warn};

/// Maximum number of recent projects to track
const MAX_RECENT_PROJECTS: usize = 10;

/// Application name for config directory
const APP_NAME: &str = "form_factor";

/// Recent projects list
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct RecentProjects {
    /// List of recent project paths (most recent first)
    #[serde(default)]
    projects: Vec<PathBuf>,
}

impl RecentProjects {
    /// Create a new empty recent projects list
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    /// Get the list of recent project paths (most recent first)
    pub fn projects(&self) -> &[PathBuf] {
        &self.projects
    }

    /// Get the number of recent projects
    pub fn len(&self) -> usize {
        self.projects.len()
    }

    /// Check if the recent projects list is empty
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }

    /// Add a project to the recent list (moves to front if already exists)
    ///
    /// If the path is already in the list, it is moved to the front.
    /// The list is automatically truncated to maintain at most 10 entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use form_factor_drawing::RecentProjects;
    /// use std::path::PathBuf;
    ///
    /// let mut recent = RecentProjects::new();
    /// recent.add(PathBuf::from("/path/to/project1.json"));
    /// recent.add(PathBuf::from("/path/to/project2.json"));
    /// assert_eq!(recent.len(), 2);
    /// assert_eq!(recent.most_recent(), Some(&PathBuf::from("/path/to/project2.json")));
    /// ```
    #[instrument(skip(self), fields(path = ?path, current_count = self.projects.len()))]
    pub fn add(&mut self, path: PathBuf) {
        // Remove if already in list
        self.projects.retain(|p| p != &path);

        // Add to front
        self.projects.insert(0, path);

        // Keep only MAX_RECENT_PROJECTS
        self.projects.truncate(MAX_RECENT_PROJECTS);
    }

    /// Get the most recent project path
    pub fn most_recent(&self) -> Option<&PathBuf> {
        self.projects.first()
    }

    /// Load recent projects from config file
    ///
    /// Returns a default empty list if the config file doesn't exist or cannot be read.
    /// Errors are logged but not propagated.
    #[instrument]
    pub fn load() -> Self {
        let config_path = Self::config_path();

        match std::fs::read_to_string(&config_path) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(recent) => {
                    debug!(path = ?config_path, "Loaded recent projects");
                    recent
                }
                Err(e) => {
                    warn!(path = ?config_path, error = %e, "Failed to parse recent projects config, starting fresh");
                    Self::new()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!("No recent projects config found, starting fresh");
                Self::new()
            }
            Err(e) => {
                warn!(path = ?config_path, error = %e, "Failed to read recent projects config");
                Self::new()
            }
        }
    }

    /// Save recent projects to config file
    ///
    /// # Errors
    ///
    /// Returns `IoError` if:
    /// - Config directory cannot be created
    /// - Serialization fails
    /// - File write fails
    #[instrument(skip(self), fields(count = self.projects.len()))]
    pub fn save(&self) -> Result<(), IoError> {
        let config_path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                IoError::new(
                    format!("Failed to create config directory: {}", e),
                    parent.to_string_lossy().to_string(),
                    IoOperation::Create,
                    line!(),
                    file!(),
                )
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            IoError::new(
                format!("Failed to serialize recent projects: {}", e),
                config_path.to_string_lossy().to_string(),
                IoOperation::Write,
                line!(),
                file!(),
            )
        })?;

        std::fs::write(&config_path, json).map_err(|e| {
            IoError::new(
                format!("Failed to write recent projects config: {}", e),
                config_path.to_string_lossy().to_string(),
                IoOperation::Write,
                line!(),
                file!(),
            )
        })?;

        debug!(path = ?config_path, count = self.projects.len(), "Saved recent projects");
        Ok(())
    }

    /// Get the config file path
    ///
    /// Returns a platform-specific path:
    /// - Linux: `$XDG_CONFIG_HOME/form_factor/recent_projects.json` or `~/.config/form_factor/recent_projects.json`
    /// - macOS: `~/Library/Application Support/form_factor/recent_projects.json`
    /// - Windows: `%APPDATA%\form_factor\recent_projects.json`
    fn config_path() -> PathBuf {
        // Use platform-specific config directory
        let config_dir = if cfg!(target_os = "linux") {
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut home =
                        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from(".")));
                    home.push(".config");
                    home
                })
        } else if cfg!(target_os = "macos") {
            let mut home =
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from(".")));
            home.push("Library");
            home.push("Application Support");
            home
        } else if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
        } else {
            PathBuf::from(".")
        };

        let mut path = config_dir;
        path.push(APP_NAME);
        path.push("recent_projects.json");
        path
    }
}
