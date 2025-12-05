//! Field manipulation helpers for the template editor.

use super::editor::{DragOperation, DragOperationType, DrawingState, TemplateEditorPanel};
use egui::{Color32, Pos2, Rect, Response, Stroke, Vec2};
use form_factor_core::{FieldBounds, FieldDefinition, FieldType};
use tracing::{debug, instrument};

const HANDLE_SIZE: f32 = 8.0;
const MIN_FIELD_SIZE: f32 = 20.0;

impl TemplateEditorPanel {
    /// Handles drawing mode interactions.
    #[instrument(skip(self, response, _painter))]
    pub(super) fn handle_draw_mode(
        &mut self,
        response: &Response,
        _painter: &egui::Painter,
        canvas_rect: Rect,
        page_index: usize,
    ) {
        if response.drag_started()
            && let Some(start_pos) = response.interact_pointer_pos()
        {
            self.drawing_state = Some(DrawingState {
                start_pos,
                current_pos: start_pos,
            });
            debug!("Started drawing field");
        }

        if response.dragged()
            && let Some(drawing) = &mut self.drawing_state
            && let Some(current_pos) = response.interact_pointer_pos()
        {
            drawing.current_pos = current_pos;
        }

        if response.drag_stopped()
            && let Some(drawing) = self.drawing_state.take()
        {
            self.create_field_from_drawing(drawing, canvas_rect, page_index);
            debug!("Completed drawing field");
        }
    }

    /// Handles select mode interactions (selection, movement, resizing).
    #[instrument(skip(self, response, fields))]
    pub(super) fn handle_select_mode(
        &mut self,
        response: &Response,
        fields: &[FieldDefinition],
        canvas_rect: Rect,
        page_index: usize,
    ) {
        // Start drag operation
        if response.drag_started()
            && let Some(start_pos) = response.interact_pointer_pos()
        {
            // Check if clicking on resize handle of selected field
            if let Some(selected_idx) = self.state.selected_field()
                && let Some(field) = fields.get(selected_idx)
                && let Some(handle_type) =
                    self.get_resize_handle_at_position(start_pos, field, canvas_rect)
            {
                // Start resize operation
                self.drag_state = Some(DragOperation {
                    field_index: selected_idx,
                    operation_type: handle_type,
                    start_pos,
                    original_bounds: field.bounds,
                });
                debug!(field_index = selected_idx, handle = ?handle_type, "Started resize");
                return;
            }

            // Check if clicking on a field (for selection or movement)
            let field_idx = self.find_field_at_position(start_pos, fields, canvas_rect);
            if let Some(idx) = field_idx {
                // Select the field
                self.state.set_selected_field(Some(idx));

                // Start move operation
                if let Some(field) = fields.get(idx) {
                    self.drag_state = Some(DragOperation {
                        field_index: idx,
                        operation_type: DragOperationType::Move,
                        start_pos,
                        original_bounds: field.bounds,
                    });
                    debug!(field_index = idx, "Started move");
                }
            } else {
                // Clicked on empty space - deselect
                self.state.set_selected_field(None);
                debug!("Deselected field");
            }
        }

        // Continue drag operation
        if response.dragged()
            && let Some(drag) = self.drag_state.clone()
            && let Some(current_pos) = response.interact_pointer_pos()
        {
            self.update_field_bounds(&drag, current_pos, canvas_rect, page_index);
        }

        // End drag operation
        if response.drag_stopped()
            && let Some(drag) = self.drag_state.take()
        {
            // Push undo snapshot after drag completes with descriptive action
            let action_desc = match drag.operation_type {
                DragOperationType::Move => "Move field",
                DragOperationType::ResizeTopLeft => "Resize field (top-left)",
                DragOperationType::ResizeTopRight => "Resize field (top-right)",
                DragOperationType::ResizeBottomLeft => "Resize field (bottom-left)",
                DragOperationType::ResizeBottomRight => "Resize field (bottom-right)",
            };
            self.state.push_snapshot(action_desc);
            debug!("Completed drag operation: {}", action_desc);
        }
    }

