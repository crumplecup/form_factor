//! Example application demonstrating the backend-agnostic architecture

use form_factor::{App, AppContext, Backend, BackendConfig, DrawingCanvas, EframeBackend};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Main application struct
struct DemoApp {
    name: String,
    canvas: DrawingCanvas,
    #[cfg(feature = "ocr")]
    ocr_results: Vec<(usize, form_factor::OCRResult)>,
    #[cfg(feature = "plugins")]
    plugin_manager: form_factor::PluginManager,
}

impl DemoApp {
    fn new() -> Self {
        #[cfg(feature = "plugins")]
        let plugin_manager = {
            let mut manager = form_factor::PluginManager::new();

            #[cfg(feature = "plugin-canvas")]
            {
                manager.register(Box::new(form_factor::canvas::CanvasPlugin::new()));
                tracing::info!("Registered canvas plugin");
            }

            #[cfg(feature = "plugin-layers")]
            {
                manager.register(Box::new(form_factor::layers::LayersPlugin::new()));
                tracing::info!("Registered layers plugin");
            }

            #[cfg(feature = "plugin-file")]
            {
                manager.register(Box::new(form_factor::file::FilePlugin::new()));
                tracing::info!("Registered file plugin");
            }

            #[cfg(feature = "plugin-detection")]
            {
                manager.register(Box::new(form_factor::detection::DetectionPlugin::new()));
                tracing::info!("Registered detection plugin");
            }

            #[cfg(feature = "plugin-ocr")]
            {
                manager.register(Box::new(form_factor::ocr::OcrPlugin::new()));
                tracing::info!("Registered OCR plugin");
            }

            manager
        };

        Self {
            name: String::from("Form Factor"),
            canvas: DrawingCanvas::new(),
            #[cfg(feature = "ocr")]
            ocr_results: Vec::new(),
            #[cfg(feature = "plugins")]
            plugin_manager,
        }
    }
}

