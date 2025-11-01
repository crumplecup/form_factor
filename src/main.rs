//! Example application demonstrating the backend-agnostic architecture

use form_factor::{App, AppContext, Backend, BackendConfig, EframeBackend};

/// Simple demo application
struct DemoApp {
    counter: i32,
    name: String,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            counter: 0,
            name: String::from("Form Factor Demo"),
        }
    }
}

impl App for DemoApp {
    fn setup(&mut self, _ctx: &egui::Context) {
        println!("Application setup complete");
    }

    fn update(&mut self, ctx: &AppContext) {
        // Create a centered window
        egui::CentralPanel::default().show(ctx.egui_ctx, |ui| {
            ui.heading("Form Factor Demo Application");

            ui.separator();

            ui.label(format!("Frame: {}", ctx.frame_count));
            ui.label(format!("Delta time: {:.3}ms", ctx.delta_time * 1000.0));

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Increment").clicked() {
                    self.counter += 1;
                }
                if ui.button("Decrement").clicked() {
                    self.counter -= 1;
                }
                ui.label(format!("Counter: {}", self.counter));
            });

            ui.separator();

            ui.label("This application demonstrates:");
            ui.label("• Backend-agnostic GUI architecture");
            ui.label("• AccessKit integration for screen readers");
            ui.label("• Extensible design for multiple backends");
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
