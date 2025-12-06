//! Integration tests for DrawingCanvas workflows
//!
//! Tests cover:
//! - Tool mode workflows (Rectangle, Circle, Freehand, Select, Edit, Rotate)
//! - Canvas state machine transitions
//! - Zoom and pan integration
//! - Layer integration with drawing

mod helpers;

use form_factor_drawing::{CanvasState, DrawingCanvas, LayerType, Shape, ToolMode};
use helpers::{
    assert_active_tool, assert_pan_offset, assert_shape_count, assert_zoom_level,
    create_canvas_with_shapes, create_test_canvas, get_shapes_on_layer,
};

// ============================================================================
// Helper Verification Tests (ensure test infrastructure works)
// ============================================================================

#[test]
fn test_helpers_create_test_canvas() {
    let canvas = create_test_canvas();
    assert_eq!(canvas.project_name(), "Test Canvas");
    assert_shape_count(&canvas, 0);
    assert_active_tool(&canvas, ToolMode::Select);
    assert_zoom_level(&canvas, 5.0);
}

// TODO: Fix builder pattern issues
// #[test]
// fn test_helpers_create_canvas_with_shapes() {
//     let canvas = create_canvas_with_shapes(3);
//     assert_shape_count(&canvas, 3);

//     let shapes = get_shapes_on_layer(&canvas, LayerType::Shapes);
//     assert_eq!(shapes.len(), 3);
// }

// #[test]
// fn test_helpers_create_canvas_with_single_shape() {
//     let canvas = create_canvas_with_single_shape();
//     assert_shape_count(&canvas, 1);

//     // Verify it's a rectangle
//     let shapes = canvas.shapes();
//     assert!(matches!(shapes[0], Shape::Rectangle(_)));
// }

// ============================================================================
// Basic Canvas Operations
// ============================================================================

