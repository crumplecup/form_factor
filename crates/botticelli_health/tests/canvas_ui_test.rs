//! UI rendering integration tests
//!
//! These tests verify that the canvas UI renders correctly without panicking
//! for various canvas states. This catches rendering bugs that might not be
//! caught by pure state tests.
//!
//! Tests cover:
//! - Basic UI rendering (smoke tests)
//! - Tool panel rendering for all tool modes  
//! - UI rendering with shapes present
//! - UI rendering with zoom/pan applied
//! - UI rendering in template and instance modes
//! - Multi-page template UI rendering

use botticelli_health::{
    assert_all_tools_render, assert_ui_renders_without_panic, create_canvas_with_shapes,
    create_test_canvas,
};
use form_factor_drawing::{DrawingCanvas, ToolMode};

// ============================================================================
// Basic UI Rendering Tests
// ============================================================================

#[test]
fn test_default_canvas_renders() {
    let mut canvas = DrawingCanvas::default();
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_canvas_with_project_name_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_project_name("Test Project".to_string());
    assert_ui_renders_without_panic(&mut canvas);
}

// ============================================================================
// Tool Panel Rendering Tests
// ============================================================================

#[test]
fn test_all_tool_modes_render() {
    let mut canvas = create_test_canvas();
    assert_all_tools_render(&mut canvas);
}

#[test]
fn test_rectangle_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_circle_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Circle);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_freehand_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Freehand);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_select_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Select);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_edit_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Edit);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_rotate_tool_selected_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rotate);
    assert_ui_renders_without_panic(&mut canvas);
}

// ============================================================================
// UI Rendering with Shapes
// ============================================================================

#[test]
fn test_canvas_with_shapes_renders() {
    let mut canvas = create_canvas_with_shapes(5);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_canvas_with_many_shapes_renders() {
    let mut canvas = create_canvas_with_shapes(20);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_canvas_with_shapes_and_tool_change_renders() {
    let mut canvas = create_canvas_with_shapes(5);
    
    // Change tools while shapes present
    for tool in [
        ToolMode::Select,
        ToolMode::Edit,
        ToolMode::Rectangle,
        ToolMode::Rotate,
    ] {
        canvas.set_tool(tool);
        assert_ui_renders_without_panic(&mut canvas);
    }
}

// ============================================================================
// UI Rendering with Zoom/Pan
// ============================================================================

#[test]
fn test_zoomed_in_canvas_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_zoom(3.0); // 300% zoom
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_zoomed_out_canvas_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_zoom(0.5); // 50% zoom
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_panned_canvas_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_pan_offset(100.0, 50.0);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_zoomed_and_panned_canvas_renders() {
    let mut canvas = create_test_canvas();
    canvas.set_zoom(2.0);
    canvas.set_pan_offset(-50.0, -50.0);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_zoomed_canvas_with_shapes_renders() {
    let mut canvas = create_canvas_with_shapes(5);
    canvas.set_zoom(2.5);
    assert_ui_renders_without_panic(&mut canvas);
}

// ============================================================================
// UI Rendering in Template Mode
// ============================================================================

#[test]
fn test_template_creation_mode_renders() {
    let mut canvas = create_test_canvas();
    canvas.start_template_creation("test_template", "Test Template");
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_template_mode_with_rectangle_tool_renders() {
    let mut canvas = create_test_canvas();
    canvas.start_template_creation("test_template", "Test Template");
    canvas.set_tool(ToolMode::Rectangle);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_template_mode_with_multiple_pages_renders() {
    let mut canvas = create_test_canvas();
    canvas.start_template_creation("test_template", "Test Template");
    
    // Add multiple pages
    let _ = canvas.add_template_page();
    let _ = canvas.add_template_page();
    
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_template_mode_page_navigation_renders() {
    let mut canvas = create_test_canvas();
    canvas.start_template_creation("test_template", "Test Template");
    
    // Add pages
    let _ = canvas.add_template_page();
    let _ = canvas.add_template_page();
    
    // Navigate between pages
    canvas.set_current_page(0);
    assert_ui_renders_without_panic(&mut canvas);
    
    canvas.set_current_page(1);
    assert_ui_renders_without_panic(&mut canvas);
    
    canvas.set_current_page(2);
    assert_ui_renders_without_panic(&mut canvas);
}

// ============================================================================
// UI Rendering with Layer Visibility
// ============================================================================

#[test]
fn test_canvas_with_hidden_layers_renders() {
    use form_factor_drawing::LayerType;
    
    let mut canvas = create_canvas_with_shapes(5);
    
    // Hide shapes layer
    canvas.layer_manager_mut().set_visible(LayerType::Shapes, false);
    assert_ui_renders_without_panic(&mut canvas);
    
    // Show shapes, hide grid
    canvas.layer_manager_mut().set_visible(LayerType::Shapes, true);
    canvas.layer_manager_mut().set_visible(LayerType::Grid, false);
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_canvas_with_all_layers_hidden_renders() {
    use form_factor_drawing::LayerType;
    
    let mut canvas = create_test_canvas();
    
    // Hide all layers
    for layer in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        canvas.layer_manager_mut().set_visible(layer, false);
    }
    
    assert_ui_renders_without_panic(&mut canvas);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_rapid_tool_switching_renders() {
    let mut canvas = create_test_canvas();
    
    // Rapidly switch tools 100 times
    for i in 0..100 {
        let tool = match i % 6 {
            0 => ToolMode::Select,
            1 => ToolMode::Rectangle,
            2 => ToolMode::Circle,
            3 => ToolMode::Freehand,
            4 => ToolMode::Edit,
            _ => ToolMode::Rotate,
        };
        canvas.set_tool(tool);
    }
    
    // Should still render fine
    assert_ui_renders_without_panic(&mut canvas);
}

#[test]
fn test_extreme_zoom_levels_render() {
    let mut canvas = create_canvas_with_shapes(3);
    
    // Test various extreme zoom levels
    for zoom in [0.1, 0.5, 1.0, 2.0, 5.0, 10.0] {
        canvas.set_zoom(zoom);
        assert_ui_renders_without_panic(&mut canvas);
    }
}

#[test]
fn test_extreme_pan_offsets_render() {
    let mut canvas = create_canvas_with_shapes(3);
    
    // Test various extreme pan offsets
    for offset in [(-1000.0, -1000.0), (1000.0, 1000.0), (-500.0, 500.0)] {
        canvas.set_pan_offset(offset.0, offset.1);
        assert_ui_renders_without_panic(&mut canvas);
    }
}
