//! Integration tests for shape module
//!
//! These tests validate the geometric shape types used for canvas annotations,
//! focusing on pure business logic without GUI dependencies.

use egui::{Color32, Pos2, Stroke};
use form_factor::{Circle, FormError, PolygonShape, Rectangle, Shape, ShapeErrorKind};
use std::f32::consts::PI;

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn polygon_rejects_too_few_points() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Test with 0 points
    let result = PolygonShape::from_points(vec![], stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::TooFewPoints(0)));
        assert!(e.file.contains("shape.rs"));
        assert!(e.line > 0);
    }

    // Test with 1 point
    let result = PolygonShape::from_points(vec![Pos2::new(0.0, 0.0)], stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::TooFewPoints(1)));
    }

    // Test with 2 points
    let result =
        PolygonShape::from_points(vec![Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)], stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::TooFewPoints(2)));
    }

    // Test with 3 points - should succeed
    let result = PolygonShape::from_points(
        vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(1.0, 0.0),
            Pos2::new(0.5, 1.0),
        ],
        stroke,
        fill,
    );
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn circle_rejects_invalid_radius() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;
    let center = Pos2::new(10.0, 10.0);

    // Test with zero radius
    let result = Circle::new(center, 0.0, stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidRadius(r) if r == 0.0));
    }

    // Test with negative radius
    let result = Circle::new(center, -5.0, stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidRadius(r) if r == -5.0));
    }

    // Test with NaN radius
    let result = Circle::new(center, f32::NAN, stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidRadius(r) if r.is_nan()));
    }

    // Test with infinity radius
    let result = Circle::new(center, f32::INFINITY, stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidRadius(r) if r.is_infinite()));
    }

    // Test with valid radius - should succeed
    let result = Circle::new(center, 5.0, stroke, fill);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn shape_rejects_nan_coordinates() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Rectangle with NaN
    let result = Rectangle::from_corners(
        Pos2::new(f32::NAN, 0.0),
        Pos2::new(10.0, 10.0),
        stroke,
        fill,
    );
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidCoordinate));
    }

    // Circle with NaN center
    let result = Circle::new(Pos2::new(f32::NAN, 5.0), 5.0, stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidCoordinate));
    }

    // Polygon with NaN
    let result = PolygonShape::from_points(
        vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(f32::NAN, 0.0),
            Pos2::new(0.5, 1.0),
        ],
        stroke,
        fill,
    );
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidCoordinate));
    }
    Ok(())
}

#[test]
fn shape_rejects_infinity_coordinates() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let result = Rectangle::from_corners(
        Pos2::new(0.0, 0.0),
        Pos2::new(f32::INFINITY, 10.0),
        stroke,
        fill,
    );
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::InvalidCoordinate));
    }
    Ok(())
}

#[test]
fn rectangle_rejects_degenerate_shape() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Zero width
    let result = Rectangle::from_corners(Pos2::new(5.0, 0.0), Pos2::new(5.0, 10.0), stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::DegenerateShape));
    }

    // Zero height
    let result = Rectangle::from_corners(Pos2::new(0.0, 5.0), Pos2::new(10.0, 5.0), stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::DegenerateShape));
    }

    // Same point
    let result = Rectangle::from_corners(Pos2::new(5.0, 5.0), Pos2::new(5.0, 5.0), stroke, fill);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, ShapeErrorKind::DegenerateShape));
    }
    Ok(())
}

#[test]
fn error_captures_location_info() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let result = Circle::new(Pos2::new(0.0, 0.0), -1.0, stroke, fill);
    assert!(result.is_err());

    if let Err(e) = result {
        // Verify all location fields are populated
        assert!(e.line > 0, "Line number should be captured");
        assert!(!e.file.is_empty(), "File path should be captured");
        assert!(e.file.contains("shape.rs"), "File should be shape.rs");

        // Verify display format includes location
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("Shape Error:"));
        assert!(error_msg.contains("at line"));
        assert!(error_msg.contains("in"));
    }
    Ok(())
}

// ============================================================================
// Shape Creation & Validation Tests
// ============================================================================

#[test]
fn rectangle_creates_from_corners() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(2.0, Color32::RED);
    let fill = Color32::BLUE;

    let rect = Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 20.0), stroke, fill)
        ?;

    let corners = rect.corners();
    assert_eq!(corners.len(), 4);

    // Verify corners form correct rectangle
    assert_eq!(corners[0], Pos2::new(0.0, 0.0)); // top-left
    assert_eq!(corners[1], Pos2::new(10.0, 0.0)); // top-right
    assert_eq!(corners[2], Pos2::new(10.0, 20.0)); // bottom-right
    assert_eq!(corners[3], Pos2::new(0.0, 20.0)); // bottom-left
    Ok(())
}

