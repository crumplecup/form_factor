//! Form Factor - Accessible GUI framework for tagging scanned forms
//!
//! This library provides a backend-agnostic architecture for building
//! accessible GUI applications using egui, with support for multiple
//! rendering backends (eframe, wgpu, etc.)

pub mod app;
pub mod backend;

pub use app::{App, AppContext};
pub use backend::Backend;
