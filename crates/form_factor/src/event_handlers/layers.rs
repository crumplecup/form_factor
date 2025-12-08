//! Layer event handlers

use crate::type_conversions::LayerParser;
use form_factor_drawing::{
    AppMode, AppState, DataEntryPanel, DrawingCanvas, InstanceManagerPanel, LayerType,
};

/// Layer event handler
pub struct LayerEventHandler;

impl LayerEventHandler {
    /// Handle layer visibility changed
    #[tracing::instrument(skip(canvas), fields(layer_name, visible))]
    pub fn handle_visibility_changed(canvas: &mut DrawingCanvas, layer_name: &str, visible: bool) {
        tracing::debug!(layer_name, visible, "Layer visibility changed");

        if let Some(layer_type) = LayerParser::from_name(layer_name)
            && canvas.layer_manager().is_visible(layer_type) != visible
        {
            canvas.layer_manager_mut().toggle_layer(layer_type);
        }
    }

    /// Handle layer selected
    #[tracing::instrument(skip(canvas), fields(layer_name))]
    pub fn handle_selected(canvas: &mut DrawingCanvas, layer_name: &str) {
        tracing::debug!(layer_name, "Layer selected");

        let layer_type = LayerParser::from_name(layer_name);
        canvas.with_selected_layer(layer_type);
    }

    /// Handle layer clear requested
    #[tracing::instrument(
        skip(canvas, app_state, instance_manager_panel, data_entry_panel),
        fields(layer_name)
    )]
    pub fn handle_clear_requested(
        canvas: &mut DrawingCanvas,
        app_state: &mut AppState,
        layer_name: &str,
        instance_manager_panel: &mut Option<InstanceManagerPanel>,
        data_entry_panel: &mut Option<DataEntryPanel>,
    ) {
        tracing::debug!(layer_name, "Layer clear requested");

        if let Some(layer_type) = LayerParser::from_name(layer_name) {
            match layer_type {
                LayerType::Shapes => {
                    canvas.clear_shapes();
                    tracing::info!("Cleared shapes layer");
                }
                LayerType::Detections => {
                    canvas.clear_detections();
                    tracing::info!("Cleared detections layer");
                }
                LayerType::Canvas => {
                    canvas.clear_canvas_image();
                    tracing::info!("Cleared canvas image");
                }
                LayerType::Template => {
                    tracing::info!("Clearing template layer");

                    // Clear current template from app state
                    app_state.set_current_template(None);

                    // If in template-related mode, return to Canvas
                    match app_state.mode() {
                        AppMode::TemplateEditor | AppMode::TemplateManager => {
                            tracing::info!("Exiting template mode due to layer clear");
                            app_state.set_mode(AppMode::Canvas);
                            app_state.mark_clean();
                        }
                        _ => {}
                    }

                    // Clear template manager panel if it exists
                    if instance_manager_panel.is_some() {
                        tracing::debug!("Clearing instance manager panel");
                        *instance_manager_panel = None;
                    }
                }
                LayerType::Instance => {
                    tracing::info!("Clearing instance layer");

                    // Clear current instance from app state
                    app_state.set_current_instance(None);

                    // Clear data entry panel if it exists
                    if data_entry_panel.is_some() {
                        tracing::debug!("Clearing data entry panel");
                        *data_entry_panel = None;
                    }

                    // If in instance filling mode, return to previous mode
                    if *app_state.mode() == AppMode::InstanceFilling {
                        tracing::info!("Exiting instance filling mode due to layer clear");
                        app_state.go_back();
                        app_state.mark_clean();
                    }
                }
                LayerType::Grid => {
                    // Grid doesn't need clearing
                }
            }
        }
    }
}
