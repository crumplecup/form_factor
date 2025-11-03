//! Shape definitions for drawing annotations
//!
//! This module provides geometric shapes for canvas annotations, leveraging
//! the `geo` crate for robust geometry operations and spatial queries.

use egui::{Color32, Pos2, Stroke};
use geo::{Contains, Point};
use geo_types::{Coord, LineString, Polygon as GeoPolygon};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during shape creation and manipulation
#[derive(Error, Debug, Clone)]
pub enum ShapeError {
    #[error("Polygon must have at least 3 points, got {0}")]
    TooFewPoints(usize),

    #[error("Invalid coordinate: point contains NaN or infinity")]
    InvalidCoordinate,

    #[error("Circle radius must be positive, got {0}")]
    InvalidRadius(f32),

    #[error("Degenerate shape: all points are collinear or coincident")]
    DegenerateShape,
}

/// Convert an egui Pos2 to a geo Coord<f64>
#[inline]
fn pos2_to_coord(p: Pos2) -> Result<Coord<f64>, ShapeError> {
    if !p.x.is_finite() || !p.y.is_finite() {
        return Err(ShapeError::InvalidCoordinate);
    }
    Ok(Coord {
        x: p.x as f64,
        y: p.y as f64,
    })
}

/// Convert a geo Coord<f64> to an egui Pos2
#[inline]
fn coord_to_pos2(c: Coord<f64>) -> Pos2 {
    Pos2::new(c.x as f32, c.y as f32)
}

/// A drawing shape on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
    Polygon(PolygonShape),
}

/// A quadrilateral annotation (4-sided polygon, initially a rectangle)
///
/// Internally uses `geo::Polygon` for robust geometric operations.
/// Corners are stored in clockwise order: top-left, top-right, bottom-right, bottom-left.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    /// Internal polygon representation for geometric operations
    polygon: GeoPolygon<f64>,
    /// Four corners for efficient rendering (cached from polygon)
    corners: [Pos2; 4],
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl Rectangle {
    /// Create a rectangle from two opposite corners
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if any coordinate is NaN or infinite.
    /// Returns `ShapeError::DegenerateShape` if start and end are the same point.
    pub fn from_corners(start: Pos2, end: Pos2, stroke: Stroke, fill: Color32) -> Result<Self, ShapeError> {
        // Validate coordinates
        pos2_to_coord(start)?;
        pos2_to_coord(end)?;

        // Calculate all 4 corners from the diagonal
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        // Check for degenerate case (zero area)
        if (max_x - min_x).abs() < f32::EPSILON || (max_y - min_y).abs() < f32::EPSILON {
            return Err(ShapeError::DegenerateShape);
        }

        let corners = [
            Pos2::new(min_x, min_y), // top-left
            Pos2::new(max_x, min_y), // top-right
            Pos2::new(max_x, max_y), // bottom-right
            Pos2::new(min_x, max_y), // bottom-left
        ];

        // Build geo polygon from corners
        let coords: Vec<Coord<f64>> = corners
            .iter()
            .map(|&p| pos2_to_coord(p).expect("Already validated"))
            .collect();
        let polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(Self {
            polygon,
            corners,
            stroke,
            fill,
            name: String::new(),
        })
    }

    /// Create a rectangle from four corners
    ///
    /// Corners should be in order (typically clockwise). This allows creating
    /// arbitrary quadrilaterals, including rotated rectangles.
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if any coordinate is NaN or infinite.
    pub fn from_four_corners(corners: [Pos2; 4], stroke: Stroke, fill: Color32) -> Result<Self, ShapeError> {
        // Validate all corners
        let coords: Result<Vec<Coord<f64>>, ShapeError> = corners
            .iter()
            .map(|&p| pos2_to_coord(p))
            .collect();
        let coords = coords?;

        let polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(Self {
            polygon,
            corners,
            stroke,
            fill,
            name: String::new(),
        })
    }

    /// Get the corners of this rectangle
    pub fn corners(&self) -> &[Pos2; 4] {
        &self.corners
    }

    /// Test if a point is inside this quadrilateral
    ///
    /// Uses the robust `geo` crate implementation for point-in-polygon testing.
    pub fn contains_point(&self, pos: Pos2) -> bool {
        let Ok(coord) = pos2_to_coord(pos) else {
            return false;
        };
        let point = Point::from(coord);
        self.polygon.contains(&point)
    }
}

