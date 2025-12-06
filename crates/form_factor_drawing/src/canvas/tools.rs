//! Tool interaction and state management for the drawing canvas
//!
//! This module handles all user interactions with the canvas tools:
//! - Selection: Clicking on shapes to select them
//! - Drawing: Creating new shapes (rectangles, circles, polygons)
//! - Editing: Dragging vertices to modify shapes
//! - Rotation: Rotating shapes, grid, or form image
//!
//! The interaction state machine prevents invalid state combinations
//! (e.g., drawing while rotating) and ensures consistent behavior.

use crate::{Circle, LayerType, PolygonShape, Rectangle, Shape, ToolMode};
use egui::Pos2;
use form_factor_core::{FieldBounds, FieldDefinition};
use tracing::{debug, instrument, trace, warn};

use super::core::DrawingCanvas;

impl DrawingCanvas {
    /// Handle input events for the current tool mode
    ///
    /// This is the main input dispatcher that delegates to specific handlers
    /// based on the current tool mode and interaction state.
    #[instrument(skip(self, response, painter, transform), fields(tool = ?self.current_tool()))]
    pub(super) fn handle_input(
        &mut self,
        response: &egui::Response,
        painter: &egui::Painter,
        transform: &egui::emath::TSTransform,
    ) {
        // Pre-compute transformation parameters to avoid borrow checker issues
        let canvas_rect = response.rect;
        let image_transform_params =
            if let (Some(image_size), Some(_)) = (self.form_image_size, &self.form_image) {
                let canvas_size = canvas_rect.size();
                let scale_x = canvas_size.x / image_size.x;
                let scale_y = canvas_size.y / image_size.y;
                let scale = scale_x.min(scale_y);
                let fitted_size = image_size * scale;
                let offset_x = (canvas_size.x - fitted_size.x) / 2.0;
                let offset_y = (canvas_size.y - fitted_size.y) / 2.0;
                let image_offset = canvas_rect.min + egui::vec2(offset_x, offset_y);
                Some((scale, image_offset))
            } else {
                None
            };

        // Helper to transform screen coordinates to canvas coordinates, then to image pixel coordinates
        // This ensures shapes are stored in the same coordinate system as detections
        let transform_pos = |screen_pos: Pos2| -> Pos2 {
            let canvas_pos = transform.inverse().mul_pos(screen_pos);
            // Convert to image pixel coordinates if image is loaded
            if let Some((scale, image_offset)) = image_transform_params {
                let image_x = (canvas_pos.x - image_offset.x) / scale;
                let image_y = (canvas_pos.y - image_offset.y) / scale;
                Pos2::new(image_x, image_y)
            } else {
                canvas_pos
            }
        };
        match self.current_tool() {
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

                // Handle field dragging in template mode
                if matches!(
                    self.template_mode(),
                    super::core::TemplateMode::Creating | super::core::TemplateMode::Editing
                ) {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = transform_pos(pos);
                        if response.drag_started() {
                            self.start_field_drag(canvas_pos);
                        } else if response.dragged()
                            && matches!(
                                self.state(),
                                super::core::CanvasState::DraggingField { .. }
                            )
                        {
                            self.continue_field_drag(canvas_pos);
                        }
                    }

                    // Check if drag ended
                    if response.drag_stopped()
                        && matches!(self.state(), super::core::CanvasState::DraggingField { .. })
                    {
                        self.finish_field_drag();
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
                    } else if response.dragged()
                        && matches!(
                            self.state(),
                            super::core::CanvasState::DraggingVertex { .. }
                        )
                    {
                        self.continue_vertex_drag(canvas_pos);
                    }
                }

                // Check if drag ended
                if response.drag_stopped()
                    && matches!(
                        self.state(),
                        super::core::CanvasState::DraggingVertex { .. }
                    )
                {
                    self.finish_vertex_drag();
                }

                // Also handle selection clicks when not dragging
                if response.clicked() && matches!(self.state(), super::core::CanvasState::Idle) {
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
                    } else if response.dragged()
                        && matches!(self.state(), super::core::CanvasState::Drawing { .. })
                    {
                        self.continue_drawing(
                            canvas_pos,
                            painter,
                            transform,
                            image_transform_params,
                        );
                    }
                }