    /// Renders resize handles for a field.
    pub(super) fn render_resize_handles(
        &self,
        field: &FieldDefinition,
        painter: &egui::Painter,
        canvas_rect: Rect,
    ) {
        let bounds = &field.bounds;
        let field_rect = Rect::from_min_size(
            canvas_rect.min + Vec2::new(bounds.x, bounds.y),
            Vec2::new(bounds.width, bounds.height),
        );

        let handle_color = Color32::from_rgb(0, 100, 200);

        // Top-left
        painter.circle_filled(field_rect.min, HANDLE_SIZE / 2.0, handle_color);
        // Top-right
        painter.circle_filled(
            Pos2::new(field_rect.max.x, field_rect.min.y),
            HANDLE_SIZE / 2.0,
            handle_color,
        );
        // Bottom-left
        painter.circle_filled(
            Pos2::new(field_rect.min.x, field_rect.max.y),
            HANDLE_SIZE / 2.0,
            handle_color,
        );
        // Bottom-right
        painter.circle_filled(field_rect.max, HANDLE_SIZE / 2.0, handle_color);
    }

    /// Renders the drawing preview rectangle.
    pub(super) fn render_drawing_preview(&self, drawing: &DrawingState, painter: &egui::Painter) {
        let rect = Rect::from_two_pos(drawing.start_pos, drawing.current_pos);

        // Draw preview rectangle
        painter.rect_filled(rect, 2.0, Color32::from_rgba_unmultiplied(255, 200, 0, 60));
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(2.0, Color32::from_rgb(255, 150, 0)),
            egui::epaint::StrokeKind::Middle,
        );

