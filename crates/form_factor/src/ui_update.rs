use crate::AppContext;
use form_factor::{AppState, DataEntryAction, DataEntryPanel};
use tracing::instrument;

/// Handles UI updates for instance filling mode
#[instrument(skip(app_state, data_entry_panel, ctx))]
pub fn update_instance_filling_mode(
    app_state: &mut AppState,
    data_entry_panel: &mut Option<DataEntryPanel>,
    ctx: &AppContext,
) {
    // Create data entry panel if it doesn't exist
    if data_entry_panel.is_none() {
        if let (Some(template), Some(instance)) =
            (app_state.current_template(), app_state.current_instance())
        {
            *data_entry_panel = Some(DataEntryPanel::new(template.clone(), instance.clone()));
            tracing::info!("Created data entry panel");
        } else {
            // No template/instance, return to canvas mode
            tracing::warn!("No template or instance available for filling");
            app_state.set_mode(form_factor::AppMode::Canvas);
            return;
        }
    }

    // Render data entry panel and handle actions
    let mut action = DataEntryAction::None;
    if let Some(panel) = data_entry_panel {
        egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
            action = panel.ui(ui);
        });
    }

    // Handle data entry actions
    handle_data_entry_action(action, app_state, data_entry_panel);
}

/// Handles data entry actions from the panel
#[instrument(skip(app_state, data_entry_panel))]
fn handle_data_entry_action(
    action: DataEntryAction,
    app_state: &mut AppState,
    data_entry_panel: &mut Option<DataEntryPanel>,
) {
    match action {
        DataEntryAction::SaveDraft => {
            tracing::info!("Saving instance draft");
            if let Some(panel) = data_entry_panel {
                let instance_name = panel
                    .instance()
                    .instance_name()
                    .clone()
                    .unwrap_or_else(|| "unnamed".to_string());
                save_instance_draft(&instance_name);
            }
        }
        DataEntryAction::Submit => {
            tracing::info!("Submitting instance");
            let (can_submit, instance_name) = if let Some(panel) = data_entry_panel {
                let valid = panel.validate().is_ok();
                let name = panel
                    .instance()
                    .instance_name()
                    .clone()
                    .unwrap_or_else(|| "unnamed".to_string());
                (valid, name)
            } else {
                (false, String::new())
            };

            if can_submit && save_instance_final(&instance_name) {
                // Clear panel and return to canvas
                *data_entry_panel = None;
                app_state.set_current_instance(None);
                app_state.set_mode(form_factor::AppMode::Canvas);
                app_state.mark_clean();
            }
        }
        DataEntryAction::Cancel => {
            tracing::info!("Cancelling instance filling");
            // Clear panel and return to previous mode
            *data_entry_panel = None;
            app_state.set_current_instance(None);
            app_state.go_back();
            app_state.mark_clean();
        }
        DataEntryAction::None => {
            // No action, update dirty state
            if let Some(panel) = data_entry_panel
                && panel.is_dirty()
            {
                app_state.mark_dirty();
            }
        }
    }
}

/// Save instance as draft (incomplete data allowed)
#[instrument]
fn save_instance_draft(instance_name: &str) {
    // TODO: Implement instance draft saving to file system
    tracing::info!(instance_name, "Saved instance draft");
}

/// Save instance as final submission (validation required)
#[instrument]
fn save_instance_final(instance_name: &str) -> bool {
    // TODO: Implement instance final saving to file system
    tracing::info!(instance_name, "Saved final instance");
    true
}
