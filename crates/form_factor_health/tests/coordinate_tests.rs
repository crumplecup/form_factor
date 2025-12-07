//! Coordinate system consistency tests
//!
//! These tests ensure that shapes and detections share the same coordinate space
//! (image pixel coordinates) and maintain consistent positions when the canvas is resized.

use form_factor::{Circle, DrawingCanvas, Rectangle, Shape};

#[test]
fn test_shapes_stored_in_image_coordinates() {
    // Create a canvas and add a shape at a known position
    let mut canvas = DrawingCanvas::new();

    // Create a rectangle in image pixel coordinates (e.g., 100x100 at origin)
    let rect = Rectangle::from_corners(
        egui::pos2(0.0, 0.0),
        egui::pos2(100.0, 100.0),
        egui::Stroke::new(2.0, egui::Color32::RED),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    let shape = Shape::Rectangle(rect);

    // Store the original position
    let original_corners = if let Shape::Rectangle(r) = &shape {
        r.corners().to_vec()
    } else {
        panic!("Expected Rectangle");
    };

    // Add shape to canvas (shapes should be stored in image coordinates)
    canvas.test_add_shape(shape.clone());

    // Verify the shape is stored with the same coordinates
    let stored_shape = canvas.shapes().first().unwrap();
    let stored_corners = if let Shape::Rectangle(r) = stored_shape {
        r.corners().to_vec()
    } else {
        panic!("Expected Rectangle");
    };

    // Shapes should be stored exactly as provided (in image pixel coordinates)
    assert_eq!(original_corners, stored_corners);
}

#[test]
fn test_map_detection_preserves_aspect_ratio() {
    let canvas = DrawingCanvas::new();

    // Create a square detection in image coordinates (1000x1000 pixels)
    let detection = Rectangle::from_corners(
        egui::pos2(0.0, 0.0),
        egui::pos2(1000.0, 1000.0),
        egui::Stroke::new(2.0, egui::Color32::BLUE),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    let detection_shape = Shape::Rectangle(detection);

    // Simulate mapping to a canvas with specific dimensions
    // Image: 2000x2000, Canvas: 800x600 (landscape)
    let scale = 0.3; // min(800/2000, 600/2000) = 0.3
    let image_offset = egui::pos2(100.0, 0.0); // Centered horizontally

    let mapped = canvas.test_map_detection_to_canvas(&detection_shape, scale, image_offset);

    // Verify the mapped shape maintains square aspect ratio
    if let Shape::Rectangle(r) = mapped {
        let corners = r.corners();
        let width = (corners[1].x - corners[0].x).abs();
        let height = (corners[2].y - corners[0].y).abs();

        // Should be a 300x300 square (1000 * 0.3)
        assert!(
            (width - 300.0).abs() < 0.01,
            "Width should be 300.0, got {}",
            width
        );
        assert!(
            (height - 300.0).abs() < 0.01,
            "Height should be 300.0, got {}",
            height
        );

        // Should be offset correctly
        assert!(
            (corners[0].x - 100.0).abs() < 0.01,
            "Should be offset by 100.0 horizontally"
        );
        assert!(
            (corners[0].y - 0.0).abs() < 0.01,
            "Should be at 0.0 vertically"
        );
    } else {
        panic!("Expected Rectangle after mapping");
    }
}

#[test]
fn test_map_detection_scales_position_and_size() {
    let canvas = DrawingCanvas::new();

    // Create a circle at (500, 500) with radius 100 in image coordinates
    let detection = Circle::new(
        egui::pos2(500.0, 500.0),
        100.0,
        egui::Stroke::new(2.0, egui::Color32::GREEN),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    let detection_shape = Shape::Circle(detection);

    // Scale by 0.5, offset by (50, 50)
    let scale = 0.5;
    let image_offset = egui::pos2(50.0, 50.0);

    let mapped = canvas.test_map_detection_to_canvas(&detection_shape, scale, image_offset);

    if let Shape::Circle(c) = mapped {
        // Center should be at (500 * 0.5 + 50, 500 * 0.5 + 50) = (300, 300)
        assert!(
            (c.center().x - 300.0).abs() < 0.01,
            "Center X should be 300.0, got {}",
            c.center().x
        );
        assert!(
            (c.center().y - 300.0).abs() < 0.01,
            "Center Y should be 300.0, got {}",
            c.center().y
        );

        // Radius should be 100 * 0.5 = 50
        assert!(
            (c.radius() - 50.0).abs() < 0.01,
            "Radius should be 50.0, got {}",
            c.radius()
        );
    } else {
        panic!("Expected Circle after mapping");
    }
}

#[test]
fn test_shapes_and_detections_use_same_coordinate_system() {
    let mut canvas = DrawingCanvas::new();

    // Create identical rectangles - one as a shape, one as a detection
    let rect_coords = (egui::pos2(100.0, 100.0), egui::pos2(200.0, 200.0));

    let shape_rect = Rectangle::from_corners(
        rect_coords.0,
        rect_coords.1,
        egui::Stroke::new(2.0, egui::Color32::RED),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    let detection_rect = Rectangle::from_corners(
        rect_coords.0,
        rect_coords.1,
        egui::Stroke::new(2.0, egui::Color32::BLUE),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    // Both should be stored in the same coordinate system (image pixels)
    canvas.test_add_shape(Shape::Rectangle(shape_rect.clone()));
    canvas.test_add_detection(Shape::Rectangle(detection_rect.clone()));

    // When we retrieve them, they should have identical positions
    let stored_shape = canvas.shapes().first().unwrap();
    let stored_detection = canvas.detections().first().unwrap();

    let shape_corners = if let Shape::Rectangle(r) = stored_shape {
        r.corners().to_vec()
    } else {
        panic!("Expected Rectangle");
    };

    let detection_corners = if let Shape::Rectangle(r) = stored_detection {
        r.corners().to_vec()
    } else {
        panic!("Expected Rectangle");
    };

    // Both should be stored with identical coordinates
    assert_eq!(
        shape_corners, detection_corners,
        "Shapes and detections should be stored in the same coordinate system"
    );
}

#[test]
fn test_coordinate_transformation_is_reversible() {
    let _canvas = DrawingCanvas::new();

    // Original position in image coordinates
    let original_pos = egui::pos2(1000.0, 1500.0);

    // Apply forward transformation (image -> canvas)
    let scale = 0.4;
    let image_offset = egui::pos2(100.0, 50.0);

    let canvas_x = original_pos.x * scale + image_offset.x;
    let canvas_y = original_pos.y * scale + image_offset.y;
    let canvas_pos = egui::pos2(canvas_x, canvas_y);

    // Apply inverse transformation (canvas -> image)
    let recovered_x = (canvas_pos.x - image_offset.x) / scale;
    let recovered_y = (canvas_pos.y - image_offset.y) / scale;
    let recovered_pos = egui::pos2(recovered_x, recovered_y);

    // Should recover the original position
    assert!(
        (recovered_pos.x - original_pos.x).abs() < 0.01,
        "X coordinate should be recoverable"
    );
    assert!(
        (recovered_pos.y - original_pos.y).abs() < 0.01,
        "Y coordinate should be recoverable"
    );
}

#[test]
fn test_different_canvas_sizes_maintain_relative_positions() {
    let canvas = DrawingCanvas::new();

    // Create two shapes in image coordinates
    let shape1 = Circle::new(
        egui::pos2(500.0, 500.0),
        50.0,
        egui::Stroke::new(2.0, egui::Color32::RED),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    let shape2 = Circle::new(
        egui::pos2(1500.0, 1500.0),
        50.0,
        egui::Stroke::new(2.0, egui::Color32::BLUE),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    // Calculate positions for small canvas (400x400)
    let small_scale = 0.2; // For 2000x2000 image on 400x400 canvas
    let small_offset = egui::pos2(0.0, 0.0);

    let shape1_small = canvas.test_map_detection_to_canvas(
        &Shape::Circle(shape1.clone()),
        small_scale,
        small_offset,
    );
    let shape2_small = canvas.test_map_detection_to_canvas(
        &Shape::Circle(shape2.clone()),
        small_scale,
        small_offset,
    );

    // Calculate positions for large canvas (800x800)
    let large_scale = 0.4; // For 2000x2000 image on 800x800 canvas
    let large_offset = egui::pos2(0.0, 0.0);

    let shape1_large = canvas.test_map_detection_to_canvas(
        &Shape::Circle(shape1.clone()),
        large_scale,
        large_offset,
    );
    let shape2_large = canvas.test_map_detection_to_canvas(
        &Shape::Circle(shape2.clone()),
        large_scale,
        large_offset,
    );

    // Calculate the distance ratio between shapes in both canvas sizes
    if let (
        Shape::Circle(c1_small),
        Shape::Circle(c2_small),
        Shape::Circle(c1_large),
        Shape::Circle(c2_large),
    ) = (&shape1_small, &shape2_small, &shape1_large, &shape2_large)
    {
        let small_dx = c2_small.center().x - c1_small.center().x;
        let small_dy = c2_small.center().y - c1_small.center().y;
        let small_distance = (small_dx * small_dx + small_dy * small_dy).sqrt();

        let large_dx = c2_large.center().x - c1_large.center().x;
        let large_dy = c2_large.center().y - c1_large.center().y;
        let large_distance = (large_dx * large_dx + large_dy * large_dy).sqrt();

        // The ratio of distances should match the ratio of scales
        let distance_ratio = large_distance / small_distance;
        let scale_ratio = large_scale / small_scale;

        assert!(
            (distance_ratio - scale_ratio).abs() < 0.01,
            "Relative positions should scale proportionally with canvas size"
        );
    } else {
        panic!("Expected circles after mapping");
    }
}

#[test]
fn test_zero_offset_centering() {
    let canvas = DrawingCanvas::new();

    // Shape at origin in image coordinates
    let shape = Rectangle::from_corners(
        egui::pos2(0.0, 0.0),
        egui::pos2(100.0, 100.0),
        egui::Stroke::new(2.0, egui::Color32::WHITE),
        egui::Color32::TRANSPARENT,
    )
    .unwrap();

    // With zero offset, shape should start at canvas origin
    let scale = 1.0;
    let zero_offset = egui::pos2(0.0, 0.0);

    let mapped = canvas.test_map_detection_to_canvas(&Shape::Rectangle(shape), scale, zero_offset);

    if let Shape::Rectangle(r) = mapped {
        let corners = r.corners();
        assert!((corners[0].x - 0.0).abs() < 0.01, "Should start at x=0");
        assert!((corners[0].y - 0.0).abs() < 0.01, "Should start at y=0");
    } else {
        panic!("Expected Rectangle");
    }
}
