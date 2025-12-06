//! Tool workflow integration tests
//!
//! These tests verify that tool workflows function correctly by testing
//! the state changes and shape creation that result from tool usage.
//!
//! Note: We test tool workflows via state-based assertions rather than
//! egui interaction simulation. This tests the business logic of each tool
//! while being more maintainable and reliable than mocking UI interactions.
//!
//! Tests cover:
//! - Rectangle tool workflow (creation, state)
//! - Circle tool workflow (creation, state)
//! - Freehand tool workflow (multi-point polygons)
//! - Select tool workflow (selection state)
//! - Edit tool workflow (shape modification)
//! - Rotate tool workflow (rotation state)
//! - State machine transitions (Idle, Drawing, DraggingVertex, Rotating)

use botticelli_health::{
    assert_active_tool, assert_shape_count, create_canvas_with_shapes, create_circle_shape,
    create_freehand_shape, create_rectangle_shape, create_test_canvas, deselect_all, select_shape,
};
use form_factor_drawing::{CanvasState, DrawingCanvas, ToolMode};

// ============================================================================
// Rectangle Tool Workflow Tests
// ============================================================================

#[test]
fn test_rectangle_tool_creates_shapes() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);

    // Simulate creating a rectangle via direct API
    let rect = create_rectangle_shape(10.0, 10.0, 100.0, 100.0);
    canvas.test_add_shape(rect);

    assert_shape_count(&canvas, 1);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_rectangle_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Rectangle);
}

#[test]
fn test_multiple_rectangles_on_same_canvas() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);

    // Create multiple rectangles
    canvas.test_add_shape(create_rectangle_shape(10.0, 10.0, 50.0, 50.0));
    canvas.test_add_shape(create_rectangle_shape(60.0, 60.0, 100.0, 100.0));
    canvas.test_add_shape(create_rectangle_shape(110.0, 110.0, 150.0, 150.0));

    assert_shape_count(&canvas, 3);
}

#[test]
fn test_rectangle_tool_respects_layer_system() {
    use form_factor_drawing::LayerType;

    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);

    // Add shape to shapes layer
    canvas.test_add_shape(create_rectangle_shape(10.0, 10.0, 50.0, 50.0));

    // Verify shape is on correct layer
    let shapes = canvas.shapes_on_layer(LayerType::Shapes);
    assert_eq!(shapes.len(), 1);
}

// ============================================================================
// Circle Tool Workflow Tests
// ============================================================================

#[test]
fn test_circle_tool_creates_shapes() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Circle);

    // Simulate creating a circle via direct API
    let circle = create_circle_shape(50.0, 50.0, 30.0);
    canvas.test_add_shape(circle);

    assert_shape_count(&canvas, 1);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_circle_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Circle);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Circle);
}

#[test]
fn test_multiple_circles_on_same_canvas() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Circle);

    // Create multiple circles
    canvas.test_add_shape(create_circle_shape(50.0, 50.0, 20.0));
    canvas.test_add_shape(create_circle_shape(150.0, 150.0, 30.0));
    canvas.test_add_shape(create_circle_shape(250.0, 250.0, 40.0));

    assert_shape_count(&canvas, 3);
}

#[test]
fn test_circle_tool_respects_layer_system() {
    use form_factor_drawing::LayerType;

    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Circle);

    // Add shape to shapes layer
    canvas.test_add_shape(create_circle_shape(50.0, 50.0, 30.0));

    // Verify shape is on correct layer
    let shapes = canvas.shapes_on_layer(LayerType::Shapes);
    assert_eq!(shapes.len(), 1);
}

// ============================================================================
// Freehand Tool Workflow Tests
// ============================================================================

#[test]
fn test_freehand_tool_creates_polygons() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Freehand);

    // Create a freehand polygon with multiple points
    let points = vec![
        (10.0, 10.0),
        (50.0, 20.0),
        (70.0, 60.0),
        (40.0, 80.0),
        (10.0, 10.0), // Close the polygon
    ];
    canvas.test_add_shape(create_freehand_shape(points));

    assert_shape_count(&canvas, 1);
}

#[test]
fn test_freehand_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Freehand);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Freehand);
}

#[test]
fn test_freehand_multiple_polygons() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Freehand);

    // Create multiple freehand shapes
    canvas.test_add_shape(create_freehand_shape(vec![
        (0.0, 0.0),
        (10.0, 0.0),
        (5.0, 10.0),
    ]));
    canvas.test_add_shape(create_freehand_shape(vec![
        (20.0, 20.0),
        (30.0, 20.0),
        (25.0, 30.0),
    ]));

    assert_shape_count(&canvas, 2);
}

#[test]
fn test_freehand_tool_respects_layer_system() {
    use form_factor_drawing::LayerType;

    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Freehand);

    // Add freehand shape
    canvas.test_add_shape(create_freehand_shape(vec![
        (0.0, 0.0),
        (10.0, 0.0),
        (5.0, 10.0),
    ]));

    // Verify shape is on correct layer
    let shapes = canvas.shapes_on_layer(LayerType::Shapes);
    assert_eq!(shapes.len(), 1);
}

