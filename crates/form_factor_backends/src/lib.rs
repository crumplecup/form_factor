//! Backend implementations for form_factor
//!
//! This crate provides the backend implementations (eframe, etc.)
//! that implement the Backend trait from form_factor_core.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "eframe")]
pub mod eframe_backend;

// Miniquad backend - reference implementation for future use
// Uncomment when egui-miniquad supports egui 0.33+
// pub mod miniquad_backend;

#[cfg(feature = "eframe")]
pub use eframe_backend::{EframeBackend, EframeError};
