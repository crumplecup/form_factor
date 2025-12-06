//! Test helpers for integration tests
//!
//! This module provides utilities for creating test fixtures, simulating
//! user interactions, testing UI rendering via accessibility, and asserting
//! expected behaviors in integration tests.
//!
//! # Modules
//!
//! - `accessibility_helpers` - UI rendering tests via AccessKit
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
//!
//! // Test UI rendering
//! use helpers::render_canvas_ui_with_accessibility;
//! let tree = render_canvas_ui_with_accessibility(&mut canvas);
//! ```

pub mod accessibility_helpers;
pub mod canvas_helpers;

// Note: plugin_helpers module is not included here because it requires
// form_factor_plugins which is not a dependency of form_factor_drawing.
// Plugin coordination tests should be in the form_factor crate.

// Re-export commonly used functions for convenience
pub use accessibility_helpers::{
    assert_all_tools_render, assert_ui_renders_without_panic, render_canvas_ui,
};
pub use canvas_helpers::{
    assert_active_tool, assert_pan_offset, assert_shape_count, assert_zoom_level,
    create_canvas_with_shapes, create_circle_shape, create_freehand_shape,
    create_rectangle_shape, create_test_canvas, deselect_all, get_shapes_on_layer,
    select_shape,
};
