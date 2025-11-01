//! Shape definitions for drawing annotations

use egui::{Color32, Pos2, Stroke};
use serde::{Deserialize, Serialize};

/// A drawing shape on the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
    Freehand(FreehandStroke),
}

/// A rectangular annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub start: Pos2,
    pub end: Pos2,
    pub stroke: Stroke,
    pub fill: Color32,
}

/// A circular annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub center: Pos2,
    pub radius: f32,
    pub stroke: Stroke,
    pub fill: Color32,
}

/// A freehand stroke annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreehandStroke {
    pub points: Vec<Pos2>,
    pub stroke: Stroke,
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
            Shape::Freehand(stroke) => {
                if stroke.points.len() > 1 {
                    painter.add(egui::Shape::line(stroke.points.clone(), stroke.stroke));
                }
            }
        }
    }
}
