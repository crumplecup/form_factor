//! Text detection using OpenCV EAST model

use opencv::{
    core::{Mat, Scalar, Size, Vector},
    dnn::{blob_from_image, read_net_from_caffe, NetTrait},
    imgcodecs, imgproc,
    prelude::*,
};
use tracing::{debug, warn};

/// A detected text region
#[derive(Debug, Clone)]
pub struct TextRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub confidence: f32,
}

/// Text detector using EAST model
pub struct TextDetector {
    model_config: String,
    model_weights: String,
}

impl TextDetector {
    /// Create a new text detector with paths to EAST model files
    pub fn new(model_config: String, model_weights: String) -> Self {
        Self {
            model_config,
            model_weights,
        }
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
        debug!("Loading EAST model from: {} and {}", self.model_config, self.model_weights);

        // Load the pre-trained EAST model
        let mut net = read_net_from_caffe(&self.model_config, &self.model_weights)
            .map_err(|e| format!("Failed to load EAST model: {}", e))?;

        // Get the image dimensions
        let orig_height = image.rows() as f32;
        let orig_width = image.cols() as f32;

        debug!("Image dimensions: {}x{}", orig_width, orig_height);

        // Create a blob from the image
        // EAST model expects 320x320 input
        let blob = blob_from_image(
            image,
            1.0,
            Size::new(320, 320),
            Scalar::new(123.68, 116.78, 103.94, 0.0),
            true,
            false,
            opencv::core::CV_32F,
        )
        .map_err(|e| format!("Failed to create blob: {}", e))?;

        // Set the blob as the input for the network
        net.set_input(&blob, "", 1.0, Scalar::all(0.0))
            .map_err(|e| format!("Failed to set input: {}", e))?;

        // Run the forward pass
        let mut scores = Mat::default();
        let mut geometry = Mat::default();
        let output_layers = Vector::from_slice(&[
            "feature_fusion/Conv_7/Sigmoid",
            "feature_fusion/concat_3",
        ]);

        let mut outs = Vector::new();
        net.forward(&mut outs, &output_layers)
            .map_err(|e| format!("Failed to run forward pass: {}", e))?;

        if outs.len() < 2 {
            return Err("Expected 2 output layers from EAST model".to_string());
        }

        scores = outs.get(0).map_err(|e| format!("Failed to get scores: {}", e))?;
        geometry = outs.get(1).map_err(|e| format!("Failed to get geometry: {}", e))?;

        debug!("Scores shape: {:?}", scores.size());
        debug!("Geometry shape: {:?}", geometry.size());

        // Decode the predictions
        let regions = self.decode_predictions(&scores, &geometry, confidence_threshold, orig_width, orig_height)
            .map_err(|e| format!("Failed to decode predictions: {}", e))?;

        debug!("Detected {} text regions", regions.len());

        Ok(regions)
    }

    /// Decode EAST model predictions into text regions
    fn decode_predictions(
        &self,
        scores: &Mat,
        geometry: &Mat,
        confidence_threshold: f32,
        orig_width: f32,
        orig_height: f32,
    ) -> Result<Vec<TextRegion>, String> {
        let mut regions = Vec::new();

        let scores_size = scores.size().map_err(|e| format!("Failed to get scores size: {}", e))?;
        let num_rows = scores_size[2] as i32;
        let num_cols = scores_size[3] as i32;

        debug!("Decoding predictions: {} rows x {} cols", num_rows, num_cols);

        // Scale factors for mapping from model output to original image
        let scale_x = orig_width / 320.0;
        let scale_y = orig_height / 320.0;

        // Iterate over each position in the score map
        for y in 0..num_rows {
            for x in 0..num_cols {
                // Get the confidence score at this position
                let score = self.get_score(scores, y, x)
                    .map_err(|e| format!("Failed to get score at ({}, {}): {}", x, y, e))?;

                if score < confidence_threshold {
                    continue;
                }

                // Get the geometry data (distances to box edges)
                let offset_x = x as f32 * 4.0;
                let offset_y = y as f32 * 4.0;

                // EAST geometry output: [top, right, bottom, left] distances
                let (h, w) = self.get_geometry(geometry, y, x)
                    .map_err(|e| format!("Failed to get geometry at ({}, {}): {}", x, y, e))?;

                // Calculate bounding box in original image coordinates
                let box_x = ((offset_x - w) * scale_x) as i32;
                let box_y = ((offset_y - h) * scale_y) as i32;
                let box_width = (w * 2.0 * scale_x) as i32;
                let box_height = (h * 2.0 * scale_y) as i32;

                regions.push(TextRegion {
                    x: box_x.max(0),
                    y: box_y.max(0),
                    width: box_width.min(orig_width as i32 - box_x),
                    height: box_height.min(orig_height as i32 - box_y),
                    confidence: score,
                });
            }
        }

        // Apply non-maximum suppression to remove overlapping boxes
        let regions = self.non_max_suppression(regions, 0.4);

        Ok(regions)
    }