                // Check if mouse was released (drag ended) for drawing tools
                if response.drag_stopped()
                    && matches!(self.state(), super::core::CanvasState::Drawing { .. })
                {
                    self.finalize_shape();
                }
            }
            ToolMode::Rotate => {
                let is_rotating = matches!(self.state(), super::core::CanvasState::Rotating { .. });
                let _span = tracing::debug_span!(
                    "rotate_tool",
                    selected_layer = ?self.selected_layer(),
                    selected_shape = ?self.selected_shape(),
                    is_rotating
                )
                .entered();

                debug!(
                    clicked = response.clicked(),
                    drag_started = response.drag_started(),
                    dragged = response.dragged(),
                    drag_stopped = response.drag_stopped(),
                    is_rotating,
                    selected_layer = ?self.selected_layer(),
                    selected_shape = ?self.selected_shape(),
                    "Rotate tool input events"
                );

                // Handle rotation based on selected layer
                if let Some(pos) = response.interact_pointer_pos() {
                    let canvas_pos = transform_pos(pos);

                    if response.drag_started() {
                        debug!(?canvas_pos, "Drag started - calling start_rotation");
                        self.start_rotation(canvas_pos);
                    } else if response.dragged()
                        && matches!(self.state(), super::core::CanvasState::Rotating { .. })
                    {
                        debug!(?canvas_pos, "Dragging - calling continue_rotation");
                        self.continue_rotation(canvas_pos);
                    } else if response.dragged()
                        && !matches!(self.state(), super::core::CanvasState::Rotating { .. })
                    {
                        debug!("Dragging but is_rotating is false");
                    }
                }

                // Check if drag ended
                if response.drag_stopped() {
                    if matches!(self.state(), super::core::CanvasState::Rotating { .. }) {
                        debug!("Drag stopped - calling finish_rotation");
                        self.finish_rotation();
                    } else {
                        debug!("Drag stopped but is_rotating was false");
                    }
                }

                // Also handle selection clicks when not rotating
                if response.clicked() && matches!(self.state(), super::core::CanvasState::Idle) {
                    debug!("Click detected - handling selection");
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = transform_pos(pos);
                        debug!(
                            ?canvas_pos,
                            "Calling handle_selection_click with interact_pointer_pos"
                        );
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

    /// Handle a selection click at the given canvas position
    ///
    /// Performs hit testing on all shapes to find the topmost shape
    /// that contains the click point. Updates selection state and
    /// automatically selects the Shapes layer if a shape is selected.
    #[instrument(skip(self), fields(pos = ?pos, total_shapes = self.shapes().len()))]
    pub(super) fn handle_selection_click(&mut self, pos: Pos2) {
        let _span = tracing::debug_span!("hit_testing").entered();

        // Check if we're in template mode
        if matches!(
            self.template_mode(),
            super::core::TemplateMode::Creating | super::core::TemplateMode::Editing
        ) {
            self.handle_field_selection_click(pos);
            return;
        }

        // Find the topmost shape that contains the click point
        // Iterate in reverse to select the most recently drawn shape first
        let mut selected = None;
        for (idx, shape) in self.shapes().iter().enumerate().rev() {
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

        debug!(?selected, selected_shape_old = ?self.selected_shape(), "Selection result");

        // Check if a shape was newly selected to auto-focus the name field
        // Only set focus if this is a different selection or a new selection
        let _span = tracing::debug_span!("shape_name_autofocus").entered();

        let should_focus = selected != *self.selected_shape() && selected.is_some();

        if should_focus {
            debug!("Setting focus flag for newly selected shape");
            self.set_focus_name_field(true);
        } else {
            trace!(
                selection_changed = (selected != *self.selected_shape()),
                selected_is_some = selected.is_some(),
                "Not setting focus flag"
            );
        }

        self.set_selected_shape(selected);
        self.set_show_properties(selected.is_some());

        // When a shape is selected, also select the Shapes layer for rotation
        if selected.is_some() {
            debug!("Shape selected - setting selected_layer to Shapes");
            self.with_selected_layer(Some(LayerType::Shapes));
        }
    }

    /// Handle field selection in template mode
    fn handle_field_selection_click(&mut self, pos: Pos2) {
        let _span = tracing::debug_span!("field_hit_testing").entered();

        let Some(template) = self.current_template() else {
            return;
        };

        // Find the topmost field that contains the click point
        let mut selected_field = None;
        for (page_idx, page) in template.pages.iter().enumerate().rev() {
            for (field_idx, field) in page.fields.iter().enumerate().rev() {
                // Check if point is within field bounds
                let contains = pos.x >= *field.bounds().x()
                    && pos.x <= *field.bounds().x() + *field.bounds().width()
                    && pos.y >= *field.bounds().y()
                    && pos.y <= *field.bounds().y() + *field.bounds().height();

                debug!(
                    field_id = %field.id(),
                    field_idx,
                    page_idx,
                    contains,
                    "Testing field"
                );

                if contains {
                    selected_field = Some(field_idx);
                    break;
                }
            }
            if selected_field.is_some() {
                break;
            }
        }

        debug!(
            ?selected_field,
            selected_field_old = ?self.selected_field(),
            "Field selection result"
        );

        self.with_selected_field(selected_field);
        self.set_show_properties(selected_field.is_some());
    }

    /// Start drawing a new shape
    ///
    /// Initializes the drawing state based on the current tool mode.
    /// For freehand polygons, starts collecting points. For rectangles
    /// and circles, records the starting position.
    pub(super) fn start_drawing(&mut self, pos: Pos2) {
        let points = if *self.current_tool() == ToolMode::Freehand {
            vec![pos]
        } else {
            Vec::new()
        };

        self.set_state(super::core::CanvasState::Drawing {
            start: pos,
            current_end: Some(pos),
            points,
        });
    }

    /// Continue drawing a shape (preview)
    ///
    /// Updates the drawing state with the current mouse position and
    /// draws a preview of the shape being created. The preview updates
    /// in real-time as the user drags the mouse.
    pub(super) fn continue_drawing(
        &mut self,
        pos: Pos2,
        painter: &egui::Painter,
        transform: &egui::emath::TSTransform,
        image_transform_params: Option<(f32, egui::Pos2)>,
    ) {
        // Store values needed for rendering before mutably borrowing state
        let current_tool = *self.current_tool();
        let fill_color = *self.fill_color();
        let stroke = *self.stroke();
        let zoom_level = *self.zoom_level();

        // Helper to transform a point from image pixel coordinates to screen coordinates
        // This applies: image coords -> canvas coords -> zoom/pan transform
        let transform_preview_pos = |image_pos: Pos2| -> Pos2 {
            if let Some((scale, image_offset)) = image_transform_params {
                // First convert from image pixel coords to canvas coords
                let canvas_x = image_pos.x * scale + image_offset.x;
                let canvas_y = image_pos.y * scale + image_offset.y;
                let canvas_pos = Pos2::new(canvas_x, canvas_y);
                // Then apply zoom/pan transformation
                transform.mul_pos(canvas_pos)
            } else {
                // No image loaded - positions are already in canvas coords
                transform.mul_pos(image_pos)
            }
        };

        // Update the drawing state with the new position
        if let super::core::CanvasState::Drawing {
            start,
            current_end,
            points,
        } = self.state_mut()
        {
            *current_end = Some(pos);

            match current_tool {
                ToolMode::Rectangle => {
                    // Transform the rectangle corners for preview (from image coords to screen)
                    let rect_min = transform_preview_pos(start.min(pos));
                    let rect_max = transform_preview_pos(start.max(pos));
                    let transformed_rect = egui::Rect::from_min_max(rect_min, rect_max);
                    painter.rect_filled(transformed_rect, 0.0, fill_color);
                    painter.rect_stroke(transformed_rect, 0.0, stroke, egui::StrokeKind::Outside);
                }
                ToolMode::Circle => {
                    // Radius is in image pixel coordinates, needs to be scaled
                    let radius = start.distance(pos);
                    let transformed_center = transform_preview_pos(*start);
                    // Scale radius based on image scale, then apply zoom
                    let scaled_radius = if let Some((scale, _)) = image_transform_params {
                        radius * scale * zoom_level
                    } else {
                        radius * zoom_level
                    };
                    painter.circle(transformed_center, scaled_radius, fill_color, stroke);
                }
                ToolMode::Freehand => {
                    points.push(pos);
                    if points.len() > 2 {
                        // Transform points for preview (from image coords to screen)
                        let transformed_points: Vec<Pos2> =
                            points.iter().map(|p| transform_preview_pos(*p)).collect();
                        // Draw preview as a closed polygon
                        painter.add(egui::Shape::convex_polygon(
                            transformed_points.clone(),
                            fill_color,
                            egui::Stroke::NONE,
                        ));
                        painter.add(egui::Shape::closed_line(transformed_points, stroke));
                    } else if points.len() > 1 {
                        // Transform points for preview line (from image coords to screen)
                        let transformed_points: Vec<Pos2> =
                            points.iter().map(|p| transform_preview_pos(*p)).collect();
                        // Draw preview line until we have enough points
                        painter.add(egui::Shape::line(transformed_points, stroke));
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
    }

    /// Finalize and create the shape
    ///
    /// Creates the final shape from the drawing state and adds it to
    /// the canvas. Automatically selects the newly created shape and
    /// focuses the name field for easy naming.
    pub(super) fn finalize_shape(&mut self) {
        // Check if we're in template creation/editing mode
        if matches!(
            self.template_mode(),
            super::core::TemplateMode::Creating | super::core::TemplateMode::Editing
        ) {
            self.finalize_template_field();
            return;
        }

        let shape = if let super::core::CanvasState::Drawing {
            start,
            current_end,
            points,
        } = self.state()
        {
            match self.current_tool() {
                ToolMode::Rectangle => {
                    if let Some(end) = current_end {
                        Rectangle::from_corners(*start, *end, *self.stroke(), *self.fill_color())
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
                    if let Some(edge) = current_end {
                        let radius = start.distance(*edge);
                        Circle::new(*start, radius, *self.stroke(), *self.fill_color())
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
                    if points.len() >= 3 {
                        // Create a closed polygon from the points
                        PolygonShape::from_points(
                            points.clone(),
                            *self.stroke(),
                            *self.fill_color(),
                        )
                        .map(Shape::Polygon)
                        .map_err(|e| {
                            warn!("Failed to create polygon: {}", e);
                            e
                        })
                        .ok()
                    } else {
                        None
                    }
                }
                ToolMode::Select => None,
                ToolMode::Edit => None,
                ToolMode::Rotate => None,
            }
        } else {
            None
        };

        if let Some(shape) = shape {
            self.add_shape(shape);

            // Automatically select the newly created shape so user can name it
            let new_shape_idx = self.shapes().len() - 1;
            self.set_selected_shape(Some(new_shape_idx));
            self.set_show_properties(true);
            self.set_focus_name_field(true);

            debug!(
                shape_index = new_shape_idx,
                "Auto-selecting newly created shape"
            );
        }

        // Reset to idle state
        self.set_state(super::core::CanvasState::Idle);
    }

    /// Finalize a template field (when in template mode)
    fn finalize_template_field(&mut self) {
        use form_factor_core::FieldType;

        let field_def = if let super::core::CanvasState::Drawing {
            start,
            current_end,
            points,
        } = self.state()
        {
            match self.current_tool() {
                ToolMode::Rectangle | ToolMode::Circle => {
                    // Both rectangle and circle create rectangular field bounds
                    if let Some(end) = current_end {
                        let min_x = start.x.min(end.x);
                        let min_y = start.y.min(end.y);
                        let max_x = start.x.max(end.x);
                        let max_y = start.y.max(end.y);
                        let width = max_x - min_x;
                        let height = max_y - min_y;

                        // Generate a unique field ID
                        let field_count = if let Some(template) = self.current_template() {
                            template.pages.iter().map(|p| p.fields.len()).sum::<usize>()
                        } else {
                            0
                        };
                        let field_id = format!("field_{}", field_count + 1);
                        let bounds = form_factor_core::FieldBounds::new(min_x, min_y, width, height);

                        Some(
                            form_factor_core::FieldDefinitionBuilder::default()
                                .id(field_id.clone())
                                .label(format!("Field {}", field_count + 1))
                                .field_type(FieldType::FreeText)
                                .page_index(0) // TODO: Use current page
                                .bounds(bounds)
                                .required(false)
                                .build()
                                .expect("Valid field definition"),
                        )
                    } else {
                        None
                    }
                }
                ToolMode::Freehand => {
                    // Freehand creates field from bounding box of polygon
                    if points.len() >= 3 {
                        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                        let width = max_x - min_x;
                        let height = max_y - min_y;

                        let field_count = if let Some(template) = self.current_template() {
                            template.pages.iter().map(|p| p.fields.len()).sum::<usize>()
                        } else {
                            0
                        };
                        let field_id = format!("field_{}", field_count + 1);
                        let bounds = form_factor_core::FieldBounds::new(min_x, min_y, width, height);

                        Some(
                            form_factor_core::FieldDefinitionBuilder::default()
                                .id(field_id.clone())
                                .label(format!("Field {}", field_count + 1))
                                .field_type(FieldType::FreeText)
                                .page_index(0) // TODO: Use current page
                                .bounds(bounds)
                                .required(false)
                                .build()
                                .expect("Valid field definition"),
                        )
                    } else {
                        None
                    }
                }
                ToolMode::Select => None,
                ToolMode::Edit => None,
                ToolMode::Rotate => None,
            }
        } else {
            None
        };

        if let Some(field) = field_def {
            // Add field to current template
            if let Some(template) = self.current_template_mut() {
                // Add to first page (TODO: support multiple pages)
                if template.pages.is_empty() {
                    let page = crate::TemplatePage::new(0);
                    template.pages.push(page);
                }

                if let Some(page) = template.pages.first_mut() {
                    page.fields.push(field.clone());
                    let field_idx = page.fields.len() - 1;

                    // Select the newly created field
                    self.with_selected_field(Some(field_idx));
                    self.set_show_properties(true);

                    debug!(
                        field_id = %field.id(),
                        field_index = field_idx,
                        "Created template field"
                    );
                }
            }
        }

        self.set_state(super::core::CanvasState::Idle);
    }

    /// Start dragging a vertex in Edit mode
    ///
    /// Determines which vertex of the selected shape is closest to the
    /// click position and begins vertex dragging if one is found within
    /// the click radius.
    pub(super) fn start_vertex_drag(&mut self, pos: Pos2) {
        const VERTEX_CLICK_RADIUS: f32 = 8.0;

        let Some(idx) = *self.selected_shape() else {
            // No shape selected, try to select one
            self.handle_selection_click(pos);
            return;
        };

        let Some(shape) = self.shapes().get(idx) else {
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
                if pos.distance(*circle.center()) < VERTEX_CLICK_RADIUS {
                    Some(0)
                } else {
                    let edge_point =
                        egui::pos2(circle.center().x + circle.radius(), circle.center().y);
                    if pos.distance(edge_point) < VERTEX_CLICK_RADIUS {
                        Some(1)
                    } else {
                        None
                    }
                }
            }
            Shape::Polygon(poly) => poly
                .to_egui_points()
                .iter()
                .enumerate()
                .find(|(_, vertex_pos)| pos.distance(**vertex_pos) < VERTEX_CLICK_RADIUS)
                .map(|(i, _)| i),
        };

        if let Some(vertex_idx) = clicked_vertex {
            debug!(vertex_idx, "Starting vertex drag");
            self.set_state(super::core::CanvasState::DraggingVertex {
                vertex_index: vertex_idx,
            });
        }
    }

    /// Continue dragging a vertex
    ///
    /// Updates the position of the vertex being dragged to follow the
    /// mouse cursor. Different shapes handle vertex updates differently:
    /// rectangles update corners, circles update center or radius, and
    /// polygons update individual vertex positions.
    pub(super) fn continue_vertex_drag(&mut self, pos: Pos2) {
        let super::core::CanvasState::DraggingVertex {
            vertex_index: vertex_idx,
        } = *self.state()
        else {
            return;
        };

        let Some(shape_idx) = *self.selected_shape() else {
            return;
        };

        let Some(shape) = self.shapes_mut().get_mut(shape_idx) else {
            return;
        };

        trace!(vertex_idx, ?pos, "Continuing vertex drag");

        // Update the vertex position based on which shape and vertex
        match shape {
            Shape::Rectangle(rect) => {
                // Update the specific corner using setter method
                if let Err(e) = rect.set_corner(vertex_idx, pos) {
                    warn!("Failed to update rectangle corner {}: {}", vertex_idx, e);
                }
            }
            Shape::Circle(circle) => {
                match vertex_idx {
                    0 => {
                        // Moving center - maintain radius
                        if let Err(e) = circle.set_center(pos) {
                            warn!("Failed to update circle center: {}", e);
                        }
                    }
                    1 => {
                        // Moving edge - update radius
                        let new_radius = circle.center().distance(pos);
                        if let Err(e) = circle.set_radius(new_radius) {
                            warn!("Failed to update circle radius: {}", e);
                        }
                    }
                    _ => {}
                }
            }
            Shape::Polygon(poly) => {
                // Update the specific vertex using setter method
                if let Err(e) = poly.set_vertex(vertex_idx, pos) {
                    warn!("Failed to update polygon vertex {}: {}", vertex_idx, e);
                }
            }
        }
    }

    /// Finish dragging a vertex
    ///
    /// Completes the vertex drag operation and returns to idle state.
    pub(super) fn finish_vertex_drag(&mut self) {
        debug!("Finishing vertex drag");
        self.set_state(super::core::CanvasState::Idle);
    }

    /// Start rotation interaction
    ///
    /// Determines what to rotate based on the selected layer:
    /// - Shapes layer: Rotates the selected shape around its center
    /// - Grid layer: Rotates the grid overlay
    /// - Canvas layer: Rotates the form image
    /// - Detections layer: Cannot be rotated
    #[instrument(skip(self), fields(
        pos = ?pos,
        selected_layer = ?self.selected_layer(),
        selected_shape = ?self.selected_shape(),
        has_form_image = self.form_image().is_some()
    ))]
    pub(super) fn start_rotation(&mut self, pos: Pos2) {
        let _span = tracing::debug_span!("start_rotation").entered();

        debug!(
            ?pos,
            selected_layer = ?self.selected_layer(),
            selected_shape = ?self.selected_shape(),
            has_form_image = self.form_image().is_some(),
            "Attempting to start rotation"
        );

        // Determine what to rotate based on selected layer
        match self.selected_layer() {
            Some(LayerType::Shapes) => {
                debug!("Shapes layer selected");
                // If a shape is selected, rotate it
                if let Some(idx) = *self.selected_shape() {
                    debug!(shape_idx = idx, "Shape is selected");
                    if let Some(shape) = self.shapes().get(idx) {
                        let center = self.get_shape_center(shape);
                        let start_angle = Self::calculate_angle(center, pos);
                        self.set_state(super::core::CanvasState::Rotating {
                            start_angle,
                            center: Some(center),
                        });
                        debug!(?center, start_angle, "Started rotating shape");
                    } else {
                        debug!(shape_idx = idx, "Shape index out of bounds");
                    }
                } else {
                    debug!("No shape selected - cannot rotate");
                }
            }
            Some(LayerType::Template) | Some(LayerType::Instance) => {
                debug!("Template/Instance layer selected - rotation not supported");
                // TODO: Add field rotation support if needed
            }
            Some(LayerType::Grid) => {
                debug!("Grid layer selected - rotating grid");
                // Rotate the grid around the canvas center
                let start_angle = Self::calculate_angle(Pos2::ZERO, pos);
                self.set_state(super::core::CanvasState::Rotating {
                    start_angle,
                    center: Some(Pos2::ZERO),
                });
                debug!(rotation_center = ?Pos2::ZERO, start_angle, "Started rotating grid");
            }
            Some(LayerType::Canvas) => {
                debug!(
                    has_form_image = self.form_image().is_some(),
                    "Canvas layer selected"
                );
                // Rotate the form image if one is loaded
                if self.form_image().is_some() {
                    let start_angle = Self::calculate_angle(Pos2::ZERO, pos);
                    self.set_state(super::core::CanvasState::Rotating {
                        start_angle,
                        center: Some(Pos2::ZERO),
                    });
                    debug!(rotation_center = ?Pos2::ZERO, start_angle, "Started rotating form image");
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

        debug!(
            is_rotating = matches!(self.state(), super::core::CanvasState::Rotating { .. }),
            "Rotation state after start_rotation"
        );
    }

    /// Continue rotation interaction
    ///
    /// Calculates the rotation delta and applies it to the selected layer:
    /// - Shapes: Applies rotation transformation to the shape geometry
    /// - Grid: Updates grid rotation angle
    /// - Canvas: Updates form image rotation angle
    pub(super) fn continue_rotation(&mut self, pos: Pos2) {
        // Store values before mutably borrowing state
        let selected_layer = *self.selected_layer();
        let selected_shape = *self.selected_shape();
        let grid_rotation_angle = *self.grid_rotation_angle();
        let form_image_rotation = *self.form_image_rotation();

        let super::core::CanvasState::Rotating {
            start_angle,
            center,
        } = self.state_mut()
        else {
            return;
        };

        let Some(center_pos) = *center else {
            return;
        };

        let current_angle = Self::calculate_angle(center_pos, pos);
        let angle_delta = current_angle - *start_angle;

        // Update start angle for next frame
        *start_angle = current_angle;

        debug!(
            ?pos,
            ?center_pos,
            current_angle,
            angle_delta,
            "Continuing rotation"
        );

        // Apply rotation based on selected layer (negated for inverted axis)
        match selected_layer {
            Some(LayerType::Shapes) => {
                // Rotate the selected shape using the new transformation method
                if let Some(idx) = selected_shape
                    && let Some(shape) = self.shapes_mut().get_mut(idx)
                {
                    let rotation_angle = -angle_delta; // Negate for inverted axis
                    match shape {
                        Shape::Rectangle(rect) => {
                            if let Err(e) = rect.rotate(rotation_angle, center_pos) {
                                warn!("Failed to rotate rectangle: {}", e);
                            }
                        }
                        Shape::Circle(circle) => {
                            if let Err(e) = circle.rotate(rotation_angle, center_pos) {
                                warn!("Failed to rotate circle: {}", e);
                            }
                        }
                        Shape::Polygon(poly) => {
                            if let Err(e) = poly.rotate(rotation_angle, center_pos) {
                                warn!("Failed to rotate polygon: {}", e);
                            }
                        }
                    }
                }
            }
            Some(LayerType::Grid) => {
                self.set_grid_rotation_angle(grid_rotation_angle - angle_delta);
            }
            Some(LayerType::Canvas) => {
                self.set_form_image_rotation(form_image_rotation - angle_delta);
            }
            Some(LayerType::Detections) => {
                // Detections cannot be rotated
            }
            Some(LayerType::Template) | Some(LayerType::Instance) => {
                // Template/Instance fields cannot be rotated (for now)
            }
            None => {}
        }
    }

    /// Finish rotation interaction
    ///
    /// Completes the rotation operation and returns to idle state.
    pub(super) fn finish_rotation(&mut self) {
        debug!("Finishing rotation");
        self.set_state(super::core::CanvasState::Idle);
    }

    /// Calculate angle from center to position in radians
    ///
    /// Uses atan2 to calculate the angle, which correctly handles all quadrants
    /// and returns a value in the range [-π, π].
    pub(super) fn calculate_angle(center: Pos2, pos: Pos2) -> f32 {
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        dy.atan2(dx)
    }

    /// Get the center point of a shape
    ///
    /// Calculates the geometric center (centroid) of the shape:
    /// - Rectangle: Average of all 4 corners
    /// - Circle: The center point
    /// - Polygon: Average of all vertices
    pub(super) fn get_shape_center(&self, shape: &Shape) -> Pos2 {
        match shape {
            Shape::Rectangle(rect) => {
                let sum_x: f32 = rect.corners().iter().map(|p| p.x).sum();
                let sum_y: f32 = rect.corners().iter().map(|p| p.y).sum();
                Pos2::new(sum_x / 4.0, sum_y / 4.0)
            }
            Shape::Circle(circle) => *circle.center(),
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                let sum_x: f32 = points.iter().map(|p| p.x).sum();
                let sum_y: f32 = points.iter().map(|p| p.y).sum();
                let count = points.len() as f32;
                Pos2::new(sum_x / count, sum_y / count)
            }
        }
    }

    /// Start dragging a template field
    fn start_field_drag(&mut self, pos: Pos2) {
        let Some(field_idx) = *self.selected_field() else {
            return;
        };

        let Some(template) = self.current_template() else {
            return;
        };

        // Find the field (currently only supporting first page)
        let Some(page) = template.pages.first() else {
            return;
        };

        let Some(field) = page.fields.get(field_idx) else {
            return;
        };

        // Store original bounds for drag operation
        self.set_state(super::core::CanvasState::DraggingField {
            field_index: field_idx,
            drag_start: pos,
            original_bounds: *field.bounds(),
        });

        debug!(field_idx, ?pos, "Started dragging field");
    }

    /// Continue dragging a template field
    fn continue_field_drag(&mut self, pos: Pos2) {
        let super::core::CanvasState::DraggingField {
            field_index,
            drag_start,
            original_bounds,
        } = self.state().clone()
        else {
            return;
        };

        // Calculate drag delta
        let delta_x = pos.x - drag_start.x;
        let delta_y = pos.y - drag_start.y;

        // Update field position by reconstructing with new bounds
        if let Some(template) = self.current_template_mut()
            && let Some(page) = template.pages.first_mut()
            && let Some(field) = page.fields.get(field_index)
        {
            let new_bounds = FieldBounds::new(
                *original_bounds.x() + delta_x,
                *original_bounds.y() + delta_y,
                *original_bounds.width(),
                *original_bounds.height(),
            );

            // Reconstruct field with new bounds
            let mut builder = FieldDefinition::builder()
                .id(field.id().to_string())
                .label(field.label().to_string())
                .field_type(field.field_type().clone())
                .page_index(*field.page_index())
                .bounds(new_bounds)
                .required(*field.required());

            if let Some(pattern) = field.validation_pattern() {
                builder = builder.validation_pattern(pattern.to_string());
            }

            if let Some(text) = field.help_text() {
                builder = builder.help_text(text.to_string());
            }

            // TODO: Copy metadata as well
            let updated_field = builder
                .build()
                .expect("Field reconstruction with valid data should not fail");

            page.fields[field_index] = updated_field;
        }
    }

    /// Finish dragging a template field
    fn finish_field_drag(&mut self) {
        debug!("Finished dragging field");
        self.set_state(super::core::CanvasState::Idle);
    }
}