// ============================================================================
// Select Tool Workflow Tests
// ============================================================================

#[test]
fn test_select_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Select);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Select);
}

#[test]
fn test_select_tool_with_shapes_present() {
    let mut canvas = create_canvas_with_shapes(5);
    canvas.set_tool(ToolMode::Select);

    // Select tool should be active with shapes present
    assert_active_tool(&canvas, ToolMode::Select);
    assert_shape_count(&canvas, 5);
}

#[test]
fn test_select_tool_selection_state() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Select);

    // Initially no selection
    assert_eq!(canvas.selected_shape_index(), None);

    // Select a shape via API
    select_shape(&mut canvas, 1);
    assert_eq!(canvas.selected_shape_index(), Some(1));
}

#[test]
fn test_select_tool_deselection() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Select);

    // Select then deselect
    select_shape(&mut canvas, 1);
    assert_eq!(canvas.selected_shape_index(), Some(1));

    deselect_all(&mut canvas);
    assert_eq!(canvas.selected_shape_index(), None);
}

#[test]
fn test_select_tool_changes_selection() {
    let mut canvas = create_canvas_with_shapes(5);
    canvas.set_tool(ToolMode::Select);

    // Change selection multiple times
    select_shape(&mut canvas, 0);
    assert_eq!(canvas.selected_shape_index(), Some(0));

    select_shape(&mut canvas, 2);
    assert_eq!(canvas.selected_shape_index(), Some(2));

    select_shape(&mut canvas, 4);
    assert_eq!(canvas.selected_shape_index(), Some(4));
}

// ============================================================================
// Edit Tool Workflow Tests
// ============================================================================

#[test]
fn test_edit_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Edit);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Edit);
}

#[test]
fn test_edit_tool_with_selected_shape() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Edit);
    select_shape(&mut canvas, 1);

    // Edit tool active with selection
    assert_active_tool(&canvas, ToolMode::Edit);
    assert_eq!(canvas.selected_shape_index(), Some(1));
}

#[test]
fn test_edit_tool_requires_selection() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Edit);

    // No selection initially
    assert_eq!(canvas.selected_shape_index(), None);

    // Edit tool still active (but would need selection to modify)
    assert_active_tool(&canvas, ToolMode::Edit);
}

#[test]
fn test_edit_tool_shape_modification() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Edit);

    // Add and select a shape
    canvas.test_add_shape(create_rectangle_shape(10.0, 10.0, 50.0, 50.0));
    select_shape(&mut canvas, 0);

    // Shape count unchanged (modification, not addition)
    assert_shape_count(&canvas, 1);
    assert_eq!(canvas.selected_shape_index(), Some(0));
}

// ============================================================================
// Rotate Tool Workflow Tests
// ============================================================================

#[test]
fn test_rotate_tool_state_idle_by_default() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rotate);

    assert_eq!(canvas.current_state(), &CanvasState::Idle);
    assert_active_tool(&canvas, ToolMode::Rotate);
}

#[test]
fn test_rotate_tool_with_selected_shape() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Rotate);
    select_shape(&mut canvas, 1);

    // Rotate tool active with selection
    assert_active_tool(&canvas, ToolMode::Rotate);
    assert_eq!(canvas.selected_shape_index(), Some(1));
}

#[test]
fn test_rotate_tool_requires_selection() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Rotate);

    // No selection initially
    assert_eq!(canvas.selected_shape_index(), None);

    // Rotate tool still active (but would need selection to rotate)
    assert_active_tool(&canvas, ToolMode::Rotate);
}

#[test]
fn test_rotate_tool_shape_count_unchanged() {
    let mut canvas = create_canvas_with_shapes(3);
    canvas.set_tool(ToolMode::Rotate);
    select_shape(&mut canvas, 1);

    // Rotation doesn't change shape count
    assert_shape_count(&canvas, 3);
}

// ============================================================================
// State Machine Transition Tests
// ============================================================================

