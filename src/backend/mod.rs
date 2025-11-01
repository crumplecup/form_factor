//! Backend abstraction layer
//!
//! This module defines the trait that rendering backends must implement
//! to run applications defined with the `App` trait.

use crate::App;

#[cfg(feature = "backend-eframe")]
pub mod eframe_backend;

/// Configuration options for initializing a backend
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Initial window width in pixels
    pub window_width: u32,

    /// Initial window height in pixels
    pub window_height: u32,

    /// Whether the window is resizable
    pub resizable: bool,

    /// Whether to enable vsync
    pub vsync: bool,

    /// MSAA sample count (1 = disabled, 2/4/8 = enabled)
    pub msaa_samples: u32,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            window_width: 1024,
            window_height: 768,
            resizable: true,
            vsync: true,
            msaa_samples: 1,
        }
    }
}

/// Trait that backend implementations must satisfy.
///
/// Backends are responsible for:
/// - Managing the event loop
/// - Creating and managing the window
/// - Rendering the egui UI
/// - Integrating with AccessKit for accessibility
pub trait Backend {
    /// Error type returned by backend operations
    type Error: std::error::Error;

    /// Runs the application with this backend.
    ///
    /// This method takes ownership of the app and starts the event loop.
    /// It typically blocks until the application exits.
    fn run(app: Box<dyn App>, config: BackendConfig) -> Result<(), Self::Error>;
}
