//! Shape definitions for drawing annotations
//!
//! This module provides geometric shapes for canvas annotations, leveraging
//! the `geo` crate for robust geometry operations and spatial queries.

use derive_builder::Builder;
use derive_getters::Getters;
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
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
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

    /// Update a corner and rebuild the internal polygon
    ///
    /// # Arguments
    ///
    /// * `index` - Corner index (0-3)
    /// * `pos` - New position for the corner
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if the new position is invalid.
    pub fn set_corner(&mut self, index: usize, pos: Pos2) -> Result<(), ShapeError> {
        if index >= 4 {
            return Ok(()); // Silently ignore out of bounds
        }

        // Validate the new position
        pos2_to_coord(pos)?;

        // Update the corner
        self.corners[index] = pos;

        // Rebuild the polygon from updated corners
        let coords: Vec<Coord<f64>> = self.corners
            .iter()
            .map(|&p| pos2_to_coord(p).expect("Already validated"))
            .collect();
        self.polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(())
    }

    /// Update all corners at once and rebuild the internal polygon
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if any coordinate is invalid.
    pub fn set_corners(&mut self, corners: [Pos2; 4]) -> Result<(), ShapeError> {
        // Validate all corners first
        for &corner in &corners {
            pos2_to_coord(corner)?;
        }

        self.corners = corners;

        // Rebuild the polygon
        let coords: Vec<Coord<f64>> = self.corners
            .iter()
            .map(|&p| pos2_to_coord(p).expect("Already validated"))
            .collect();
        self.polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(())
    }

    /// Get the center point of this rectangle
    pub fn center(&self) -> Pos2 {
        let sum_x: f32 = self.corners.iter().map(|p| p.x).sum();
        let sum_y: f32 = self.corners.iter().map(|p| p.y).sum();
        Pos2::new(sum_x / 4.0, sum_y / 4.0)
    }

    /// Rotate this rectangle around a pivot point
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    /// * `pivot` - Point to rotate around
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if rotation produces invalid coordinates.
    pub fn rotate(&mut self, angle: f32, pivot: Pos2) -> Result<(), ShapeError> {
        // Rotate each corner around the pivot
        let cos = angle.cos();
        let sin = angle.sin();

        let mut new_corners = [Pos2::ZERO; 4];
        for (i, corner) in self.corners.iter().enumerate() {
            // Translate to origin
            let dx = corner.x - pivot.x;
            let dy = corner.y - pivot.y;

            // Rotate
            let rotated_x = dx * cos - dy * sin;
            let rotated_y = dx * sin + dy * cos;

            // Translate back
            new_corners[i] = Pos2::new(
                rotated_x + pivot.x,
                rotated_y + pivot.y,
            );
        }

        // Update using the setter to maintain consistency
        self.set_corners(new_corners)?;
        Ok(())
    }

    /// Translate this rectangle by a delta vector
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if translation produces invalid coordinates.
    pub fn translate(&mut self, delta: egui::Vec2) -> Result<(), ShapeError> {
        let new_corners = [
            self.corners[0] + delta,
            self.corners[1] + delta,
            self.corners[2] + delta,
            self.corners[3] + delta,
        ];
        self.set_corners(new_corners)?;
        Ok(())
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
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[builder(setter(into))]
pub struct Circle {
    pub center: Pos2,
    pub radius: f32,
    pub stroke: Stroke,
    pub fill: Color32,
    #[builder(default = "String::new()")]
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

    /// Set the center position
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if the position is invalid.
    pub fn set_center(&mut self, center: Pos2) -> Result<(), ShapeError> {
        pos2_to_coord(center)?;
        self.center = center;
        Ok(())
    }

    /// Set the radius
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidRadius` if the radius is not positive and finite.
    pub fn set_radius(&mut self, radius: f32) -> Result<(), ShapeError> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ShapeError::InvalidRadius(radius));
        }
        self.radius = radius;
        Ok(())
    }

    /// Rotate this circle around a pivot point
    ///
    /// For circles, only the center rotates; the radius remains unchanged.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    /// * `pivot` - Point to rotate around
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if rotation produces invalid coordinates.
    pub fn rotate(&mut self, angle: f32, pivot: Pos2) -> Result<(), ShapeError> {
        // Rotate the center around the pivot
        let cos = angle.cos();
        let sin = angle.sin();

        // Translate to origin
        let dx = self.center.x - pivot.x;
        let dy = self.center.y - pivot.y;

        // Rotate
        let rotated_x = dx * cos - dy * sin;
        let rotated_y = dx * sin + dy * cos;

        // Translate back
        let new_center = Pos2::new(
            rotated_x + pivot.x,
            rotated_y + pivot.y,
        );

        self.set_center(new_center)?;
        Ok(())
    }

    /// Translate this circle by a delta vector
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if translation produces invalid coordinates.
    pub fn translate(&mut self, delta: egui::Vec2) -> Result<(), ShapeError> {
        self.set_center(self.center + delta)?;
        Ok(())
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
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
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

    /// Update a vertex of the polygon
    ///
    /// # Arguments
    ///
    /// * `index` - Vertex index
    /// * `pos` - New position for the vertex
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if the new position is invalid.
    pub fn set_vertex(&mut self, index: usize, pos: Pos2) -> Result<(), ShapeError> {
        // Get current points
        let mut points = self.to_egui_points();

        if index >= points.len() {
            return Ok(()); // Silently ignore out of bounds
        }

        // Update the vertex
        points[index] = pos;

        // Rebuild the polygon using from_points logic
        if points.len() < 3 {
            return Err(ShapeError::TooFewPoints(points.len()));
        }

        // Validate and convert all points
        let coords: Result<Vec<Coord<f64>>, ShapeError> = points
            .iter()
            .map(|&p| pos2_to_coord(p))
            .collect();
        let coords = coords?;

        self.polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(())
    }

    /// Update all vertices at once
    ///
    /// # Errors
    ///
    /// Returns error if points are invalid or fewer than 3.
    pub fn set_vertices(&mut self, points: Vec<Pos2>) -> Result<(), ShapeError> {
        if points.len() < 3 {
            return Err(ShapeError::TooFewPoints(points.len()));
        }

        // Validate and convert all points
        let coords: Result<Vec<Coord<f64>>, ShapeError> = points
            .iter()
            .map(|&p| pos2_to_coord(p))
            .collect();
        let coords = coords?;

        self.polygon = GeoPolygon::new(LineString::from(coords), vec![]);

        Ok(())
    }

    /// Get the center point of this polygon (centroid)
    pub fn center(&self) -> Pos2 {
        let points = self.to_egui_points();
        if points.is_empty() {
            return Pos2::ZERO;
        }

        let sum_x: f32 = points.iter().map(|p| p.x).sum();
        let sum_y: f32 = points.iter().map(|p| p.y).sum();
        let count = points.len() as f32;

        Pos2::new(sum_x / count, sum_y / count)
    }

    /// Rotate this polygon around a pivot point
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    /// * `pivot` - Point to rotate around
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if rotation produces invalid coordinates.
    pub fn rotate(&mut self, angle: f32, pivot: Pos2) -> Result<(), ShapeError> {
        let points = self.to_egui_points();
        let cos = angle.cos();
        let sin = angle.sin();

        let rotated_points: Vec<Pos2> = points
            .iter()
            .map(|point| {
                // Translate to origin
                let dx = point.x - pivot.x;
                let dy = point.y - pivot.y;

                // Rotate
                let rotated_x = dx * cos - dy * sin;
                let rotated_y = dx * sin + dy * cos;

                // Translate back
                Pos2::new(
                    rotated_x + pivot.x,
                    rotated_y + pivot.y,
                )
            })
            .collect();

        self.set_vertices(rotated_points)?;
        Ok(())
    }

    /// Translate this polygon by a delta vector
    ///
    /// # Errors
    ///
    /// Returns `ShapeError::InvalidCoordinate` if translation produces invalid coordinates.
    pub fn translate(&mut self, delta: egui::Vec2) -> Result<(), ShapeError> {
        let points = self.to_egui_points();
        let translated_points: Vec<Pos2> = points
            .iter()
            .map(|p| *p + delta)
            .collect();

        self.set_vertices(translated_points)?;
        Ok(())
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
