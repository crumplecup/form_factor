//! Form Factor - GUI application for tagging scanned forms with OCR metadata

use form_factor::{App, AppContext, DrawingCanvas};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "backend-eframe")]
use form_factor::{Backend, BackendConfig, EframeBackend};

/// Main application struct
struct FormFactorApp {
    name: String,
    canvas: DrawingCanvas,
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
            #[cfg(feature = "plugins")]
            plugin_manager,
        }
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
                        self.canvas.set_selected_layer(layer_type);
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
                                self.plugin_manager.event_bus().sender().emit(
                                    AppEvent::DetectionComplete {
                                        count,
                                        detection_type: "text".to_string(),
                                    },
                                );
                            }
                            Err(e) => {
                                tracing::error!("Failed to detect text: {}", e);
                            }
                        }
                    }
                    #[cfg(feature = "logo-detection")]
                    AppEvent::LogoDetectionRequested => match self.canvas.detect_logos() {
                        Ok(count) => {
                            tracing::info!("Detected {} logos", count);
                            self.plugin_manager.event_bus().sender().emit(
                                AppEvent::DetectionComplete {
                                    count,
                                    detection_type: "logo".to_string(),
                                },
                            );
                        }
                        Err(e) => {
                            tracing::error!("Failed to detect logos: {}", e);
                        }
                    },
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
                                    tracing::info!(
                                        "Extracted text from {} detections",
                                        results.len()
                                    );
                                    let texts: Vec<String> = results
                                        .iter()
                                        .map(|(_, result)| result.text().trim().to_string())
                                        .collect();

                                    // Emit custom event with extracted text
                                    if let Ok(event) =
                                        AppEvent::custom("ocr", "text_extracted", &texts)
                                    {
                                        self.plugin_manager.event_bus().sender().emit(event);
                                    }
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
