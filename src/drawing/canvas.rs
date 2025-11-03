//! Drawing canvas with interactive annotation tools

use crate::drawing::{Circle, LayerManager, LayerType, PolygonShape, Rectangle, RecentProjects, Shape, ToolMode};
#[cfg(feature = "text-detection")]
use crate::text_detection::TextDetector;
use egui::{Color32, Pos2, Stroke};
use geo::CoordsIter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, instrument, trace, warn};

/// Default zoom level for new canvases
fn default_zoom_level() -> f32 {
    5.0
}

/// Drawing canvas state
#[derive(Clone, Serialize, Deserialize)]
pub struct DrawingCanvas {
    /// Project name
    pub project_name: String,
    /// All completed shapes
    pub shapes: Vec<Shape>,
    /// Detected text regions
    pub detections: Vec<Shape>,
    /// Currently active tool
    pub current_tool: ToolMode,
    /// Layer management
    pub layer_manager: LayerManager,
    /// Path to the loaded form image (for serialization)
    pub form_image_path: Option<String>,

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
    pub selected_layer: Option<LayerType>,
    #[serde(skip)]
    show_properties: bool,
    #[serde(skip)]
    focus_name_field: bool,
    #[serde(skip)]
    pub editing_project_name: bool,

    // Edit mode vertex dragging state (not serialized)
    #[serde(skip)]
    dragging_vertex: Option<usize>,
    #[serde(skip)]
    is_dragging_vertex: bool,

    // Rotation state (not serialized)
    #[serde(skip)]
    is_rotating: bool,
    #[serde(skip)]
    rotation_start_angle: f32,
    #[serde(skip)]
    rotation_center: Option<Pos2>,

    // Form image state (not serialized)
    #[serde(skip)]
    form_image: Option<egui::TextureHandle>,
    #[serde(skip)]
    form_image_size: Option<egui::Vec2>,
    #[serde(skip)]
    pending_image_load: Option<String>,

    // Zoom and pan state
    #[serde(default = "default_zoom_level")]
    pub zoom_level: f32,
    #[serde(default)]
    pub pan_offset: egui::Vec2,

    // Settings state (not serialized)
    #[serde(skip)]
    show_settings: bool,
    #[serde(skip)]
    zoom_sensitivity: f32,
    #[serde(skip)]
    grid_spacing_horizontal: f32,
    #[serde(skip)]
    grid_spacing_vertical: f32,
    #[serde(default)]
    pub grid_rotation_angle: f32,

    // Form image rotation
    #[serde(default)]
    pub form_image_rotation: f32,

    // Style settings
    pub stroke: Stroke,
    pub fill_color: Color32,
}

impl Default for DrawingCanvas {
    fn default() -> Self {
        Self {
            project_name: String::from("Untitled"),
            shapes: Vec::new(),
            detections: Vec::new(),
            current_tool: ToolMode::default(),
            layer_manager: LayerManager::new(),
            form_image_path: None,
            drawing_start: None,
            current_end: None,
            current_points: Vec::new(),
            is_drawing: false,
            selected_shape: None,
            selected_layer: None,
            show_properties: false,
            focus_name_field: false,
            editing_project_name: false,
            dragging_vertex: None,
            is_dragging_vertex: false,
            is_rotating: false,
            rotation_start_angle: 0.0,
            rotation_center: None,
            form_image: None,
            form_image_size: None,
            pending_image_load: None,
            zoom_level: 5.0,
            pan_offset: egui::Vec2::ZERO,
            show_settings: false,
            zoom_sensitivity: 5.0,
            grid_spacing_horizontal: 10.0,
            grid_spacing_vertical: 10.0,
            grid_rotation_angle: 0.0,
            form_image_rotation: 0.0,
            stroke: Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
            fill_color: Color32::from_rgba_premultiplied(0, 120, 215, 30),
        }
    }
}

impl std::fmt::Debug for DrawingCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawingCanvas")
            .field("shapes", &self.shapes)
            .field("detections", &self.detections)
            .field("current_tool", &self.current_tool)
            .field("layer_manager", &self.layer_manager)
            .field("form_image_path", &self.form_image_path)
            .field("form_image_loaded", &self.form_image.is_some())
            .field("form_image_size", &self.form_image_size)
            .field("selected_shape", &self.selected_shape)
            .field("stroke", &self.stroke)
            .field("fill_color", &self.fill_color)
            .finish()
    }
}

impl DrawingCanvas {
    /// Create a new drawing canvas
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the canvas UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Log state at frame start (only when we have detections to avoid spam)
        if !self.detections.is_empty() {
            trace!("Frame start: detections={}, shapes={}", self.detections.len(), self.shapes.len());
        }

        // Process any pending image loads (deferred from startup)
        if let Some(pending_path) = self.pending_image_load.take() {
            tracing::debug!("Processing deferred image load: {}", pending_path);
            if let Err(e) = self.load_form_image(&pending_path, ui.ctx()) {
                tracing::warn!("Could not load deferred form image from {}: {}", pending_path, e);
            }
        }

