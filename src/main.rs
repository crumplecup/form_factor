//! Example application demonstrating the backend-agnostic architecture

use form_factor::{App, AppContext, Backend, BackendConfig, DrawingCanvas, EframeBackend};

/// Simple demo application
struct DemoApp {
    counter: i32,
    name: String,
    canvas: DrawingCanvas,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            counter: 0,
            name: String::from("Form Factor Demo"),
            canvas: DrawingCanvas::new(),
        }
    }
}

impl App for DemoApp {
    fn setup(&mut self, _ctx: &egui::Context) {
        println!("Application setup complete");
    }

    fn update(&mut self, ctx: &AppContext) {
        // Side panel for controls and info
        egui::SidePanel::left("control_panel")
            .default_width(250.0)
            .show(ctx.egui_ctx, |ui| {
                ui.heading("Form Factor Demo");

                ui.separator();

                ui.label(format!("Frame: {}", ctx.frame_count));
                ui.label(format!("FPS: {:.1}", 1.0 / ctx.delta_time));

                ui.separator();

                ui.heading("Canvas Controls");

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

                ui.heading("Counter Demo");
                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.counter += 1;
                    }
                    if ui.button("-").clicked() {
                        self.counter -= 1;
                    }
                    ui.label(format!("{}", self.counter));
                });

                ui.separator();

                ui.label("This demo shows:");
                ui.label("• Drawing tools (rect, circle, freehand)");
                ui.label("• Backend-agnostic architecture");
                ui.label("• AccessKit integration");
            });

        // Main canvas area
        egui::CentralPanel::default().show(ctx.egui_ctx, |ui| {
            self.canvas.ui(ui);
        });
    }

    fn on_exit(&mut self) {
        println!("Application exiting. Final counter value: {}", self.counter);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Box::new(DemoApp::new());
    let config = BackendConfig::default();

    // Run with the backend specified by feature flags
    #[cfg(feature = "backend-eframe")]
    {
        println!("Starting application with eframe backend...");
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
