//! Text detection using OpenCV DB (Differentiable Binarization) model

use opencv::{
    core::{Mat, Point2f, RotatedRect, Vector},
    dnn::TextDetectionModel_DB,
    imgcodecs,
    prelude::*,
};
use tracing::debug;

/// A detected text region
#[derive(Debug, Clone)]
pub struct TextRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub confidence: f32,
}

/// Text detector using DB (Differentiable Binarization) model
pub struct TextDetector {
    model_path: String,
    binary_threshold: f32,
    polygon_threshold: f32,
    unclip_ratio: f64,
    max_candidates: i32,
}

impl TextDetector {
    /// Create a new text detector with path to DB model file
    pub fn new(model_path: String) -> Self {
        Self {
            model_path,
            binary_threshold: 0.3,
            polygon_threshold: 0.5,
            unclip_ratio: 2.0,
            max_candidates: 200,
        }
    }

    /// Set the binary threshold (default: 0.3)
    pub fn with_binary_threshold(mut self, threshold: f32) -> Self {
        self.binary_threshold = threshold;
        self
    }

    /// Set the polygon threshold (default: 0.5)
    pub fn with_polygon_threshold(mut self, threshold: f32) -> Self {
        self.polygon_threshold = threshold;
        self
    }

    /// Set the unclip ratio (default: 2.0)
    pub fn with_unclip_ratio(mut self, ratio: f64) -> Self {
        self.unclip_ratio = ratio;
        self
    }

    /// Set the max candidates (default: 200)
    pub fn with_max_candidates(mut self, max: i32) -> Self {
        self.max_candidates = max;
        self
    }

    /// Detect text regions in an image file
    pub fn detect_from_file(&self, image_path: &str, confidence_threshold: f32) -> Result<Vec<TextRegion>, String> {
        debug!("Loading image from: {}", image_path);

        // Load the input image
        let image = imgcodecs::imread(image_path, imgcodecs::IMREAD_COLOR)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        if image.empty() {
            return Err("Loaded image is empty".to_string());
        }

        self.detect_from_mat(&image, confidence_threshold)
    }

    /// Detect text regions in an OpenCV Mat
    pub fn detect_from_mat(&self, image: &Mat, confidence_threshold: f32) -> Result<Vec<TextRegion>, String> {
        debug!("Loading DB model from: {}", self.model_path);

        // Load the pre-trained DB model (ONNX format)
        let mut detector = TextDetectionModel_DB::new_1(&self.model_path, "")
            .map_err(|e| format!("Failed to load DB model: {}", e))?;

        // Configure the detector
        detector.set_binary_threshold(self.binary_threshold)
            .map_err(|e| format!("Failed to set binary threshold: {}", e))?;
        detector.set_polygon_threshold(self.polygon_threshold)
            .map_err(|e| format!("Failed to set polygon threshold: {}", e))?;
        detector.set_unclip_ratio(self.unclip_ratio)
            .map_err(|e| format!("Failed to set unclip ratio: {}", e))?;
        detector.set_max_candidates(self.max_candidates)
            .map_err(|e| format!("Failed to set max candidates: {}", e))?;

        debug!("Running text detection...");

        // Detect text regions
        let mut detections = Vector::<RotatedRect>::new();
        let mut confidences = Vector::<f32>::new();

        detector.detect_text_rectangles(image, &mut detections, &mut confidences)
            .map_err(|e| format!("Failed to detect text: {}", e))?;

        debug!("Detected {} text regions", detections.len());

        // Convert RotatedRect to TextRegion and filter by confidence
        let mut regions = Vec::new();
        for i in 0..detections.len() {
            let rect = detections.get(i)
                .map_err(|e| format!("Failed to get detection {}: {}", i, e))?;
            let confidence = confidences.get(i)
                .map_err(|e| format!("Failed to get confidence {}: {}", i, e))?;

            if confidence < confidence_threshold {
                continue;
            }

            // Convert rotated rect to axis-aligned bounding box
            let mut points = [Point2f::default(); 4];
            rect.points(&mut points)
                .map_err(|e| format!("Failed to get rect points: {}", e))?;

            // Find bounding box
            let min_x = points.iter().map(|p| p.x as i32).min().unwrap_or(0);
            let max_x = points.iter().map(|p| p.x as i32).max().unwrap_or(0);
            let min_y = points.iter().map(|p| p.y as i32).min().unwrap_or(0);
            let max_y = points.iter().map(|p| p.y as i32).max().unwrap_or(0);

            regions.push(TextRegion {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
                confidence,
            });
        }

        Ok(regions)
    }
}

impl Default for TextDetector {
    fn default() -> Self {
        Self::new(String::from("models/DB_TD500_resnet50.onnx"))
    }
}
