//! Form Factor - GUI application for tagging scanned forms with OCR metadata

mod detection_results;
mod detection_tasks;
mod event_handlers;
mod file_dialogs;
mod plugin_setup;
mod type_conversions;
mod ui_properties;
mod ui_template;
mod ui_update;

use form_factor::{App, AppContext, DrawingCanvas};
#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use form_factor_drawing::Shape;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
#[cfg(feature = "text-detection")]
use detection_tasks::TextDetectionTask;
#[cfg(feature = "logo-detection")]
use detection_tasks::LogoDetectionTask;
#[cfg(feature = "ocr")]
use detection_tasks::OcrExtractionTask;
use event_handlers::FileEventHandler;
use file_dialogs::FileDialogs;
#[cfg(feature = "plugins")]
use plugin_setup::PluginSetup;
use type_conversions::{LayerParser, ToolParser};
use ui_properties::PropertyRenderer;

#[cfg(feature = "backend-eframe")]
use form_factor::{Backend, BackendConfig, EframeBackend};

#[cfg(feature = "plugins")]
use form_factor_plugins::TemplateBrowserPlugin;

/// Main application struct
struct FormFactorApp {
    name: String,
    canvas: DrawingCanvas,
    app_state: form_factor::AppState,
    mode_switcher: form_factor::ModeSwitcher,
    /// Data entry panel for instance filling (created when entering InstanceFilling mode)
    data_entry_panel: Option<form_factor::DataEntryPanel>,
    /// Instance manager panel for creating and managing instances
    instance_manager_panel: Option<form_factor::InstanceManagerPanel>,
    /// Template browser for template management
    #[cfg(feature = "plugins")]
    template_browser: Option<TemplateBrowserPlugin>,
    /// Toast notifications for user feedback
    toasts: egui_notify::Toasts,
    #[cfg(feature = "plugins")]
    plugin_manager: form_factor::PluginManager,
    /// Previous selected shape index for change detection
    prev_selected_shape: Option<usize>,
    /// Previous selected detection for change detection
    prev_selected_detection: Option<(form_factor_drawing::DetectionType, usize)>,
    /// Field type selector for assigning fields to shapes/detections
    field_type_selector: Option<form_factor_drawing::FieldTypeSelector>,
    /// Whether field type selector dialog is open
    show_field_selector: bool,
}

impl FormFactorApp {
    fn new() -> Self {
        #[cfg(feature = "plugins")]
        let plugin_manager = PluginSetup::create_manager();

        Self {
            name: String::from("Form Factor"),
            canvas: DrawingCanvas::new(),
            app_state: form_factor::AppState::new(),
            mode_switcher: form_factor::ModeSwitcher::new(),
            data_entry_panel: None,
            instance_manager_panel: None,
            #[cfg(feature = "plugins")]
            template_browser: None,
            toasts: egui_notify::Toasts::default(),
            #[cfg(feature = "plugins")]
            plugin_manager,
            prev_selected_shape: None,
            prev_selected_detection: None,
            field_type_selector: None,
            show_field_selector: false,
        }
    }

    /// Render the instance filling mode layout
    fn render_instance_filling_mode(&mut self, ctx: &AppContext) {
        ui_update::update_instance_filling_mode(&mut self.app_state, &mut self.data_entry_panel, ctx);
    }

    /// Render the template manager mode layout
    fn render_template_manager_mode(&mut self, ctx: &AppContext) {
        #[cfg(feature = "plugins")]
        ui_template::update_template_manager_mode(
            &self.app_state,
            &mut self.template_browser,
            &mut self.canvas,
            ctx,
        );

        #[cfg(not(feature = "plugins"))]
        ui_template::update_template_manager_mode(ctx);
    }

}

impl App for FormFactorApp {
    fn setup(&mut self, ctx: &egui::Context) {
        // Try to load the most recent project (defers image loading)
        match self.canvas.load_recent_on_startup(ctx) {
            Ok(()) => {
                tracing::info!("Auto-loaded most recent project");
            }
            Err(e) => {
                tracing::debug!("No recent project to load: {}", e);
                tracing::info!("Starting with default workspace");
            }
        }

        tracing::info!("Application setup complete");
    }