    /// Get confidence score at a specific position in the score map
    fn get_score(&self, scores: &Mat, y: i32, x: i32) -> Result<f32, String> {
        unsafe {
            // scores is shape [1, 1, H, W]
            let data = scores.ptr(0).map_err(|e| format!("Failed to get scores ptr: {}", e))?;
            let width = scores.size().map_err(|e| format!("Failed to get size: {}", e))?[3] as i32;
            let idx = (y * width + x) as isize;
            let score_ptr = data.offset(idx) as *const f32;
            Ok(*score_ptr)
        }
    }

    /// Get geometry data at a specific position
    fn get_geometry(&self, geometry: &Mat, y: i32, x: i32) -> Result<(f32, f32), String> {
        unsafe {
            // geometry is shape [1, 5, H, W] where the 5 channels are:
            // [distance_top, distance_right, distance_bottom, distance_left, rotation_angle]
            let size = geometry.size().map_err(|e| format!("Failed to get size: {}", e))?;
            let height = size[2] as i32;
            let width = size[3] as i32;
            let channel_size = (height * width) as isize;

            let data = geometry.ptr(0).map_err(|e| format!("Failed to get geometry ptr: {}", e))?;
            let idx = (y * width + x) as isize;

            // Get distances
            let top_ptr = data.offset(idx) as *const f32;
            let right_ptr = data.offset(channel_size + idx) as *const f32;
            let bottom_ptr = data.offset(2 * channel_size + idx) as *const f32;
            let left_ptr = data.offset(3 * channel_size + idx) as *const f32;

            let h = (*top_ptr + *bottom_ptr) / 2.0;
            let w = (*left_ptr + *right_ptr) / 2.0;

            Ok((h, w))
        }
    }

    /// Apply non-maximum suppression to remove overlapping detections
    fn non_max_suppression(&self, mut regions: Vec<TextRegion>, iou_threshold: f32) -> Vec<TextRegion> {
        if regions.is_empty() {
            return regions;
        }

        // Sort by confidence (descending)
        regions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut keep = Vec::new();
        let mut suppressed = vec![false; regions.len()];

        for i in 0..regions.len() {
            if suppressed[i] {
                continue;
            }

            keep.push(regions[i].clone());

            // Suppress overlapping boxes
            for j in (i + 1)..regions.len() {
                if suppressed[j] {
                    continue;
                }

                let iou = self.compute_iou(&regions[i], &regions[j]);
                if iou > iou_threshold {
                    suppressed[j] = true;
                }
            }
        }

        keep
    }

    /// Compute Intersection over Union (IoU) between two regions
    fn compute_iou(&self, a: &TextRegion, b: &TextRegion) -> f32 {
        let x1 = a.x.max(b.x);
        let y1 = a.y.max(b.y);
        let x2 = (a.x + a.width).min(b.x + b.width);
        let y2 = (a.y + a.height).min(b.y + b.height);

        if x2 < x1 || y2 < y1 {
            return 0.0;
        }

        let intersection = ((x2 - x1) * (y2 - y1)) as f32;
        let area_a = (a.width * a.height) as f32;
        let area_b = (b.width * b.height) as f32;
        let union = area_a + area_b - intersection;

        if union <= 0.0 {
            return 0.0;
        }

        intersection / union
    }
}

impl Default for TextDetector {
    fn default() -> Self {
        Self {
            model_config: String::from("models/frozen_east_text_detection.pb.prototxt"),
            model_weights: String::from("models/frozen_east_text_detection.pb"),
        }
    }
}
