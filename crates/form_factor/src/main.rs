//! Form Factor - GUI application for tagging scanned forms with OCR metadata

use form_factor::{App, AppContext, DrawingCanvas};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "backend-eframe")]
use form_factor::{Backend, BackendConfig, EframeBackend};

#[cfg(all(feature = "plugins", feature = "plugin-canvas"))]
use form_factor_plugins::CanvasPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-layers"))]
use form_factor_plugins::LayersPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-file"))]
use form_factor_plugins::FilePlugin;

#[cfg(all(feature = "plugins", feature = "plugin-detection"))]
use form_factor_plugins::DetectionPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-ocr"))]
use form_factor_plugins::OcrPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-properties"))]
use form_factor_plugins::PropertiesPlugin;

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
    /// Toast notifications for user feedback
    toasts: egui_notify::Toasts,
    #[cfg(feature = "plugins")]
    plugin_manager: form_factor::PluginManager,
}

impl FormFactorApp {
    fn new() -> Self {
        #[cfg(feature = "plugins")]
        let plugin_manager = {
            let mut manager = form_factor::PluginManager::new();

            #[cfg(feature = "plugin-canvas")]
            {
                manager.register(Box::new(CanvasPlugin::new()));
                tracing::info!("Registered canvas plugin");
            }

            #[cfg(feature = "plugin-layers")]
            {
                manager.register(Box::new(LayersPlugin::new()));
                tracing::info!("Registered layers plugin");
            }

            #[cfg(feature = "plugin-file")]
            {
                manager.register(Box::new(FilePlugin::new()));
                tracing::info!("Registered file plugin");
            }

            #[cfg(feature = "plugin-detection")]
            {
                manager.register(Box::new(DetectionPlugin::new()));
                tracing::info!("Registered detection plugin");
            }

            #[cfg(feature = "plugin-ocr")]
            {
                manager.register(Box::new(OcrPlugin::new()));
                tracing::info!("Registered OCR plugin");
            }

            #[cfg(feature = "plugin-properties")]
            {
                manager.register(Box::new(PropertiesPlugin::new()));
                tracing::info!("Registered properties plugin");
            }

            manager
        };

        Self {
            name: String::from("Form Factor"),
            canvas: DrawingCanvas::new(),
            app_state: form_factor::AppState::new(),
            mode_switcher: form_factor::ModeSwitcher::new(),
            data_entry_panel: None,
            instance_manager_panel: None,
            toasts: egui_notify::Toasts::default(),
            #[cfg(feature = "plugins")]
            plugin_manager,
        }
    }

