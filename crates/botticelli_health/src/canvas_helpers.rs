//! Canvas test helpers for simulating user interactions

use egui::Pos2;
use form_factor_drawing::{DrawingCanvas, LayerType, Shape, ToolMode};

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

/// Helper to create a rectangle shape
pub fn create_rectangle_shape(x: f32, y: f32, width: f32, height: f32) -> Shape {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::Rectangle;

    let rect = Rectangle::from_corners(
        Pos2::new(x, y),
        Pos2::new(x + width, y + height),
        Stroke::new(2.0, Color32::BLUE),
        Color32::from_rgba_premultiplied(0, 0, 255, 50),
    )
    .expect("Valid rectangle");

    Shape::Rectangle(rect)
}

/// Helper to create a circle shape
pub fn create_circle_shape(center_x: f32, center_y: f32, radius: f32) -> Shape {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::Circle;

    let circle = Circle::new(
        Pos2::new(center_x, center_y),
        radius,
        Stroke::new(2.0, Color32::GREEN),
        Color32::from_rgba_premultiplied(0, 255, 0, 50),
    )
    .expect("Valid circle");

    Shape::Circle(circle)
}

/// Helper to create a freehand/polygon shape
pub fn create_freehand_shape(points: Vec<(f32, f32)>) -> Shape {
    use egui::{Color32, Pos2, Stroke};
    use form_factor_drawing::PolygonShape;

    let pos_points: Vec<Pos2> = points.iter().map(|(x, y)| Pos2::new(*x, *y)).collect();
    let polygon = PolygonShape::from_points(
        pos_points,
        Stroke::new(2.0, Color32::YELLOW),
        Color32::from_rgba_premultiplied(255, 255, 0, 50),
    )
    .expect("Valid polygon");

    Shape::Polygon(polygon)
}

#[allow(dead_code)]
pub fn simulate_click(_canvas: &mut DrawingCanvas, _position: Pos2) {
    // TODO: Implement
}

#[allow(dead_code)]
pub fn simulate_drag(_canvas: &mut DrawingCanvas, _from: Pos2, _to: Pos2) {
    // TODO: Implement
}

/// Gets all shapes on a specific layer
pub fn get_shapes_on_layer(canvas: &DrawingCanvas, layer: LayerType) -> Vec<&Shape> {
    canvas.shapes_on_layer(layer)
}

/// Asserts the total number of shapes on the canvas
pub fn assert_shape_count(canvas: &DrawingCanvas, expected: usize) {
    let actual = canvas.shapes().len();
    assert_eq!(
        actual, expected,
        "Expected {} shapes, but found {}",
        expected, actual
    );
}

/// Asserts the currently active tool mode
pub fn assert_active_tool(canvas: &DrawingCanvas, expected: ToolMode) {
    let actual = canvas.current_tool();
    assert_eq!(
        actual, &expected,
        "Expected tool {:?}, but found {:?}",
        expected, actual
    );
}

/// Asserts the canvas zoom level
pub fn assert_zoom_level(canvas: &DrawingCanvas, expected: f32) {
    let actual = canvas.zoom_level();
    assert!(
        (actual - expected).abs() < 0.01,
        "Expected zoom {}, but found {}",
        expected,
        actual
    );
}

/// Asserts the canvas pan offset
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

/// Helper to select a shape by index
pub fn select_shape(canvas: &mut DrawingCanvas, index: usize) {
    canvas.test_set_selected_shape(Some(index));
}

/// Helper to deselect all shapes
pub fn deselect_all(canvas: &mut DrawingCanvas) {
    canvas.test_set_selected_shape(None);
}
