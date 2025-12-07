use form_factor::{Circle, DrawingCanvas, FormError, Rectangle, Shape};
use egui::{Color32, Pos2, Stroke};

#[test]
fn test_shape_creation_and_persistence() -> Result<(), FormError> {
    // Create drawing canvas
    let mut canvas = DrawingCanvas::new();

    // Verify no shapes initially
    assert_eq!(
        canvas.shapes().len(),
        0,
        "Should start with no shapes"
    );

    // Create a rectangle shape
    let rect = Rectangle::from_corners(
        Pos2::new(10.0, 10.0),
        Pos2::new(100.0, 100.0),
        Stroke::new(2.0, Color32::RED),
        Color32::TRANSPARENT,
    )?;
    let shape = Shape::Rectangle(rect);

    // Add shape to canvas
    canvas.add_shape(shape.clone());

    // Verify shape was added
    assert_eq!(
        canvas.shapes().len(),
        1,
        "Shape should be persisted in canvas"
    );

    // Verify the shape matches what we added
    let added_shape = &canvas.shapes()[0];
    assert_eq!(added_shape, &shape);
    Ok(())
}

#[test]
fn test_multiple_shape_creation() -> Result<(), FormError> {
    let mut canvas = DrawingCanvas::new();

    // Add multiple shapes
    for i in 0..5 {
        let x = (i * 20) as f32;
        let rect = Rectangle::from_corners(
            Pos2::new(x, x),
            Pos2::new(x + 50.0, x + 50.0),
            Stroke::new(1.0, Color32::BLUE),
            Color32::TRANSPARENT,
        )?;
        let shape = Shape::Rectangle(rect);
        canvas.add_shape(shape);
    }

    // Verify all shapes were added
    assert_eq!(
        canvas.shapes().len(),
        5,
        "All shapes should be persisted"
    );
    Ok(())
}

#[test]
fn test_shape_drawing_workflow() -> Result<(), FormError> {
    let mut canvas = DrawingCanvas::new();

    // Simulate drawing workflow:
    // 1. User starts drawing at a point
    let start_point = Pos2::new(10.0, 10.0);

    // 2. User drags to create shape
    let end_point = Pos2::new(100.0, 100.0);
    let rect = Rectangle::from_corners(
        start_point,
        end_point,
        Stroke::new(2.0, Color32::GREEN),
        Color32::TRANSPARENT,
    )?;
    let shape = Shape::Rectangle(rect);

    // 3. User finishes drawing - shape should be added
    canvas.add_shape(shape);

    // Verify shape persists after drawing completes
    assert_eq!(
        canvas.shapes().len(),
        1,
        "Completed shape should persist"
    );
    Ok(())
}

#[test]
fn test_mixed_shape_types() -> Result<(), FormError> {
    let mut canvas = DrawingCanvas::new();

    // Add a rectangle
    let rect = Rectangle::from_corners(
        Pos2::new(0.0, 0.0),
        Pos2::new(50.0, 50.0),
        Stroke::new(1.0, Color32::RED),
        Color32::TRANSPARENT,
    )?;
    canvas.add_shape(Shape::Rectangle(rect));

    // Add a circle
    let circle = Circle::new(
        Pos2::new(100.0, 100.0),
        25.0,
        Stroke::new(1.0, Color32::BLUE),
        Color32::TRANSPARENT,
    )?;
    canvas.add_shape(Shape::Circle(circle));

    // Verify both shapes were added
    assert_eq!(
        canvas.shapes().len(),
        2,
        "Both rectangle and circle should persist"
    );

    // Verify shape types
    match &canvas.shapes()[0] {
        Shape::Rectangle(_) => {}, // Expected
        _ => panic!("First shape should be Rectangle"),
    }
    match &canvas.shapes()[1] {
        Shape::Circle(_) => {}, // Expected
        _ => panic!("Second shape should be Circle"),
    }
    Ok(())
}
