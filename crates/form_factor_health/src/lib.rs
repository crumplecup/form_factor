//! Health checks, testing helpers, and integration tests for Form Factor.
//!
//! This crate provides public testing utilities and integration tests
//! that exercise the Form Factor application through its public API.
//! It is a leaf dependency that imports the main facade crate.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod accessibility_helpers;
mod canvas_helpers;
mod performance_helpers;
mod plugin_helpers;
mod test_plugins;

pub use accessibility_helpers::{
    assert_all_tools_render, assert_ui_renders_without_panic, render_canvas_ui,
};
pub use canvas_helpers::{
    assert_active_tool, assert_pan_offset, assert_shape_count, assert_zoom_level,
    create_canvas_with_shapes, create_circle_shape, create_freehand_shape, create_rectangle_shape,
    create_test_canvas, deselect_all, get_shapes_on_layer, select_shape,
};
pub use performance_helpers::{BenchmarkResult, measure_operation};
pub use plugin_helpers::*;
pub use test_plugins::{
    CountingPlugin, EventCollectorPlugin, ResponsePlugin, create_test_plugin,
    create_test_plugin_with_name,
};