        // Tool selection toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tool, ToolMode::Select, "✋ Select");
            ui.selectable_value(&mut self.current_tool, ToolMode::Rectangle, "▭ Rectangle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Circle, "◯ Circle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Freehand, "✏ Freehand");
            ui.selectable_value(&mut self.current_tool, ToolMode::Edit, "✎ Edit");
            ui.selectable_value(&mut self.current_tool, ToolMode::Rotate, "🔄 Rotate");
        });

        ui.separator();

        // Canvas area
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        // Handle zoom input
        let mut zoom_delta = 0.0;

        // Mouse wheel zoom (only when hovering the canvas)
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                zoom_delta = scroll_delta * 0.001 * self.zoom_sensitivity; // Apply zoom sensitivity
            }
        }

        // Keyboard zoom with Ctrl+/- (works when canvas is focused/clicked)
        if response.clicked() || response.has_focus() {
            ui.input(|i| {
                if i.modifiers.ctrl || i.modifiers.command {
                    if i.key_pressed(egui::Key::Minus) {
                        zoom_delta = -0.1 * self.zoom_sensitivity;
                    } else if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                        zoom_delta = 0.1 * self.zoom_sensitivity;
                    }
                }
            });
        }

        // Apply zoom delta and clamp to zoom range (1.0 - 10.0)
        if zoom_delta != 0.0 {
            let old_zoom = self.zoom_level;
            self.zoom_level = (self.zoom_level + zoom_delta).clamp(1.0, 10.0);

            // Adjust pan offset to zoom toward the center of the viewport
            if let Some(hover_pos) = response.hover_pos() {
                let canvas_center = response.rect.center();
                let zoom_point = hover_pos - canvas_center;
                let zoom_factor = self.zoom_level / old_zoom;
                self.pan_offset = self.pan_offset * zoom_factor + zoom_point * (1.0 - zoom_factor);
            }
        }

        // Paint background if Canvas layer is visible
        if self.layer_manager.is_visible(crate::drawing::LayerType::Canvas) {
            painter.rect_filled(
                response.rect,
                0.0,
                Color32::from_rgb(245, 245, 245),
            );
        }

        // Apply zoom transformation to a child painter
        let canvas_center = response.rect.center();
        let to_screen = egui::emath::TSTransform::from_translation(canvas_center.to_vec2() + self.pan_offset)
            * egui::emath::TSTransform::from_scaling(self.zoom_level)
            * egui::emath::TSTransform::from_translation(-canvas_center.to_vec2());

        // Draw form image on Canvas layer if loaded
        if self.layer_manager.is_visible(crate::drawing::LayerType::Canvas)
            && let (Some(texture), Some(image_size)) = (&self.form_image, self.form_image_size)
        {
            // Calculate scaling to fit image within canvas while maintaining aspect ratio
            let canvas_size = response.rect.size();
            let scale_x = canvas_size.x / image_size.x;
            let scale_y = canvas_size.y / image_size.y;
            let scale = scale_x.min(scale_y); // Use the smaller scale to fit entirely

            let fitted_size = image_size * scale;

            // Center the image within the canvas
            let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
            let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
            let image_pos = response.rect.min + egui::vec2(offset_x, offset_y);

            let image_rect = egui::Rect::from_min_size(image_pos, fitted_size);

            // If rotation is applied, use textured mesh for rotation
            if self.form_image_rotation != 0.0 {
                // Get the center of the image for rotation
                let image_center = image_rect.center();

                // Define the four corners of the image in world coordinates
                let corners = [
                    image_rect.min,
                    egui::pos2(image_rect.max.x, image_rect.min.y),
                    image_rect.max,
                    egui::pos2(image_rect.min.x, image_rect.max.y),
                ];

                // Rotate corners around the image center, then apply zoom/pan transform
                let transformed_corners: Vec<Pos2> = corners
                    .iter()
                    .map(|&corner| {
                        let rotated = Self::rotate_point(corner, image_center, self.form_image_rotation);
                        to_screen.mul_pos(rotated)
                    })
                    .collect();

                // Create a textured mesh for the rotated image
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                let mesh = egui::Mesh {
                    indices: vec![0, 1, 2, 0, 2, 3],
                    vertices: vec![
                        egui::epaint::Vertex {
                            pos: transformed_corners[0],
                            uv: uv.min,
                            color: Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: transformed_corners[1],
                            uv: egui::pos2(uv.max.x, uv.min.y),
                            color: Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: transformed_corners[2],
                            uv: uv.max,
                            color: Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: transformed_corners[3],
                            uv: egui::pos2(uv.min.x, uv.max.y),
                            color: Color32::WHITE,
                        },
                    ],
                    texture_id: texture.id(),
                };

                painter.add(egui::Shape::mesh(mesh));
            } else {
                // No rotation - use simple image rendering
                let transformed_image_rect = egui::Rect::from_min_max(
                    to_screen.mul_pos(image_rect.min),
                    to_screen.mul_pos(image_rect.max),
                );

                painter.image(
                    texture.id(),
                    transformed_image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }

        // Draw detections if Detections layer is visible (with zoom transformation)
        // Note: Detections are stored in image pixel coordinates and need to be mapped to canvas space
        let detections_visible = self.layer_manager.is_visible(crate::drawing::LayerType::Detections);
        if !self.detections.is_empty() {
            debug!("Rendering frame: detections={}, layer_visible={}, image_size={:?}, canvas_size={:?}",
                   self.detections.len(), detections_visible, self.form_image_size, response.rect.size());
        }
        if detections_visible && let (Some(image_size), Some(_texture)) = (self.form_image_size, &self.form_image) {
            // Calculate the image-to-canvas coordinate transform
            let canvas_size = response.rect.size();
            let scale_x = canvas_size.x / image_size.x;
            let scale_y = canvas_size.y / image_size.y;
            let scale = scale_x.min(scale_y);
            let fitted_size = image_size * scale;
            let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
            let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
            let image_offset = response.rect.min + egui::vec2(offset_x, offset_y);

            debug!("Image transform: scale={:.3}, offset=({:.1}, {:.1})", scale, offset_x, offset_y);

            for (idx, detection) in self.detections.iter().enumerate() {
                trace!("Rendering detection {}/{}: {:?}", idx + 1, self.detections.len(), detection);

                // Convert detection from image pixel coordinates to canvas coordinates
                let detection_in_canvas_space = self.map_detection_to_canvas(detection, scale, image_offset);
                self.render_shape_transformed(&detection_in_canvas_space, &painter, &to_screen);
            }
        } else if detections_visible && !self.detections.is_empty() {
            debug!("Detections layer visible but image not loaded: {} detections not rendered", self.detections.len());
        } else if !self.detections.is_empty() {
            debug!("Detections layer hidden: {} detections not rendered", self.detections.len());
        }

        // Draw existing shapes if Shapes layer is visible (with zoom transformation)
        let shapes_visible = self.layer_manager.is_visible(crate::drawing::LayerType::Shapes);
        if shapes_visible {
            for (idx, shape) in self.shapes.iter().enumerate() {
                self.render_shape_transformed(shape, &painter, &to_screen);

                // Draw selection highlight
                if Some(idx) == self.selected_shape {
                    let highlight_stroke = Stroke::new(4.0, Color32::from_rgb(255, 215, 0));

                    match shape {
                        Shape::Rectangle(rect) => {
                            let transformed_corners: Vec<Pos2> = rect.corners()
                                .iter()
                                .map(|p| to_screen.mul_pos(*p))
                                .collect();
                            painter.add(egui::Shape::closed_line(
                                transformed_corners,
                                highlight_stroke,
                            ));
                        }
                        Shape::Circle(circle) => {
                            let transformed_center = to_screen.mul_pos(circle.center);
                            let transformed_radius = circle.radius * self.zoom_level;
                            painter.circle_stroke(transformed_center, transformed_radius, highlight_stroke);
                        }
                        Shape::Polygon(poly) => {
                            let points: Vec<Pos2> = poly.to_egui_points()
                                .iter()
                                .map(|p| to_screen.mul_pos(*p))
                                .collect();
                            painter.add(egui::Shape::closed_line(points, highlight_stroke));
                        }
                    }

                    // Draw edit vertices if in Edit mode
                    if self.current_tool == ToolMode::Edit && Some(idx) == self.selected_shape {
                        self.draw_edit_vertices_transformed(shape, &painter, &to_screen);
                    }
                }
            }
        }

        // Draw grid on top of everything if Grid layer is visible
        if self.layer_manager.is_visible(crate::drawing::LayerType::Grid) {
            debug!(
                grid_visible = true,
                grid_spacing_h = self.grid_spacing_horizontal,
                grid_spacing_v = self.grid_spacing_vertical,
                "Calling draw_grid (rendering on top)"
            );
            self.draw_grid(&painter, &response.rect, &to_screen);
        } else {
            trace!("Grid layer is not visible, skipping grid render");
        }

        // Handle mouse interactions and draw preview (with zoom transformation)
        self.handle_input(&response, &painter, &to_screen);
    }

    #[instrument(skip(self, response, painter, transform), fields(tool = ?self.current_tool))]
    fn handle_input(&mut self, response: &egui::Response, painter: &egui::Painter, transform: &egui::emath::TSTransform) {
        // Helper to transform screen coordinates to canvas coordinates
        let transform_pos = |screen_pos: Pos2| -> Pos2 {
            transform.inverse().mul_pos(screen_pos)
        };
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
                        let canvas_pos = transform_pos(pos);
                        trace!(?pos, ?canvas_pos, "Using interact_pointer_pos");
                        self.handle_selection_click(canvas_pos);
                    } else if let Some(pos) = response.hover_pos() {
                        let canvas_pos = transform_pos(pos);
                        trace!(?pos, ?canvas_pos, "Using hover_pos fallback");
                        self.handle_selection_click(canvas_pos);
                    } else {
                        debug!("No position available for click");
                    }
                }
            }
            ToolMode::Edit => {
                let _span = tracing::debug_span!("edit_vertices").entered();

                // Handle vertex editing
                if let Some(pos) = response.interact_pointer_pos() {
                    let canvas_pos = transform_pos(pos);
                    if response.drag_started() {
                        self.start_vertex_drag(canvas_pos);
                    } else if response.dragged() && self.is_dragging_vertex {
                        self.continue_vertex_drag(canvas_pos);
                    }
                }

                // Check if drag ended
                if response.drag_stopped() && self.is_dragging_vertex {
                    self.finish_vertex_drag();
                }

                // Also handle selection clicks when not dragging
                if response.clicked() && !self.is_dragging_vertex {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = transform_pos(pos);
                        self.handle_selection_click(canvas_pos);
                    } else if let Some(pos) = response.hover_pos() {
                        let canvas_pos = transform_pos(pos);
                        self.handle_selection_click(canvas_pos);
                    }
                }
            }
            ToolMode::Rectangle | ToolMode::Circle | ToolMode::Freehand => {
                // Handle drawing tools
                if let Some(pos) = response.interact_pointer_pos() {
                    let canvas_pos = transform_pos(pos);
                    if response.drag_started() {
                        self.start_drawing(canvas_pos);
                    } else if response.dragged() && self.is_drawing {
                        self.continue_drawing(canvas_pos, painter, transform);
                    }
                }

                // Check if mouse was released (drag ended) for drawing tools
                if response.drag_stopped() && self.is_drawing {
                    self.finalize_shape();
                }
            }
            ToolMode::Rotate => {
                let _span = tracing::debug_span!(
                    "rotate_tool",
                    selected_layer = ?self.selected_layer,
                    selected_shape = ?self.selected_shape,
                    is_rotating = self.is_rotating
                ).entered();

                debug!(
                    clicked = response.clicked(),
                    drag_started = response.drag_started(),
                    dragged = response.dragged(),
                    drag_stopped = response.drag_stopped(),
                    is_rotating = self.is_rotating,
                    ?self.selected_layer,
                    ?self.selected_shape,
                    "Rotate tool input events"
                );

                // Handle rotation based on selected layer
                if let Some(pos) = response.interact_pointer_pos() {
                    let canvas_pos = transform_pos(pos);

                    if response.drag_started() {
                        debug!(?canvas_pos, "Drag started - calling start_rotation");
                        self.start_rotation(canvas_pos);
                    } else if response.dragged() && self.is_rotating {
                        debug!(?canvas_pos, "Dragging - calling continue_rotation");
                        self.continue_rotation(canvas_pos);
                    } else if response.dragged() && !self.is_rotating {
                        debug!("Dragging but is_rotating is false");
                    }
                }

                // Check if drag ended
                if response.drag_stopped() {
                    if self.is_rotating {
                        debug!("Drag stopped - calling finish_rotation");
                        self.finish_rotation();
                    } else {
                        debug!("Drag stopped but is_rotating was false");
                    }
                }

                // Also handle selection clicks when not rotating
                if response.clicked() && !self.is_rotating {
                    debug!("Click detected - handling selection");
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = transform_pos(pos);
                        debug!(?canvas_pos, "Calling handle_selection_click with interact_pointer_pos");
                        self.handle_selection_click(canvas_pos);
                    } else if let Some(pos) = response.hover_pos() {
                        let canvas_pos = transform_pos(pos);
                        debug!(?canvas_pos, "Calling handle_selection_click with hover_pos");
                        self.handle_selection_click(canvas_pos);
                    }
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

        debug!(?selected, ?self.selected_shape, "Selection result");

        // Check if a shape was newly selected to auto-focus the name field
        // Only set focus if this is a different selection or a new selection
        let _span = tracing::debug_span!("shape_name_autofocus").entered();

        let should_focus = selected != self.selected_shape && selected.is_some();

        if should_focus {
            debug!("Setting focus flag for newly selected shape");
            self.focus_name_field = true;
        } else {
            trace!(
                selection_changed = (selected != self.selected_shape),
                selected_is_some = selected.is_some(),
                "Not setting focus flag"
            );
        }

        self.selected_shape = selected;
        self.show_properties = selected.is_some();

        // When a shape is selected, also select the Shapes layer for rotation
        if selected.is_some() {
            debug!("Shape selected - setting selected_layer to Shapes");
            self.selected_layer = Some(LayerType::Shapes);
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

    fn continue_drawing(&mut self, pos: Pos2, painter: &egui::Painter, transform: &egui::emath::TSTransform) {
        self.current_end = Some(pos);

        match self.current_tool {
            ToolMode::Rectangle => {
                if let Some(start) = self.drawing_start {
                    // Transform the rectangle corners for preview
                    let rect = egui::Rect::from_two_pos(start, pos);
                    let transformed_rect = egui::Rect::from_min_max(
                        transform.mul_pos(rect.min),
                        transform.mul_pos(rect.max),
                    );
                    painter.rect_filled(transformed_rect, 0.0, self.fill_color);
                    painter.rect_stroke(transformed_rect, 0.0, self.stroke, egui::StrokeKind::Outside);
                }
            }
            ToolMode::Circle => {
                if let Some(center) = self.drawing_start {
                    let radius = center.distance(pos);
                    let transformed_center = transform.mul_pos(center);
                    let transformed_radius = radius * self.zoom_level;
                    painter.circle(transformed_center, transformed_radius, self.fill_color, self.stroke);
                }
            }
            ToolMode::Freehand => {
                self.current_points.push(pos);
                if self.current_points.len() > 2 {
                    // Transform points for preview
                    let transformed_points: Vec<Pos2> = self.current_points
                        .iter()
                        .map(|p| transform.mul_pos(*p))
                        .collect();
                    // Draw preview as a closed polygon
                    painter.add(egui::Shape::convex_polygon(
                        transformed_points.clone(),
                        self.fill_color,
                        egui::Stroke::NONE,
                    ));
                    painter.add(egui::Shape::closed_line(
                        transformed_points,
                        self.stroke,
                    ));
                } else if self.current_points.len() > 1 {
                    // Transform points for preview line
                    let transformed_points: Vec<Pos2> = self.current_points
                        .iter()
                        .map(|p| transform.mul_pos(*p))
                        .collect();
                    // Draw preview line until we have enough points
                    painter.add(egui::Shape::line(
                        transformed_points,
                        self.stroke,
                    ));
                }
            }
            ToolMode::Select => {
                // Selection preview could go here
            }
            ToolMode::Edit => {
                // Edit mode doesn't draw new shapes
            }
            ToolMode::Rotate => {
                // Rotate mode doesn't draw new shapes
            }
        }
    }

    fn finalize_shape(&mut self) {
        let shape = match self.current_tool {
            ToolMode::Rectangle => {
                if let (Some(start), Some(end)) = (self.drawing_start, self.current_end) {
                    Rectangle::from_corners(start, end, self.stroke, self.fill_color)
                        .map(Shape::Rectangle)
                        .map_err(|e| {
                            warn!("Failed to create rectangle: {}", e);
                            e
                        })
                        .ok()
                } else {
                    None
                }
            }
            ToolMode::Circle => {
                if let (Some(center), Some(edge)) = (self.drawing_start, self.current_end) {
                    let radius = center.distance(edge);
                    Circle::new(center, radius, self.stroke, self.fill_color)
                        .map(Shape::Circle)
                        .map_err(|e| {
                            warn!("Failed to create circle: {}", e);
                            e
                        })
                        .ok()
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
                        .map_err(|e| {
                            warn!("Failed to create polygon: {}", e);
                            e
                        })
                        .ok()
                } else {
                    // Clear points if we don't have enough for a polygon
                    self.current_points.clear();
                    None
                }
            }
            ToolMode::Select => None,
            ToolMode::Edit => None,
            ToolMode::Rotate => None,
        };

        if let Some(shape) = shape {
            self.shapes.push(shape);

            // Automatically select the newly created shape so user can name it
            let new_shape_idx = self.shapes.len() - 1;
            self.selected_shape = Some(new_shape_idx);
            self.show_properties = true;
            self.focus_name_field = true;

            debug!(
                shape_index = new_shape_idx,
                "Auto-selecting newly created shape"
            );
        }

        // Reset drawing state
        self.drawing_start = None;
        self.current_end = None;
        self.current_points.clear();
        self.is_drawing = false;
    }

    /// Clear all shapes and detections from the canvas
    pub fn clear(&mut self) {
        debug!("Clearing canvas: shapes={}, detections={}", self.shapes.len(), self.detections.len());
        self.shapes.clear();
        self.detections.clear();
    }

    /// Remove the last shape (undo)
    pub fn undo(&mut self) {
        self.shapes.pop();
    }

    /// Get the number of shapes on the canvas
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Clear only shapes from the canvas
    pub fn clear_shapes(&mut self) {
        debug!("Clearing shapes: count={}", self.shapes.len());
        self.shapes.clear();
        self.selected_shape = None;
    }

    /// Clear only detections from the canvas
    pub fn clear_detections(&mut self) {
        debug!("Clearing detections: count={}", self.detections.len());
        self.detections.clear();
    }

    /// Clear the canvas image (form image)
    pub fn clear_canvas_image(&mut self) {
        debug!("Clearing canvas image: path={:?}", self.form_image_path);
        self.form_image = None;
        self.form_image_size = None;
        self.form_image_path = None;
        self.pending_image_load = None;
    }

    /// Detect text regions in the loaded form image
    #[cfg(feature = "text-detection")]
    #[instrument(skip(self), fields(confidence_threshold, existing_detections = self.detections.len()))]
    pub fn detect_text_regions(&mut self, confidence_threshold: f32) -> Result<usize, String> {
        // Check if we have a form image loaded
        let form_path = self.form_image_path.as_ref()
            .ok_or_else(|| "No form image loaded".to_string())?;

        tracing::info!("Detecting text regions in: {}", form_path);

        // Create text detector with default model paths
        let detector = TextDetector::default();

        // Detect text regions
        let regions = detector.detect_from_file(form_path, confidence_threshold)
            .map_err(|e| format!("Text detection failed: {}", e))?;

        let count = regions.len();
        tracing::info!("Detected {} text regions", count);

        // Create rectangle shapes for each detected region
        for (i, region) in regions.iter().enumerate() {
            let top_left = Pos2::new(region.x as f32, region.y as f32);
            let bottom_right = Pos2::new(
                (region.x + region.width) as f32,
                (region.y + region.height) as f32,
            );

            // Create a rectangle shape with a distinctive color for text regions
            let stroke = Stroke::new(2.0, Color32::from_rgb(255, 165, 0)); // Orange
            let fill = Color32::TRANSPARENT; // No fill, outline only

            match Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                Ok(mut rect) => {
                    rect.name = format!("Text Region {} ({:.2}%)", i + 1, region.confidence * 100.0);
                    self.detections.push(Shape::Rectangle(rect));
                }
                Err(e) => {
                    warn!("Failed to create detection rectangle for region {}: {}", i, e);
                }
            }
        }

        debug!("Added {} detections, total now: {}", count, self.detections.len());

        Ok(count)
    }

    /// Extract text from all detections using OCR
    ///
    /// Returns a vector of (detection_index, OCR_result) pairs
    #[cfg(feature = "ocr")]
    #[instrument(skip(self, ocr), fields(detections = self.detections.len()))]
    pub fn extract_text_from_detections(
        &self,
        ocr: &crate::ocr::OCREngine,
    ) -> Result<Vec<(usize, crate::ocr::OCRResult)>, String> {
        let form_path = self.form_image_path.as_ref()
            .ok_or_else(|| "No form image loaded".to_string())?;

        tracing::info!("Extracting text from {} detections", self.detections.len());

        let mut results = Vec::new();

        for (idx, detection) in self.detections.iter().enumerate() {
            match self.extract_text_from_shape(ocr, form_path, detection) {
                Ok(result) => {
                    debug!(
                        "Detection {}: extracted {} chars with {:.1}% confidence",
                        idx,
                        result.text.len(),
                        result.confidence
                    );
                    results.push((idx, result));
                }
                Err(e) => {
                    warn!("Failed to extract text from detection {}: {}", idx, e);
                }
            }
        }

        tracing::info!("Extracted text from {}/{} detections", results.len(), self.detections.len());
        Ok(results)
    }

    /// Extract text from a specific shape using OCR
    #[cfg(feature = "ocr")]
    fn extract_text_from_shape(
        &self,
        ocr: &crate::ocr::OCREngine,
        image_path: &str,
        shape: &Shape,
    ) -> Result<crate::ocr::OCRResult, String> {
        use crate::drawing::Shape;

        // Get bounding box of the shape in image pixel coordinates
        let bbox = match shape {
            Shape::Rectangle(rect) => {
                // Find min/max coords
                let xs: Vec<f32> = rect.corners().iter().map(|p| p.x).collect();
                let ys: Vec<f32> = rect.corners().iter().map(|p| p.y).collect();

                let x_min = xs.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let y_min = ys.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let x_max = xs.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;
                let y_max = ys.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;

                let width = x_max.saturating_sub(x_min);
                let height = y_max.saturating_sub(y_min);

                (x_min, y_min, width, height)
            }
            Shape::Circle(circle) => {
                let x = (circle.center.x - circle.radius).max(0.0) as u32;
                let y = (circle.center.y - circle.radius).max(0.0) as u32;
                let diameter = (circle.radius * 2.0) as u32;
                (x, y, diameter, diameter)
            }
            Shape::Polygon(poly) => {
                // Get bounding box from polygon points
                let points = poly.to_egui_points();
                let xs: Vec<f32> = points.iter().map(|p| p.x).collect();
                let ys: Vec<f32> = points.iter().map(|p| p.y).collect();

                let x_min = xs.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let y_min = ys.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let x_max = xs.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;
                let y_max = ys.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;

                let width = x_max.saturating_sub(x_min);
                let height = y_max.saturating_sub(y_min);

                (x_min, y_min, width, height)
            }
        };

        trace!("Shape bbox in image coords: {:?}", bbox);

        // Extract text from this region
        ocr.extract_text_from_region_file(image_path, bbox)
    }

    /// Load a form image from a file path
    pub fn load_form_image(&mut self, path: &str, ctx: &egui::Context) -> Result<(), String> {
        // Load the image from disk
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

        // Convert to RGBA8
        let size = [img.width() as usize, img.height() as usize];
        let img_rgba = img.to_rgba8();
        let pixels = img_rgba.as_flat_samples();

        // Create egui ColorImage
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        // Create texture
        let texture = ctx.load_texture(
            "form_image",
            color_image,
            egui::TextureOptions::default(),
        );

        // Store the texture and metadata
        self.form_image_size = Some(egui::Vec2::new(img.width() as f32, img.height() as f32));
        self.form_image = Some(texture);
        self.form_image_path = Some(path.to_string());

        // Reset zoom and pan to fit image to window
        self.zoom_level = 1.0;
        self.pan_offset = egui::Vec2::ZERO;

        tracing::info!("Loaded form image: {} ({}x{})", path, img.width(), img.height());
        Ok(())
    }

    /// Clear the loaded form image
    pub fn clear_form_image(&mut self) {
        self.form_image = None;
        self.form_image_size = None;
        self.form_image_path = None;
    }

    /// Save the project state to a file
    #[instrument(skip(self), fields(path, shapes = self.shapes.len(), detections = self.detections.len()))]
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        debug!("Saving project: shapes={}, detections={}", self.shapes.len(), self.detections.len());

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;

        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Add to recent projects
        let mut recent = RecentProjects::load();
        recent.add(PathBuf::from(path));
        if let Err(e) = recent.save() {
            tracing::warn!("Failed to save recent projects: {}", e);
        }

        tracing::info!("Saved project to: {}", path);
        Ok(())
    }

    /// Load the project state from a file
    pub fn load_from_file(&mut self, path: &str, ctx: &egui::Context) -> Result<(), String> {
        self.load_from_file_impl(path, ctx, false)
    }

    /// Load the project state from a file (internal implementation)
    /// If defer_image_load is true, the image will be loaded on the next update() call
    #[instrument(skip(self, ctx), fields(path, defer_image_load))]
    fn load_from_file_impl(&mut self, path: &str, ctx: &egui::Context, defer_image_load: bool) -> Result<(), String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let loaded: DrawingCanvas = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize project: {}", e))?;

        debug!("Deserialized project state: shapes={}, detections={}",
               loaded.shapes.len(), loaded.detections.len());

        // Copy all the serialized state
        self.project_name = loaded.project_name;
        self.shapes = loaded.shapes;
        self.detections = loaded.detections;
        self.current_tool = loaded.current_tool;
        self.layer_manager = loaded.layer_manager;
        self.stroke = loaded.stroke;
        self.fill_color = loaded.fill_color;
        self.zoom_level = loaded.zoom_level;
        self.pan_offset = loaded.pan_offset;
        self.grid_rotation_angle = loaded.grid_rotation_angle;
        self.form_image_rotation = loaded.form_image_rotation;

        debug!("Loaded project state: shapes={}, detections={}, detections_layer_visible={}",
               self.shapes.len(),
               self.detections.len(),
               self.layer_manager.is_visible(LayerType::Detections));

        // If there was a form image saved, try to reload it
        if let Some(form_path) = &loaded.form_image_path {
            if defer_image_load {
                // Defer image loading until the first update() call
                self.pending_image_load = Some(form_path.clone());
                self.form_image_path = Some(form_path.clone());
                tracing::debug!("Deferred loading of form image: {}", form_path);
            } else {
                // Load image immediately
                if let Err(e) = self.load_form_image(form_path, ctx) {
                    tracing::warn!("Could not reload form image from {}: {}", form_path, e);
                    // Don't fail the entire load if the image is missing
                    self.form_image_path = loaded.form_image_path;
                }
            }
        } else {
            self.form_image_path = None;
            self.form_image = None;
            self.form_image_size = None;
        }

        // Add to recent projects
        let mut recent = RecentProjects::load();
        recent.add(PathBuf::from(path));
        if let Err(e) = recent.save() {
            tracing::warn!("Failed to save recent projects: {}", e);
        }

        tracing::info!("Loaded project from: {}", path);
        Ok(())
    }

    /// Load the most recent project on startup (defers image loading)
    pub fn load_recent_on_startup(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let recent = RecentProjects::load();
        if let Some(recent_path) = recent.most_recent()
            && let Some(path_str) = recent_path.to_str()
        {
            return self.load_from_file_impl(path_str, ctx, true);
        }
        Err("No recent projects found".to_string())
    }

    /// Show inline properties UI for the selected shape
    pub fn show_inline_properties(&mut self, ui: &mut egui::Ui) {
        if !self.show_properties {
            trace!("Not showing properties panel");
            return;
        }

        let Some(idx) = self.selected_shape else {
            trace!("No shape selected");
            return;
        };

        let Some(shape) = self.shapes.get_mut(idx) else {
            trace!("Selected shape index {} out of bounds", idx);
            self.selected_shape = None;
            self.show_properties = false;
            return;
        };

        debug!(
            shape_type = ?shape,
            focus_flag = self.focus_name_field,
            "Showing properties panel"
        );

        ui.heading("Shape Properties");
        ui.separator();

        match shape {
            Shape::Rectangle(rect) => {
                ui.label("Type: Rectangle");
                ui.separator();

                ui.horizontal(|ui| {
                    let _span = tracing::debug_span!("shape_name_autofocus").entered();

                    ui.label("Name:");

                    // Create text edit with explicit ID for focusing
                    let text_edit = egui::TextEdit::singleline(&mut rect.name)
                        .id_salt("rectangle_name");
                    let response = ui.add(text_edit);

                    debug!(
                        has_focus = response.has_focus(),
                        focus_flag = self.focus_name_field,
                        widget_id = ?response.id,
                        "Rectangle name field rendered"
                    );

                    // Auto-focus the name field when rectangle is first selected
                    if self.focus_name_field {
                        debug!("Requesting focus on rectangle name field");
                        response.request_focus();
                        self.focus_name_field = false;
                        debug!(
                            has_focus_after_request = response.has_focus(),
                            "Focus requested, checking result"
                        );
                    }
                });

                ui.separator();

                // Calculate bounding box from all 4 corners
                let min_x = rect.corners().iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = rect.corners().iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = rect.corners().iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = rect.corners().iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                let rect_geom = egui::Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
                ui.label(format!("Width: {:.1}", rect_geom.width()));
                ui.label(format!("Height: {:.1}", rect_geom.height()));
            }
            Shape::Circle(circle) => {
                ui.label("Type: Circle");
                ui.separator();

                ui.horizontal(|ui| {
                    let _span = tracing::debug_span!("shape_name_autofocus").entered();

                    ui.label("Name:");

                    // Create text edit with explicit ID for focusing
                    let text_edit = egui::TextEdit::singleline(&mut circle.name)
                        .id_salt("circle_name");
                    let response = ui.add(text_edit);

                    debug!(
                        has_focus = response.has_focus(),
                        focus_flag = self.focus_name_field,
                        widget_id = ?response.id,
                        "Circle name field rendered"
                    );

                    // Auto-focus the name field when circle is first selected
                    if self.focus_name_field {
                        debug!("Requesting focus on circle name field");
                        response.request_focus();
                        self.focus_name_field = false;
                        debug!(
                            has_focus_after_request = response.has_focus(),
                            "Focus requested, checking result"
                        );
                    }
                });

                ui.separator();

                ui.label(format!("Radius: {:.1}", circle.radius));
                ui.label(format!("Center: ({:.1}, {:.1})", circle.center.x, circle.center.y));
            }
            Shape::Polygon(poly) => {
                ui.label("Type: Polygon");
                ui.separator();

                ui.horizontal(|ui| {
                    let _span = tracing::debug_span!("shape_name_autofocus").entered();

                    ui.label("Name:");

                    // Create text edit with explicit ID for focusing
                    let text_edit = egui::TextEdit::singleline(&mut poly.name)
                        .id_salt("polygon_name");
                    let response = ui.add(text_edit);

                    debug!(
                        has_focus = response.has_focus(),
                        focus_flag = self.focus_name_field,
                        widget_id = ?response.id,
                        "Polygon name field rendered"
                    );

                    // Auto-focus the name field when polygon is first selected
                    if self.focus_name_field {
                        debug!("Requesting focus on polygon name field");
                        response.request_focus();
                        self.focus_name_field = false;
                        debug!(
                            has_focus_after_request = response.has_focus(),
                            "Focus requested, checking result"
                        );
                    }
                });

                ui.separator();

                ui.label(format!("Points: {}", poly.polygon().exterior().coords_count()));
            }
        }

        ui.separator();

        if ui.button("Deselect").clicked() {
            self.selected_shape = None;
            self.show_properties = false;
        }
    }

    /// Show properties panel for the selected shape (popup window version)
    /// Returns true if a properties panel was shown
    #[allow(dead_code)]
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

                    // Calculate bounding box from all 4 corners
                    let min_x = rect.corners().iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                    let max_x = rect.corners().iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                    let min_y = rect.corners().iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                    let max_y = rect.corners().iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                    let rect_geom = egui::Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
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

                    ui.label(format!("Points: {}", poly.polygon().exterior().coords_count()));

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

    /// Show settings inline in the side panel
    pub fn show_inline_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();

        ui.label("Zoom Sensitivity:");
        ui.add(
            egui::Slider::new(&mut self.zoom_sensitivity, 0.1..=10.0)
                .text("Sensitivity")
                .logarithmic(true)
        );
        ui.label("Higher values make zoom more responsive");

        ui.separator();

        ui.label("Grid Spacing (Horizontal):");
        ui.add(
            egui::Slider::new(&mut self.grid_spacing_horizontal, 0.1..=100.0)
                .text("Horizontal")
                .logarithmic(true)
        );

        ui.label("Grid Spacing (Vertical):");
        ui.add(
            egui::Slider::new(&mut self.grid_spacing_vertical, 0.1..=100.0)
                .text("Vertical")
                .logarithmic(true)
        );
        ui.label("Distance between grid lines");
    }

    /// Show settings panel
    /// Returns true if the settings panel was shown
    #[allow(dead_code)]
    pub fn show_settings_panel(&mut self, ctx: &egui::Context) -> bool {
        if !self.show_settings {
            return false;
        }

        let mut panel_open = true;
        egui::Window::new("Settings")
            .open(&mut panel_open)
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                ui.label("Zoom Sensitivity:");
                ui.add(
                    egui::Slider::new(&mut self.zoom_sensitivity, 0.1..=10.0)
                        .text("Sensitivity")
                        .logarithmic(true)
                );
                ui.label("Higher values make zoom more responsive");

                ui.separator();

                if ui.button("Close").clicked() {
                    self.show_settings = false;
                }
            });

        // Close if window was closed
        if !panel_open {
            self.show_settings = false;
        }

        true
    }

    /// Start dragging a vertex
    fn start_vertex_drag(&mut self, pos: Pos2) {
        const VERTEX_CLICK_RADIUS: f32 = 8.0;

        let Some(idx) = self.selected_shape else {
            // No shape selected, try to select one
            self.handle_selection_click(pos);
            return;
        };

        let Some(shape) = self.shapes.get(idx) else {
            return;
        };

        // Find which vertex was clicked
        let clicked_vertex = match shape {
            Shape::Rectangle(rect) => {
                // Check all 4 corners
                rect.corners()
                    .iter()
                    .enumerate()
                    .find(|(_, corner)| pos.distance(**corner) < VERTEX_CLICK_RADIUS)
                    .map(|(i, _)| i)
            }
            Shape::Circle(circle) => {
                if pos.distance(circle.center) < VERTEX_CLICK_RADIUS {
                    Some(0)
                } else {
                    let edge_point = egui::pos2(circle.center.x + circle.radius, circle.center.y);
                    if pos.distance(edge_point) < VERTEX_CLICK_RADIUS {
                        Some(1)
                    } else {
                        None
                    }
                }
            }
            Shape::Polygon(poly) => {
                poly.to_egui_points()
                    .iter()
                    .enumerate()
                    .find(|(_, vertex_pos)| pos.distance(**vertex_pos) < VERTEX_CLICK_RADIUS)
                    .map(|(i, _)| i)
            }
        };

        if let Some(vertex_idx) = clicked_vertex {
            debug!(vertex_idx, "Starting vertex drag");
            self.dragging_vertex = Some(vertex_idx);
            self.is_dragging_vertex = true;
        }
    }

    /// Continue dragging a vertex
    fn continue_vertex_drag(&mut self, pos: Pos2) {
        let Some(vertex_idx) = self.dragging_vertex else {
            return;
        };

        let Some(shape_idx) = self.selected_shape else {
            return;
        };

        let Some(shape) = self.shapes.get_mut(shape_idx) else {
            return;
        };

        trace!(vertex_idx, ?pos, "Continuing vertex drag");

        // Update the vertex position based on which shape and vertex
        match shape {
            Shape::Rectangle(_rect) => {
                // Note: vertex editing for rectangles not yet implemented with new geo-based structure
                // Would need to rebuild polygon from updated corners
                warn!("Rectangle vertex editing not implemented");
            }
            Shape::Circle(circle) => {
                match vertex_idx {
                    0 => {
                        // Moving center - maintain radius
                        circle.center = pos;
                    }
                    1 => {
                        // Moving edge - update radius
                        circle.radius = circle.center.distance(pos);
                    }
                    _ => {}
                }
            }
            Shape::Polygon(_poly) => {
                // Note: vertex editing for polygons not yet implemented with private polygon field
                // Would need to rebuild polygon from updated points using from_points constructor
                warn!("Polygon vertex editing not implemented");
            }
        }
    }

    /// Finish dragging a vertex
    fn finish_vertex_drag(&mut self) {
        debug!("Finishing vertex drag");
        self.dragging_vertex = None;
        self.is_dragging_vertex = false;
    }

    /// Start rotation interaction
    #[instrument(skip(self), fields(
        pos = ?pos,
        selected_layer = ?self.selected_layer,
        selected_shape = ?self.selected_shape,
        has_form_image = self.form_image.is_some()
    ))]
    fn start_rotation(&mut self, pos: Pos2) {
        let _span = tracing::debug_span!("start_rotation").entered();

        debug!(
            ?pos,
            ?self.selected_layer,
            ?self.selected_shape,
            has_form_image = self.form_image.is_some(),
            "Attempting to start rotation"
        );

        // Determine what to rotate based on selected layer
        match self.selected_layer {
            Some(LayerType::Shapes) => {
                debug!("Shapes layer selected");
                // If a shape is selected, rotate it
                if let Some(idx) = self.selected_shape {
                    debug!(shape_idx = idx, "Shape is selected");
                    if let Some(shape) = self.shapes.get(idx) {
                        let center = self.get_shape_center(shape);
                        self.rotation_center = Some(center);
                        self.rotation_start_angle = Self::calculate_angle(center, pos);
                        self.is_rotating = true;
                        debug!(?center, start_angle = self.rotation_start_angle, "Started rotating shape");
                    } else {
                        debug!(shape_idx = idx, "Shape index out of bounds");
                    }
                } else {
                    debug!("No shape selected - cannot rotate");
                }
            }
            Some(LayerType::Grid) => {
                debug!("Grid layer selected - rotating grid");
                // Rotate the grid around the canvas center
                self.rotation_center = Some(Pos2::ZERO);
                self.rotation_start_angle = Self::calculate_angle(Pos2::ZERO, pos);
                self.is_rotating = true;
                debug!(rotation_center = ?Pos2::ZERO, start_angle = self.rotation_start_angle, "Started rotating grid");
            }
            Some(LayerType::Canvas) => {
                debug!(has_form_image = self.form_image.is_some(), "Canvas layer selected");
                // Rotate the form image if one is loaded
                if self.form_image.is_some() {
                    self.rotation_center = Some(Pos2::ZERO);
                    self.rotation_start_angle = Self::calculate_angle(Pos2::ZERO, pos);
                    self.is_rotating = true;
                    debug!(rotation_center = ?Pos2::ZERO, start_angle = self.rotation_start_angle, "Started rotating form image");
                } else {
                    debug!("No form image loaded - cannot rotate");
                }
            }
            Some(LayerType::Detections) => {
                debug!("Detections layer selected - detections cannot be rotated");
            }
            None => {
                debug!("No layer selected for rotation - user must select a layer first");
            }
        }

        debug!(is_rotating = self.is_rotating, "Rotation state after start_rotation");
    }

    /// Continue rotation interaction
    fn continue_rotation(&mut self, pos: Pos2) {
        if let Some(center) = self.rotation_center {
            let current_angle = Self::calculate_angle(center, pos);
            let angle_delta = current_angle - self.rotation_start_angle;

            debug!(?pos, ?center, current_angle, angle_delta, "Continuing rotation");

            // Apply rotation based on selected layer (negated for inverted axis)
            match self.selected_layer {
                Some(LayerType::Shapes) => {
                    // Note: rotation_angle removed from shapes
                    // Individual shape rotation would need to be implemented differently
                    // For now, only grid rotation is supported
                }
                Some(LayerType::Grid) => {
                    self.grid_rotation_angle -= angle_delta;
                }
                Some(LayerType::Canvas) => {
                    self.form_image_rotation -= angle_delta;
                }
                Some(LayerType::Detections) => {
                    // Detections cannot be rotated
                }
                None => {}
            }

            // Update start angle for next frame
            self.rotation_start_angle = current_angle;
        }
    }

    /// Finish rotation interaction
    fn finish_rotation(&mut self) {
        debug!("Finishing rotation");
        self.is_rotating = false;
        self.rotation_center = None;
    }

    /// Calculate angle from center to position in radians
    fn calculate_angle(center: Pos2, pos: Pos2) -> f32 {
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        dy.atan2(dx)
    }

    /// Get the center point of a shape
    fn get_shape_center(&self, shape: &Shape) -> Pos2 {
        match shape {
            Shape::Rectangle(rect) => {
                let sum_x: f32 = rect.corners().iter().map(|p| p.x).sum();
                let sum_y: f32 = rect.corners().iter().map(|p| p.y).sum();
                Pos2::new(sum_x / 4.0, sum_y / 4.0)
            }
            Shape::Circle(circle) => circle.center,
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                let sum_x: f32 = points.iter().map(|p| p.x).sum();
                let sum_y: f32 = points.iter().map(|p| p.y).sum();
                let count = points.len() as f32;
                Pos2::new(sum_x / count, sum_y / count)
            }
        }
    }

    /// Draw grid overlay on the canvas
    fn draw_grid(&self, painter: &egui::Painter, canvas_rect: &egui::Rect, transform: &egui::emath::TSTransform) {
        let _span = tracing::debug_span!(
            "draw_grid",
            spacing_h = self.grid_spacing_horizontal,
            spacing_v = self.grid_spacing_vertical
        ).entered();

        debug!(
            canvas_rect = ?canvas_rect,
            zoom_level = self.zoom_level,
            "Starting grid render"
        );

        // Use a more visible grid color - darker with higher opacity
        let grid_color = Color32::from_rgba_premultiplied(100, 100, 100, 180);
        let grid_stroke = Stroke::new(1.0, grid_color);

        debug!(
            grid_color = ?grid_color,
            stroke_width = grid_stroke.width,
            "Grid stroke configuration"
        );

        // Calculate the canvas bounds in world coordinates
        let canvas_min = transform.inverse().mul_pos(canvas_rect.min);
        let canvas_max = transform.inverse().mul_pos(canvas_rect.max);

        debug!(
            canvas_min = ?canvas_min,
            canvas_max = ?canvas_max,
            "Canvas bounds in world coordinates"
        );

        // Determine grid line positions in world coordinates
        let spacing_h = self.grid_spacing_horizontal;
        let spacing_v = self.grid_spacing_vertical;
        let start_x = (canvas_min.x / spacing_h).floor() * spacing_h;
        let start_y = (canvas_min.y / spacing_v).floor() * spacing_v;

        debug!(
            start_x = start_x,
            start_y = start_y,
            spacing_h = spacing_h,
            spacing_v = spacing_v,
            "Starting grid positions"
        );

        // Rotation center for the grid (canvas origin)
        let grid_center = Pos2::ZERO;

        // Draw vertical lines (spaced horizontally)
        let mut x = start_x;
        let mut vertical_count = 0;
        while x <= canvas_max.x {
            // Create line endpoints in world coordinates
            let top = Pos2::new(x, canvas_min.y);
            let bottom = Pos2::new(x, canvas_max.y);

            // Apply rotation around grid center
            let rotated_top = Self::rotate_point(top, grid_center, self.grid_rotation_angle);
            let rotated_bottom = Self::rotate_point(bottom, grid_center, self.grid_rotation_angle);

            // Apply zoom/pan transform
            let screen_x_top = transform.mul_pos(rotated_top);
            let screen_x_bottom = transform.mul_pos(rotated_bottom);

            trace!(
                x = x,
                screen_top = ?screen_x_top,
                screen_bottom = ?screen_x_bottom,
                rotation_angle = self.grid_rotation_angle,
                "Drawing vertical grid line"
            );
            painter.line_segment([screen_x_top, screen_x_bottom], grid_stroke);
            x += spacing_h;
            vertical_count += 1;
        }

        // Draw horizontal lines (spaced vertically)
        let mut y = start_y;
        let mut horizontal_count = 0;
        while y <= canvas_max.y {
            // Create line endpoints in world coordinates
            let left = Pos2::new(canvas_min.x, y);
            let right = Pos2::new(canvas_max.x, y);

            // Apply rotation around grid center
            let rotated_left = Self::rotate_point(left, grid_center, self.grid_rotation_angle);
            let rotated_right = Self::rotate_point(right, grid_center, self.grid_rotation_angle);

            // Apply zoom/pan transform
            let screen_y_left = transform.mul_pos(rotated_left);
            let screen_y_right = transform.mul_pos(rotated_right);

            trace!(
                y = y,
                screen_left = ?screen_y_left,
                screen_right = ?screen_y_right,
                rotation_angle = self.grid_rotation_angle,
                "Drawing horizontal grid line"
            );
            painter.line_segment([screen_y_left, screen_y_right], grid_stroke);
            y += spacing_v;
            horizontal_count += 1;
        }

        debug!(
            vertical_lines = vertical_count,
            horizontal_lines = horizontal_count,
            "Grid render complete"
        );
    }

    /// Render a shape with zoom transformation applied
    fn render_shape_transformed(&self, shape: &Shape, painter: &egui::Painter, transform: &egui::emath::TSTransform) {
        match shape {
            Shape::Rectangle(rect) => {
                // Note: rotation removed from shapes
                // let center = self.get_shape_center(shape);

                // Apply rotation then zoom/pan transform
                let transformed_corners: Vec<Pos2> = rect.corners()
                    .iter()
                    .map(|p| {
                        // Note: rotation_angle removed - shapes no longer have implicit rotation
                        // Apply zoom/pan transform
                        transform.mul_pos(*p)
                    })
                    .collect();

                // Draw filled quadrilateral
                painter.add(egui::Shape::convex_polygon(
                    transformed_corners.clone(),
                    rect.fill,
                    egui::Stroke::NONE,
                ));
                // Draw outline
                painter.add(egui::Shape::closed_line(
                    transformed_corners,
                    rect.stroke,
                ));
            }
            Shape::Circle(circle) => {
                // Note: rotation_angle removed - circles are symmetric anyway
                let transformed_center = transform.mul_pos(circle.center);
                let transformed_radius = circle.radius * self.zoom_level;
                painter.circle(transformed_center, transformed_radius, circle.fill, circle.stroke);
            }
            Shape::Polygon(poly) => {
                // Note: rotation removed from shapes
                // let center = self.get_shape_center(shape);

                let points: Vec<Pos2> = poly.to_egui_points()
                    .iter()
                    .map(|p| {
                        // Note: rotation_angle removed - apply zoom/pan transform only
                        transform.mul_pos(*p)
                    })
                    .collect();

                if points.len() > 2 {
                    // Draw filled polygon
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        poly.fill,
                        egui::Stroke::NONE,
                    ));
                    // Draw outline
                    painter.add(egui::Shape::closed_line(points, poly.stroke));
                }
            }
        }
    }

    /// Rotate a point around a center by the given angle (in radians)
    fn rotate_point(point: Pos2, center: Pos2, angle: f32) -> Pos2 {
        if angle == 0.0 {
            return point;
        }

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Translate to origin
        let dx = point.x - center.x;
        let dy = point.y - center.y;

        // Rotate
        let rotated_x = dx * cos_a - dy * sin_a;
        let rotated_y = dx * sin_a + dy * cos_a;

        // Translate back
        Pos2::new(center.x + rotated_x, center.y + rotated_y)
    }

    /// Map a detection shape from image pixel coordinates to canvas coordinates
    /// Detections are stored in image pixel space (e.g., 0-3400 x 0-4400),
    /// but need to be converted to canvas space where the image is scaled and centered
    fn map_detection_to_canvas(&self, detection: &Shape, scale: f32, image_offset: Pos2) -> Shape {
        match detection {
            Shape::Rectangle(rect) => {
                let mapped_corners: Vec<Pos2> = rect.corners()
                    .iter()
                    .map(|p| {
                        // Scale from image pixels to fitted canvas size, then offset
                        Pos2::new(
                            p.x * scale + image_offset.x,
                            p.y * scale + image_offset.y,
                        )
                    })
                    .collect();

                // Rectangle now uses from_four_corners constructor
                Rectangle::from_four_corners(
                    [mapped_corners[0], mapped_corners[1], mapped_corners[2], mapped_corners[3]],
                    rect.stroke,
                    rect.fill,
                )
                .map(|mut r| {
                    r.name = rect.name.clone();
                    Shape::Rectangle(r)
                })
                .unwrap_or_else(|e| {
                    warn!("Failed to map rectangle during serialization: {}", e);
                    // Fallback to original rectangle (unscaled)
                    Shape::Rectangle(rect.clone())
                })
            }
            Shape::Circle(circle) => {
                let mapped_center = Pos2::new(
                    circle.center.x * scale + image_offset.x,
                    circle.center.y * scale + image_offset.y,
                );
                let mapped_radius = circle.radius * scale;

                // Circle::new returns Result, but we're mapping from existing valid circle
                // so this should not fail unless coordinates became invalid during transformation
                Circle::new(mapped_center, mapped_radius, circle.stroke, circle.fill)
                    .map(|mut c| {
                        c.name = circle.name.clone();
                        Shape::Circle(c)
                    })
                    .unwrap_or_else(|e| {
                        warn!("Failed to map circle during serialization: {}", e);
                        // Fallback to original circle (unscaled)
                        Shape::Circle(circle.clone())
                    })
            }
            Shape::Polygon(poly) => {
                // Map polygon points from image space to canvas space
                let image_points = poly.to_egui_points();
                let mapped_points: Vec<Pos2> = image_points
                    .iter()
                    .map(|p| {
                        Pos2::new(
                            p.x * scale + image_offset.x,
                            p.y * scale + image_offset.y,
                        )
                    })
                    .collect();

                // Convert back to geo coordinates for storage (geo uses f64)
                let _geo_points: Vec<geo::Coord<f64>> = mapped_points
                    .iter()
                    .map(|p| geo::Coord { x: p.x as f64, y: p.y as f64 })
                    .collect();

                // Use from_points constructor
                PolygonShape::from_points(mapped_points, poly.stroke, poly.fill)
                    .map(|mut p| {
                        p.name = poly.name.clone();
                        Shape::Polygon(p)
                    })
                    .unwrap_or_else(|e| {
                        warn!("Failed to map polygon: {}", e);
                        Shape::Polygon(poly.clone())
                    })
            }
        }
    }

    /// Draw edit vertices with zoom transformation applied
    fn draw_edit_vertices_transformed(&self, shape: &Shape, painter: &egui::Painter, transform: &egui::emath::TSTransform) {
        const VERTEX_SIZE: f32 = 6.0;
        let vertex_stroke = Stroke::new(2.0, Color32::from_rgb(0, 120, 215));
        let vertex_fill = Color32::from_rgb(255, 255, 255);

        match shape {
            Shape::Rectangle(rect) => {
                // Draw control points at all 4 corners
                for corner in rect.corners() {
                    let transformed_corner = transform.mul_pos(*corner);
                    painter.rect_filled(
                        egui::Rect::from_center_size(transformed_corner, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                        0.0,
                        vertex_fill,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(transformed_corner, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                        0.0,
                        vertex_stroke,
                        egui::StrokeKind::Outside,
                    );
                }
            }
            Shape::Circle(circle) => {
                let transformed_center = transform.mul_pos(circle.center);

                // Draw control point at center
                painter.rect_filled(
                    egui::Rect::from_center_size(transformed_center, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                    0.0,
                    vertex_fill,
                );
                painter.rect_stroke(
                    egui::Rect::from_center_size(transformed_center, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                    0.0,
                    vertex_stroke,
                    egui::StrokeKind::Outside,
                );

                // Draw control point on edge
                let edge_point = egui::pos2(circle.center.x + circle.radius, circle.center.y);
                let transformed_edge = transform.mul_pos(edge_point);
                painter.rect_filled(
                    egui::Rect::from_center_size(transformed_edge, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                    0.0,
                    vertex_fill,
                );
                painter.rect_stroke(
                    egui::Rect::from_center_size(transformed_edge, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                    0.0,
                    vertex_stroke,
                    egui::StrokeKind::Outside,
                );
            }
            Shape::Polygon(poly) => {
                for vertex_pos in poly.to_egui_points() {
                    let transformed_vertex = transform.mul_pos(vertex_pos);
                    painter.rect_filled(
                        egui::Rect::from_center_size(transformed_vertex, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                        0.0,
                        vertex_fill,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(transformed_vertex, egui::vec2(VERTEX_SIZE, VERTEX_SIZE)),
                        0.0,
                        vertex_stroke,
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
    }
}
