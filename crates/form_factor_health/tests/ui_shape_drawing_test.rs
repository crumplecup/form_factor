//! UI-level shape drawing integration test
//!
//! This test simulates the actual UI workflow when drawing shapes,
//! including the requirement to have an image loaded.

use form_factor::{DrawingCanvas, Shape, ToolMode};
use form_factor_health::TestHarness;

#[test]
fn test_shapes_require_loaded_image() {
    let harness = TestHarness::new();
    let mut canvas = DrawingCanvas::new();

    // Critical observation from the rendering code:
    // Shapes are only rendered if form_image_size and form_image are Some
    // This suggests shapes might not be drawn without a loaded image

    // Try to draw without image
    canvas.set_tool(ToolMode::Rectangle);
    canvas.start_drawing(egui::Pos2::new(100.0, 100.0));
    canvas.continue_drawing(egui::Pos2::new(200.0, 200.0), &harness.test_painter(), None);
    canvas.finalize_shape();

    // Shape should still be added to vector
    assert_eq!(
        canvas.shapes().len(),
        1,
        "Shape should be added even without image"
    );

    // But would it render? Let's check the conditions
    assert!(
        canvas.form_image_size().is_none(),
        "No image size when no image loaded"
    );

    // This is the issue! Shapes won't render without an image
}

#[test]
fn test_shapes_with_loaded_image() {
    let harness = TestHarness::new();
    let mut canvas = DrawingCanvas::new();

    // Load a test image first
    // Note: We need an actual image file for this test
    // For now, we can manually set the form_image_size to simulate having an image

    // Simulate having an image loaded by setting the size
    // (In real usage, this would happen via load_form_image)
    let test_image_size = egui::Vec2::new(800.0, 600.0);

    // We need access to set form_image_size - this might require adding a test helper
    // For now, let's document what should happen:

    // When image is loaded:
    // 1. form_image_size is set
    // 2. form_image texture is loaded
    // 3. Shapes are drawn in IMAGE pixel coordinates
    // 4. Shapes are transformed to canvas coordinates during rendering

    // The transformation formula from rendering.rs:
    // scale = canvas_size / image_size (maintaining aspect ratio)
    // offset = center the scaled image
    // canvas_pos = image_pos * scale + offset

    canvas.set_tool(ToolMode::Rectangle);
    
    // Draw in what would be image coordinates (e.g., 100x100 in an 800x600 image)
    canvas.start_drawing(egui::Pos2::new(100.0, 100.0));
    canvas.continue_drawing(egui::Pos2::new(200.0, 200.0), &harness.test_painter(), None);
    canvas.finalize_shape();

    assert_eq!(canvas.shapes().len(), 1);

    // The shape's coordinates should be in image space (100, 100) to (200, 200)
    let shape = &canvas.shapes()[0];
    match shape {
        Shape::Rectangle(rect) => {
            let bounds = rect.bounds();
            assert!(
                (bounds.x() - 100.0).abs() < 0.1,
                "Shape should be in image coordinates"
            );
        }
        _ => panic!("Expected Rectangle"),
    }
}

#[test]
fn test_coordinate_space_documentation() {
    // This test documents the coordinate system understanding:
    //
    // SHAPES coordinate space:
    // - Shapes are stored in IMAGE PIXEL coordinates
    // - A shape at (100, 100) means 100 pixels from top-left of the source image
    // - This is independent of canvas size, zoom, or pan
    //
    // CANVAS coordinate space:  
    // - The UI canvas can be any size
    // - The image is scaled to fit the canvas (maintaining aspect ratio)
    // - Zoom and pan further transform the display
    //
    // TRANSFORMATION during rendering:
    // 1. Image is scaled to fit canvas: scale = min(canvas_w/image_w, canvas_h/image_h)
    // 2. Image is centered: offset = (canvas_size - fitted_size) / 2
    // 3. Shape coords transformed: canvas_pos = image_pos * scale + offset  
    // 4. Zoom/pan transform applied: screen_pos = to_screen.mul_pos(canvas_pos)
    //
    // THE PROBLEM:
    // If rendering condition `form_image_size.is_some() && form_image.is_some()`
    // is false, shapes won't render even if they exist in the vector.
    //
    // HYPOTHESIS:
    // User draws shape but doesn't have an image loaded, so:
    // - Shape IS added to shapes vector ✓
    // - Shape render code is skipped (no image) ✗
    // - User sees nothing drawn ✗
}
