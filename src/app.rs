//! Core application traits and types
//!
//! This module defines the trait interface that applications implement,
//! keeping them independent of the underlying event loop backend.

use egui::Context;

/// Context information provided to the application each frame
pub struct AppContext<'a> {
    /// The egui context for building UI
    pub egui_ctx: &'a Context,

    /// Time elapsed since the last frame (in seconds)
    pub delta_time: f32,

    /// Frame number (increments each frame)
    pub frame_count: u64,
}

/// Core trait that all applications must implement.
///
/// This trait provides the interface between the GUI logic and the
/// event loop backend, allowing the same application code to run
/// on different backends (eframe, wgpu, etc.).
pub trait App {
    /// Called once before the event loop starts.
    ///
    /// Use this for initialization that requires access to the egui context.
    fn setup(&mut self, ctx: &Context) {
        let _ = ctx; // Default implementation does nothing
    }

    /// Called every frame to update state and build the UI.
    ///
    /// This is where you define your application logic and UI layout.
    fn update(&mut self, ctx: &AppContext);

    /// Called when the application is about to exit.
    ///
    /// Use this for cleanup operations, saving state, etc.
    fn on_exit(&mut self) {
        // Default implementation does nothing
    }

    /// Returns the application name shown in the window title.
    fn name(&self) -> &str {
        "Form Factor Application"
    }
}
