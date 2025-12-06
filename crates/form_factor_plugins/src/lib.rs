//! Plugin system for Form Factor application.
//!
//! This crate provides a flexible plugin architecture that allows the UI to be
//! modularized into independent, composable components. Plugins communicate through
//! an event bus using message passing, enabling loose coupling between components.
//!
//! # Architecture
//!
//! The plugin system consists of:
//!
//! - **Plugin Trait**: Interface that all plugins must implement
//! - **Event Bus**: Message passing system using `tokio::sync::mpsc`
//! - **Plugin Manager**: Coordinates plugin lifecycle and event distribution
//! - **App Events**: Typed events for inter-plugin communication
//!
//! # Features
//!
//! Plugins are enabled at compile time using feature flags:
//!
//! - `plugin-canvas` - Canvas drawing and manipulation tools
//! - `plugin-layers` - Layer management UI
//! - `plugin-file` - File open/save operations
//! - `plugin-detection` - Computer vision detection features
//! - `plugin-ocr` - OCR text extraction
//! - `plugin-template` - Template creation and management
//! - `all-plugins` - Enable all available plugins
//!
//! # Example
//!
//! ```rust
//! use form_factor_plugins::{PluginManager, Plugin, PluginContext};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str {
//!         "my-plugin"
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
//!         ui.label("Hello from my plugin!");
//!     }
//! }
//!
//! // Create manager and register plugins
//! let mut manager = PluginManager::new();
//! manager.register(Box::new(MyPlugin));
//!
//! // In your main loop:
//! // manager.process_events();
//! // manager.render_plugins(&mut ui);
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod bus;
mod event;
mod manager;
mod plugin;

// Re-export public API
pub use bus::{EventBus, EventSender, SendError, SendErrorKind};
pub use event::{AppEvent, DecodeError};
pub use manager::PluginManager;
pub use plugin::{Plugin, PluginBuilder, PluginContext};

// Feature-gated plugin modules
#[cfg(feature = "plugin-canvas")]
pub mod canvas;

#[cfg(feature = "plugin-layers")]
pub mod layers;

#[cfg(feature = "plugin-file")]
pub mod file;

#[cfg(feature = "plugin-detection")]
pub mod detection;

#[cfg(feature = "plugin-ocr")]
pub mod ocr;

#[cfg(feature = "plugin-template")]
pub mod template;