impl App for DemoApp {
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
                    AppEvent::LayerVisibilityChanged { layer_name, visible } => {
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
                        self.canvas.set_selected_layer(layer_type);
                    }
                    AppEvent::OpenFileRequested => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Form Factor Project", &["ffp"])
                            .pick_file()
                            && let Some(path_str) = path.to_str()
                        {
                                match self.canvas.load_from_file(path_str, ctx.egui_ctx) {
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
                    #[cfg(feature = "text-detection")]
                    AppEvent::TextDetectionRequested => {
                        match self.canvas.detect_text_regions(0.5) {
                            Ok(count) => {
                                tracing::info!("Detected {} text regions", count);
                                self.plugin_manager
                                    .event_bus()
                                    .sender()
                                    .emit(AppEvent::DetectionComplete {
                                        count,
                                        detection_type: "text".to_string(),
                                    });
                            }
                            Err(e) => {
                                tracing::error!("Failed to detect text: {}", e);
                            }
                        }
                    }
                    #[cfg(feature = "logo-detection")]
                    AppEvent::LogoDetectionRequested => {
                        match self.canvas.detect_logos() {
                            Ok(count) => {
                                tracing::info!("Detected {} logos", count);
                                self.plugin_manager
                                    .event_bus()
                                    .sender()
                                    .emit(AppEvent::DetectionComplete {
                                        count,
                                        detection_type: "logo".to_string(),
                                    });
                            }
                            Err(e) => {
                                tracing::error!("Failed to detect logos: {}", e);
                            }
                        }
                    }
                    #[cfg(feature = "ocr")]
                    AppEvent::OcrExtractionRequested => {
                        use form_factor::{OCRConfig, OCREngine, PageSegmentationMode};

                        match OCREngine::new(
                            OCRConfig::new()
                                .with_psm(PageSegmentationMode::Auto)
                                .with_min_confidence(60),
                        ) {
                            Ok(ocr) => match self.canvas.extract_text_from_detections(&ocr) {
                                Ok(results) => {
                                    tracing::info!("Extracted text from {} detections", results.len());
                                    let texts: Vec<String> = results
                                        .iter()
                                        .map(|(_, result)| result.text().trim().to_string())
                                        .collect();

                                    // Emit custom event with extracted text
                                    if let Ok(event) = AppEvent::custom("ocr", "text_extracted", &texts) {
                                        self.plugin_manager.event_bus().sender().emit(event);
                                    }

                                    self.ocr_results = results;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to extract text: {}", e);
                                }
                            },
                            Err(e) => {
                                tracing::error!("Failed to initialize OCR engine: {}", e);
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

        // Plugin sidebar (if plugins feature is enabled)
        #[cfg(feature = "plugins")]
        egui::SidePanel::right("plugin_panel")
            .default_width(280.0)
            .show(ctx.egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.plugin_manager.render_plugins(ui);
                });
            });

        // Side panel for controls and info
        egui::SidePanel::left("control_panel")
            .default_width(280.0)
            .show(ctx.egui_ctx, |ui| {
                ui.heading("Canvas Controls");

                ui.separator();

                // Load Form button
                if ui.button("📁 Load Form").clicked() {
                    let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    else {
                        return;
                    };

                    let Some(path_str) = path.to_str() else {
                        return;
                    };

                    match self.canvas.load_form_image(path_str, ctx.egui_ctx) {
                        Ok(()) => {
                            tracing::info!("Successfully loaded form image");
                        }
                        Err(e) => {
                            tracing::error!("Failed to load form image: {}", e);
                        }
                    }
                }

                // Detect Text button (only available with text-detection feature)
                #[cfg(feature = "text-detection")]
                if ui.button("🔍 Detect Text").clicked() {
                    match self.canvas.detect_text_regions(0.5) {
                        Ok(count) => {
                            tracing::info!("Detected {} text regions", count);
                        }
                        Err(e) => {
                            tracing::error!("Failed to detect text: {}", e);
                        }
                    }
                }

                // Detect Logos button (only available with logo-detection feature)
                #[cfg(feature = "logo-detection")]
                if ui.button("🏢 Detect Logos").clicked() {
                    match self.canvas.detect_logos() {
                        Ok(count) => {
                            tracing::info!("Detected {} logos", count);
                        }
                        Err(e) => {
                            tracing::error!("Failed to detect logos: {}", e);
                        }
                    }
                }

                // Extract Text (OCR) button (only available with ocr feature)
                #[cfg(feature = "ocr")]
                if ui.button("📝 Extract Text (OCR)").clicked() {
                    use form_factor::{OCREngine, OCRConfig, PageSegmentationMode};

                    // Create OCR engine
                    match OCREngine::new(OCRConfig::new()
                        .with_psm(PageSegmentationMode::Auto)
                        .with_min_confidence(60))
                    {
                        Ok(ocr) => {
                            // Extract text from all detections
                            match self.canvas.extract_text_from_detections(&ocr) {
                                Ok(results) => {
                                    tracing::info!("Extracted text from {} detections", results.len());
                                    for (idx, result) in &results {
                                        tracing::info!(
                                            "Detection {}: '{}' ({:.1}% confidence)",
                                            idx,
                                            result.text().trim(),
                                            result.confidence()
                                        );
                                    }
                                    self.ocr_results = results;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to extract text: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to initialize OCR engine: {}", e);
                            tracing::error!("Make sure Tesseract is installed on your system");
                        }
                    }
                }

                // Show OCR results if available
                #[cfg(feature = "ocr")]
                if !self.ocr_results.is_empty() {
                    ui.label(format!("OCR Results: {} regions", self.ocr_results.len()));
                    if ui.button("Clear OCR Results").clicked() {
                        self.ocr_results.clear();
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Clear All").clicked() {
                        self.canvas.clear();
                    }
                    if ui.button("Undo").clicked() {
                        self.canvas.undo();
                    }
                });

                ui.label(format!("Shapes: {}", self.canvas.shape_count()));

                ui.separator();

                // Layers section
                ui.heading("Layers");
                ui.separator();

                // Clone layers data to avoid borrow checker issues
                // Display in reverse order (Grid at top, Canvas at bottom) to match visual z-order
                let mut layers_data: Vec<_> = self.canvas.layer_manager().layers_in_order()
                    .map(|l| (*l.layer_type(), *l.visible(), *l.locked(), l.name().clone()))
                    .collect();
                layers_data.reverse();

                use form_factor::LayerType;
                for (layer_type, visible, locked, name) in layers_data {
                    let is_selected = *self.canvas.selected_layer() == Some(layer_type);

                    // Frame for row with background highlight when selected
                    let frame = if is_selected {
                        egui::Frame::default()
                            .fill(ui.visuals().selection.bg_fill)
                            .inner_margin(4.0)
                    } else {
                        egui::Frame::default()
                            .inner_margin(4.0)
                    };

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let visible_icon = if visible { "👁" } else { "🚫" };
                            if ui.button(visible_icon).clicked() {
                                self.canvas.layer_manager_mut().toggle_layer(layer_type);
                            }

                            let lock_icon = if locked { "🔒" } else { "🔓" };
                            if ui.button(lock_icon).clicked() {
                                self.canvas.layer_manager_mut().toggle_locked(layer_type);
                            }

                            // Clear button for each layer type
                            if ui.button("🗑").on_hover_text("Clear layer").clicked() {
                                match layer_type {
                                    LayerType::Shapes => self.canvas.clear_shapes(),
                                    LayerType::Detections => self.canvas.clear_detections(),
                                    LayerType::Canvas => self.canvas.clear_canvas_image(),
                                    LayerType::Grid => {} // Grid doesn't need clearing
                                }
                            }

                            // Special handling for Detections layer with dropdown
                            if layer_type == LayerType::Detections {
                                let is_expanded = self.canvas.is_detections_expanded();
                                let arrow = if is_expanded { "▼" } else { "▶" };

                                if ui.button(arrow).clicked() {
                                    self.canvas.toggle_detections_expanded();
                                }

                                if ui.label(&name).clicked() {
                                    // Toggle selection: if already selected, unselect; otherwise select
                                    if is_selected {
                                        self.canvas.set_selected_layer(None);
                                    } else {
                                        self.canvas.set_selected_layer(Some(layer_type));
                                    }
                                }
                            } else {
                                // Use regular label for other layers
                                if ui.label(&name).clicked() {
                                    // Toggle selection: if already selected, unselect; otherwise select
                                    if is_selected {
                                        self.canvas.set_selected_layer(None);
                                    } else {
                                        self.canvas.set_selected_layer(Some(layer_type));
                                    }
                                }
                            }
                        });
                    });

                    // Show sub-items if Detections layer is expanded
                    if layer_type == LayerType::Detections && self.canvas.is_detections_expanded() {
                        use form_factor::DetectionSubtype;

                        let logo_count = self.canvas.logo_detection_count();
                        let text_count = self.canvas.text_detection_count();

                        // Alphabetical order: Logos first, then Text

                        // Logos sub-item
                        let logos_selected = *self.canvas.selected_detection_subtype() == Some(DetectionSubtype::Logos);

                        ui.horizontal(|ui| {
                            ui.add_space(40.0); // Indent for sub-items

                            if logos_selected {
                                ui.visuals_mut().selection.bg_fill = ui.visuals().selection.bg_fill;
                            }

                            if ui.selectable_label(logos_selected, format!("🏢 Logos: {}", logo_count)).clicked() {
                                if logos_selected {
                                    self.canvas.set_selected_detection_subtype(None);
                                } else {
                                    self.canvas.set_selected_detection_subtype(Some(DetectionSubtype::Logos));
                                }
                            }
                        });

                        // Text sub-item
                        let text_selected = *self.canvas.selected_detection_subtype() == Some(DetectionSubtype::Text);

                        ui.horizontal(|ui| {
                            ui.add_space(40.0); // Indent for sub-items

                            if text_selected {
                                ui.visuals_mut().selection.bg_fill = ui.visuals().selection.bg_fill;
                            }

                            if ui.selectable_label(text_selected, format!("📝 Text: {}", text_count)).clicked() {
                                if text_selected {
                                    self.canvas.set_selected_detection_subtype(None);
                                } else {
                                    self.canvas.set_selected_detection_subtype(Some(DetectionSubtype::Text));
                                }
                            }
                        });
                    }
                }

                ui.separator();

                // Projects section
                ui.heading("Project");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Name:");

                    if *self.canvas.editing_project_name() {
                        // Use a temporary variable for editing since we need a mutable reference
                        let mut temp_name = self.canvas.project_name().clone();
                        let response = ui.text_edit_singleline(&mut temp_name);

                        // Update the canvas with the modified name
                        self.canvas.set_project_name(temp_name);

                        // Stop editing on Enter or focus loss
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.canvas.set_editing_project_name(false);
                        }

                        // Request focus when starting to edit
                        if !response.has_focus() {
                            response.request_focus();
                        }
                    } else {
                        // Show project name as clickable label
                        if ui.selectable_label(false, self.canvas.project_name()).clicked() {
                            self.canvas.set_editing_project_name(true);
                        }
                    }
                });

                // Save and Load buttons
                ui.horizontal(|ui| {
                    // Save button
                    if ui.button("💾 Save").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Form Factor Project", &["ffp"])
                        .set_file_name(format!("{}.ffp", self.canvas.project_name()))
                        .save_file()
                    && let Some(path_str) = path.to_str()
                {
                    match self.canvas.save_to_file(path_str) {
                        Ok(()) => {
                            tracing::info!("Successfully saved project to {}", path_str);
                        }
                        Err(e) => {
                            tracing::error!("Failed to save project: {}", e);
                        }
                    }
                    }

                    // Load button
                    if ui.button("📁 Load").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Form Factor Project", &["ffp"])
                            .pick_file()
                        && let Some(path_str) = path.to_str()
                    {
                        match self.canvas.load_from_file(path_str, ctx.egui_ctx) {
                            Ok(()) => {
                                tracing::info!("Successfully loaded project from {}", path_str);
                            }
                            Err(e) => {
                                tracing::error!("Failed to load project: {}", e);
                            }
                        }
                    }
                });

                ui.separator();

                // Settings section
                self.canvas.show_inline_settings(ui);

                ui.separator();

                // Show inline properties panel for selected shape
                self.canvas.show_inline_properties(ui);
            });

        // Main canvas area
        egui::CentralPanel::default().show(ctx.egui_ctx, |ui| {
            self.canvas.ui(ui);
        });
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

    let app = Box::new(DemoApp::new());
    let config = BackendConfig::default();

    // Run with the backend specified by feature flags
    #[cfg(feature = "backend-eframe")]
    {
        tracing::info!("Using eframe backend");
        EframeBackend::run(app, config)?;
    }

    // Miniquad backend support - ready for when egui-miniquad updates to egui 0.33+
    // #[cfg(all(feature = "backend-miniquad", not(feature = "backend-eframe")))]
    // {
    //     use form_factor::backends::miniquad_backend::MiniquadBackend;
    //     println!("Starting application with miniquad backend...");
    //     MiniquadBackend::run(app, config)?;
    // }

    Ok(())
}