    fn update(&mut self, ctx: &AppContext) {
        // Show mode switcher in top panel
        egui::TopBottomPanel::top("mode_panel").show(ctx.egui_ctx(), |ui| {
            ui.horizontal(|ui| {
                self.mode_switcher.ui(ui, &mut self.app_state);
            });
        });
        
        // Process plugin events and wire them to canvas operations
        #[cfg(feature = "plugins")]
        {
            // First, drain events for the application to handle
            // This must happen BEFORE process_events() which also drains
            let events = self.plugin_manager.event_bus_mut().drain_events();

            // Handle application events
            for event in &events {
                use form_factor::AppEvent;
                match event {
                    AppEvent::CanvasZoomChanged { zoom } => {
                        self.canvas.set_zoom(*zoom);
                    }
                    AppEvent::CanvasPanChanged { x, y } => {
                        self.canvas.set_pan_offset(*x, *y);
                    }
                    AppEvent::ToolSelected { tool_name } => {
                        // Parse tool name and set tool mode
                        if let Some(tool) = ToolParser::from_name(tool_name) {
                            self.canvas.set_tool(tool);
                        }
                    }
                    AppEvent::LayerVisibilityChanged {
                        layer_name,
                        visible,
                    } => {
                        // Find layer by name and toggle
                        if let Some(layer_type) = LayerParser::from_name(layer_name)
                            && self.canvas.layer_manager().is_visible(layer_type) != *visible
                        {
                            self.canvas.layer_manager_mut().toggle_layer(layer_type);
                        }
                    }
                    AppEvent::LayerSelected { layer_name } => {
                        let layer_type = LayerParser::from_name(layer_name);
                        self.canvas.with_selected_layer(layer_type);
                    }
                    AppEvent::LayerClearRequested { layer_name } => {
                        if let Some(layer_type) = LayerParser::from_name(layer_name) {
                            match layer_type {
                                LayerType::Shapes => {
                                    self.canvas.clear_shapes();
                                    tracing::info!("Cleared shapes layer");
                                }
                                LayerType::Detections => {
                                    self.canvas.clear_detections();
                                    tracing::info!("Cleared detections layer");
                                }
                                LayerType::Canvas => {
                                    self.canvas.clear_canvas_image();
                                    tracing::info!("Cleared canvas image");
                                }
                                LayerType::Template => {
                                    tracing::info!("Clearing template layer");

                                    // Clear current template from app state
                                    self.app_state.set_current_template(None);

                                    // If in template-related mode, return to Canvas
                                    match self.app_state.mode() {
                                        form_factor::AppMode::TemplateEditor
                                        | form_factor::AppMode::TemplateManager => {
                                            tracing::info!(
                                                "Exiting template mode due to layer clear"
                                            );
                                            self.app_state.set_mode(form_factor::AppMode::Canvas);
                                            self.app_state.mark_clean();
                                        }
                                        _ => {}
                                    }

                                    // Clear template manager panel if it exists
                                    if self.instance_manager_panel.is_some() {
                                        tracing::debug!("Clearing instance manager panel");
                                        self.instance_manager_panel = None;
                                    }
                                }
                                LayerType::Instance => {
                                    tracing::info!("Clearing instance layer");

                                    // Clear current instance from app state
                                    self.app_state.set_current_instance(None);

                                    // Clear data entry panel if it exists
                                    if self.data_entry_panel.is_some() {
                                        tracing::debug!("Clearing data entry panel");
                                        self.data_entry_panel = None;
                                    }

                                    // If in instance filling mode, return to previous mode
                                    if *self.app_state.mode()
                                        == form_factor::AppMode::InstanceFilling
                                    {
                                        tracing::info!(
                                            "Exiting instance filling mode due to layer clear"
                                        );
                                        self.app_state.go_back();
                                        self.app_state.mark_clean();
                                    }
                                }
                                LayerType::Grid => {
                                    // Grid doesn't need clearing
                                }
                            }
                        }
                    }
                    AppEvent::ObjectDeleteRequested {
                        layer_type,
                        object_index,
                    } => {
                        use form_factor::LayerType;
                        tracing::info!(?layer_type, object_index, "Deleting object from layer");
                        match layer_type {
                            LayerType::Shapes => {
                                self.canvas.delete_shape(*object_index);
                            }
                            LayerType::Detections => {
                                self.canvas.delete_detection(*object_index);
                            }
                            _ => {
                                tracing::warn!(
                                    ?layer_type,
                                    "Layer does not support object deletion"
                                );
                            }
                        }
                    }
                    AppEvent::ObjectVisibilityChanged {
                        layer_type,
                        object_index,
                        visible,
                    } => {
                        use form_factor::LayerType;
                        tracing::info!(
                            ?layer_type,
                            object_index,
                            visible,
                            "Changing object visibility"
                        );
                        match layer_type {
                            LayerType::Shapes => {
                                if let Err(e) =
                                    self.canvas.set_shape_visibility(*object_index, *visible)
                                {
                                    tracing::error!("Failed to set shape visibility: {}", e);
                                }
                            }
                            LayerType::Detections => {
                                if let Err(e) = self
                                    .canvas
                                    .set_detection_visibility(*object_index, *visible)
                                {
                                    tracing::error!("Failed to set detection visibility: {}", e);
                                }
                            }
                            _ => {
                                tracing::warn!(
                                    ?layer_type,
                                    "Layer does not support object visibility"
                                );
                            }
                        }
                    }
                    #[cfg(feature = "plugin-layers")]
                    AppEvent::OcrObjectDeleteRequested { index } => {
                        tracing::info!(index, "Deleting OCR detection");
                        self.canvas.delete_ocr_detection(*index);
                    }
                    #[cfg(feature = "plugin-layers")]
                    AppEvent::OcrObjectVisibilityChanged { index, visible } => {
                        tracing::info!(index, visible, "Changing OCR detection visibility");
                        if let Err(e) = self.canvas.set_ocr_detection_visibility(*index, *visible) {
                            tracing::error!("Failed to change OCR detection visibility: {}", e);
                        }
                    }
                    AppEvent::OpenFileRequested => {
                        FileEventHandler::handle_open_requested(
                            &mut self.canvas,
                            self.plugin_manager.event_bus().sender(),
                            ctx.egui_ctx(),
                        );
                    }
                    AppEvent::SaveFileRequested => {
                        FileEventHandler::handle_save_requested(
                            &mut self.canvas,
                            self.plugin_manager.event_bus().sender(),
                            self.canvas.project_name(),
                        );
                    }
                    AppEvent::SaveAsRequested => {
                        FileEventHandler::handle_save_as_requested(
                            &mut self.canvas,
                            self.plugin_manager.event_bus().sender(),
                            self.canvas.project_name(),
                        );
                    }
                    AppEvent::LoadImageRequested => {
                        FileEventHandler::handle_load_image_requested(
                            &mut self.canvas,
                            ctx.egui_ctx(),
                        );
                    }
                    #[cfg(feature = "text-detection")]
                    AppEvent::TextDetectionRequested => {
                        // Show toast immediately that detection started
                        self.toasts.info("Text detection started...");

                        // Get form image path for background thread
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            let sender = self.plugin_manager.event_bus().sender();
                            TextDetectionTask::spawn(form_path, sender);
                        } else {
                            tracing::error!("No form image loaded for text detection");
                            self.plugin_manager.event_bus().sender().emit(
                                AppEvent::DetectionFailed {
                                    detection_type: "text".to_string(),
                                    error: "No form image loaded".to_string(),
                                },
                            );
                        }
                    }
                    #[cfg(feature = "logo-detection")]
                    AppEvent::LogoDetectionRequested => {
                        // Show toast immediately that detection started
                        self.toasts.info("Logo detection started...");

                        // Get form image path for background thread
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            let sender = self.plugin_manager.event_bus().sender();
                            LogoDetectionTask::spawn(form_path, sender);
                        } else {
                            tracing::error!("No form image loaded for logo detection");
                            self.plugin_manager.event_bus().sender().emit(
                                AppEvent::DetectionFailed {
                                    detection_type: "logo".to_string(),
                                    error: "No form image loaded".to_string(),
                                },
                            );
                        }
                    }
                    #[cfg(feature = "ocr")]
                    AppEvent::OcrExtractionRequested => {
                        // Show toast immediately that OCR started
                        self.toasts.info("OCR extraction started...");

                        // Get form image path and detections for background thread
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            // Clone detections to pass to background thread
                            let detections: Vec<Shape> = self.canvas.detections().to_vec();
                            let sender = self.plugin_manager.event_bus().sender();
                            
                            OcrExtractionTask::spawn(form_path, detections, sender);
                        } else {
                            self.toasts.error("No image loaded");
                        }
                    }
                    #[cfg(feature = "ocr")]
                    AppEvent::OcrComplete { results_json } => {
                        // Deserialize results from JSON
                        match serde_json::from_str::<Vec<(Shape, String)>>(results_json) {
                            Ok(results) => {
                                tracing::info!("Extracted text from {} detections", results.len());

                                // Clear old OCR detections and add new ones with text
                                self.canvas.clear_ocr_detections();
                                for (shape, text) in results {
                                    self.canvas.add_ocr_detection(shape, text);
                                }

                                // Show success toast
                                self.toasts.success(format!(
                                    "OCR complete: extracted text from {} region{}",
                                    self.canvas.ocr_detections().len(),
                                    if self.canvas.ocr_detections().len() == 1 {
                                        ""
                                    } else {
                                        "s"
                                    }
                                ));
                            }
                            Err(e) => {
                                tracing::error!("Failed to deserialize OCR results: {}", e);
                                self.toasts.error(format!("OCR processing failed: {}", e));
                            }
                        }
                    }
                    AppEvent::DetectionComplete {
                        count,
                        detection_type,
                    } => {
                        // Show success toast with count
                        self.toasts.success(format!(
                            "{} detection complete: found {} region{}",
                            match detection_type.as_str() {
                                "text" => "Text",
                                "logo" => "Logo",
                                _ => "Detection",
                            },
                            count,
                            if *count == 1 { "" } else { "s" }
                        ));
                    }
                    AppEvent::DetectionFailed {
                        detection_type,
                        error,
                    } => {
                        // Show error toast
                        self.toasts.error(format!(
                            "{} detection failed: {}",
                            match detection_type.as_str() {
                                "text" => "Text",
                                "logo" => "Logo",
                                _ => "Detection",
                            },
                            error
                        ));
                    }
                    AppEvent::DetectionResultsReady {
                        detection_type,
                        shapes_json,
                    } => {
                        // Deserialize shapes and add to canvas detections
                        match serde_json::from_str::<Vec<form_factor::Shape>>(shapes_json) {
                            Ok(shapes) => {
                                tracing::info!(
                                    "Received {} {} detection results",
                                    shapes.len(),
                                    detection_type
                                );
                                for shape in shapes {
                                    self.canvas.add_detection(shape);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to deserialize detection results: {}", e);
                            }
                        }
                    }
                    AppEvent::CanvasImageVisibilityChanged { visible } => {
                        self.canvas.with_form_image_visible(*visible);
                        tracing::debug!(visible = visible, "Canvas image visibility changed");
                    }
                    AppEvent::CanvasImageLockChanged { locked } => {
                        self.canvas.with_form_image_locked(*locked);
                        tracing::debug!(locked = locked, "Canvas image lock state changed");
                    }
                    AppEvent::CanvasImageClearRequested => {
                        self.canvas.with_form_image_path(None);
                        tracing::info!("Canvas image cleared");
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }

            // Now distribute those same events to plugins for their reaction
            // Re-emit them so plugins can process them
            for event in events {
                self.plugin_manager.event_bus().sender().emit(event);
            }

            // Process plugin events (which now includes the re-emitted events)
            self.plugin_manager.process_events();
        }

        // Top panel with mode switcher
        egui::TopBottomPanel::top("mode_switcher_panel").show(ctx.egui_ctx(), |ui| {
            ui.add_space(4.0);
            self.mode_switcher.ui(ui, &mut self.app_state);
            ui.add_space(4.0);
        });

        // Render mode-specific layout
        match self.app_state.mode() {
            form_factor::AppMode::InstanceFilling => {
                self.render_instance_filling_mode(ctx);
            }
            form_factor::AppMode::TemplateManager => {
                self.render_template_manager_mode(ctx);
            }
            _ => {
                // Default layout: plugin sidebar + canvas
                #[cfg(feature = "plugins")]
                egui::SidePanel::right("plugin_panel")
                    .default_width(280.0)
                    .show(ctx.egui_ctx(), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            self.plugin_manager.render_plugins(ui, &self.canvas);

                            // Show property editor when something is selected
                            if *self.canvas.show_properties() {
                                ui.separator();
                                ui.heading("Properties");

                                if let Some(shape_idx) = *self.canvas.selected_shape() {
                                    if let Err(e) = PropertyRenderer::render_shape_properties(
                                        ui,
                                        &self.canvas,
                                        shape_idx,
                                    ) {
                                        tracing::error!(error = %e, "Failed to render shape properties");
                                        ui.label(format!("Error: {}", e));
                                    } else {
                                        // Show field type selector button (maintained in FormFactorApp for state)
                                        if ui.button("Assign to Field...").clicked() {
                                            self.show_field_selector = true;
                                            if self.field_type_selector.is_none() {
                                                self.field_type_selector =
                                                    Some(form_factor_drawing::FieldTypeSelector::new());
                                            }
                                        }
                                    }
                                } else if let Some((det_type, det_idx)) =
                                    *self.canvas.selected_detection()
                                {
                                    if let Err(e) = PropertyRenderer::render_detection_properties(
                                        ui,
                                        &self.canvas,
                                        det_type,
                                        det_idx,
                                    ) {
                                        tracing::error!(error = %e, "Failed to render detection properties");
                                        ui.label(format!("Error: {}", e));
                                    } else {
                                        // Show field type selector button (maintained in FormFactorApp for state)
                                        if ui.button("Assign to Field...").clicked() {
                                            self.show_field_selector = true;
                                            if self.field_type_selector.is_none() {
                                                self.field_type_selector =
                                                    Some(form_factor_drawing::FieldTypeSelector::new());
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    });

                egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
                    self.canvas.ui(ui);
                });
            }
        }

        // Check for selection changes and emit events
        #[cfg(feature = "plugins")]
        self.handle_selection_changes();

        // Show field type selector dialog if open
        if self.show_field_selector {
            if let Some(selector) = &mut self.field_type_selector {
                let mut should_close = false;
                egui::Window::new("Select Field Type")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(400.0)
                    .show(ctx.egui_ctx(), |ui| {
                        selector.show(ui);
                        
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
                            
                            if ui.button("Assign").clicked() && selector.selected().is_some() {
                                // TODO: Actually assign the field to the selected shape/detection
                                tracing::info!("Assigned field type: {:?}", selector.selected());
                                should_close = true;
                            }
                        });
                    });
                    
                if should_close {
                    self.show_field_selector = false;
                }
            }
        }

        // Render toast notifications (shown on top of everything)
        self.toasts.show(ctx.egui_ctx());
    }

