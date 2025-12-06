//! Test helpers for integration tests
//!
//! This module provides utilities for creating test fixtures, simulating
//! user interactions, and asserting expected behaviors in integration tests.
//!
//! # Modules
//!
//! - `canvas_helpers` - Canvas creation and interaction simulation
//!
//! Note: Plugin helpers are in the form_factor crate tests, not here,
//! since they require access to the form_factor_plugins crate.
//!
//! # Examples
//!
//! ```
//! // Create a test canvas
//! use helpers::create_test_canvas;
//! let canvas = create_test_canvas();
//! ```

pub mod canvas_helpers;

// Note: plugin_helpers module is not included here because it requires
// form_factor_plugins which is not a dependency of form_factor_drawing.
// Plugin coordination tests should be in the form_factor crate.

// Re-export commonly used functions for convenience
pub use canvas_helpers::{
    assert_active_tool, assert_pan_offset, assert_shape_count, assert_zoom_level,
    create_canvas_with_shapes, create_test_canvas, get_shapes_on_layer,
};
