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
use ui_properties::PropertyRenderer;

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

#[cfg(all(feature = "plugins", feature = "plugin-properties"))]
use form_factor_plugins::PropertiesPlugin;

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
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            let sender = self.plugin_manager.event_bus().sender();

                            // Spawn background thread for text detection
                            std::thread::spawn(move || {
                                use egui::{Color32, Pos2, Stroke};
                                use form_factor::{Rectangle, Shape};
                                use form_factor_cv::TextDetector;

                                tracing::info!("Starting text detection in background thread");

                                // Perform detection in background
                                let result = (|| -> Result<Vec<Shape>, String> {
                                    // Create text detector
                                    let detector = TextDetector::new(
                                        "models/DB_TD500_resnet50.onnx".to_string(),
                                    )
                                    .map_err(|e| format!("Failed to create detector: {}", e))?;

                                    // Detect text regions
                                    let regions = detector
                                        .detect_from_file(&form_path, 0.5)
                                        .map_err(|e| format!("Detection failed: {}", e))?;

                                    // Convert to shapes
                                    let mut shapes = Vec::new();
                                    for (i, region) in regions.iter().enumerate() {
                                        let top_left =
                                            Pos2::new(*region.x() as f32, *region.y() as f32);
                                        let bottom_right = Pos2::new(
                                            (*region.x() + *region.width()) as f32,
                                            (*region.y() + *region.height()) as f32,
                                        );

                                        let stroke =
                                            Stroke::new(2.0, Color32::from_rgb(255, 165, 0));
                                        let fill = Color32::TRANSPARENT;

                                        if let Ok(mut rect) = Rectangle::from_corners(
                                            top_left,
                                            bottom_right,
                                            stroke,
                                            fill,
                                        ) {
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
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            let sender = self.plugin_manager.event_bus().sender();

                            // Spawn background thread for logo detection
                            std::thread::spawn(move || {
                                use egui::{Color32, Pos2, Stroke};
                                use form_factor::{Rectangle, Shape};
                                use form_factor_cv::LogoDetector;

                                tracing::info!("Starting logo detection in background thread");

                                // Perform detection in background
                                let result = (|| -> Result<Vec<Shape>, String> {
                                    // Create logo detector
                                    let mut detector = LogoDetector::builder()
                                        .template_matching()
                                        .with_confidence_threshold(0.5)
                                        .with_scales(vec![
                                            0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.65, 0.75, 1.0, 1.25,
                                            1.5, 2.0,
                                        ])
                                        .build();

                                    // Load logo templates from logos directory
                                    let logos_dir = std::path::Path::new("logos");
                                    if !logos_dir.exists() {
                                        return Err("logos directory does not exist".to_string());
                                    }

                                    let mut logo_count = 0;
                                    for entry in std::fs::read_dir(logos_dir).map_err(|e| {
                                        format!("Failed to read logos directory: {}", e)
                                    })? {
                                        let entry = entry.map_err(|e| {
                                            format!("Failed to read directory entry: {}", e)
                                        })?;
                                        let path = entry.path();
                                        if path.is_file()
                                            && let Some(ext) = path.extension()
                                        {
                                            let ext_str = ext.to_string_lossy().to_lowercase();
                                            if ext_str == "png"
                                                || ext_str == "jpg"
                                                || ext_str == "jpeg"
                                                || ext_str == "webp"
                                            {
                                                let logo_name = path
                                                    .file_stem()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("unknown");
                                                if let Err(e) = detector.add_logo(logo_name, &path)
                                                {
                                                    tracing::warn!(
                                                        "Failed to load logo {}: {}",
                                                        logo_name,
                                                        e
                                                    );
                                                } else {
                                                    logo_count += 1;
                                                }
                                            }
                                        }
                                    }

                                    if logo_count == 0 {
                                        return Err("No logo templates found in logos directory"
                                            .to_string());
                                    }

                                    tracing::info!("Loaded {} logo templates", logo_count);

                                    // Detect logos
                                    let results = detector
                                        .detect_logos_from_path(&form_path)
                                        .map_err(|e| format!("Detection failed: {}", e))?;

                                    // Convert to shapes
                                    let mut shapes = Vec::new();
                                    for result in results.iter() {
                                        let top_left = Pos2::new(
                                            result.location.x as f32,
                                            result.location.y as f32,
                                        );
                                        let bottom_right = Pos2::new(
                                            (result.location.x + result.size.width) as f32,
                                            (result.location.y + result.size.height) as f32,
                                        );

                                        let stroke =
                                            Stroke::new(2.0, Color32::from_rgb(0, 128, 255)); // Blue for logos
                                        let fill = Color32::TRANSPARENT;

                                        if let Ok(mut rect) = Rectangle::from_corners(
                                            top_left,
                                            bottom_right,
                                            stroke,
                                            fill,
                                        ) {
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
                        // Show toast immediately that OCR started
                        self.toasts.info("OCR extraction started...");

                        // Get form image path and detections for background thread
                        if let Some(form_path) = self.canvas.form_image_path().clone() {
                            // Clone detections to pass to background thread
                            let detections: Vec<Shape> = self.canvas.detections().to_vec();

                            let sender = self.plugin_manager.event_bus().sender();

                            // Spawn background thread for OCR extraction
                            std::thread::spawn(move || {
                                use form_factor::{OCRConfig, OCREngine, PageSegmentationMode};
                                use image;

                                tracing::info!("Starting OCR extraction in background thread");

                                // Perform OCR in background
                                let result = (|| -> Result<String, String> {
                                    // Load the image
                                    let img = image::open(&form_path)
                                        .map_err(|e| format!("Failed to load image: {}", e))?;

                                    // Create OCR engine
                                    let ocr = OCREngine::new(
                                        OCRConfig::new()
                                            .with_psm(PageSegmentationMode::Auto)
                                            .with_min_confidence(60),
                                    )
                                    .map_err(|e| format!("Failed to create OCR engine: {}", e))?;

                                    // Extract text from each detection
                                    let mut results = Vec::new();
                                    for shape in detections {
                                        // Get bounding box from shape
                                        let bbox = match &shape {
                                            Shape::Rectangle(rect) => {
                                                let xs: Vec<f32> =
                                                    rect.corners().iter().map(|p| p.x).collect();
                                                let ys: Vec<f32> =
                                                    rect.corners().iter().map(|p| p.y).collect();

                                                let x_min =
                                                    xs.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                                                        as u32;
                                                let y_min =
                                                    ys.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                                                        as u32;
                                                let x_max = xs
                                                    .iter()
                                                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                                                    as u32;
                                                let y_max = ys
                                                    .iter()
                                                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                                                    as u32;

                                                let width = x_max.saturating_sub(x_min);
                                                let height = y_max.saturating_sub(y_min);

                                                (x_min, y_min, width, height)
                                            }
                                            Shape::Circle(circle) => {
                                                let center = circle.center();
                                                let radius = circle.radius();
                                                let x_min = (center.x - radius) as u32;
                                                let y_min = (center.y - radius) as u32;
                                                let width = (radius * 2.0) as u32;
                                                let height = (radius * 2.0) as u32;

                                                (x_min, y_min, width, height)
                                            }
                                            Shape::Polygon(poly) => {
                                                // Get coords from geo polygon
                                                let coords: Vec<_> =
                                                    poly.polygon().exterior().coords().collect();
                                                let xs: Vec<f32> =
                                                    coords.iter().map(|c| c.x as f32).collect();
                                                let ys: Vec<f32> =
                                                    coords.iter().map(|c| c.y as f32).collect();

                                                let x_min =
                                                    xs.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                                                        as u32;
                                                let y_min =
                                                    ys.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                                                        as u32;
                                                let x_max = xs
                                                    .iter()
                                                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                                                    as u32;
                                                let y_max = ys
                                                    .iter()
                                                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                                                    as u32;

                                                let width = x_max.saturating_sub(x_min);
                                                let height = y_max.saturating_sub(y_min);

                                                (x_min, y_min, width, height)
                                            }
                                        };

                                        match ocr.extract_text_from_region(&img, bbox) {
                                            Ok(result) => {
                                                let text = result.text().trim().to_string();
                                                if !text.is_empty() {
                                                    results.push((shape, text));
                                                }
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to extract text from region: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }

                                    // Serialize results to JSON
                                    serde_json::to_string(&results)
                                        .map_err(|e| format!("Failed to serialize results: {}", e))
                                })();

                                // Send result back to main thread
                                match result {
                                    Ok(results_json) => {
                                        tracing::info!("OCR extraction complete");
                                        let _ = sender.send(AppEvent::OcrComplete { results_json });
                                    }
                                    Err(e) => {
                                        tracing::error!("OCR extraction failed: {}", e);
                                        let _ = sender.send(AppEvent::DetectionFailed {
                                            detection_type: "ocr".to_string(),
                                            error: e,
                                        });
                                    }
                                }
                            });
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
