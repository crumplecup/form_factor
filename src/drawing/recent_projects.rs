//! Recent projects tracking

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum number of recent projects to track
const MAX_RECENT_PROJECTS: usize = 10;

/// Recent projects list
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentProjects {
    /// List of recent project paths (most recent first)
    pub projects: Vec<PathBuf>,
}

impl RecentProjects {
    /// Create a new empty recent projects list
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    /// Add a project to the recent list (moves to front if already exists)
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
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if let Ok(json) = std::fs::read_to_string(&config_path)
            && let Ok(recent) = serde_json::from_str(&json)
        {
            tracing::debug!("Loaded recent projects from {:?}", config_path);
            return recent;
        }

        tracing::debug!("No recent projects config found, starting fresh");
        Self::new()
    }

    /// Save recent projects to config file
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize recent projects: {}", e))?;

        std::fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write recent projects config: {}", e))?;

        tracing::debug!("Saved recent projects to {:?}", config_path);
        Ok(())
    }

    /// Get the config file path
    fn config_path() -> PathBuf {
        // Use platform-specific config directory
        let config_dir = if cfg!(target_os = "linux") {
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from(".")));
                    home.push(".config");
                    home
                })
        } else if cfg!(target_os = "macos") {
            let mut home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from(".")));
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
        path.push("form_factor");
        path.push("recent_projects.json");
        path
    }
}
