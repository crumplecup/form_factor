//! Rendering and UI methods for the drawing canvas
//!
//! This module contains all rendering, painting, and UI interaction code for the DrawingCanvas.
//! It handles:
//! - Main UI update loop with toolbar and canvas rendering
//! - Shape and detection rendering with zoom/pan transformations
//! - Grid overlay rendering with rotation support
//! - Form image rendering with rotation support
//! - Property panels and settings UI
//! - Vertex editing handles
//! - Coordinate transformation utilities
//! - Template field overlay rendering

use super::core::DrawingCanvas;
use crate::{LayerType, Shape, ToolMode};
use egui::{Color32, Pos2, Stroke};
use form_factor_core::FieldDefinition;
use geo::CoordsIter;
use tracing::{debug, trace, warn};

impl DrawingCanvas {
    /// Render the canvas UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Log state at frame start (only when we have detections to avoid spam)
        if !self.detections.is_empty() {
            trace!(
                "Frame start: detections={}, shapes={}",
                self.detections.len(),
                self.shapes.len()
            );
        }

        // Process any pending image loads (deferred from startup)
        if let Some(pending_path) = self.pending_image_load.take() {
            tracing::debug!("Processing deferred image load: {}", pending_path);
            if let Err(e) = self.load_form_image(&pending_path, ui.ctx()) {
                tracing::warn!(
                    "Could not load deferred form image from {}: {}",
                    pending_path,
                    e
                );
            }
        }

        // Canvas area
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

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
                
                // Delete key for shapes and fields
                if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                    self.handle_delete_key();
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
        if self.layer_manager.is_visible(LayerType::Canvas) {
            painter.rect_filled(response.rect, 0.0, Color32::from_rgb(245, 245, 245));
        }

        // Apply zoom transformation to a child painter
        let canvas_center = response.rect.center();
        let to_screen =
            egui::emath::TSTransform::from_translation(canvas_center.to_vec2() + self.pan_offset)
                * egui::emath::TSTransform::from_scaling(self.zoom_level)
                * egui::emath::TSTransform::from_translation(-canvas_center.to_vec2());

        // Draw form image on Canvas layer if loaded
        if self.layer_manager.is_visible(LayerType::Canvas)
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
                        let rotated =
                            Self::rotate_point(corner, image_center, self.form_image_rotation);
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
        let detections_visible = self.layer_manager.is_visible(LayerType::Detections);
        if !self.detections.is_empty() {
            debug!(
                "Rendering frame: detections={}, layer_visible={}, image_size={:?}, canvas_size={:?}",
                self.detections.len(),
                detections_visible,
                self.form_image_size,
                response.rect.size()
            );
        }
        if detections_visible
            && let (Some(image_size), Some(_texture)) = (self.form_image_size, &self.form_image)
        {
            // Calculate the image-to-canvas coordinate transform
            let canvas_size = response.rect.size();
            let scale_x = canvas_size.x / image_size.x;
            let scale_y = canvas_size.y / image_size.y;
            let scale = scale_x.min(scale_y);
            let fitted_size = image_size * scale;
            let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
            let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
            let image_offset = response.rect.min + egui::vec2(offset_x, offset_y);

            debug!(
                "Image transform: scale={:.3}, offset=({:.1}, {:.1})",
                scale, offset_x, offset_y
            );

            for (idx, detection) in self.detections.iter().enumerate() {
                trace!(
                    "Rendering detection {}/{}: {:?}",
                    idx + 1,
                    self.detections.len(),
                    detection
                );

                // Convert detection from image pixel coordinates to canvas coordinates
                let detection_in_canvas_space =
                    self.map_detection_to_canvas(detection, scale, image_offset);
                self.render_shape_transformed(&detection_in_canvas_space, &painter, &to_screen);
            }
        } else if detections_visible && !self.detections.is_empty() {
            debug!(
                "Detections layer visible but image not loaded: {} detections not rendered",
                self.detections.len()
            );
        } else if !self.detections.is_empty() {
            debug!(
                "Detections layer hidden: {} detections not rendered",
                self.detections.len()
            );
        }

