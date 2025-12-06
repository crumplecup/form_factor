//! Mode switcher UI component for template/instance workflow.
//!
//! Provides toolbar buttons for switching between application modes
//! and handles state transitions with validation.

use crate::{AppMode, AppState};
use tracing::instrument;

/// UI component for mode switching toolbar.
#[derive(Debug, Clone)]
pub struct ModeSwitcher {
    /// Whether to show confirmation dialogs for mode changes
    confirm_mode_changes: bool,
    /// Pending mode change awaiting confirmation
    pending_mode_change: Option<AppMode>,
}

impl ModeSwitcher {
    /// Creates a new mode switcher.
    pub fn new() -> Self {
        Self {
            confirm_mode_changes: true,
            pending_mode_change: None,
        }
    }

    /// Renders the mode switcher toolbar.
    ///
    /// Returns true if the UI was interacted with.
    #[instrument(skip(self, ui, state))]
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut AppState) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.separator();

            // Canvas mode button
            if ui
                .selectable_label(*state.mode() == AppMode::Canvas, "Canvas")
                .clicked()
                && *state.mode() != AppMode::Canvas
            {
                self.request_mode_change(state, AppMode::Canvas);
                changed = true;
            }

            ui.separator();

            // Template Manager button
            if ui
                .selectable_label(
                    *state.mode() == AppMode::TemplateManager,
                    "📋 Templates",
                )
                .on_hover_text("Browse and manage form templates")
                .clicked()
                && *state.mode() != AppMode::TemplateManager
            {
                self.request_mode_change(state, AppMode::TemplateManager);
                changed = true;
            }

            // Template Editor button (only show if template is loaded)
            if state.current_template().is_some()
                && ui
                    .selectable_label(
                        *state.mode() == AppMode::TemplateEditor,
                        "✏ Edit Template",
                    )
                    .on_hover_text("Edit the current template")
                    .clicked()
                    && *state.mode() != AppMode::TemplateEditor
                {
                    self.request_mode_change(state, AppMode::TemplateEditor);
                    changed = true;
                }

            ui.separator();

            // Instance Filling button (only show if instance is loaded)
            if state.current_instance().is_some() {
                if ui
                    .selectable_label(*state.mode() == AppMode::InstanceFilling, "📝 Fill Form")
                    .on_hover_text("Fill out the form fields")
                    .clicked()
                    && *state.mode() != AppMode::InstanceFilling
                {
                    self.request_mode_change(state, AppMode::InstanceFilling);
                    changed = true;
                }

                // Instance Viewing button
                if ui
                    .selectable_label(*state.mode() == AppMode::InstanceViewing, "👁 View Form")
                    .on_hover_text("View completed form")
                    .clicked()
                    && *state.mode() != AppMode::InstanceViewing
                {
                    self.request_mode_change(state, AppMode::InstanceViewing);
                    changed = true;
                }
            }

            // Show unsaved changes indicator
            if *state.has_unsaved_changes() {
                ui.separator();
                ui.label("⚠")
                    .on_hover_text("Unsaved changes - save before switching modes");
            }

            // Back button (if previous mode exists)
            if state.previous_mode().is_some() {
                ui.separator();
                if ui.button("⬅ Back").clicked()
                    && let Some(prev_mode) = state.go_back() {
                        tracing::info!("Returned to {:?} mode", prev_mode);
                        changed = true;
                    }
            }
        });

        // Handle confirmation dialog
        if let Some(pending) = self.pending_mode_change {
            self.show_confirmation_dialog(ui, state, pending);
        }

        changed
    }

    /// Requests a mode change, showing confirmation if needed.
    #[instrument(skip(self, state))]
    fn request_mode_change(&mut self, state: &mut AppState, new_mode: AppMode) {
        // Check if we can change without confirmation
        if !*state.has_unsaved_changes() || !self.confirm_mode_changes {
            match state.transition_to(new_mode) {
                Ok(()) => {
                    tracing::info!("Switched to {:?} mode", new_mode);
                }
                Err(e) => {
                    tracing::warn!("Cannot switch to {:?} mode: {}", new_mode, e);
                }
            }
        } else {
            // Store pending change for confirmation dialog
            self.pending_mode_change = Some(new_mode);
            tracing::debug!("Mode change pending confirmation: {:?}", new_mode);
        }
    }

    /// Shows confirmation dialog for mode changes with unsaved changes.
    #[instrument(skip(self, ui, state))]
    fn show_confirmation_dialog(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        pending: AppMode,
    ) {
        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("You have unsaved changes. What would you like to do?");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save & Continue").clicked() {
                        // TODO: Trigger save operation
                        state.mark_clean();
                        if let Ok(()) = state.transition_to(pending) {
                            tracing::info!("Saved and switched to {:?} mode", pending);
                        }
                        self.pending_mode_change = None;
                    }

                    if ui.button("Discard & Continue").clicked() {
                        state.mark_clean();
                        if let Ok(()) = state.transition_to(pending) {
                            tracing::info!("Discarded changes and switched to {:?} mode", pending);
                        }
                        self.pending_mode_change = None;
                    }

                    if ui.button("Cancel").clicked() {
                        tracing::debug!("Mode change cancelled");
                        self.pending_mode_change = None;
                    }
                });
            });
    }

    /// Sets whether to confirm mode changes when there are unsaved changes.
    pub fn set_confirm_mode_changes(&mut self, confirm: bool) {
        self.confirm_mode_changes = confirm;
    }
}

impl Default for ModeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}