    /// Render the instance filling mode layout
    fn render_instance_filling_mode(&mut self, ctx: &AppContext) {
        use form_factor::{DataEntryAction, DataEntryPanel};

        // Create data entry panel if it doesn't exist
        if self.data_entry_panel.is_none() {
            if let (Some(template), Some(instance)) = (
                self.app_state.current_template(),
                self.app_state.current_instance(),
            ) {
                self.data_entry_panel = Some(DataEntryPanel::new(
                    template.clone(),
                    instance.clone(),
                ));
                tracing::info!("Created data entry panel");
            } else {
                // No template/instance, return to canvas mode
                tracing::warn!("No template or instance available for filling");
                self.app_state.set_mode(form_factor::AppMode::Canvas);
                return;
            }
        }

        // Render data entry panel and handle actions
        let mut action = DataEntryAction::None;
        if let Some(panel) = &mut self.data_entry_panel {
            egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
                action = panel.ui(ui);
            });
        }

        // Handle data entry actions outside the borrow
        match action {
            DataEntryAction::SaveDraft => {
                tracing::info!("Saving instance draft");
                if let Some(panel) = &self.data_entry_panel {
                    let instance_name = panel
                        .instance()
                        .instance_name()
                        .as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "unnamed".to_string());
                    self.save_instance_draft(instance_name);
                }
            }
            DataEntryAction::Submit => {
                tracing::info!("Submitting instance");
                let (can_submit, instance_name) = if let Some(panel) = &mut self.data_entry_panel {
                    let valid = panel.validate().is_ok();
                    let name = panel
                        .instance()
                        .instance_name()
                        .as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "unnamed".to_string());
                    (valid, name)
                } else {
                    (false, String::new())
                };

                if can_submit && self.save_instance_final(instance_name) {
                    // Clear panel and return to canvas
                    self.data_entry_panel = None;
                    self.app_state.set_current_instance(None);
                    self.app_state.set_mode(form_factor::AppMode::Canvas);
                    self.app_state.mark_clean();
                }
            }
            DataEntryAction::Cancel => {
                tracing::info!("Cancelling instance filling");
                // Clear panel and return to previous mode
                self.data_entry_panel = None;
                self.app_state.set_current_instance(None);
                self.app_state.go_back();
                self.app_state.mark_clean();
            }
            DataEntryAction::None => {
                // No action, update dirty state
                if let Some(panel) = &self.data_entry_panel {
                    if panel.is_dirty() {
                        self.app_state.mark_dirty();
                    }
                }
            }
        }
    }

    /// Save instance as draft (incomplete data allowed)
    fn save_instance_draft(&mut self, instance_name: String) {
        // TODO: Implement instance draft saving to file system
        tracing::info!(instance_name, "Saved instance draft");
        self.app_state.mark_clean();
    }

    /// Save instance as final (requires validation)
    fn save_instance_final(&mut self, instance_name: String) -> bool {
        // TODO: Implement instance final save to file system
        tracing::info!(instance_name, "Saved final instance");
        self.app_state.mark_clean();
        true
    }

    /// Render the template manager mode layout
    fn render_template_manager_mode(&mut self, ctx: &AppContext) {
        use form_factor::{InstanceManagerAction, InstanceManagerPanel};
        use std::collections::HashMap;

        // Create instance manager panel if it doesn't exist
        if self.instance_manager_panel.is_none() {
            // TODO: Load templates and instances from file system
            let templates = HashMap::new();
            let instances = HashMap::new();

            self.instance_manager_panel = Some(InstanceManagerPanel::new(templates, instances));
            tracing::info!("Created instance manager panel");
        }

        // Render instance manager panel and handle actions
        let mut action = InstanceManagerAction::None;
        if let Some(panel) = &mut self.instance_manager_panel {
            egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
                action = panel.ui(ui);
            });
        }

        // Handle instance manager actions
        match action {
            InstanceManagerAction::CreateInstance { template_id } => {
                tracing::info!(template_id, "Creating new instance from template");
                if let Some(panel) = &self.instance_manager_panel {
                    if let Some(template) = panel.get_template(&template_id) {
                        let instance = self.create_instance_from_template(template);
                        self.app_state.set_current_template(Some(template.clone()));
                        self.app_state.set_current_instance(Some(instance));

                        // Transition to instance filling mode
                        if let Err(e) =
                            self.app_state.transition_to(form_factor::AppMode::InstanceFilling)
                        {
                            tracing::error!("Failed to transition to instance filling mode: {}", e);
                        } else {
                            // Clear the instance manager panel for next time
                            self.instance_manager_panel = None;
                        }
                    }
                }
            }
            InstanceManagerAction::LoadInstance { instance_id } => {
                tracing::info!(instance_id, "Loading instance for editing");
                if let Some(panel) = &self.instance_manager_panel {
                    if let Some(instance) = panel.get_instance(&instance_id) {
                        let template_id = instance.template_id();
                        if let Some(template) = panel.get_template(template_id) {
                            self.app_state.set_current_template(Some(template.clone()));
                            self.app_state.set_current_instance(Some(instance.clone()));

                            // Transition to instance filling mode
                            if let Err(e) = self
                                .app_state
                                .transition_to(form_factor::AppMode::InstanceFilling)
                            {
                                tracing::error!(
                                    "Failed to transition to instance filling mode: {}",
                                    e
                                );
                            } else {
                                // Clear the instance manager panel for next time
                                self.instance_manager_panel = None;
                            }
                        }
                    }
                }
            }
            InstanceManagerAction::DeleteInstance { instance_id } => {
                tracing::info!(instance_id, "Deleting instance");
                if let Some(panel) = &mut self.instance_manager_panel {
                    panel.remove_instance(&instance_id);
                    // TODO: Delete from file system
                }
            }
            InstanceManagerAction::None => {}
        }
    }

    /// Create a new instance from a template
    fn create_instance_from_template(
        &self,
        template: &form_factor::DrawingTemplate,
    ) -> form_factor::DrawingInstance {
        tracing::info!(
            template_id = template.id(),
            template_name = template.name(),
            "Creating new instance"
        );

        // Create instance with auto-generated name
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let instance_name = format!("{} - New Instance {}", template.name(), timestamp);

        let mut instance = form_factor::DrawingInstance::from_template(
            template.id(),
            template.page_count(),
        );
        // Set the instance name
        instance.set_instance_name(instance_name);

        instance
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
                        use form_factor::ToolMode;
                        let tool = match tool_name.as_str() {
                            "Select" => Some(ToolMode::Select),
                            "Rectangle" => Some(ToolMode::Rectangle),
                            "Circle" => Some(ToolMode::Circle),
                            "Freehand" => Some(ToolMode::Freehand),
                            "Edit" => Some(ToolMode::Edit),
                            "Rotate" => Some(ToolMode::Rotate),
                            _ => None,
                        };
                        if let Some(tool) = tool {
                            self.canvas.set_tool(tool);
                        }
                    }
                    AppEvent::LayerVisibilityChanged {
                        layer_name,
                        visible,
                    } => {
                        // Find layer by name and toggle
                        use form_factor::LayerType;
                        let layer_type = match layer_name.as_str() {
                            "Canvas" => Some(LayerType::Canvas),
                            "Detections" => Some(LayerType::Detections),
                            "Shapes" => Some(LayerType::Shapes),
                            "Grid" => Some(LayerType::Grid),
                            _ => None,
                        };
                        if let Some(layer_type) = layer_type
                            && self.canvas.layer_manager().is_visible(layer_type) != *visible
                        {
                            self.canvas.layer_manager_mut().toggle_layer(layer_type);
                        }
                    }
                    AppEvent::LayerSelected { layer_name } => {
                        use form_factor::LayerType;
                        let layer_type = match layer_name.as_str() {
                            "Canvas" => Some(LayerType::Canvas),
                            "Detections" => Some(LayerType::Detections),
                            "Shapes" => Some(LayerType::Shapes),
                            "Grid" => Some(LayerType::Grid),
                            _ => None,
                        };
                        self.canvas.with_selected_layer(layer_type);
                    }
                    AppEvent::LayerClearRequested { layer_name } => {
                        use form_factor::LayerType;
                        let layer_type = match layer_name.as_str() {
                            "Canvas" => Some(LayerType::Canvas),
                            "Detections" => Some(LayerType::Detections),
                            "Shapes" => Some(LayerType::Shapes),
                            "Grid" => Some(LayerType::Grid),
                            _ => None,
                        };
                        if let Some(layer_type) = layer_type {
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
                                    if *self.app_state.mode() == form_factor::AppMode::InstanceFilling {
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
                    AppEvent::OpenFileRequested => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Form Factor Project", &["ffp"])
                            .pick_file()
                            && let Some(path_str) = path.to_str()
                        {
                            match self.canvas.load_from_file(path_str, ctx.egui_ctx()) {
                                Ok(()) => {
                                    tracing::info!("Loaded project from {}", path_str);
                                    // Emit FileOpened event
                                    self.plugin_manager
                                        .event_bus()
                                        .sender()
                                        .emit(AppEvent::FileOpened { path });
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load project: {}", e);
                                }
                            }
                        }
                    }
                    AppEvent::SaveFileRequested => {
                        // Save to current file or show save dialog
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Form Factor Project", &["ffp"])
                            .set_file_name(format!("{}.ffp", self.canvas.project_name()))
                            .save_file()
                            && let Some(path_str) = path.to_str()
                        {
                            match self.canvas.save_to_file(path_str) {
                                Ok(()) => {
                                    tracing::info!("Saved project to {}", path_str);
                                    self.plugin_manager
                                        .event_bus()
                                        .sender()
                                        .emit(AppEvent::FileSaved { path });
                                }
                                Err(e) => {
                                    tracing::error!("Failed to save project: {}", e);
                                }
                            }
                        }
                    }
                    AppEvent::SaveAsRequested => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Form Factor Project", &["ffp"])
                            .set_file_name(format!("{}.ffp", self.canvas.project_name()))
                            .save_file()
                            && let Some(path_str) = path.to_str()
                        {
                            match self.canvas.save_to_file(path_str) {
                                Ok(()) => {
                                    tracing::info!("Saved project to {}", path_str);
                                    self.plugin_manager
                                        .event_bus()
                                        .sender()
                                        .emit(AppEvent::FileSaved { path });
                                }
                                Err(e) => {
                                    tracing::error!("Failed to save project: {}", e);
                                }
                            }
                        }
                    }
                    AppEvent::LoadImageRequested => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                            .pick_file()
                            && let Some(path_str) = path.to_str()
                        {
                            match self.canvas.load_form_image(path_str, ctx.egui_ctx()) {
                                Ok(()) => {
                                    tracing::info!("Loaded image from {}", path_str);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load image: {}", e);
                                }
                            }
                        }
                    }
                    #[cfg(feature = "text-detection")]
                    AppEvent::TextDetectionRequested => {
                        // Show toast immediately that detection started
                        self.toasts.info("Text detection started...");

                        // Get form image path for background thread
                        if let Some(form_path) = self.canvas.form_image_path().as_ref().map(|s| s.clone()) {
                            let sender = self.plugin_manager.event_bus().sender();

                            // Spawn background thread for text detection
                            std::thread::spawn(move || {
                                use form_factor::{Rectangle, Shape};
                                use form_factor_cv::TextDetector;
                                use egui::{Color32, Pos2, Stroke};

                                tracing::info!("Starting text detection in background thread");

                                // Perform detection in background
                                let result = (|| -> Result<Vec<Shape>, String> {
                                    // Create text detector
                                    let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())
                                        .map_err(|e| format!("Failed to create detector: {}", e))?;

                                    // Detect text regions
                                    let regions = detector
                                        .detect_from_file(&form_path, 0.5)
                                        .map_err(|e| format!("Detection failed: {}", e))?;

                                    // Convert to shapes
                                    let mut shapes = Vec::new();
                                    for (i, region) in regions.iter().enumerate() {
                                        let top_left = Pos2::new(*region.x() as f32, *region.y() as f32);
                                        let bottom_right = Pos2::new(
                                            (*region.x() + *region.width()) as f32,
                                            (*region.y() + *region.height()) as f32,
                                        );

                                        let stroke = Stroke::new(2.0, Color32::from_rgb(255, 165, 0));
                                        let fill = Color32::TRANSPARENT;

                                        if let Ok(mut rect) = Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                                            rect.set_name(format!(
                                                "Text Region {} ({:.2}%)",
                                                i + 1,
                                                *region.confidence() * 100.0
                                            ));
                                            shapes.push(Shape::Rectangle(rect));
                                        }
                                    }

                                    Ok(shapes)
                                })();

                                match result {
                                    Ok(shapes) => {
                                        let count = shapes.len();
                                        tracing::info!("Detected {} text regions", count);

                                        // Serialize shapes as JSON
                                        if let Ok(shapes_json) = serde_json::to_string(&shapes) {
                                            sender.emit(AppEvent::DetectionResultsReady {
                                                detection_type: "text".to_string(),
                                                shapes_json,
                                            });
                                        }

                                        sender.emit(AppEvent::DetectionComplete {
                                            count,
                                            detection_type: "text".to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!("Text detection failed: {}", e);
                                        sender.emit(AppEvent::DetectionFailed {
                                            detection_type: "text".to_string(),
                                            error: e,
                                        });
                                    }
                                }
                            });
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
                        if let Some(form_path) = self.canvas.form_image_path().as_ref().map(|s| s.clone()) {
                            let sender = self.plugin_manager.event_bus().sender();

                            // Spawn background thread for logo detection
                            std::thread::spawn(move || {
                                use form_factor::{Rectangle, Shape};
                                use form_factor_cv::LogoDetector;
                                use egui::{Color32, Pos2, Stroke};

                                tracing::info!("Starting logo detection in background thread");

                                // Perform detection in background
                                let result = (|| -> Result<Vec<Shape>, String> {
                                    // Create logo detector
                                    let mut detector = LogoDetector::builder()
                                        .template_matching()
                                        .with_confidence_threshold(0.5)
                                        .with_scales(vec![
                                            0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.65, 0.75, 1.0, 1.25, 1.5, 2.0,
                                        ])
                                        .build();

                                    // Load logo templates from logos directory
                                    let logos_dir = std::path::Path::new("logos");
                                    if !logos_dir.exists() {
                                        return Err("logos directory does not exist".to_string());
                                    }

                                    let mut logo_count = 0;
                                    for entry in std::fs::read_dir(logos_dir)
                                        .map_err(|e| format!("Failed to read logos directory: {}", e))?
                                    {
                                        let entry = entry
                                            .map_err(|e| format!("Failed to read directory entry: {}", e))?;
                                        let path = entry.path();
                                        if path.is_file() {
                                            if let Some(ext) = path.extension() {
                                                let ext_str = ext.to_string_lossy().to_lowercase();
                                                if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" || ext_str == "webp" {
                                                    let logo_name = path
                                                        .file_stem()
                                                        .and_then(|s| s.to_str())
                                                        .unwrap_or("unknown");
                                                    if let Err(e) = detector.add_logo(logo_name, &path) {
                                                        tracing::warn!("Failed to load logo {}: {}", logo_name, e);
                                                    } else {
                                                        logo_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if logo_count == 0 {
                                        return Err("No logo templates found in logos directory".to_string());
                                    }

                                    tracing::info!("Loaded {} logo templates", logo_count);

                                    // Detect logos
                                    let results = detector
                                        .detect_logos_from_path(&form_path)
                                        .map_err(|e| format!("Detection failed: {}", e))?;

                                    // Convert to shapes
                                    let mut shapes = Vec::new();
                                    for result in results.iter() {
                                        let top_left = Pos2::new(result.location.x as f32, result.location.y as f32);
                                        let bottom_right = Pos2::new(
                                            (result.location.x + result.size.width) as f32,
                                            (result.location.y + result.size.height) as f32,
                                        );

                                        let stroke = Stroke::new(2.0, Color32::from_rgb(0, 128, 255)); // Blue for logos
                                        let fill = Color32::TRANSPARENT;

                                        if let Ok(mut rect) = Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                                            rect.set_name(format!(
                                                "Logo: {} ({:.2}%)",
                                                result.logo_name,
                                                result.confidence * 100.0
                                            ));
                                            shapes.push(Shape::Rectangle(rect));
                                        }
                                    }

                                    Ok(shapes)
                                })();

                                match result {
                                    Ok(shapes) => {
                                        let count = shapes.len();
                                        tracing::info!("Detected {} logos", count);

                                        // Serialize shapes as JSON
                                        if let Ok(shapes_json) = serde_json::to_string(&shapes) {
                                            sender.emit(AppEvent::DetectionResultsReady {
                                                detection_type: "logo".to_string(),
                                                shapes_json,
                                            });
                                        }

                                        sender.emit(AppEvent::DetectionComplete {
                                            count,
                                            detection_type: "logo".to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!("Logo detection failed: {}", e);
                                        sender.emit(AppEvent::DetectionFailed {
                                            detection_type: "logo".to_string(),
                                            error: e,
                                        });
                                    }
                                }
                            });
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
                        use form_factor::{OCRConfig, OCREngine, PageSegmentationMode};

                        // Show toast immediately that OCR started
                        self.toasts.info("OCR extraction started...");

                        match OCREngine::new(
                            OCRConfig::new()
                                .with_psm(PageSegmentationMode::Auto)
                                .with_min_confidence(60),
                        ) {
                            Ok(ocr) => match self.canvas.extract_text_from_detections(&ocr) {
                                Ok(results) => {
                                    tracing::info!(
                                        "Extracted text from {} detections",
                                        results.len()
                                    );
                                    let texts: Vec<String> = results
                                        .iter()
                                        .map(|(_, result)| result.text().trim().to_string())
                                        .collect();

                                    // Show success toast
                                    self.toasts.success(format!(
                                        "OCR complete: extracted text from {} region{}",
                                        results.len(),
                                        if results.len() == 1 { "" } else { "s" }
                                    ));

                                    // Emit custom event with extracted text
                                    if let Ok(event) =
                                        AppEvent::custom("ocr", "text_extracted", &texts)
                                    {
                                        self.plugin_manager.event_bus().sender().emit(event);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to extract text: {}", e);
                                    self.toasts.error(format!("OCR extraction failed: {}", e));
                                }
                            },
                            Err(e) => {
                                tracing::error!("Failed to initialize OCR engine: {}", e);
                                self.toasts.error(format!("OCR initialization failed: {}", e));
                            }
                        }
                    }
                    AppEvent::DetectionComplete { count, detection_type } => {
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
                    AppEvent::DetectionFailed { detection_type, error } => {
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
                        match serde_json::from_str::<Vec<form_factor::Shape>>(&shapes_json) {
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
                            self.plugin_manager.render_plugins(ui);
                        });
                    });

                egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
                    self.canvas.ui(ui);
                });
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
