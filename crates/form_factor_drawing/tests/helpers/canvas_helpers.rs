//! Canvas test helpers for simulating user interactions

use form_factor_drawing::{DrawingCanvas, LayerType, Shape, ToolMode};
use egui::Pos2;

/// Creates a canvas with default test settings
pub fn create_test_canvas() -> DrawingCanvas {
    let mut canvas = DrawingCanvas::default();
    canvas.set_project_name("Test Canvas");
    canvas
}

/// Creates a canvas with the specified number of test shapes
pub fn create_canvas_with_shapes(count: usize) -> DrawingCanvas {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::Rectangle;

    let mut canvas = create_test_canvas();

    for i in 0..count {
        let x = (i as f32) * 50.0;
        let y = (i as f32) * 50.0;

        let rect = Rectangle::from_corners(
            Pos2::new(x, y),
            Pos2::new(x + 40.0, y + 40.0),
            Stroke::new(2.0, Color32::RED),
            Color32::from_rgba_premultiplied(255, 0, 0, 50),
        )
        .expect("Valid rectangle");

        canvas.test_add_shape(Shape::Rectangle(rect));
    }

    canvas
}


#[allow(dead_code)]
pub fn simulate_click(_canvas: &mut DrawingCanvas, _position: Pos2) {
    // TODO: Implement
}

#[allow(dead_code)]
pub fn simulate_drag(_canvas: &mut DrawingCanvas, _from: Pos2, _to: Pos2) {
    // TODO: Implement
}

pub fn get_shapes_on_layer(canvas: &DrawingCanvas, layer: LayerType) -> Vec<&Shape> {
    canvas.shapes_on_layer(layer)
}

#[allow(dead_code)]
pub fn count_shapes_on_layer(canvas: &DrawingCanvas, layer: LayerType) -> usize {
    get_shapes_on_layer(canvas, layer).len()
}

pub fn assert_shape_count(canvas: &DrawingCanvas, expected: usize) {
    let actual = canvas.shapes().len();
    assert_eq!(
        actual, expected,
        "Expected {} shapes, but found {}",
        expected, actual
    );
}

#[allow(dead_code)]
pub fn assert_detection_count(canvas: &DrawingCanvas, expected: usize) {
    let actual = canvas.detections().len();
    assert_eq!(
        actual, expected,
        "Expected {} detections, but found {}",
        expected, actual
    );
}

pub fn assert_active_tool(canvas: &DrawingCanvas, expected: ToolMode) {
    let actual = canvas.current_tool();
    assert_eq!(
        actual, &expected,
        "Expected tool {:?}, but found {:?}",
        expected, actual
    );
}

pub fn assert_zoom_level(canvas: &DrawingCanvas, expected: f32) {
    let actual = canvas.zoom_level();
    assert!(
        (actual - expected).abs() < 0.01,
        "Expected zoom {}, but found {}",
        expected,
        actual
    );
}

pub fn assert_pan_offset(canvas: &DrawingCanvas, expected_x: f32, expected_y: f32) {
    let offset = canvas.pan_offset();
    assert!(
        (offset.x - expected_x).abs() < 0.01 && (offset.y - expected_y).abs() < 0.01,
        "Expected pan offset ({}, {}), but found ({}, {})",
        expected_x,
        expected_y,
        offset.x,
        offset.y
    );
}
