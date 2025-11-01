//! Shape definitions for drawing annotations

use egui::{Color32, Pos2, Stroke};
use geo::{Contains, Point};
use geo_types::{Coord, Polygon as GeoPolygon};
use serde::{Deserialize, Serialize};

/// A drawing shape on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
    Polygon(PolygonShape),
}

/// A rectangular annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub start: Pos2,
    pub end: Pos2,
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl Rectangle {
    /// Test if a point is inside this rectangle
    pub fn contains_point(&self, pos: Pos2) -> bool {
        let rect = egui::Rect::from_two_pos(self.start, self.end);
        rect.contains(pos)
    }
}

/// A circular annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub center: Pos2,
    pub radius: f32,
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl Circle {
    /// Test if a point is inside this circle
    pub fn contains_point(&self, pos: Pos2) -> bool {
        let distance = self.center.distance(pos);
        distance <= self.radius
    }
}

/// A polygon annotation (closed shape)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonShape {
    pub polygon: GeoPolygon<f64>,
    pub stroke: Stroke,
    pub fill: Color32,
    pub name: String,
}

impl PolygonShape {
    /// Create a polygon from a vector of egui positions
    /// Automatically closes the polygon by connecting the last point to the first
    pub fn from_points(points: Vec<Pos2>, stroke: Stroke, fill: Color32) -> Option<Self> {
        if points.len() < 3 {
            return None; // Need at least 3 points for a polygon
        }

        // Convert egui Pos2 to geo_types Coord
        let coords: Vec<Coord<f64>> = points
            .iter()
            .map(|p| Coord {
                x: p.x as f64,
                y: p.y as f64,
            })
            .collect();

        // Create a closed LineString (polygon exterior)
        // geo_types automatically closes the polygon
        let polygon = GeoPolygon::new(coords.into(), vec![]);

        Some(PolygonShape {
            polygon,
            stroke,
            fill,
            name: String::new(),
        })
    }

    /// Convert polygon to egui points for rendering
    pub fn to_egui_points(&self) -> Vec<Pos2> {
        self.polygon
            .exterior()
            .points()
            .map(|p| Pos2::new(p.x() as f32, p.y() as f32))
            .collect()
    }

    /// Test if a point is inside this polygon
    pub fn contains_point(&self, pos: Pos2) -> bool {
        let point = Point::new(pos.x as f64, pos.y as f64);
        self.polygon.contains(&point)
    }
}

impl Shape {
    /// Render this shape to the given painter
    pub fn render(&self, painter: &egui::Painter) {
        match self {
            Shape::Rectangle(rect) => {
                let rect_shape = egui::Rect::from_two_pos(rect.start, rect.end);
                painter.rect_filled(rect_shape, 0.0, rect.fill);
                painter.rect_stroke(
                    rect_shape,
                    0.0,
                    rect.stroke,
                    egui::StrokeKind::Outside,
                );
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
}
