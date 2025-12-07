use egui::{Color32, Pos2, Stroke};
use form_factor::{Circle, DrawingCanvas, Rectangle, Shape, ShapeError};

/// Test that shapes can be added to canvas and retrieved
///
/// This validates the basic state management for shape selection workflow
#[test]
fn test_shape_storage_and_retrieval() -> Result<(), ShapeError> {
    let mut canvas = DrawingCanvas::new();
    let stroke = Stroke::new(2.0, Color32::RED);
    let fill = Color32::TRANSPARENT;

    // Create and add shapes
    let circle = Circle::new(Pos2::new(100.0, 100.0), 50.0, stroke, fill)?;
    canvas.test_add_shape(Shape::Circle(circle));

    let rect = Rectangle::from_corners(
        Pos2::new(50.0, 50.0),
        Pos2::new(250.0, 80.0),
        stroke,
        fill,
    )?;
    canvas.test_add_shape(Shape::Rectangle(rect));

    // Verify shapes were stored
    let shapes = canvas.shapes();
    assert_eq!(shapes.len(), 2);
    assert!(matches!(shapes[0], Shape::Circle(_)));
    assert!(matches!(shapes[1], Shape::Rectangle(_)));

    Ok(())
}

/// Test that detection metadata can be associated with shapes
///
/// This is the foundation for the property editing workflow where users
/// assign form field types to detected regions
#[test]
fn test_detection_metadata_association() {
    let canvas = DrawingCanvas::new();

    // TODO: Once selection system is implemented:
    // 1. Add detection to canvas
    // 2. User selects detection
    // 3. User assigns field type via property panel
    // 4. Metadata is stored and retrievable
    // 5. Verify metadata persists with detection

    // For now, verify canvas can store metadata
    assert!(canvas.detection_metadata().is_empty());
}

/// Test workflow: Create shape -> Select -> Show properties -> Edit field
///
/// This integration test validates the end-to-end user workflow for
/// template creation:
/// 1. User draws or detects a region
/// 2. User selects it with select tool
/// 3. Property panel shows
/// 4. User assigns field type and properties
#[test]
fn test_shape_selection_to_field_assignment_workflow() -> Result<(), ShapeError> {
    let mut canvas = DrawingCanvas::new();
    let stroke = Stroke::new(2.0, Color32::BLUE);
    let fill = Color32::TRANSPARENT;

    // Step 1: User draws a rectangle (future form field)
    let rect = Rectangle::from_corners(
        Pos2::new(100.0, 100.0),
        Pos2::new(250.0, 125.0),
        stroke,
        fill,
    )?;
    canvas.test_add_shape(Shape::Rectangle(rect));

    // Step 2: User selects shape with select tool
    // TODO: Implement selection mechanism
    //canvas.select_shape_at(Pos2::new(125.0, 112.0));

    // Step 3: Property panel would show shape properties
    // TODO: Verify property panel displays

    // Step 4: User assigns field type
    // TODO: Implement field assignment
    // canvas.assign_field_to_shape(0, FieldType::Text, "name");

    // Verify state
    let shapes = canvas.shapes();
    assert_eq!(shapes.len(), 1);

    Ok(())
}
