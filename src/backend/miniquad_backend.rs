//! miniquad backend implementation
//!
//! This module provides a backend that uses miniquad for window management
//! and rendering. miniquad is a lightweight cross-platform framework with
//! excellent mobile support (Android/iOS) in addition to desktop and web.
//!
//! # Status: Awaiting Ecosystem Update
//!
//! This backend is currently **non-functional** because:
//! - We use `egui 0.33.0` (released October 2025)
//! - Latest `egui-miniquad 0.16.0` only supports `egui 0.31.1`
//!
//! This code serves as a reference implementation for when `egui-miniquad`
//! catches up to the latest egui version.
//!
//! ## To Enable When Ready
//!
//! 1. In `Cargo.toml`:
//!    - Uncomment the `egui-miniquad` dependency
//!    - Update version to match current egui support
//!    - Uncomment the `backend-miniquad` feature
//!
//! 2. In `src/backend/mod.rs`:
//!    - Uncomment the `#[cfg(feature = "backend-miniquad")]` guard
//!
//! 3. In `src/main.rs`:
//!    - The miniquad backend selection is already implemented
//!
//! # Example Usage (When Enabled)
//!
//! ```ignore
//! cargo run --no-default-features --features backend-miniquad
//! ```

// Reference implementation - kept for when ecosystem catches up
// This will not be used unless miniquad backend is uncommented in mod.rs
#![allow(dead_code)]

/*
use crate::{App, AppContext};
use super::{Backend, BackendConfig};
use egui_miniquad::EguiMq;
use miniquad as mq;

/// miniquad-based backend implementation
pub struct MiniquadBackend;

/// Application state wrapper for miniquad
struct Stage {
    egui_mq: EguiMq,
    app: Box<dyn App>,
    frame_count: u64,
    last_frame_time: f64,
}

impl Stage {
    fn new(ctx: &mut mq::Context, mut app: Box<dyn App>) -> Self {
        // Call app setup with egui context
        let egui_ctx = egui::Context::default();
        app.setup(&egui_ctx);

        Self {
            egui_mq: EguiMq::new(ctx),
            app,
            frame_count: 0,
            last_frame_time: mq::date::now(),
        }
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, ctx: &mut mq::Context) {
        // Calculate delta time
        let now = mq::date::now();
        let delta_time = (now - self.last_frame_time) as f32;
        self.last_frame_time = now;

        // Begin egui frame
        self.egui_mq.begin_frame(ctx);

        // Call app update
        let app_ctx = AppContext {
            egui_ctx: self.egui_mq.egui_ctx(),
            delta_time,
            frame_count: self.frame_count,
        };
        self.app.update(&app_ctx);

        // End egui frame and render
        self.egui_mq.end_frame(ctx);
        self.frame_count += 1;
    }

    fn draw(&mut self, ctx: &mut mq::Context) {
        // Clear the screen
        ctx.begin_default_pass(mq::PassAction::clear_color(0.1, 0.1, 0.1, 1.0));
        ctx.end_render_pass();

        // Draw egui
        self.egui_mq.draw(ctx);

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(ctx, dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, button, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, button, x, y);
    }

    fn char_event(&mut self, _ctx: &mut mq::Context, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }

    fn quit_requested_event(&mut self) {
        self.app.on_exit();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MiniquadError {
    #[error("Failed to initialize miniquad")]
    InitError,
}

impl Backend for MiniquadBackend {
    type Error = MiniquadError;

    fn run(app: Box<dyn App>, config: BackendConfig) -> Result<(), Self::Error> {
        let app_name = app.name().to_string();

        let conf = mq::conf::Conf {
            window_title: app_name,
            window_width: config.window_width as i32,
            window_height: config.window_height as i32,
            window_resizable: config.resizable,
            high_dpi: true,
            sample_count: config.msaa_samples as i32,
            ..Default::default()
        };

        mq::start(conf, |ctx| Box::new(Stage::new(ctx, app)));

        Ok(())
    }
}
*/
