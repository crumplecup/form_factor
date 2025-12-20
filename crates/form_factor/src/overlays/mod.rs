//! Overlay system for modal and side panel UI components.
//!
//! This module provides infrastructure for displaying overlays on top of the main canvas,
//! including modal dialogs, side panels, and contextual menus.

mod manager;
mod template_browser;

pub use manager::{Overlay, OverlayManager, OverlayResponse};
pub use template_browser::{TemplateBrowserAction, TemplateBrowserOverlay};
