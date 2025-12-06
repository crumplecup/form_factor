//! Application mode management for form factor UI.
//!
//! Provides state machine for managing transitions between different
//! application modes (canvas, template manager, template editor, instance filling).

use crate::{DrawingInstance, DrawingTemplate};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Application modes defining the current UI state and available operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppMode {
    /// Default canvas mode with drawing tools and plugins
    Canvas,
    /// Template library browser and management
    TemplateManager,
    /// Template editor for creating/modifying templates
    TemplateEditor,
    /// Instance filling mode for data entry
    InstanceFilling,
    /// Instance viewing mode for reviewing completed forms
    InstanceViewing,
}

impl fmt::Display for AppMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppMode::Canvas => write!(f, "Canvas"),
            AppMode::TemplateManager => write!(f, "Template Manager"),
            AppMode::TemplateEditor => write!(f, "Template Editor"),
            AppMode::InstanceFilling => write!(f, "Fill Form"),
            AppMode::InstanceViewing => write!(f, "View Form"),
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Canvas
    }
}

/// Application state managing current mode and associated data.
#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
pub struct AppState {
    /// Current application mode
    mode: AppMode,
    /// Previous mode for navigation history
    previous_mode: Option<AppMode>,
    /// Currently active template (in editor or for instance filling)
    current_template: Option<DrawingTemplate>,
    /// Currently active instance (in filling or viewing mode)
    current_instance: Option<DrawingInstance>,
    /// Whether there are unsaved changes
    has_unsaved_changes: bool,
}

impl AppState {
    /// Creates a new application state in Canvas mode.
    pub fn new() -> Self {
        Self {
            mode: AppMode::Canvas,
            previous_mode: None,
            current_template: None,
            current_instance: None,
            has_unsaved_changes: false,
        }
    }

    /// Sets the current mode and updates history.
    pub fn set_mode(&mut self, mode: AppMode) {
        if self.mode != mode {
            self.previous_mode = Some(self.mode);
            self.mode = mode;
        }
    }

    /// Returns to the previous mode if one exists.
    ///
    /// Returns the new mode, or None if there was no previous mode.
    pub fn go_back(&mut self) -> Option<AppMode> {
        if let Some(prev) = self.previous_mode {
            self.mode = prev;
            self.previous_mode = None;
            Some(prev)
        } else {
            None
        }
    }

    /// Sets the current template.
    pub fn set_current_template(&mut self, template: Option<DrawingTemplate>) {
        self.current_template = template;
    }

    /// Sets the current instance.
    pub fn set_current_instance(&mut self, instance: Option<DrawingInstance>) {
        self.current_instance = instance;
    }

    /// Marks the state as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.has_unsaved_changes = true;
    }

    /// Clears the unsaved changes flag.
    pub fn mark_clean(&mut self) {
        self.has_unsaved_changes = false;
    }

    /// Checks if it's safe to change modes (no unsaved changes).
    pub fn can_change_mode(&self) -> bool {
        !self.has_unsaved_changes
    }

    /// Validates a mode transition.
    ///
    /// Returns Ok if the transition is valid, Err with a message if not.
    pub fn validate_transition(&self, new_mode: AppMode) -> Result<(), String> {
        // Check for unsaved changes
        if self.has_unsaved_changes && new_mode != self.mode {
            return Err("Cannot switch modes with unsaved changes. Save or discard first.".to_string());
        }

        // Validate state requirements for target mode
        match new_mode {
            AppMode::TemplateEditor => {
                if self.current_template.is_none() {
                    return Err("No template selected for editing".to_string());
                }
            }
            AppMode::InstanceFilling | AppMode::InstanceViewing => {
                if self.current_instance.is_none() {
                    return Err("No instance selected".to_string());
                }
            }
            AppMode::Canvas | AppMode::TemplateManager => {
                // These modes don't require specific state
            }
        }

        Ok(())
    }

    /// Attempts to transition to a new mode.
    ///
    /// Returns Ok if successful, Err with a message if the transition is invalid.
    pub fn transition_to(&mut self, new_mode: AppMode) -> Result<(), String> {
        self.validate_transition(new_mode)?;
        self.set_mode(new_mode);
        Ok(())
    }

    /// Resets the application state to default (Canvas mode).
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        let state = AppState::new();
        assert_eq!(*state.mode(), AppMode::Canvas);
        assert_eq!(*state.previous_mode(), None);
        assert!(!state.has_unsaved_changes());
    }

    #[test]
    fn test_mode_transition() {
        let mut state = AppState::new();
        
        state.set_mode(AppMode::TemplateManager);
        assert_eq!(*state.mode(), AppMode::TemplateManager);
        assert_eq!(*state.previous_mode(), Some(AppMode::Canvas));
    }

    #[test]
    fn test_go_back() {
        let mut state = AppState::new();
        
        state.set_mode(AppMode::TemplateManager);
        let prev = state.go_back();
        
        assert_eq!(prev, Some(AppMode::Canvas));
        assert_eq!(*state.mode(), AppMode::Canvas);
        assert_eq!(*state.previous_mode(), None);
    }

    #[test]
    fn test_unsaved_changes_blocking() {
        let mut state = AppState::new();
        state.mark_dirty();
        
        let result = state.transition_to(AppMode::TemplateManager);
        assert!(result.is_err());
        assert_eq!(*state.mode(), AppMode::Canvas); // Mode unchanged
    }

    #[test]
    fn test_can_change_mode_with_clean_state() {
        let state = AppState::new();
        assert!(state.can_change_mode());
    }

    #[test]
    fn test_cannot_change_mode_with_dirty_state() {
        let mut state = AppState::new();
        state.mark_dirty();
        assert!(!state.can_change_mode());
    }
}
