//! Property rendering for shapes and detections

use form_factor_drawing::{DrawingCanvas, Shape};
use form_factor_drawing::DetectionType;
use tracing::instrument;

/// Renders property editors for shapes and detections
pub struct PropertyRenderer;

impl PropertyRenderer {
    /// Render shape property editor
    #[instrument(skip(ui, canvas), fields(shape_idx))]
    pub fn render_shape_properties(
        ui: &mut egui::Ui,
        canvas: &DrawingCanvas,
        shape_idx: usize,
    ) -> Result<(), String> {
        tracing::debug!(shape_idx, "Rendering shape properties");
        
        let shape = canvas.shapes().get(shape_idx).ok_or_else(|| {
            tracing::warn!(shape_idx, "Shape index out of bounds");
            format!("Shape {} not found", shape_idx)
        })?;

        // Shape property rendering
        ui.label(format!("Shape #{}", shape_idx));
        ui.add_space(8.0);

        // Show shape type and bounds
        match shape {
            Shape::Rectangle(rect) => {
                ui.label("Type: Rectangle");
                let corners = rect.corners();
                let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = corners
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::NEG_INFINITY, f32::max);
                let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = corners
                    .iter()
                    .map(|p| p.y)
                    .fold(f32::NEG_INFINITY, f32::max);
                ui.label(format!("Position: ({:.1}, {:.1})", min_x, min_y));
                ui.label(format!("Size: {:.1} × {:.1}", max_x - min_x, max_y - min_y));
            }
            Shape::Circle(circle) => {
                ui.label("Type: Circle");
                let center = circle.center();
                let radius = circle.radius();
                ui.label(format!("Center: ({:.1}, {:.1})", center.x, center.y));
                ui.label(format!("Radius: {:.1}", radius));
            }
            Shape::Polygon(poly) => {
                ui.label("Type: Polygon");
                let points = poly.to_egui_points();
                ui.label(format!("Vertices: {}", points.len()));
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Field assignment section
        ui.heading("Field Assignment");

        // Note: Field selector button handling moved to caller
        // to maintain state in FormFactorApp

        Ok(())
    }

    /// Render detection property editor
    #[instrument(skip(ui, canvas), fields(det_type = ?det_type, det_idx))]
    pub fn render_detection_properties(
        ui: &mut egui::Ui,
        canvas: &DrawingCanvas,
        det_type: DetectionType,
        det_idx: usize,
    ) -> Result<(), String> {
        tracing::debug!(det_type = ?det_type, det_idx, "Rendering detection properties");

        ui.label(format!("{:?} Detection #{}", det_type, det_idx));
        ui.add_space(8.0);

        // Show detection-specific information
        match det_type {
            DetectionType::Logo => {
                if canvas.detections().get(det_idx).is_some() {
                    ui.label("Logo detection");
                } else {
                    return Err(format!("Detection {} not found", det_idx));
                }
            }
            DetectionType::Text => {
                if canvas.detections().get(det_idx).is_some() {
                    ui.label("Text detection");
                } else {
                    return Err(format!("Detection {} not found", det_idx));
                }
            }
            DetectionType::Ocr => {
                if let Some((_shape, text)) = canvas.ocr_detections().get(det_idx) {
                    ui.label("Detected text:");
                    ui.text_edit_singleline(&mut text.clone());
                } else {
                    return Err(format!("OCR detection {} not found", det_idx));
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Field assignment section
        ui.heading("Field Assignment");

        // Note: Field selector button handling moved to caller
        // to maintain state in FormFactorApp

        Ok(())
    }
}
