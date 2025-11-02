//! Drawing canvas with interactive annotation tools

use crate::drawing::{Circle, LayerManager, LayerType, PolygonShape, Rectangle, Shape, ToolMode};
use egui::{Color32, Pos2, Stroke};
use geo::CoordsIter;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

/// Drawing canvas state
#[derive(Clone, Serialize, Deserialize)]
pub struct DrawingCanvas {
    /// All completed shapes
    pub shapes: Vec<Shape>,
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

    // Edit mode vertex dragging state (not serialized)
    #[serde(skip)]
    dragging_vertex: Option<usize>,
    #[serde(skip)]
    is_dragging_vertex: bool,

    // Form image state (not serialized)
    #[serde(skip)]
    form_image: Option<egui::TextureHandle>,
    #[serde(skip)]
    form_image_size: Option<egui::Vec2>,

    // Zoom and pan state (not serialized)
    #[serde(skip)]
    zoom_level: f32,
    #[serde(skip)]
    pan_offset: egui::Vec2,

    // Settings state (not serialized)
    #[serde(skip)]
    show_settings: bool,
    #[serde(skip)]
    zoom_sensitivity: f32,
    #[serde(skip)]
    grid_spacing_horizontal: f32,
    #[serde(skip)]
    grid_spacing_vertical: f32,

    // Style settings
    pub stroke: Stroke,
    pub fill_color: Color32,
}

impl Default for DrawingCanvas {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
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
            dragging_vertex: None,
            is_dragging_vertex: false,
            form_image: None,
            form_image_size: None,
            zoom_level: 5.0,
            pan_offset: egui::Vec2::ZERO,
            show_settings: false,
            zoom_sensitivity: 1.0,
            grid_spacing_horizontal: 10.0,
            grid_spacing_vertical: 10.0,
            stroke: Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
            fill_color: Color32::from_rgba_premultiplied(0, 120, 215, 30),
        }
    }
}

impl std::fmt::Debug for DrawingCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawingCanvas")
            .field("shapes", &self.shapes)
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
        // Tool selection toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tool, ToolMode::Select, "✋ Select");
            ui.selectable_value(&mut self.current_tool, ToolMode::Rectangle, "▭ Rectangle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Circle, "◯ Circle");
            ui.selectable_value(&mut self.current_tool, ToolMode::Freehand, "✏ Freehand");
            ui.selectable_value(&mut self.current_tool, ToolMode::Edit, "✎ Edit");
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

            // Transform the image rect for zoom
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
                            let transformed_corners: Vec<Pos2> = rect.corners
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
        }
    }

    fn finalize_shape(&mut self) {
        let shape = match self.current_tool {
            ToolMode::Rectangle => {
                if let (Some(start), Some(end)) = (self.drawing_start, self.current_end) {
                    Some(Shape::Rectangle(Rectangle::from_corners(
                        start,
                        end,
                        self.stroke,
                        self.fill_color,
                    )))
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
            ToolMode::Edit => None,
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
                let min_x = rect.corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = rect.corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = rect.corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = rect.corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
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

                ui.label(format!("Points: {}", poly.polygon.exterior().coords_count()));
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
                    let min_x = rect.corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                    let max_x = rect.corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                    let min_y = rect.corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                    let max_y = rect.corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
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

    /// Show settings inline in the side panel
    pub fn show_inline_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();

        ui.label("Zoom Sensitivity:");
        ui.add(
            egui::Slider::new(&mut self.zoom_sensitivity, 0.1..=5.0)
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
                    egui::Slider::new(&mut self.zoom_sensitivity, 0.1..=5.0)
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
                rect.corners
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
            Shape::Rectangle(rect) => {
                // Update the specific corner
                if vertex_idx < 4 {
                    rect.corners[vertex_idx] = pos;
                }
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
            Shape::Polygon(poly) => {
                // Update the polygon vertex
                let mut coords: Vec<geo_types::Coord<f64>> = poly.polygon
                    .exterior()
                    .coords()
                    .copied()
                    .collect();

                if vertex_idx < coords.len() {
                    coords[vertex_idx] = geo_types::Coord {
                        x: pos.x as f64,
                        y: pos.y as f64,
                    };

                    // Reconstruct the polygon with updated coordinates
                    poly.polygon = geo_types::Polygon::new(coords.into(), vec![]);
                }
            }
        }
    }

    /// Finish dragging a vertex
    fn finish_vertex_drag(&mut self) {
        debug!("Finishing vertex drag");
        self.dragging_vertex = None;
        self.is_dragging_vertex = false;
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

        // Draw vertical lines (spaced horizontally)
        let mut x = start_x;
        let mut vertical_count = 0;
        while x <= canvas_max.x {
            let screen_x_top = transform.mul_pos(Pos2::new(x, canvas_min.y));
            let screen_x_bottom = transform.mul_pos(Pos2::new(x, canvas_max.y));
            trace!(
                x = x,
                screen_top = ?screen_x_top,
                screen_bottom = ?screen_x_bottom,
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
            let screen_y_left = transform.mul_pos(Pos2::new(canvas_min.x, y));
            let screen_y_right = transform.mul_pos(Pos2::new(canvas_max.x, y));
            trace!(
                y = y,
                screen_left = ?screen_y_left,
                screen_right = ?screen_y_right,
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
                let transformed_corners: Vec<Pos2> = rect.corners
                    .iter()
                    .map(|p| transform.mul_pos(*p))
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
                let transformed_center = transform.mul_pos(circle.center);
                let transformed_radius = circle.radius * self.zoom_level;
                painter.circle(transformed_center, transformed_radius, circle.fill, circle.stroke);
            }
            Shape::Polygon(poly) => {
                let points: Vec<Pos2> = poly.to_egui_points()
                    .iter()
                    .map(|p| transform.mul_pos(*p))
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

    /// Draw edit vertices with zoom transformation applied
    fn draw_edit_vertices_transformed(&self, shape: &Shape, painter: &egui::Painter, transform: &egui::emath::TSTransform) {
        const VERTEX_SIZE: f32 = 6.0;
        let vertex_stroke = Stroke::new(2.0, Color32::from_rgb(0, 120, 215));
        let vertex_fill = Color32::from_rgb(255, 255, 255);

        match shape {
            Shape::Rectangle(rect) => {
                // Draw control points at all 4 corners
                for corner in &rect.corners {
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
