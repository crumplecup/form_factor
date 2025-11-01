//! Drawing canvas with interactive annotation tools

use crate::drawing::{Circle, PolygonShape, Rectangle, Shape, ToolMode};
use egui::{Color32, Pos2, Stroke};
use geo::CoordsIter;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

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

    // Selection state (not serialized)
    #[serde(skip)]
    selected_shape: Option<usize>,
    #[serde(skip)]
    show_properties: bool,

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
            selected_shape: None,
            show_properties: false,
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
        for (idx, shape) in self.shapes.iter().enumerate() {
            shape.render(&painter);

            // Draw selection highlight
            if Some(idx) == self.selected_shape {
                let highlight_stroke = Stroke::new(4.0, Color32::from_rgb(255, 215, 0));

                match shape {
                    Shape::Rectangle(rect) => {
                        let rect_shape = egui::Rect::from_two_pos(rect.start, rect.end);
                        painter.rect_stroke(
                            rect_shape,
                            0.0,
                            highlight_stroke,
                            egui::StrokeKind::Outside,
                        );
                    }
                    Shape::Circle(circle) => {
                        painter.circle_stroke(circle.center, circle.radius, highlight_stroke);
                    }
                    Shape::Polygon(poly) => {
                        let points = poly.to_egui_points();
                        painter.add(egui::Shape::closed_line(points, highlight_stroke));
                    }
                }
            }
        }

        // Handle mouse interactions and draw preview
        self.handle_input(&response, &painter);
    }

    #[instrument(skip(self, response, painter), fields(tool = ?self.current_tool))]
    fn handle_input(&mut self, response: &egui::Response, painter: &egui::Painter) {
        match self.current_tool {
            ToolMode::Select => {
                let _span = tracing::debug_span!("selection").entered();

                // Handle selection clicks
                // Use interact_pointer_pos which works for both clicks and drags
                if response.clicked() {
                    debug!(
                        interact_pos = ?response.interact_pointer_pos(),
                        hover_pos = ?response.hover_pos(),
                        "Canvas clicked"
                    );

                    if let Some(pos) = response.interact_pointer_pos() {
                        trace!(?pos, "Using interact_pointer_pos");
                        self.handle_selection_click(pos);
                    } else if let Some(pos) = response.hover_pos() {
                        trace!(?pos, "Using hover_pos fallback");
                        self.handle_selection_click(pos);
                    } else {
                        debug!("No position available for click");
                    }
                }
            }
            _ => {
                // Handle drawing tools
                if let Some(pos) = response.interact_pointer_pos() {
                    if response.drag_started() {
                        self.start_drawing(pos);
                    } else if response.dragged() && self.is_drawing {
                        self.continue_drawing(pos, painter);
                    }
                }

                // Check if mouse was released (drag ended) for drawing tools
                if response.drag_stopped() && self.is_drawing {
                    self.finalize_shape();
                }
            }
        }
    }

    #[instrument(skip(self), fields(pos = ?pos, total_shapes = self.shapes.len()))]
    fn handle_selection_click(&mut self, pos: Pos2) {
        let _span = tracing::debug_span!("hit_testing").entered();

        // Find the topmost shape that contains the click point
        // Iterate in reverse to select the most recently drawn shape first
        let mut selected = None;
        for (idx, shape) in self.shapes.iter().enumerate().rev() {
            let contains = match shape {
                Shape::Rectangle(rect) => {
                    let contains = rect.contains_point(pos);
                    debug!(idx, contains, "Testing rectangle");
                    contains
                }
                Shape::Circle(circle) => {
                    let contains = circle.contains_point(pos);
                    debug!(idx, contains, "Testing circle");
                    contains
                }
                Shape::Polygon(poly) => {
                    let contains = poly.contains_point(pos);
                    debug!(idx, contains, "Testing polygon");
                    contains
                }
            };

            if contains {
                selected = Some(idx);
                break;
            }
        }

        debug!(?selected, "Selection result");
        self.selected_shape = selected;
        self.show_properties = selected.is_some();
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
                        name: String::new(),
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
                            name: String::new(),
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

    /// Show properties panel for the selected shape
    /// Returns true if a properties panel was shown
    pub fn show_properties_panel(&mut self, ctx: &egui::Context) -> bool {
        if !self.show_properties {
            return false;
        }

        let Some(idx) = self.selected_shape else {
            self.show_properties = false;
            return false;
        };

        let Some(shape) = self.shapes.get_mut(idx) else {
            self.selected_shape = None;
            self.show_properties = false;
            return false;
        };

        let mut panel_open = true;
        let close_clicked = match shape {
            Shape::Rectangle(rect) => egui::Window::new("Rectangle Properties")
                .open(&mut panel_open)
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Selected Rectangle");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut rect.name);
                    });

                    ui.separator();

                    let rect_geom = egui::Rect::from_two_pos(rect.start, rect.end);
                    ui.label(format!("Width: {:.1}", rect_geom.width()));
                    ui.label(format!("Height: {:.1}", rect_geom.height()));

                    ui.separator();

                    ui.button("Close").clicked()
                }),
            Shape::Circle(circle) => egui::Window::new("Circle Properties")
                .open(&mut panel_open)
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Selected Circle");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut circle.name);
                    });

                    ui.separator();

                    ui.label(format!("Radius: {:.1}", circle.radius));
                    ui.label(format!(
                        "Center: ({:.1}, {:.1})",
                        circle.center.x, circle.center.y
                    ));

                    ui.separator();

                    ui.button("Close").clicked()
                }),
            Shape::Polygon(poly) => egui::Window::new("Polygon Properties")
                .open(&mut panel_open)
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Selected Polygon");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut poly.name);
                    });

                    ui.separator();

                    ui.label(format!("Points: {}", poly.polygon.exterior().coords_count()));

                    ui.separator();

                    ui.button("Close").clicked()
                }),
        };

        // Close if window was closed or Close button was clicked
        if !panel_open || close_clicked.is_some_and(|r| r.inner.unwrap_or(false)) {
            self.show_properties = false;
            self.selected_shape = None;
        }

        true
    }
}
