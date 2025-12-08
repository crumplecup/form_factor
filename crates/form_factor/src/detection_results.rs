//! Detection result processing

#[cfg(any(
    feature = "text-detection",
    feature = "logo-detection",
    feature = "ocr"
))]
use form_factor_drawing::DrawingCanvas;

#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use form_factor_drawing::Shape;

/// Detection result handler
pub struct DetectionResultHandler;

impl DetectionResultHandler {
    /// Handle OCR complete
    #[cfg(feature = "ocr")]
    #[tracing::instrument(skip(canvas, toasts, results_json))]
    pub fn handle_ocr_complete(
        canvas: &mut DrawingCanvas,
        toasts: &mut egui_notify::Toasts,
        results_json: &str,
    ) {
        tracing::debug!("Processing OCR complete event");

        match serde_json::from_str::<Vec<(Shape, String)>>(results_json) {
            Ok(results) => {
                tracing::info!("Extracted text from {} detections", results.len());

                // Clear old OCR detections and add new ones with text
                canvas.clear_ocr_detections();
                for (shape, text) in results {
                    canvas.add_ocr_detection(shape, text);
                }

                // Show success toast
                let count = canvas.ocr_detections().len();
                toasts.success(format!(
                    "OCR complete: extracted text from {} region{}",
                    count,
                    if count == 1 { "" } else { "s" }
                ));
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to deserialize OCR results");
                toasts.error(format!("OCR processing failed: {}", e));
            }
        }
    }

    /// Handle detection complete
    #[tracing::instrument(skip(toasts), fields(detection_type, count))]
    pub fn handle_detection_complete(
        toasts: &mut egui_notify::Toasts,
        detection_type: &str,
        count: usize,
    ) {
        tracing::info!(detection_type, count, "Detection complete");

        let type_name = match detection_type {
            "text" => "Text",
            "logo" => "Logo",
            _ => "Detection",
        };

        toasts.success(format!(
            "{} detection complete: found {} region{}",
            type_name,
            count,
            if count == 1 { "" } else { "s" }
        ));
    }

    /// Handle detection failed
    #[tracing::instrument(skip(toasts), fields(detection_type, error))]
    pub fn handle_detection_failed(
        toasts: &mut egui_notify::Toasts,
        detection_type: &str,
        error: &str,
    ) {
        tracing::error!(detection_type, error, "Detection failed");

        let type_name = match detection_type {
            "text" => "Text",
            "logo" => "Logo",
            _ => "Detection",
        };

        toasts.error(format!("{} detection failed: {}", type_name, error));
    }

    /// Handle detection results ready
    #[cfg(any(feature = "text-detection", feature = "logo-detection"))]
    #[tracing::instrument(skip(canvas, shapes_json), fields(detection_type))]
    pub fn handle_detection_results_ready(
        canvas: &mut DrawingCanvas,
        detection_type: &str,
        shapes_json: &str,
    ) {
        tracing::debug!(detection_type, "Processing detection results");

        match serde_json::from_str::<Vec<Shape>>(shapes_json) {
            Ok(shapes) => {
                tracing::info!(
                    "Received {} {} detection results",
                    shapes.len(),
                    detection_type
                );
                for shape in shapes {
                    canvas.add_detection(shape);
                }
            }
            Err(e) => {
                tracing::error!(error = %e, detection_type, "Failed to deserialize detection results");
            }
        }
    }
}