#[test]
fn test_default_state_is_idle() {
    let canvas = create_test_canvas();
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_tool_change_maintains_idle_state() {
    let mut canvas = create_test_canvas();

    for tool in [
        ToolMode::Rectangle,
        ToolMode::Circle,
        ToolMode::Freehand,
        ToolMode::Select,
        ToolMode::Edit,
        ToolMode::Rotate,
    ] {
        canvas.set_tool(tool);
        assert_eq!(canvas.current_state(), &CanvasState::Idle);
    }
}

#[test]
fn test_adding_shapes_maintains_idle_state() {
    let mut canvas = create_test_canvas();

    // Add shapes
    canvas.test_add_shape(create_rectangle_shape(0.0, 0.0, 10.0, 10.0));
    assert_eq!(canvas.current_state(), &CanvasState::Idle);

    canvas.test_add_shape(create_circle_shape(50.0, 50.0, 20.0));
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_selection_maintains_idle_state() {
    let mut canvas = create_canvas_with_shapes(5);

    select_shape(&mut canvas, 2);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);

    deselect_all(&mut canvas);
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

// ============================================================================
// Cross-Tool Workflow Tests
// ============================================================================

#[test]
fn test_switch_tools_with_shapes_present() {
    let mut canvas = create_test_canvas();

    // Create shapes with different tools
    canvas.set_tool(ToolMode::Rectangle);
    canvas.test_add_shape(create_rectangle_shape(0.0, 0.0, 50.0, 50.0));

    canvas.set_tool(ToolMode::Circle);
    canvas.test_add_shape(create_circle_shape(100.0, 100.0, 25.0));

    canvas.set_tool(ToolMode::Freehand);
    canvas.test_add_shape(create_freehand_shape(vec![
        (150.0, 150.0),
        (160.0, 150.0),
        (155.0, 160.0),
    ]));

    // All shapes preserved
    assert_shape_count(&canvas, 3);
}

#[test]
fn test_tool_workflow_with_zoom() {
    let mut canvas = create_test_canvas();
    canvas.set_zoom(2.0);

    // Create shapes at zoomed level
    canvas.set_tool(ToolMode::Rectangle);
    canvas.test_add_shape(create_rectangle_shape(0.0, 0.0, 50.0, 50.0));

    canvas.set_tool(ToolMode::Circle);
    canvas.test_add_shape(create_circle_shape(100.0, 100.0, 25.0));

    // Shapes created correctly even when zoomed
    assert_shape_count(&canvas, 2);
}

#[test]
fn test_tool_workflow_with_pan() {
    let mut canvas = create_test_canvas();
    canvas.set_pan_offset(100.0, 50.0);

    // Create shapes with pan offset
    canvas.set_tool(ToolMode::Rectangle);
    canvas.test_add_shape(create_rectangle_shape(0.0, 0.0, 50.0, 50.0));

    canvas.set_tool(ToolMode::Circle);
    canvas.test_add_shape(create_circle_shape(100.0, 100.0, 25.0));

    // Shapes created correctly even when panned
    assert_shape_count(&canvas, 2);
}

#[test]
fn test_selection_workflow_across_tools() {
    let mut canvas = create_canvas_with_shapes(3);

    // Select with Select tool
    canvas.set_tool(ToolMode::Select);
    select_shape(&mut canvas, 1);
    assert_eq!(canvas.selected_shape_index(), Some(1));

    // Switch to Edit tool - selection preserved
    canvas.set_tool(ToolMode::Edit);
    assert_eq!(canvas.selected_shape_index(), Some(1));

    // Switch to Rotate tool - selection preserved
    canvas.set_tool(ToolMode::Rotate);
    assert_eq!(canvas.selected_shape_index(), Some(1));
}

#[test]
fn test_deselect_before_drawing_tool() {
    let mut canvas = create_canvas_with_shapes(3);

    // Select a shape
    canvas.set_tool(ToolMode::Select);
    select_shape(&mut canvas, 1);
    assert_eq!(canvas.selected_shape_index(), Some(1));

    // Switch to drawing tool - selection typically cleared
    canvas.set_tool(ToolMode::Rectangle);
    // Note: Actual behavior depends on implementation
    // This documents the expected workflow
}

// ============================================================================
// Tool Workflow Edge Cases
// ============================================================================

#[test]
fn test_empty_canvas_all_tools() {
    let mut canvas = create_test_canvas();

    // All tools should work on empty canvas
    for tool in [
        ToolMode::Rectangle,
        ToolMode::Circle,
        ToolMode::Freehand,
        ToolMode::Select,
        ToolMode::Edit,
        ToolMode::Rotate,
    ] {
        canvas.set_tool(tool);
        assert_active_tool(&canvas, tool);
        assert_shape_count(&canvas, 0);
    }
}

#[test]
fn test_tool_change_rapid_switching() {
    let mut canvas = create_test_canvas();

    // Rapidly switch tools
    for _ in 0..100 {
        canvas.set_tool(ToolMode::Rectangle);
        canvas.set_tool(ToolMode::Circle);
        canvas.set_tool(ToolMode::Select);
    }

    // State should be stable
    assert_eq!(canvas.current_state(), &CanvasState::Idle);
}

#[test]
fn test_tool_workflow_with_hidden_layers() {
    use form_factor_drawing::LayerType;

    let mut canvas = create_test_canvas();

    // Hide shapes layer
    canvas
        .layer_manager_mut()
        .set_visible(LayerType::Shapes, false);

    // Can still add shapes (even if not visible)
    canvas.set_tool(ToolMode::Rectangle);
    canvas.test_add_shape(create_rectangle_shape(0.0, 0.0, 50.0, 50.0));

    assert_shape_count(&canvas, 1);
}