        // Draw existing shapes if Shapes layer is visible (with zoom transformation)
        // Note: Like detections, shapes are stored in image pixel coordinates and need to be mapped to canvas space
        let shapes_visible = self.layer_manager.is_visible(LayerType::Shapes);
        if shapes_visible
            && let (Some(image_size), Some(_texture)) = (self.form_image_size, &self.form_image)
        {
            // Calculate the image-to-canvas coordinate transform (same as for detections)
            let canvas_size = response.rect.size();
            let scale_x = canvas_size.x / image_size.x;
            let scale_y = canvas_size.y / image_size.y;
            let scale = scale_x.min(scale_y);
            let fitted_size = image_size * scale;
            let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
            let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
            let image_offset = response.rect.min + egui::vec2(offset_x, offset_y);

            for (idx, shape) in self.shapes.iter().enumerate() {
                // Convert shape from image pixel coordinates to canvas coordinates
                let shape_in_canvas_space =
                    self.map_detection_to_canvas(shape, scale, image_offset);
                self.render_shape_transformed(&shape_in_canvas_space, &painter, &to_screen);

                // Draw selection highlight
                if Some(idx) == self.selected_shape {
                    let highlight_stroke = Stroke::new(4.0, Color32::from_rgb(255, 215, 0));

                    match &shape_in_canvas_space {
                        Shape::Rectangle(rect) => {
                            let transformed_corners: Vec<Pos2> = rect
                                .corners()
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
                            painter.circle_stroke(
                                transformed_center,
                                transformed_radius,
                                highlight_stroke,
                            );
                        }
                        Shape::Polygon(poly) => {
                            let points: Vec<Pos2> = poly
                                .to_egui_points()
                                .iter()
                                .map(|p| to_screen.mul_pos(*p))
                                .collect();
                            painter.add(egui::Shape::closed_line(points, highlight_stroke));
                        }
                    }

                    // Draw edit vertices if in Edit mode
                    if self.current_tool == ToolMode::Edit && Some(idx) == self.selected_shape {
                        self.draw_edit_vertices_transformed(
                            &shape_in_canvas_space,
                            &painter,
                            &to_screen,
                        );
                    }
                }
            }
        } else if shapes_visible && !self.shapes.is_empty() {
            debug!(
                "Shapes layer visible but image not loaded: {} shapes not rendered",
                self.shapes.len()
            );
        }

        // Draw template fields if Template layer is visible
        let template_visible = self.layer_manager.is_visible(LayerType::Template);
        if template_visible
            && let (Some(image_size), Some(_texture)) = (self.form_image_size, &self.form_image)
            && let Some(template) = self.current_template()
        {
            // Calculate the same transform as for shapes
            let canvas_size = response.rect.size();
            let scale_x = canvas_size.x / image_size.x;
            let scale_y = canvas_size.y / image_size.y;
            let scale = scale_x.min(scale_y);
            let fitted_size = image_size * scale;
            let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
            let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
            let image_offset = response.rect.min + egui::vec2(offset_x, offset_y);

            // Render all fields from all pages
            for (_page_idx, page) in template.pages.iter().enumerate() {
                for (idx, field) in page.fields.iter().enumerate() {
                    self.render_template_field(
                        field,
                        idx,
                        scale,
                        image_offset,
                        &painter,
                        &to_screen,
                    );
                }
            }
        }