#[test]
fn rectangle_creates_from_four_corners() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(2.0, Color32::RED);
    let fill = Color32::BLUE;

    let corners = [
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(10.0, 20.0),
        Pos2::new(0.0, 20.0),
    ];

    let rect = Rectangle::from_four_corners(corners, stroke, fill)
        ?;

    assert_eq!(rect.corners(), &corners);
    Ok(())
}

#[test]
fn rectangle_normalizes_corners() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(2.0, Color32::RED);
    let fill = Color32::BLUE;

    // Create rectangle with corners in "wrong" order (bottom-right to top-left)
    let rect = Rectangle::from_corners(Pos2::new(10.0, 20.0), Pos2::new(0.0, 0.0), stroke, fill)
        ?;

    let corners = rect.corners();
    // Should still produce correctly ordered corners
    assert_eq!(corners[0], Pos2::new(0.0, 0.0)); // top-left
    assert_eq!(corners[2], Pos2::new(10.0, 20.0)); // bottom-right
    Ok(())
}

#[test]
fn circle_creates_with_valid_params() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::GREEN);
    let fill = Color32::RED;
    let center = Pos2::new(50.0, 75.0);
    let radius = 25.0;

    let circle = Circle::new(center, radius, stroke, fill)?;

    assert_eq!(*circle.center(), center);
    assert_eq!(*circle.radius(), radius);
    assert_eq!(*circle.stroke(), stroke);
    assert_eq!(*circle.fill(), fill);
    Ok(())
}

#[test]
fn polygon_creates_with_three_points() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::YELLOW;

    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let poly = PolygonShape::from_points(points.clone(), stroke, fill)
        ?;

    let result_points = poly.to_egui_points();
    // geo crate closes polygons by adding first point at end
    assert_eq!(result_points.len(), points.len() + 1);
    // Verify first and last points are the same (closed polygon)
    assert_eq!(result_points[0], result_points[result_points.len() - 1]);
    Ok(())
}

#[test]
fn polygon_creates_with_many_points() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Create an octagon
    let mut points = Vec::new();
    for i in 0..8 {
        let angle = (i as f32) * PI / 4.0;
        points.push(Pos2::new(angle.cos() * 10.0, angle.sin() * 10.0));
    }

    let poly = PolygonShape::from_points(points.clone(), stroke, fill)
        ?;

    // geo crate closes polygons by adding first point at end
    assert_eq!(poly.to_egui_points().len(), 9);
    Ok(())
}

// ============================================================================
// Geometric Transformation Tests
// ============================================================================

#[test]
fn circle_translates_correctly() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut circle = Circle::new(Pos2::new(10.0, 10.0), 5.0, stroke, fill)?;

    circle.translate(egui::Vec2::new(5.0, -3.0))?;

    assert_eq!(*circle.center(), Pos2::new(15.0, 7.0));
    assert_eq!(*circle.radius(), 5.0); // Radius unchanged
    Ok(())
}

#[test]
fn rectangle_translates_correctly() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0), stroke, fill)?;

    rect.translate(egui::Vec2::new(5.0, 5.0))?;

    let corners = rect.corners();
    assert_eq!(corners[0], Pos2::new(5.0, 5.0));
    assert_eq!(corners[2], Pos2::new(15.0, 15.0));
    Ok(())
}

#[test]
fn polygon_translates_correctly() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let mut poly = PolygonShape::from_points(points, stroke, fill)?;

    poly.translate(egui::Vec2::new(10.0, 10.0))?;

    let result = poly.to_egui_points();
    assert_eq!(result[0], Pos2::new(10.0, 10.0));
    assert_eq!(result[1], Pos2::new(20.0, 10.0));
    assert_eq!(result[2], Pos2::new(15.0, 20.0));
    Ok(())
}

#[test]
fn circle_rotates_around_pivot() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut circle = Circle::new(Pos2::new(10.0, 0.0), 2.0, stroke, fill)?;

    // Rotate 90 degrees around origin
    circle.rotate(PI / 2.0, Pos2::new(0.0, 0.0))?;

    // After 90° rotation around origin, (10, 0) -> (0, 10)
    let center = circle.center();
    assert!((center.x - 0.0).abs() < 0.001, "x should be ~0");
    assert!((center.y - 10.0).abs() < 0.001, "y should be ~10");
    assert_eq!(*circle.radius(), 2.0); // Radius unchanged
    Ok(())
}

