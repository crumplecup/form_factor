//! Background detection task spawning

#[cfg(feature = "text-detection")]
use form_factor::AppEvent;
#[cfg(feature = "text-detection")]
use form_factor_plugins::EventSender;
use tracing::instrument;

/// Text detection task
#[cfg(feature = "text-detection")]
pub struct TextDetectionTask;

#[cfg(feature = "text-detection")]
impl TextDetectionTask {
    /// Spawn background thread for text detection
    #[instrument(skip(sender), fields(form_path))]
    pub fn spawn(form_path: String, sender: EventSender) {
        tracing::info!("Spawning text detection background task");

        std::thread::spawn(move || {
            tracing::debug!("Text detection thread started");

            match Self::run_detection(&form_path) {
                Ok(shapes) => {
                    let count = shapes.len();
                    tracing::info!(count, "Text detection complete");

                    // Serialize shapes as JSON
                    if let Ok(shapes_json) = serde_json::to_string(&shapes) {
                        sender.emit(AppEvent::DetectionResultsReady {
                            detection_type: "text".to_string(),
                            shapes_json,
                        });
                    }

                    sender.emit(AppEvent::DetectionComplete {
                        count,
                        detection_type: "text".to_string(),
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, "Text detection failed");
                    sender.emit(AppEvent::DetectionFailed {
                        detection_type: "text".to_string(),
                        error: e,
                    });
                }
            }
        });
    }

    /// Run text detection on image
    #[instrument(fields(form_path))]
    fn run_detection(form_path: &str) -> Result<Vec<form_factor::Shape>, String> {
        use egui::{Color32, Pos2, Stroke};
        use form_factor::{Rectangle, Shape};
        use form_factor_cv::TextDetector;

        tracing::debug!("Creating text detector");

        // Create text detector
        let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create detector");
                format!("Failed to create detector: {}", e)
            })?;

        tracing::debug!("Running text detection");

        // Detect text regions
        let regions = detector.detect_from_file(form_path, 0.5).map_err(|e| {
            tracing::error!(error = %e, "Detection failed");
            format!("Detection failed: {}", e)
        })?;

        tracing::debug!(count = regions.len(), "Converting regions to shapes");

        // Convert to shapes
        let mut shapes = Vec::new();
        for (i, region) in regions.iter().enumerate() {
            let top_left = Pos2::new(*region.x() as f32, *region.y() as f32);
            let bottom_right = Pos2::new(
                (*region.x() + *region.width()) as f32,
                (*region.y() + *region.height()) as f32,
            );

            let stroke = Stroke::new(2.0, Color32::from_rgb(255, 165, 0));
            let fill = Color32::TRANSPARENT;

            if let Ok(mut rect) = Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                rect.set_name(format!(
                    "Text Region {} ({:.2}%)",
                    i + 1,
                    *region.confidence() * 100.0
                ));
                shapes.push(Shape::Rectangle(rect));
            }
        }

        Ok(shapes)
    }
}