        // Draw grid on top of everything if Grid layer is visible
        if self.layer_manager.is_visible(LayerType::Grid) {
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
                    let text_edit =
                        egui::TextEdit::singleline(&mut rect.name).id_salt("rectangle_name");
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
                let min_x = rect
                    .corners()
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::INFINITY, f32::min);
                let max_x = rect
                    .corners()
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::NEG_INFINITY, f32::max);
                let min_y = rect
                    .corners()
                    .iter()
                    .map(|p| p.y)
                    .fold(f32::INFINITY, f32::min);
                let max_y = rect
                    .corners()
                    .iter()
                    .map(|p| p.y)
                    .fold(f32::NEG_INFINITY, f32::max);
                let rect_geom =
                    egui::Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
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
                    let text_edit =
                        egui::TextEdit::singleline(&mut circle.name).id_salt("circle_name");
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
                ui.label(format!(
                    "Center: ({:.1}, {:.1})",
                    circle.center.x, circle.center.y
                ));
            }
            Shape::Polygon(poly) => {
                ui.label("Type: Polygon");
                ui.separator();

                ui.horizontal(|ui| {
                    let _span = tracing::debug_span!("shape_name_autofocus").entered();

                    ui.label("Name:");

                    // Create text edit with explicit ID for focusing
                    let text_edit =
                        egui::TextEdit::singleline(&mut poly.name).id_salt("polygon_name");
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

                ui.label(format!(
                    "Points: {}",
                    poly.polygon().exterior().coords_count()
                ));
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
                    let min_x = rect
                        .corners()
                        .iter()
                        .map(|p| p.x)
                        .fold(f32::INFINITY, f32::min);
                    let max_x = rect
                        .corners()
                        .iter()
                        .map(|p| p.x)
                        .fold(f32::NEG_INFINITY, f32::max);
                    let min_y = rect
                        .corners()
                        .iter()
                        .map(|p| p.y)
                        .fold(f32::INFINITY, f32::min);
                    let max_y = rect
                        .corners()
                        .iter()
                        .map(|p| p.y)
                        .fold(f32::NEG_INFINITY, f32::max);
                    let rect_geom =
                        egui::Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
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

                    ui.label(format!(
                        "Points: {}",
                        poly.polygon().exterior().coords_count()
                    ));

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
                .logarithmic(true),
        );
        ui.label("Higher values make zoom more responsive");

        ui.separator();

        ui.label("Grid Spacing (Horizontal):");
        ui.add(
            egui::Slider::new(&mut self.grid_spacing_horizontal, 0.1..=100.0)
                .text("Horizontal")
                .logarithmic(true),
        );

        ui.label("Grid Spacing (Vertical):");
        ui.add(
            egui::Slider::new(&mut self.grid_spacing_vertical, 0.1..=100.0)
                .text("Vertical")
                .logarithmic(true),
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
                        .logarithmic(true),
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

    /// Draw grid overlay on the canvas
    fn draw_grid(
        &self,
        painter: &egui::Painter,
        canvas_rect: &egui::Rect,
        transform: &egui::emath::TSTransform,
    ) {
        let _span = tracing::debug_span!(
            "draw_grid",
            spacing_h = self.grid_spacing_horizontal,
            spacing_v = self.grid_spacing_vertical
        )
        .entered();

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
    fn render_shape_transformed(
        &self,
        shape: &Shape,
        painter: &egui::Painter,
        transform: &egui::emath::TSTransform,
    ) {
        match shape {
            Shape::Rectangle(rect) => {
                // Note: rotation removed from shapes
                // let center = self.get_shape_center(shape);

                // Apply rotation then zoom/pan transform
                let transformed_corners: Vec<Pos2> = rect
                    .corners()
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
                painter.add(egui::Shape::closed_line(transformed_corners, rect.stroke));
            }
            Shape::Circle(circle) => {
                // Note: rotation_angle removed - circles are symmetric anyway
                let transformed_center = transform.mul_pos(circle.center);
                let transformed_radius = circle.radius * self.zoom_level;
                painter.circle(
                    transformed_center,
                    transformed_radius,
                    circle.fill,
                    circle.stroke,
                );
            }
            Shape::Polygon(poly) => {
                // Note: rotation removed from shapes
                // let center = self.get_shape_center(shape);

                let points: Vec<Pos2> = poly
                    .to_egui_points()
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

    /// Render a template field with zoom transformation applied
    fn render_template_field(
        &self,
        field: &FieldDefinition,
        field_idx: usize,
        scale: f32,
        image_offset: Pos2,
        painter: &egui::Painter,
        transform: &egui::emath::TSTransform,
    ) {
        // Convert field bounds from image coordinates to canvas coordinates
        let x = field.bounds.x * scale + image_offset.x;
        let y = field.bounds.y * scale + image_offset.y;
        let width = field.bounds.width * scale;
        let height = field.bounds.height * scale;

        // Create rectangle corners in canvas space
        let top_left = Pos2::new(x, y);
        let top_right = Pos2::new(x + width, y);
        let bottom_right = Pos2::new(x + width, y + height);
        let bottom_left = Pos2::new(x, y + height);

        // Apply zoom/pan transform
        let transformed_corners: Vec<Pos2> = vec![top_left, top_right, bottom_right, bottom_left]
            .iter()
            .map(|p| transform.mul_pos(*p))
            .collect();

        // Draw field rectangle
        let is_selected = self.selected_field() == &Some(field_idx);
        let stroke = if is_selected {
            Stroke::new(3.0, Color32::from_rgb(0, 150, 255)) // Blue for selected
        } else {
            Stroke::new(2.0, Color32::from_rgb(100, 100, 255)) // Lighter blue for unselected
        };
        
        let fill = if is_selected {
            Color32::from_rgba_premultiplied(0, 150, 255, 30) // Translucent blue
        } else {
            Color32::from_rgba_premultiplied(100, 100, 255, 15) // More translucent
        };

        // Draw filled field
        painter.add(egui::Shape::convex_polygon(
            transformed_corners.clone(),
            fill,
            egui::Stroke::NONE,
        ));
        
        // Draw outline
        painter.add(egui::Shape::closed_line(transformed_corners.clone(), stroke));

        // Draw field label
        if let Some(center_pos) = transformed_corners.get(0) {
            let label_pos = Pos2::new(center_pos.x + 5.0, center_pos.y + 5.0);
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                &field.label,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 100, 200),
            );
        }
    }

    /// Draw edit vertices with zoom transformation applied
    fn draw_edit_vertices_transformed(
        &self,
        shape: &Shape,
        painter: &egui::Painter,
        transform: &egui::emath::TSTransform,
    ) {
        const VERTEX_SIZE: f32 = 6.0;
        let vertex_stroke = Stroke::new(2.0, Color32::from_rgb(0, 120, 215));
        let vertex_fill = Color32::from_rgb(255, 255, 255);

        match shape {
            Shape::Rectangle(rect) => {
                // Draw control points at all 4 corners
                for corner in rect.corners() {
                    let transformed_corner = transform.mul_pos(*corner);
                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            transformed_corner,
                            egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                        ),
                        0.0,
                        vertex_fill,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(
                            transformed_corner,
                            egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                        ),
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
                    egui::Rect::from_center_size(
                        transformed_center,
                        egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                    ),
                    0.0,
                    vertex_fill,
                );
                painter.rect_stroke(
                    egui::Rect::from_center_size(
                        transformed_center,
                        egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                    ),
                    0.0,
                    vertex_stroke,
                    egui::StrokeKind::Outside,
                );

                // Draw control point on edge
                let edge_point = egui::pos2(circle.center.x + circle.radius, circle.center.y);
                let transformed_edge = transform.mul_pos(edge_point);
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        transformed_edge,
                        egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                    ),
                    0.0,
                    vertex_fill,
                );
                painter.rect_stroke(
                    egui::Rect::from_center_size(
                        transformed_edge,
                        egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                    ),
                    0.0,
                    vertex_stroke,
                    egui::StrokeKind::Outside,
                );
            }
            Shape::Polygon(poly) => {
                for vertex_pos in poly.to_egui_points() {
                    let transformed_vertex = transform.mul_pos(vertex_pos);
                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            transformed_vertex,
                            egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                        ),
                        0.0,
                        vertex_fill,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_center_size(
                            transformed_vertex,
                            egui::vec2(VERTEX_SIZE, VERTEX_SIZE),
                        ),
                        0.0,
                        vertex_stroke,
                        egui::StrokeKind::Outside,
                    );
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
    pub(super) fn map_detection_to_canvas(
        &self,
        detection: &Shape,
        scale: f32,
        image_offset: Pos2,
    ) -> Shape {
        use crate::{Circle, PolygonShape, Rectangle};

        match detection {
            Shape::Rectangle(rect) => {
                let mapped_corners: Vec<Pos2> = rect
                    .corners()
                    .iter()
                    .map(|p| {
                        // Scale from image pixels to fitted canvas size, then offset
                        Pos2::new(p.x * scale + image_offset.x, p.y * scale + image_offset.y)
                    })
                    .collect();

                // Rectangle now uses from_four_corners constructor
                Rectangle::from_four_corners(
                    [
                        mapped_corners[0],
                        mapped_corners[1],
                        mapped_corners[2],
                        mapped_corners[3],
                    ],
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
                    .map(|p| Pos2::new(p.x * scale + image_offset.x, p.y * scale + image_offset.y))
                    .collect();

                // Convert back to geo coordinates for storage (geo uses f64)
                let _geo_points: Vec<geo::Coord<f64>> = mapped_points
                    .iter()
                    .map(|p| geo::Coord {
                        x: p.x as f64,
                        y: p.y as f64,
                    })
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

    /// Render template field overlays
    ///
    /// Renders field bounds as semi-transparent rectangles with labels.
    /// Fields are rendered in image coordinates and transformed to canvas space.
    ///
    /// # Arguments
    ///
    /// * `fields` - Field definitions to render
    /// * `painter` - egui painter for rendering
    /// * `canvas_rect` - Canvas rectangle for coordinate transformation
    /// * `transform` - Zoom/pan transformation
    pub fn render_field_overlays(
        &self,
        fields: &[FieldDefinition],
        painter: &egui::Painter,
        canvas_rect: egui::Rect,
        transform: &egui::emath::TSTransform,
    ) {
        // Only render if we have an image loaded (fields are in image coordinates)
        let Some(image_size) = self.form_image_size else {
            return;
        };

        // Calculate the image-to-canvas coordinate transform (same as detections/shapes)
        let canvas_size = canvas_rect.size();
        let scale_x = canvas_size.x / image_size.x;
        let scale_y = canvas_size.y / image_size.y;
        let scale = scale_x.min(scale_y);
        let fitted_size = image_size * scale;
        let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
        let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
        let image_offset = canvas_rect.min + egui::vec2(offset_x, offset_y);

        // Render each field
        for field in fields {
            let bounds = &field.bounds;

            // Convert field bounds from image coordinates to canvas coordinates
            let min = Pos2::new(
                bounds.x * scale + image_offset.x,
                bounds.y * scale + image_offset.y,
            );
            let max = Pos2::new(
                (bounds.x + bounds.width) * scale + image_offset.x,
                (bounds.y + bounds.height) * scale + image_offset.y,
            );

            // Apply zoom/pan transformation
            let transformed_min = transform.mul_pos(min);
            let transformed_max = transform.mul_pos(max);
            let field_rect = egui::Rect::from_min_max(transformed_min, transformed_max);

            // Draw field bounds as semi-transparent rectangle
            let field_color = Color32::from_rgba_premultiplied(100, 200, 100, 50);
            let field_stroke = Stroke::new(2.0, Color32::from_rgb(50, 150, 50));
            painter.rect(
                field_rect,
                0.0,
                field_color,
                field_stroke,
                egui::StrokeKind::Outside,
            );

            // Draw field label
            let label_pos = transformed_min + egui::vec2(4.0, 2.0);
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                &field.label,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(20, 80, 20),
            );
        }
    }

    /// Handle delete key press for shapes and fields
    fn handle_delete_key(&mut self) {
        // Check if we're in template mode - delete field
        if matches!(
            self.template_mode(),
            super::core::TemplateMode::Creating | super::core::TemplateMode::Editing
        ) {
            if let Some(field_idx) = *self.selected_field() {
                let current_page = *self.current_page();
                if let Some(template) = self.current_template_mut() {
                    if let Some(page) = template.pages.get_mut(current_page) {
                        if field_idx < page.fields.len() {
                            let field_id = page.fields[field_idx].id.clone();
                            page.fields.remove(field_idx);
                            self.set_selected_field(None);
                            self.set_show_properties(false);
                            debug!(field_id, field_idx, "Deleted field");
                        }
                    }
                }
            }
        } else {
            // Delete shape
            if let Some(idx) = *self.selected_shape() {
                if idx < self.shapes().len() {
                    let shape_name = match &self.shapes()[idx] {
                        crate::Shape::Rectangle(r) => r.name().to_string(),
                        crate::Shape::Circle(c) => c.name().to_string(),
                        crate::Shape::Polygon(p) => p.name().to_string(),
                    };
                    self.shapes.remove(idx);
                    self.set_selected_shape(None);
                    self.set_show_properties(false);
                    debug!(shape_name, shape_idx = idx, "Deleted shape");
                }
            }
        }
    }
}
