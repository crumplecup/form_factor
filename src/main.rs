//! Example application demonstrating the backend-agnostic architecture

use form_factor::{App, AppContext, Backend, BackendConfig, DrawingCanvas, EframeBackend};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Simple demo application
struct DemoApp {
    name: String,
    canvas: DrawingCanvas,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            name: String::from("Form Factor Demo"),
            canvas: DrawingCanvas::new(),
        }
    }
}

impl App for DemoApp {
    fn setup(&mut self, _ctx: &egui::Context) {
        tracing::info!("Application setup complete");
    }

    fn update(&mut self, ctx: &AppContext) {
        // Side panel for controls and info
        egui::SidePanel::left("control_panel")
            .default_width(280.0)
            .show(ctx.egui_ctx, |ui| {
                ui.heading("Canvas Controls");

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

                for layer in self.canvas.layer_manager.layers_mut() {
                    ui.horizontal(|ui| {
                        let visible_icon = if layer.visible { "👁" } else { "🚫" };
                        if ui.button(visible_icon).clicked() {
                            layer.toggle_visibility();
                        }
                        ui.label(&layer.name);
                    });
                }

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
    // Initialize tracing subscriber with environment filter
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