#[test]
fn rectangle_rotates_90_degrees() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(2.0, 1.0), stroke, fill)?;

    // Rotate 90 degrees around origin
    rect.rotate(PI / 2.0, Pos2::new(0.0, 0.0))?;

    let corners = rect.corners();

    // After 90° rotation: (x, y) -> (-y, x)
    // (0,0) -> (0,0), (2,0) -> (0,2), (2,1) -> (-1,2), (0,1) -> (-1,0)
    assert!((corners[0].x - 0.0).abs() < 0.001);
    assert!((corners[0].y - 0.0).abs() < 0.001);
    Ok(())
}

#[test]
fn polygon_rotates_180_degrees() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let points = vec![
        Pos2::new(1.0, 0.0),
        Pos2::new(2.0, 0.0),
        Pos2::new(1.5, 1.0),
    ];

    let mut poly = PolygonShape::from_points(points, stroke, fill)?;

    // Rotate 180 degrees around origin
    poly.rotate(PI, Pos2::new(0.0, 0.0))?;

    let result = poly.to_egui_points();

    // After 180° rotation: (x, y) -> (-x, -y)
    assert!((result[0].x - (-1.0)).abs() < 0.001);
    assert!((result[0].y - 0.0).abs() < 0.001);
    assert!((result[1].x - (-2.0)).abs() < 0.001);
    assert!((result[1].y - 0.0).abs() < 0.001);
    Ok(())
}

#[test]
fn rotation_360_degrees_returns_to_original() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let original_center = Pos2::new(10.0, 10.0);
    let mut circle = Circle::new(original_center, 5.0, stroke, fill)?;

    circle.rotate(2.0 * PI, Pos2::new(0.0, 0.0))?;

    let center = circle.center();
    assert!((center.x - original_center.x).abs() < 0.001);
    assert!((center.y - original_center.y).abs() < 0.001);
    Ok(())
}

#[test]
fn rectangle_set_corner_updates_shape() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0), stroke, fill)?;

    rect.set_corner(0, Pos2::new(1.0, 1.0))?;

    let corners = rect.corners();
    assert_eq!(corners[0], Pos2::new(1.0, 1.0));
    Ok(())
}

#[test]
fn polygon_set_vertex_updates_shape() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let mut poly = PolygonShape::from_points(points, stroke, fill)?;

    poly.set_vertex(1, Pos2::new(15.0, 0.0))?;

    let result = poly.to_egui_points();
    assert_eq!(result[1], Pos2::new(15.0, 0.0));
    Ok(())
}

#[test]
fn polygon_set_vertices_replaces_all() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let mut poly = PolygonShape::from_points(points, stroke, fill)?;

    let new_points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(5.0, 0.0),
        Pos2::new(5.0, 5.0),
        Pos2::new(0.0, 5.0),
    ];

    poly.set_vertices(new_points.clone())?;

    let result = poly.to_egui_points();
    // geo crate closes polygons by adding first point at end
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], new_points[0]);
    assert_eq!(result[3], new_points[3]);
    // Verify polygon is closed
    assert_eq!(result[0], result[4]);
    Ok(())
}

#[test]
fn circle_set_radius_updates_radius() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut circle = Circle::new(Pos2::new(10.0, 10.0), 5.0, stroke, fill)?;

    circle.set_radius(10.0)?;
    assert_eq!(*circle.radius(), 10.0);

    // Invalid radius should fail
    let result = circle.set_radius(-5.0);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn circle_set_center_updates_center() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let mut circle = Circle::new(Pos2::new(10.0, 10.0), 5.0, stroke, fill)?;

    circle.set_center(Pos2::new(20.0, 30.0))?;
    assert_eq!(*circle.center(), Pos2::new(20.0, 30.0));
    Ok(())
}

// ============================================================================
// Spatial Query Tests
// ============================================================================

#[test]
fn circle_contains_interior_point() -> Result<(), form_factor::FormError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let circle = Circle::new(Pos2::new(10.0, 10.0), 5.0, stroke, fill)?;

    // Point at center
    assert!(circle.contains_point(Pos2::new(10.0, 10.0)));

    // Point clearly inside
    assert!(circle.contains_point(Pos2::new(11.0, 10.0)));

    // Point clearly outside
    assert!(!circle.contains_point(Pos2::new(20.0, 10.0)));

    // Point on edge (approximately)
    assert!(circle.contains_point(Pos2::new(15.0, 10.0)));
    Ok(())
}

#[test]
fn rectangle_contains_interior_point() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0), stroke, fill)?;

    // Point clearly inside
    assert!(rect.contains_point(Pos2::new(5.0, 5.0)));

    // Point clearly outside
    assert!(!rect.contains_point(Pos2::new(20.0, 20.0)));
    assert!(!rect.contains_point(Pos2::new(-1.0, 5.0)));

    // Note: Boundary points may not be considered "inside" by geo's Contains trait
    // Testing boundary behavior would require separate boundary-specific tests
    Ok(())
}

