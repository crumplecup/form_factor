//! Example application demonstrating the backend-agnostic architecture

use form_factor::{App, AppContext, Backend, BackendConfig, DrawingCanvas, EframeBackend};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Main application struct
struct DemoApp {
    name: String,
    canvas: DrawingCanvas,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            name: String::from("Form Factor"),
            canvas: DrawingCanvas::new(),
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
                let layers_data: Vec<_> = self.canvas.layer_manager.layers()
                    .iter()
                    .map(|l| (l.layer_type, l.visible, l.locked, l.name.clone()))
                    .collect();

                for (layer_type, visible, locked, name) in layers_data {
                    let is_selected = self.canvas.selected_layer == Some(layer_type);

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
                            if ui.button(visible_icon).clicked()
                                && let Some(layer) = self.canvas.layer_manager.layers_mut()
                                    .iter_mut()
                                    .find(|l| l.layer_type == layer_type) {
                                layer.toggle_visibility();
                            }

                            let lock_icon = if locked { "🔒" } else { "🔓" };
                            if ui.button(lock_icon).clicked()
                                && let Some(layer) = self.canvas.layer_manager.layers_mut()
                                    .iter_mut()
                                    .find(|l| l.layer_type == layer_type) {
                                layer.toggle_locked();
                            }

                            // Use regular label instead of selectable_label since we're highlighting the whole row
                            if ui.label(&name).clicked() {
                                // Toggle selection: if already selected, unselect; otherwise select
                                if is_selected {
                                    self.canvas.selected_layer = None;
                                } else {
                                    self.canvas.selected_layer = Some(layer_type);
                                }
                            }
                        });
                    });
                }

                ui.separator();

                // Projects section
                ui.heading("Project");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Name:");

                    if self.canvas.editing_project_name {
                        let response = ui.text_edit_singleline(&mut self.canvas.project_name);

                        // Stop editing on Enter or focus loss
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.canvas.editing_project_name = false;
                        }

                        // Request focus when starting to edit
                        if !response.has_focus() {
                            response.request_focus();
                        }
                    } else {
                        // Show project name as clickable label
                        if ui.selectable_label(false, &self.canvas.project_name).clicked() {
                            self.canvas.editing_project_name = true;
                        }
                    }
                });

                // Save and Load buttons
                ui.horizontal(|ui| {
                    // Save button
                    if ui.button("💾 Save").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Form Factor Project", &["ffp"])
                        .set_file_name(format!("{}.ffp", self.canvas.project_name))
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
