//! Background detection task spawning

#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use form_factor::AppEvent;
#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
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

/// Logo detection task
#[cfg(feature = "logo-detection")]
pub struct LogoDetectionTask;

#[cfg(feature = "logo-detection")]
impl LogoDetectionTask {
    /// Spawn background thread for logo detection
    #[instrument(skip(sender), fields(form_path))]
    pub fn spawn(form_path: String, sender: EventSender) {
        tracing::info!("Spawning logo detection background task");

        std::thread::spawn(move || {
            tracing::debug!("Logo detection thread started");

            match Self::run_detection(&form_path) {
                Ok(shapes) => {
                    let count = shapes.len();
                    tracing::info!(count, "Logo detection complete");

                    // Serialize shapes as JSON
                    if let Ok(shapes_json) = serde_json::to_string(&shapes) {
                        sender.emit(AppEvent::DetectionResultsReady {
                            detection_type: "logo".to_string(),
                            shapes_json,
                        });
                    }

                    sender.emit(AppEvent::DetectionComplete {
                        count,
                        detection_type: "logo".to_string(),
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, "Logo detection failed");
                    sender.emit(AppEvent::DetectionFailed {
                        detection_type: "logo".to_string(),
                        error: e,
                    });
                }
            }
        });
    }

    /// Run logo detection on image
    #[instrument(fields(form_path))]
    fn run_detection(form_path: &str) -> Result<Vec<form_factor::Shape>, String> {
        use egui::{Color32, Pos2, Stroke};
        use form_factor::{Rectangle, Shape};
        use form_factor_cv::LogoDetector;

        tracing::debug!("Creating logo detector");

        // Create logo detector
        let mut detector = LogoDetector::builder()
            .template_matching()
            .with_confidence_threshold(0.5)
            .with_scales(vec![
                0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.65, 0.75, 1.0, 1.25, 1.5, 2.0,
            ])
            .build();

        // Load logo templates from logos directory
        let logos_dir = std::path::Path::new("logos");
        if !logos_dir.exists() {
            tracing::error!("logos directory does not exist");
            return Err("logos directory does not exist".to_string());
        }

        tracing::debug!("Loading logo templates from logos directory");

        let mut logo_count = 0;
        for entry in std::fs::read_dir(logos_dir).map_err(|e| {
            tracing::error!(error = %e, "Failed to read logos directory");
            format!("Failed to read logos directory: {}", e)
        })? {
            let entry = entry.map_err(|e| {
                tracing::error!(error = %e, "Failed to read directory entry");
                format!("Failed to read directory entry: {}", e)
            })?;
            let path = entry.path();
            if path.is_file() && let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" || ext_str == "webp"
                {
                    let logo_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    if let Err(e) = detector.add_logo(logo_name, &path) {
                        tracing::warn!(logo = logo_name, error = %e, "Failed to load logo");
                    } else {
                        logo_count += 1;
                    }
                }
            }
        }

        if logo_count == 0 {
            tracing::error!("No logo templates found in logos directory");
            return Err("No logo templates found in logos directory".to_string());
        }

        tracing::info!(count = logo_count, "Loaded logo templates");

        // Detect logos
        tracing::debug!("Running logo detection");
        let results = detector.detect_logos_from_path(form_path).map_err(|e| {
            tracing::error!(error = %e, "Detection failed");
            format!("Detection failed: {}", e)
        })?;

        tracing::debug!(count = results.len(), "Converting results to shapes");

        // Convert to shapes
        let mut shapes = Vec::new();
        for result in results.iter() {
            let top_left = Pos2::new(result.location.x as f32, result.location.y as f32);
            let bottom_right = Pos2::new(
                (result.location.x + result.size.width) as f32,
                (result.location.y + result.size.height) as f32,
            );

            let stroke = Stroke::new(2.0, Color32::from_rgb(0, 128, 255)); // Blue for logos
            let fill = Color32::TRANSPARENT;

            if let Ok(mut rect) = Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                rect.set_name(format!(
                    "Logo: {} ({:.2}%)",
                    result.logo_name,
                    result.confidence * 100.0
                ));
                shapes.push(Shape::Rectangle(rect));
            }
        }

        Ok(shapes)
    }
}