/// A circular annotation
///
/// Uses egui's native circle representation. Point-in-circle testing is
/// performed using simple distance calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub center: Pos2,
    pub radius: f32,
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl Circle {
    /// Create a new circle
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if center contains NaN or infinity.
    /// Returns `ShapeError::InvalidRadius` if radius is not positive and finite.
    pub fn new(center: Pos2, radius: f32, stroke: Stroke, fill: Color32) -> Result<Self, ShapeError> {
        // Validate center coordinate
        pos2_to_coord(center)?;

        // Validate radius
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ShapeError::InvalidRadius(radius));
        }

        Ok(Self {
            center,
            radius,
            stroke,
            fill,
            name: String::new(),
        })
    }

    /// Test if a point is inside this circle
    pub fn contains_point(&self, pos: Pos2) -> bool {
        // Early return if point is invalid
        if !pos.x.is_finite() || !pos.y.is_finite() {
            return false;
        }

        let distance = self.center.distance(pos);
        distance <= self.radius
    }
}

/// A polygon annotation (closed shape)
///
/// Uses `geo::Polygon` for all geometric operations. The polygon is automatically
/// closed (first point connects to last point).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonShape {
    polygon: GeoPolygon<f64>,
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl PolygonShape {
    /// Create a polygon from a vector of egui positions
    ///
    /// The polygon is automatically closed by connecting the last point to the first.
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::TooFewPoints` if fewer than 3 points are provided.
    /// Returns `ShapeError::InvalidCoordinate` if any point contains NaN or infinity.
    pub fn from_points(points: Vec<Pos2>, stroke: Stroke, fill: Color32) -> Result<Self, ShapeError> {
        if points.len() < 3 {
            return Err(ShapeError::TooFewPoints(points.len()));
        }

        // Convert egui Pos2 to geo_types Coord with validation
        let coords: Result<Vec<Coord<f64>>, ShapeError> = points
            .iter()
            .map(|&p| pos2_to_coord(p))
            .collect();
        let coords = coords?;

        // Create a closed LineString (polygon exterior)
        // geo_types automatically closes the polygon
        let polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(PolygonShape {
            polygon,
            stroke,
            fill,
            name: String::new(),
        })
    }

    /// Get access to the underlying geo polygon
    pub fn polygon(&self) -> &GeoPolygon<f64> {
        &self.polygon
    }

    /// Convert polygon to egui points for rendering
    ///
    /// Returns the exterior ring of the polygon as a vector of egui positions.
    pub fn to_egui_points(&self) -> Vec<Pos2> {
        self.polygon
            .exterior()
            .points()
            .map(|p| coord_to_pos2(Coord { x: p.x(), y: p.y() }))
            .collect()
    }

    /// Test if a point is inside this polygon
    ///
    /// Uses the robust `geo` crate implementation for point-in-polygon testing.
    pub fn contains_point(&self, pos: Pos2) -> bool {
        let Ok(coord) = pos2_to_coord(pos) else {
            return false;
        };
        let point = Point::from(coord);
        self.polygon.contains(&point)
    }
}

impl Shape {
    /// Render this shape to the given painter
    pub fn render(&self, painter: &egui::Painter) {
        match self {
            Shape::Rectangle(rect) => {
                // Draw as a filled quadrilateral
                painter.add(egui::Shape::convex_polygon(
                    rect.corners().to_vec(),
                    rect.fill,
                    egui::Stroke::NONE,
                ));
                // Draw the outline
                painter.add(egui::Shape::closed_line(
                    rect.corners().to_vec(),
                    rect.stroke,
                ));
            }
            Shape::Circle(circle) => {
                painter.circle(circle.center, circle.radius, circle.fill, circle.stroke);
            }
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                if points.len() > 2 {
                    // Draw filled polygon
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        poly.fill,
                        egui::Stroke::NONE,
                    ));
                    // Draw polygon outline as a closed path
                    painter.add(egui::Shape::closed_line(points, poly.stroke));
                }
            }
        }
    }

    /// Test if a point is inside this shape
    pub fn contains_point(&self, pos: Pos2) -> bool {
        match self {
            Shape::Rectangle(rect) => rect.contains_point(pos),
            Shape::Circle(circle) => circle.contains_point(pos),
            Shape::Polygon(poly) => poly.contains_point(pos),
        }
    }
}