    fn on_exit(&mut self) {
        tracing::info!("Application exiting");

        #[cfg(feature = "plugins")]
        {
            tracing::info!("Shutting down plugins");
            self.plugin_manager.shutdown();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Helper methods for FormFactorApp
impl FormFactorApp {
    /// Check for selection changes and emit appropriate events
    #[cfg(feature = "plugins")]
    fn handle_selection_changes(&mut self) {
        use form_factor::AppEvent;

        let current_shape = *self.canvas.selected_shape();
        let current_detection = *self.canvas.selected_detection();

        // Check if shape selection changed
        if current_shape != self.prev_selected_shape {
            if let Some(index) = current_shape {
                tracing::debug!(index, "Shape selection changed, emitting event");
                self.plugin_manager
                    .event_bus()
                    .sender()
                    .emit(AppEvent::ShapeSelected { index });
            } else if self.prev_selected_shape.is_some() {
                tracing::debug!("Shape deselected, emitting clear event");
                self.plugin_manager
                    .event_bus()
                    .sender()
                    .emit(AppEvent::SelectionCleared);
            }
            self.prev_selected_shape = current_shape;
        }

        // Check if detection selection changed
        if current_detection != self.prev_selected_detection {
            if let Some((detection_type, index)) = current_detection {
                let detection_id = format!(
                    "{}_{}",
                    match detection_type {
                        form_factor_drawing::DetectionType::Logo => "logo",
                        form_factor_drawing::DetectionType::Text => "text",
                        form_factor_drawing::DetectionType::Ocr => "ocr",
                    },
                    index
                );
                tracing::debug!(detection_id, "Detection selection changed, emitting event");
                self.plugin_manager
                    .event_bus()
                    .sender()
                    .emit(AppEvent::DetectionSelected { detection_id });
            } else if self.prev_selected_detection.is_some() {
                tracing::debug!("Detection deselected, emitting clear event");
                self.plugin_manager
                    .event_bus()
                    .sender()
                    .emit(AppEvent::SelectionCleared);
            }
            self.prev_selected_detection = current_detection;
        }
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file for configuration (RUST_LOG, etc.)
    // This allows setting defaults in .env that can be overridden by environment variables
    let _ = dotenvy::dotenv(); // Ignore error if .env doesn't exist

    // Initialize tracing subscriber with environment filter
    // Priority: environment variable > .env file > default fallback
    // Use RUST_LOG env var to control logging, e.g.:
    // RUST_LOG=form_factor=debug cargo run
    // RUST_LOG=form_factor::drawing=trace cargo run
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "form_factor=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Form Factor application");

    // Run with the backend specified by feature flags
    #[cfg(feature = "backend-eframe")]
    {
        let app = Box::new(FormFactorApp::new());
        let config = BackendConfig::default();
        tracing::info!("Using eframe backend");
        EframeBackend::run(app, config)?;
    }

    // Miniquad backend support - ready for when egui-miniquad updates to egui 0.33+
    // #[cfg(all(feature = "backend-miniquad", not(feature = "backend-eframe")))]
    // {
    //     let app = Box::new(FormFactorApp::new());
    //     let config = BackendConfig::default();
    //     tracing::info!("Using miniquad backend");
    //     MiniquadBackend::run(app, config)?;
    // }

    #[cfg(not(any(feature = "backend-eframe")))]
    {
        compile_error!("At least one backend feature must be enabled (backend-eframe)");
    }

    Ok(())
}