#[test]
fn test_canvas_default_state() {
    let canvas = DrawingCanvas::new();

    // Check initial state
    assert_eq!(canvas.project_name(), "Untitled");
    assert_eq!(canvas.shapes().len(), 0);
    assert_eq!(canvas.detections().len(), 0);
    assert_eq!(canvas.current_tool(), &ToolMode::Select);
    assert_eq!(canvas.zoom_level(), &5.0);

    // Check initial canvas state (using test-only API)
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_tool_mode_switching() {
    let mut canvas = create_test_canvas();

    // Start with Select tool
    assert_active_tool(&canvas, ToolMode::Select);

    // Switch to Rectangle
    canvas.set_tool(ToolMode::Rectangle);
    assert_active_tool(&canvas, ToolMode::Rectangle);

    // Switch to Circle
    canvas.set_tool(ToolMode::Circle);
    assert_active_tool(&canvas, ToolMode::Circle);

    // Switch to Freehand
    canvas.set_tool(ToolMode::Freehand);
    assert_active_tool(&canvas, ToolMode::Freehand);

    // Switch to Edit
    canvas.set_tool(ToolMode::Edit);
    assert_active_tool(&canvas, ToolMode::Edit);

    // Switch to Rotate
    canvas.set_tool(ToolMode::Rotate);
    assert_active_tool(&canvas, ToolMode::Rotate);
}

#[test]
fn test_zoom_level_modification() {
    let mut canvas = create_test_canvas();

    // Initial zoom
    assert_zoom_level(&canvas, 5.0);

    // Change zoom
    canvas.set_zoom(10.0);
    assert_zoom_level(&canvas, 10.0);

    // Change zoom again
    canvas.set_zoom(2.5);
    assert_zoom_level(&canvas, 2.5);
}

#[test]
fn test_shape_addition() {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::{Circle, Rectangle};

    let mut canvas = create_test_canvas();
    assert_shape_count(&canvas, 0);

    // Add a rectangle
    let rect = Rectangle::from_corners(
        Pos2::new(10.0, 20.0),
        Pos2::new(110.0, 70.0),
        Stroke::new(2.0, Color32::RED),
        Color32::from_rgba_premultiplied(255, 0, 0, 50),
    )
    .expect("Valid rectangle");

    canvas.test_add_shape(Shape::Rectangle(rect));
    assert_shape_count(&canvas, 1);

    // Add a circle
    let circle = Circle::new(
        Pos2::new(50.0, 50.0),
        25.0,
        Stroke::new(2.0, Color32::BLUE),
        Color32::from_rgba_premultiplied(0, 0, 255, 50),
    )
    .expect("Valid circle");

    canvas.test_add_shape(Shape::Circle(circle));
    assert_shape_count(&canvas, 2);
}

#[test]
fn test_project_name_modification() {
    let mut canvas = create_test_canvas();
    assert_eq!(canvas.project_name(), "Test Canvas");

    canvas.set_project_name("My Project");
    assert_eq!(canvas.project_name(), "My Project");

    canvas.set_project_name("Another Name");
    assert_eq!(canvas.project_name(), "Another Name");
}

// ============================================================================
// Layer Integration Tests
// ============================================================================

#[test]
fn test_shapes_on_shapes_layer() {
    let canvas = create_canvas_with_shapes(5);

    let shapes_layer = get_shapes_on_layer(&canvas, LayerType::Shapes);
    assert_eq!(shapes_layer.len(), 5);

    let detections_layer = get_shapes_on_layer(&canvas, LayerType::Detections);
    assert_eq!(detections_layer.len(), 0);
}

#[test]
fn test_detections_separate_from_shapes() {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::Rectangle;

    let mut canvas = create_canvas_with_shapes(3);

    // Add a detection (these go to detections layer)
    let detection_rect = Rectangle::from_corners(
        Pos2::new(500.0, 500.0),
        Pos2::new(550.0, 530.0),
        Stroke::new(2.0, Color32::BLUE),
        Color32::from_rgba_premultiplied(0, 0, 255, 50),
    )
    .expect("Valid rectangle");

    canvas.test_add_detection(Shape::Rectangle(detection_rect));

    // Shapes and detections are separate
    assert_eq!(canvas.shapes().len(), 3);
    assert_eq!(canvas.detections().len(), 1);

    let shapes_layer = get_shapes_on_layer(&canvas, LayerType::Shapes);
    assert_eq!(shapes_layer.len(), 3);

    let detections_layer = get_shapes_on_layer(&canvas, LayerType::Detections);
    assert_eq!(detections_layer.len(), 1);
}

// ============================================================================
// TODO: Tool Workflow Tests (to be implemented in Phase 1)
// ============================================================================

// TODO: test_rectangle_tool_workflow
// TODO: test_circle_tool_workflow
// TODO: test_freehand_tool_workflow
// TODO: test_select_tool_selects_shapes
// TODO: test_edit_tool_vertex_dragging
// TODO: test_rotate_tool_rotates_selection

// ============================================================================
// State Machine Tests
// ============================================================================

#[test]
fn test_initial_state_is_idle() {
    let canvas = create_test_canvas();
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_state_persists_across_tool_changes() {
    let mut canvas = create_test_canvas();

    // State should be Idle initially
    assert_eq!(canvas.current_state(), &CanvasState::Idle);

    // State remains Idle when switching tools
    canvas.set_tool(ToolMode::Rectangle);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);

    canvas.set_tool(ToolMode::Circle);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);

    canvas.set_tool(ToolMode::Select);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_state_remains_idle_with_shapes() {
    let canvas = create_canvas_with_shapes(5);

    // Even with shapes present, state should be Idle
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_shape_count(&canvas, 5);
}

// ============================================================================
// Zoom and Pan Integration Tests
// ============================================================================

#[test]
fn test_pan_offset_state() {
    let mut canvas = create_test_canvas();

    // Initial pan offset should be (0, 0)
    assert_pan_offset(&canvas, 0.0, 0.0);

    // Set pan offset
    canvas.set_pan_offset(10.0, 20.0);
    assert_pan_offset(&canvas, 10.0, 20.0);

    // Change pan offset again
    canvas.set_pan_offset(-5.0, 15.0);
    assert_pan_offset(&canvas, -5.0, 15.0);
}

#[test]
fn test_zoom_and_pan_persistence() {
    let mut canvas = create_test_canvas();

    // Set custom zoom and pan
    canvas.set_zoom(10.0);
    canvas.set_pan_offset(25.0, -10.0);

    // Verify persistence across tool changes
    canvas.set_tool(ToolMode::Rectangle);
    assert_zoom_level(&canvas, 10.0);
    assert_pan_offset(&canvas, 25.0, -10.0);

    canvas.set_tool(ToolMode::Circle);
    assert_zoom_level(&canvas, 10.0);
    assert_pan_offset(&canvas, 25.0, -10.0);
}

#[test]
fn test_zoom_with_shapes() {
    let canvas = create_canvas_with_shapes(3);

    // Shapes should exist regardless of zoom level
    assert_shape_count(&canvas, 3);
    assert_zoom_level(&canvas, 5.0);

    // Shapes remain when we change zoom (zoom doesn't affect shape storage)
    let mut canvas = canvas;
    canvas.set_zoom(20.0);
    assert_shape_count(&canvas, 3);
    assert_zoom_level(&canvas, 20.0);
}

#[test]
fn test_pan_with_shapes() {
    let canvas = create_canvas_with_shapes(3);

    // Initial state
    assert_shape_count(&canvas, 3);
    assert_pan_offset(&canvas, 0.0, 0.0);

    // Pan doesn't affect shape storage
    let mut canvas = canvas;
    canvas.set_pan_offset(100.0, 50.0);
    assert_shape_count(&canvas, 3);
    assert_pan_offset(&canvas, 100.0, 50.0);
}
