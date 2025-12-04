//! eframe backend implementation
//!
//! This module provides a backend that uses eframe for window management
//! and rendering. eframe is a high-level framework that handles the event
//! loop and integrates with multiple rendering backends (glow, wgpu).

use form_factor_core::{App, AppContext, Backend, BackendConfig};

/// eframe-based backend implementation
pub struct EframeBackend;

/// Wrapper that adapts our App trait to eframe's epi::App trait
struct EframeApp {
    app: Box<dyn App>,
    frame_count: u64,
    last_frame_time: Option<std::time::Instant>,
}

impl EframeApp {
    fn new(app: Box<dyn App>) -> Self {
        Self {
            app,
            frame_count: 0,
            last_frame_time: None,
        }
    }
}

impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        let delta_time = self
            .last_frame_time
            .map(|last| now.duration_since(last).as_secs_f32())
            .unwrap_or(0.016); // Default to ~60fps for first frame

        self.last_frame_time = Some(now);

        let app_ctx = AppContext {
            egui_ctx: ctx,
            delta_time,
            frame_count: self.frame_count,
        };

        self.app.update(&app_ctx);
        self.frame_count += 1;
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app.on_exit();
    }
}

/// Errors that can occur when using the eframe backend
#[derive(Debug, thiserror::Error)]
pub enum EframeError {
    /// Failed to initialize the eframe backend
    #[error("Failed to initialize eframe: {0}")]
    InitError(#[from] eframe::Error),
}

impl Backend for EframeBackend {
    type Error = EframeError;

    fn run(mut app: Box<dyn App>, config: BackendConfig) -> Result<(), Self::Error> {
        // Get the app name before moving it
        let app_name = app.name().to_string();

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([config.window_width as f32, config.window_height as f32])
                .with_resizable(config.resizable)
                .with_title(&app_name),
            vsync: config.vsync,
            multisampling: config.msaa_samples as u16,
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };

        // Call setup before starting the event loop
        let ctx = egui::Context::default();
        app.setup(&ctx);

        let eframe_app = EframeApp::new(app);

        eframe::run_native(
            &app_name,
            native_options,
            Box::new(|_cc| Ok(Box::new(eframe_app))),
        )?;

        Ok(())
    }
}
