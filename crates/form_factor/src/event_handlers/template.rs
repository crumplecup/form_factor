//! Event handlers for template operations.

use form_factor_drawing::{DrawingCanvas, FieldCreator, Shape};
use tracing::{debug, error, instrument, warn};

/// Handles template-related events
pub struct TemplateEventHandler;

impl TemplateEventHandler {
    /// Handles AddDetectionToTemplate event
    #[instrument(skip(canvas), fields(detection_id))]
    pub fn handle_add_detection_to_template(
        canvas: &mut DrawingCanvas,
        detection_id: &str,
    ) -> Option<crate::AppEvent> {
        debug!(detection_id, "Handling AddDetectionToTemplate event");
        
        // Get detection metadata
        let metadata = match canvas.get_detection_metadata(detection_id) {
            Some(m) => m.clone(),
            None => {
                warn!(detection_id, "Detection metadata not found");
                return None;
            }
        };

        // Parse detection index from ID (format: "logo_0", "text_1", etc.)
        let index = match Self::parse_detection_index(detection_id) {
            Some(idx) => idx,
            None => {
                warn!(detection_id, "Could not parse detection index from ID");
                return None;
            }
        };

        // Get detection shape to extract bounds
        let shape = match canvas.get_detection(index) {
            Some(s) => s,
            None => {
                warn!(detection_id, index, "Detection shape not found at index");
                return None;
            }
        };

        // Extract bounds from shape
        let bounds = Self::extract_bounds(shape);

        // Create field from detection
        let mut field_creator = FieldCreator::new();
        let field = match field_creator.create_field_from_detection(&metadata, bounds) {
            Ok(f) => f,
            Err(e) => {
                error!(error = %e, "Failed to create field from detection");
                return None;
            }
        };

        let field_id = field.id().to_string();

        // Add to template
        if let Err(e) = canvas.add_field_to_template(field) {
            error!(error = %e, "Failed to add field to template");
            None
        } else {
            debug!(detection_id, "Successfully added detection to template");
            Some(crate::AppEvent::FieldAddedToTemplate { field_id })
        }
    }

    /// Handles AddShapeToTemplate event
    #[instrument(skip(canvas))]
    pub fn handle_add_shape_to_template(
        canvas: &mut DrawingCanvas,
        shape_id: usize,
    ) -> Option<crate::AppEvent> {
        debug!(shape_id, "Handling AddShapeToTemplate event");
        
        // Get shape from canvas
        let shape = match canvas.get_shape(shape_id) {
            Some(s) => s,
            None => {
                warn!(shape_id, "Shape not found at index");
                return None;
            }
        };

        // Create field from shape
        let mut field_creator = FieldCreator::new();
        let field = match field_creator.create_field(shape, None, None) {
            Ok(f) => f,
            Err(e) => {
                error!(error = %e, "Failed to create field from shape");
                return None;
            }
        };

        let field_id = field.id().to_string();

        // Add to template
        if let Err(e) = canvas.add_field_to_template(field) {
            error!(error = %e, "Failed to add field to template");
            None
        } else {
            debug!(shape_id, "Successfully added shape to template");
            Some(crate::AppEvent::FieldAddedToTemplate { field_id })
        }
    }

    /// Handles RemoveFieldFromTemplate event
    #[instrument(skip(canvas), fields(field_id))]
    pub fn handle_remove_field_from_template(
        canvas: &mut DrawingCanvas,
        field_id: &str,
    ) {
        debug!(field_id, "Handling RemoveFieldFromTemplate event");
        
        if let Err(e) = canvas.remove_field_from_template(field_id) {
            warn!(error = %e, "Failed to remove field from template");
        } else {
            debug!(field_id, "Successfully removed field from template");
        }
    }

    /// Parses the index from a detection ID (e.g., "logo_0" -> 0)
    fn parse_detection_index(detection_id: &str) -> Option<usize> {
        detection_id
            .rsplit('_')
            .next()
            .and_then(|s| s.parse().ok())
    }

    /// Extracts bounds (x, y, width, height) from a shape
    fn extract_bounds(shape: &Shape) -> (f32, f32, f32, f32) {
        match shape {
            Shape::Rectangle(rect) => {
                let corners = rect.corners();
                let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
            Shape::Circle(circle) => {
                let center = circle.center();
                let radius = *circle.radius();
                (
                    center.x - radius,
                    center.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                )
            }
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }
}