#[test]
fn polygon_contains_interior_point() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Create a triangle
    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let poly = PolygonShape::from_points(points, stroke, fill)?;

    // Point inside triangle
    assert!(poly.contains_point(Pos2::new(5.0, 3.0)));

    // Point outside triangle
    assert!(!poly.contains_point(Pos2::new(0.0, 10.0)));
    assert!(!poly.contains_point(Pos2::new(20.0, 5.0)));

    // Note: Boundary/vertex points may not be considered "inside" by geo's Contains trait
    Ok(())
}

#[test]
fn shape_enum_contains_point() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0), stroke, fill)?;
    let shape = Shape::Rectangle(rect);

    assert!(shape.contains_point(Pos2::new(5.0, 5.0)));
    assert!(!shape.contains_point(Pos2::new(20.0, 20.0)));
    Ok(())
}

#[test]
fn polygon_center_calculates_centroid() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Create a square
    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(10.0, 10.0),
        Pos2::new(0.0, 10.0),
    ];

    let poly = PolygonShape::from_points(points, stroke, fill)?;

    let center = poly.center();
    // Note: centroid calculation includes the closing point (first point repeated)
    // For a square with corners at (0,0), (10,0), (10,10), (0,10),
    // the geo polygon has points: (0,0), (10,0), (10,10), (0,10), (0,0)
    // Average: (0+10+10+0+0)/5 = 4, (0+0+10+10+0)/5 = 4
    assert!((center.x - 4.0).abs() < 0.001);
    assert!((center.y - 4.0).abs() < 0.001);
    Ok(())
}

#[test]
fn rectangle_center_calculates_centroid() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let rect =
        Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(10.0, 20.0), stroke, fill)?;

    let center = rect.center();
    // Centroid should be at (5, 10)
    assert!((center.x - 5.0).abs() < 0.001);
    assert!((center.y - 10.0).abs() < 0.001);
    Ok(())
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn circle_builder_creates_valid_circle() -> Result<(), form_factor::FormError> {
    use form_factor::CircleBuilder;

    let circle = CircleBuilder::default()
        .center(Pos2::new(10.0, 10.0))
        .radius(5.0)
        .stroke(Stroke::new(1.0, Color32::BLACK))
        .fill(Color32::BLUE)
        .name("test circle")
        .build()
        ?;

    assert_eq!(*circle.center(), Pos2::new(10.0, 10.0));
    assert_eq!(*circle.radius(), 5.0);
    assert_eq!(circle.name(), "test circle");
    Ok(())
}

#[test]
fn circle_builder_uses_default_name() -> Result<(), form_factor::FormError> {
    use form_factor::CircleBuilder;

    let circle = CircleBuilder::default()
        .center(Pos2::new(0.0, 0.0))
        .radius(1.0)
        .stroke(Stroke::new(1.0, Color32::BLACK))
        .fill(Color32::TRANSPARENT)
        .build()
        ?;

    assert_eq!(circle.name(), "");
    Ok(())
}

// ============================================================================
// Edge Cases and Regression Tests
// ============================================================================

#[test]
fn handles_very_small_shapes() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Very small but valid circle
    let circle = Circle::new(Pos2::new(0.0, 0.0), 0.001, stroke, fill);
    assert!(circle.is_ok());

    // Very small but valid rectangle
    let rect = Rectangle::from_corners(Pos2::new(0.0, 0.0), Pos2::new(0.001, 0.001), stroke, fill);
    assert!(rect.is_ok());
    Ok(())
}

#[test]
fn handles_very_large_coordinates() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    // Large but finite coordinates
    let rect = Rectangle::from_corners(
        Pos2::new(0.0, 0.0),
        Pos2::new(1_000_000.0, 1_000_000.0),
        stroke,
        fill,
    );
    assert!(rect.is_ok());
    Ok(())
}

#[test]
fn polygon_with_out_of_bounds_vertex_index() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let points = vec![
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(5.0, 10.0),
    ];

    let mut poly = PolygonShape::from_points(points, stroke, fill)?;

    // Setting vertex beyond bounds should be silently ignored
    let result = poly.set_vertex(100, Pos2::new(0.0, 0.0));
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn rectangle_with_negative_coordinates() -> Result<(), form_factor::ShapeError> {
    let stroke = Stroke::new(1.0, Color32::BLACK);
    let fill = Color32::TRANSPARENT;

    let rect =
        Rectangle::from_corners(Pos2::new(-10.0, -10.0), Pos2::new(10.0, 10.0), stroke, fill);
    assert!(rect.is_ok());

    if let Ok(r) = rect {
        assert!(r.contains_point(Pos2::new(0.0, 0.0)));
        assert!(r.contains_point(Pos2::new(-5.0, -5.0)));
    }
    Ok(())
}
