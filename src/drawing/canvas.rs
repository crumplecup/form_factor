//! Drawing canvas with interactive annotation tools

use crate::drawing::{Circle, PolygonShape, Rectangle, Shape, ToolMode};
use egui::{Color32, Pos2, Stroke};
use serde::{Deserialize, Serialize};

/// Drawing canvas state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingCanvas {
    /// All completed shapes
    pub shapes: Vec<Shape>,
    /// Currently active tool
    pub current_tool: ToolMode,

    // Drawing state (not serialized)
    #[serde(skip)]
    drawing_start: Option<Pos2>,
    #[serde(skip)]
    current_end: Option<Pos2>,
    #[serde(skip)]
    current_points: Vec<Pos2>,
    #[serde(skip)]
    is_drawing: bool,

    // Style settings
    pub stroke: Stroke,
    pub fill_color: Color32,
}

impl Default for DrawingCanvas {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            current_tool: ToolMode::default(),
            drawing_start: None,
            current_end: None,
            current_points: Vec::new(),
            is_drawing: false,
            stroke: Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
            fill_color: Color32::from_rgba_premultiplied(0, 120, 215, 30),
        }
    }
}

impl DrawingCanvas {
    /// Create a new drawing canvas
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the canvas UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Tool selection toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tool, ToolMode::Select, "✋ Select");
            ui.selectable_value(&mut self.current_tool, ToolMode::Rectangle, "▭ Rectangle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Circle, "◯ Circle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Freehand, "✏ Freehand");
        });

        ui.separator();

        // Canvas area
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        // Paint background
        painter.rect_filled(
            response.rect,
            0.0,
            Color32::from_rgb(245, 245, 245),
        );

        // Draw existing shapes
        for shape in &self.shapes {
            shape.render(&painter);
        }

        // Handle mouse interactions and draw preview
        self.handle_input(&response, &painter);
    }

    fn handle_input(&mut self, response: &egui::Response, painter: &egui::Painter) {
        if let Some(pos) = response.interact_pointer_pos() {
            // Mouse is over the canvas
            if response.drag_started() {
                self.start_drawing(pos);
            } else if response.dragged() && self.is_drawing {
                self.continue_drawing(pos, painter);
            }
        }

        // Check if mouse was released (drag ended)
        if response.drag_stopped() && self.is_drawing {
            self.finalize_shape();
        }
    }

    fn start_drawing(&mut self, pos: Pos2) {
        self.drawing_start = Some(pos);
        self.current_end = Some(pos);
        self.is_drawing = true;

        if self.current_tool == ToolMode::Freehand {
            self.current_points.clear();
            self.current_points.push(pos);
        }
    }

    fn continue_drawing(&mut self, pos: Pos2, painter: &egui::Painter) {
        self.current_end = Some(pos);

        match self.current_tool {
            ToolMode::Rectangle => {
                if let Some(start) = self.drawing_start {
                    let rect = egui::Rect::from_two_pos(start, pos);
                    painter.rect_filled(rect, 0.0, self.fill_color);
                    painter.rect_stroke(rect, 0.0, self.stroke, egui::StrokeKind::Outside);
                }
            }
            ToolMode::Circle => {
                if let Some(center) = self.drawing_start {
                    let radius = center.distance(pos);
                    painter.circle(center, radius, self.fill_color, self.stroke);
                }
            }
            ToolMode::Freehand => {
                self.current_points.push(pos);
                if self.current_points.len() > 2 {
                    // Draw preview as a closed polygon
                    painter.add(egui::Shape::convex_polygon(
                        self.current_points.clone(),
                        self.fill_color,
                        egui::Stroke::NONE,
                    ));
                    painter.add(egui::Shape::closed_line(
                        self.current_points.clone(),
                        self.stroke,
                    ));
                } else if self.current_points.len() > 1 {
                    // Draw preview line until we have enough points
                    painter.add(egui::Shape::line(
                        self.current_points.clone(),
                        self.stroke,
                    ));
                }
            }
            ToolMode::Select => {
                // Selection preview could go here
            }
        }
    }

    fn finalize_shape(&mut self) {
        let shape = match self.current_tool {
            ToolMode::Rectangle => {
                if let (Some(start), Some(end)) = (self.drawing_start, self.current_end) {
                    Some(Shape::Rectangle(Rectangle {
                        start,
                        end,
                        stroke: self.stroke,
                        fill: self.fill_color,
                    }))
                } else {
                    None
                }
            }
            ToolMode::Circle => {
                if let (Some(center), Some(edge)) = (self.drawing_start, self.current_end) {
                    let radius = center.distance(edge);
                    if radius > 0.0 {
                        Some(Shape::Circle(Circle {
                            center,
                            radius,
                            stroke: self.stroke,
                            fill: self.fill_color,
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ToolMode::Freehand => {
                if self.current_points.len() >= 3 {
                    // Create a closed polygon from the points
                    let points: Vec<Pos2> = self.current_points.drain(..).collect();
                    PolygonShape::from_points(points, self.stroke, self.fill_color)
                        .map(Shape::Polygon)
                } else {
                    // Clear points if we don't have enough for a polygon
                    self.current_points.clear();
                    None
                }
            }
            ToolMode::Select => None,
        };

        if let Some(shape) = shape {
            self.shapes.push(shape);
        }

        // Reset drawing state
        self.drawing_start = None;
        self.current_end = None;
        self.current_points.clear();
        self.is_drawing = false;
    }

    /// Clear all shapes from the canvas
    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    /// Remove the last shape (undo)
    pub fn undo(&mut self) {
        self.shapes.pop();
    }

    /// Get the number of shapes on the canvas
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }
}