        // Show dimensions
        let size_text = format!("{}x{}", rect.width() as i32, rect.height() as i32);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            size_text,
            egui::FontId::proportional(14.0),
            Color32::BLACK,
        );
    }

    /// Creates a new field from a drawing operation.
    fn create_field_from_drawing(
        &mut self,
        drawing: DrawingState,
        canvas_rect: Rect,
        page_index: usize,
    ) {
        let rect = Rect::from_two_pos(drawing.start_pos, drawing.current_pos);

        // Enforce minimum size
        if rect.width() < MIN_FIELD_SIZE || rect.height() < MIN_FIELD_SIZE {
            debug!("Field too small, ignoring");
            return;
        }

        // Convert to canvas-relative coordinates
        let canvas_pos = rect.min - canvas_rect.min;

        // Generate field ID
        let field_count = self
            .state
            .current_template()
            .map(|t| t.fields().len())
            .unwrap_or(0);
        let field_id = format!("field_{}", field_count + 1);

        // Create field definition
        let field = FieldDefinition {
            id: field_id.clone(),
            label: format!("Field {}", field_count + 1),
            field_type: FieldType::FreeText,
            page_index,
            bounds: FieldBounds {
                x: canvas_pos.x,
                y: canvas_pos.y,
                width: rect.width(),
                height: rect.height(),
            },
            required: false,
            validation_pattern: None,
            help_text: None,
            metadata: std::collections::HashMap::new(),
        };

        // Add field to template
        if let Some(template) = self.state.current_template_mut() {
            // Get or create page
            while template.page_count() <= page_index {
                use crate::TemplatePage;
                let page = TemplatePage::new(template.pages.len());
                template.pages.push(page);
            }

            // Add field to page
            if let Some(page) = template.pages.get_mut(page_index) {
                page.add_field(field);
                self.state
                    .push_snapshot(format!("Create field '{}'", field_id));
                debug!(field_id = %field_id, "Created new field");
            }
        }
    }

    /// Deletes a field from the template.
    pub(super) fn delete_field(&mut self, field_index: usize, page_index: usize) {
        if let Some(template) = self.state.current_template_mut()
            && let Some(page) = template.pages.get_mut(page_index)
            && field_index < page.fields.len()
        {
            let field_id = page.fields[field_index].id.clone();
            page.fields.remove(field_index);
            self.state.set_selected_field(None);
            self.state
                .push_snapshot(format!("Delete field '{}'", field_id));
            debug!(field_id = %field_id, "Deleted field");
        }
    }

    /// Determines which resize handle (if any) is at the given position.
    fn get_resize_handle_at_position(
        &self,
        pos: Pos2,
        field: &FieldDefinition,
        canvas_rect: Rect,
    ) -> Option<DragOperationType> {
        let bounds = &field.bounds;
        let field_rect = Rect::from_min_size(
            canvas_rect.min + Vec2::new(bounds.x, bounds.y),
            Vec2::new(bounds.width, bounds.height),
        );

        let handle_radius = HANDLE_SIZE;

        // Check each corner
        if pos.distance(field_rect.min) < handle_radius {
            return Some(DragOperationType::ResizeTopLeft);
        }
        if pos.distance(Pos2::new(field_rect.max.x, field_rect.min.y)) < handle_radius {
            return Some(DragOperationType::ResizeTopRight);
        }
        if pos.distance(Pos2::new(field_rect.min.x, field_rect.max.y)) < handle_radius {
            return Some(DragOperationType::ResizeBottomLeft);
        }
        if pos.distance(field_rect.max) < handle_radius {
            return Some(DragOperationType::ResizeBottomRight);
        }

        None
    }

    /// Updates field bounds based on drag operation.
    fn update_field_bounds(
        &mut self,
        drag: &DragOperation,
        current_pos: Pos2,
        _canvas_rect: Rect,
        page_index: usize,
    ) {
        let delta = current_pos - drag.start_pos;

        let new_bounds = match drag.operation_type {
            DragOperationType::Move => {
                // Move the entire field
                FieldBounds {
                    x: drag.original_bounds.x + delta.x,
                    y: drag.original_bounds.y + delta.y,
                    width: drag.original_bounds.width,
                    height: drag.original_bounds.height,
                }
            }
            DragOperationType::ResizeTopLeft => {
                // Resize from top-left corner
                let new_x = drag.original_bounds.x + delta.x;
                let new_y = drag.original_bounds.y + delta.y;
                let new_width = drag.original_bounds.width - delta.x;
                let new_height = drag.original_bounds.height - delta.y;

                FieldBounds {
                    x: new_x,
                    y: new_y,
                    width: new_width.max(MIN_FIELD_SIZE),
                    height: new_height.max(MIN_FIELD_SIZE),
                }
            }
            DragOperationType::ResizeTopRight => {
                // Resize from top-right corner
                let new_y = drag.original_bounds.y + delta.y;
                let new_width = drag.original_bounds.width + delta.x;
                let new_height = drag.original_bounds.height - delta.y;

                FieldBounds {
                    x: drag.original_bounds.x,
                    y: new_y,
                    width: new_width.max(MIN_FIELD_SIZE),
                    height: new_height.max(MIN_FIELD_SIZE),
                }
            }
            DragOperationType::ResizeBottomLeft => {
                // Resize from bottom-left corner
                let new_x = drag.original_bounds.x + delta.x;
                let new_width = drag.original_bounds.width - delta.x;
                let new_height = drag.original_bounds.height + delta.y;

                FieldBounds {
                    x: new_x,
                    y: drag.original_bounds.y,
                    width: new_width.max(MIN_FIELD_SIZE),
                    height: new_height.max(MIN_FIELD_SIZE),
                }
            }
            DragOperationType::ResizeBottomRight => {
                // Resize from bottom-right corner
                let new_width = drag.original_bounds.width + delta.x;
                let new_height = drag.original_bounds.height + delta.y;

                FieldBounds {
                    x: drag.original_bounds.x,
                    y: drag.original_bounds.y,
                    width: new_width.max(MIN_FIELD_SIZE),
                    height: new_height.max(MIN_FIELD_SIZE),
                }
            }
        };

        // Update the field bounds in the template
        if let Some(template) = self.state.current_template_mut()
            && let Some(page) = template.pages.get_mut(page_index)
            && let Some(field) = page.fields.get_mut(drag.field_index)
        {
            field.bounds = new_bounds;
        }
    }
}
