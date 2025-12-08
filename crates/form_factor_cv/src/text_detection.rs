//! Text detection using OpenCV DB (Differentiable Binarization) model
//!
//! This module provides text detection capabilities using the DB (Differentiable Binarization)
//! model, which is optimized for scene text detection in natural images and document scans.
//!
//! # Examples
//!
//! ```no_run
//! use form_factor_cv::TextDetector;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())?
//!     .with_binary_threshold(0.3)?
//!     .with_polygon_threshold(0.5)?;
//!
//! let regions = detector.detect_from_file("form.png", 0.7)?;
//! println!("Found {} text regions", regions.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Model Requirements
//!
//! The detector requires a pre-trained DB model in ONNX format. The model should be
//! compatible with OpenCV's DNN text detection API.

use derive_getters::Getters;
use opencv::{
    core::{Mat, Point2f, RotatedRect, Scalar, Size, Vector},
    dnn::TextDetectionModel_DB,
    imgcodecs,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, instrument};

// ============================================================================
// Constants
// ============================================================================

/// Default binary threshold for text detection
const DEFAULT_BINARY_THRESHOLD: f32 = 0.3;

/// Default polygon threshold for text detection
const DEFAULT_POLYGON_THRESHOLD: f32 = 0.5;

/// Default unclip ratio for expanding detected regions
const DEFAULT_UNCLIP_RATIO: f64 = 2.0;

/// Default maximum number of candidate regions
const DEFAULT_MAX_CANDIDATES: i32 = 200;

/// Input size for DB model (models typically use 736x736)
const DB_INPUT_SIZE: i32 = 736;

/// ImageNet mean values for BGR channels (used for normalization)
const IMAGENET_MEAN_BGR: [f64; 3] = [122.67891434, 116.66876762, 104.00698793];

/// Scale factor for input normalization
const INPUT_SCALE: f64 = 1.0 / 255.0;

// ============================================================================
// Error Types
// ============================================================================

/// Kinds of errors that can occur during text detection
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum TextDetectionErrorKind {
    /// Failed to load image file
    #[display("Failed to load image: {}", _0)]
    ImageLoad(String),
    /// Image is empty or corrupted
    #[display("Image is empty")]
    ImageEmpty,
    /// Failed to load or configure detection model
    #[display("Failed to load model: {}", _0)]
    ModelLoad(String),
    /// Detection operation failed
    #[display("Detection failed: {}", _0)]
    Detection(String),
    /// Invalid parameter value
    #[display("Invalid parameter: {}", _0)]
    InvalidParameter(String),
}

/// Text detection error with location information
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("Text Detection: {} at {}:{}", kind, file, line)]
pub struct TextDetectionError {
    /// Error category
    pub kind: TextDetectionErrorKind,
    /// Line number where error occurred
    pub line: u32,
    /// File where error occurred
    pub file: &'static str,
}

impl TextDetectionError {
    /// Create a new text detection error
    #[track_caller]
    pub fn new(kind: TextDetectionErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

// ============================================================================
// Text Region
// ============================================================================

/// A detected text region
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct TextRegion {
    /// X coordinate of the top-left corner
    #[serde(default)]
    x: i32,
    /// Y coordinate of the top-left corner
    #[serde(default)]
    y: i32,
    /// Width of the text region in pixels
    #[serde(default)]
    width: i32,
    /// Height of the text region in pixels
    #[serde(default)]
    height: i32,
    /// Detection confidence score between 0.0 and 1.0
    #[serde(default)]
    confidence: f32,
}

impl TextRegion {
    /// Create a new text region
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Width or height is negative
    /// - Confidence is not in range [0.0, 1.0]
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        confidence: f32,
    ) -> Result<Self, TextDetectionError> {
        if width < 0 {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Width cannot be negative: {}",
                    width
                )),
            ));
        }
        if height < 0 {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Height cannot be negative: {}",
                    height
                )),
            ));
        }
        if !(0.0..=1.0).contains(&confidence) {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Confidence must be between 0.0 and 1.0, got: {}",
                    confidence
                )),
            ));
        }

        Ok(Self {
            x,
            y,
            width,
            height,
            confidence,
        })
    }
}

// ============================================================================
// Text Detector
// ============================================================================

/// Text detector using DB (Differentiable Binarization) model
///
/// The detector loads and configures a DB model once during construction,
/// which improves performance for multiple detection calls.
pub struct TextDetector {
    detector: TextDetectionModel_DB,
    binary_threshold: f32,
    polygon_threshold: f32,
    unclip_ratio: f64,
    max_candidates: i32,
}

impl TextDetector {
    /// Create a new text detector with path to DB model file
    ///
    /// Loads and configures the model with default parameters.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Model file cannot be loaded
    /// - Model configuration fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use form_factor_cv::TextDetector;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, fields(model_path = %model_path))]
    pub fn new(model_path: String) -> Result<Self, TextDetectionError> {
        debug!(model_path = %model_path, "Loading DB model");

        let mut detector = TextDetectionModel_DB::new_1(&model_path, "").map_err(|e| {
            TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!("{}", e)))
        })?;

        // Configure input parameters (done once)
        detector
            .set_input_params(
                INPUT_SCALE,
                Size::new(DB_INPUT_SIZE, DB_INPUT_SIZE),
                Scalar::new(
                    IMAGENET_MEAN_BGR[0],
                    IMAGENET_MEAN_BGR[1],
                    IMAGENET_MEAN_BGR[2],
                    0.0,
                ),
                true,  // swapRB
                false, // crop
            )
            .map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!(
                    "Failed to set input params: {}",
                    e
                )))
            })?;

        debug!("DB model loaded successfully");

        Ok(Self {
            detector,
            binary_threshold: DEFAULT_BINARY_THRESHOLD,
            polygon_threshold: DEFAULT_POLYGON_THRESHOLD,
            unclip_ratio: DEFAULT_UNCLIP_RATIO,
            max_candidates: DEFAULT_MAX_CANDIDATES,
        })
    }

    /// Set the binary threshold (default: 0.3)
    ///
    /// # Errors
    ///
    /// Returns error if threshold is not in range [0.0, 1.0]
    pub fn with_binary_threshold(mut self, threshold: f32) -> Result<Self, TextDetectionError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Binary threshold must be between 0.0 and 1.0, got: {}",
                    threshold
                )),
            ));
        }
        self.binary_threshold = threshold;
        Ok(self)
    }

    /// Set the polygon threshold (default: 0.5)
    ///
    /// # Errors
    ///
    /// Returns error if threshold is not in range [0.0, 1.0]
    pub fn with_polygon_threshold(mut self, threshold: f32) -> Result<Self, TextDetectionError> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Polygon threshold must be between 0.0 and 1.0, got: {}",
                    threshold
                )),
            ));
        }
        self.polygon_threshold = threshold;
        Ok(self)
    }

    /// Set the unclip ratio (default: 2.0)
    ///
    /// # Errors
    ///
    /// Returns error if ratio is not positive
    pub fn with_unclip_ratio(mut self, ratio: f64) -> Result<Self, TextDetectionError> {
        if ratio <= 0.0 {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Unclip ratio must be positive, got: {}",
                    ratio
                )),
            ));
        }
        self.unclip_ratio = ratio;
        Ok(self)
    }

    /// Set the max candidates (default: 200)
    ///
    /// # Errors
    ///
    /// Returns error if max is not positive
    pub fn with_max_candidates(mut self, max: i32) -> Result<Self, TextDetectionError> {
        if max <= 0 {
            return Err(TextDetectionError::new(
                TextDetectionErrorKind::InvalidParameter(format!(
                    "Max candidates must be positive, got: {}",
                    max
                )),
            ));
        }
        self.max_candidates = max;
        Ok(self)
    }

    /// Detect text regions in an image file
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Image file cannot be read
    /// - Image is empty or corrupted
    /// - Detection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use form_factor_cv::TextDetector;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())?;
    /// let regions = detector.detect_from_file("scan.png", 0.7)?;
    /// println!("Found {} text regions", regions.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(image_path, confidence_threshold))]
    pub fn detect_from_file(
        &self,
        image_path: impl AsRef<Path>,
        confidence_threshold: f32,
    ) -> Result<Vec<TextRegion>, TextDetectionError> {
        let path = image_path.as_ref();
        debug!(path = ?path, "Loading image");

        // Load the input image
        let image = imgcodecs::imread(
            path.to_str().ok_or_else(|| {
                TextDetectionError::new(TextDetectionErrorKind::ImageLoad(
                    "Invalid UTF-8 in path".to_string(),
                ))
            })?,
            imgcodecs::IMREAD_COLOR,
        )
        .map_err(|e| {
            TextDetectionError::new(TextDetectionErrorKind::ImageLoad(format!("{}", e)))
        })?;

        if image.empty() {
            return Err(TextDetectionError::new(TextDetectionErrorKind::ImageEmpty));
        }

        self.detect_from_mat(&image, confidence_threshold)
    }

    /// Detect text regions in an OpenCV Mat
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Model configuration fails
    /// - Detection operation fails
    /// - Invalid text regions are detected
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use form_factor_cv::TextDetector;
    /// use opencv::{core::Mat, imgcodecs};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())?;
    /// let image = imgcodecs::imread("scan.png", imgcodecs::IMREAD_COLOR)?;
    /// let regions = detector.detect_from_mat(&image, 0.7)?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, image), fields(
        confidence_threshold,
        image_size = ?(image.rows(), image.cols())
    ))]
    pub fn detect_from_mat(
        &self,
        image: &Mat,
        confidence_threshold: f32,
    ) -> Result<Vec<TextRegion>, TextDetectionError> {
        debug!("Running text detection");

        // Need mutable reference to detector for detection
        let mut detector = self.detector.clone();

        // Configure the detector with stored parameters
        detector
            .set_binary_threshold(self.binary_threshold)
            .map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!(
                    "Failed to set binary threshold: {}",
                    e
                )))
            })?;
        detector
            .set_polygon_threshold(self.polygon_threshold)
            .map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!(
                    "Failed to set polygon threshold: {}",
                    e
                )))
            })?;
        detector.set_unclip_ratio(self.unclip_ratio).map_err(|e| {
            TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!(
                "Failed to set unclip ratio: {}",
                e
            )))
        })?;
        detector
            .set_max_candidates(self.max_candidates)
            .map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::ModelLoad(format!(
                    "Failed to set max candidates: {}",
                    e
                )))
            })?;

        // Detect text regions
        let mut detections = Vector::<RotatedRect>::new();
        let mut confidences = Vector::<f32>::new();

        detector
            .detect_text_rectangles(image, &mut detections, &mut confidences)
            .map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::Detection(format!("{}", e)))
            })?;

        debug!(count = detections.len(), "Text detection complete");

        // Convert RotatedRect to TextRegion and filter by confidence
        let mut regions = Vec::new();
        for i in 0..detections.len() {
            let rect = detections.get(i).map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::Detection(format!(
                    "Failed to get detection {}: {}",
                    i, e
                )))
            })?;
            let confidence = confidences.get(i).map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::Detection(format!(
                    "Failed to get confidence {}: {}",
                    i, e
                )))
            })?;

            if confidence < confidence_threshold {
                continue;
            }

            // Convert rotated rect to axis-aligned bounding box
            let mut points = [Point2f::default(); 4];
            rect.points(&mut points).map_err(|e| {
                TextDetectionError::new(TextDetectionErrorKind::Detection(format!(
                    "Failed to get rect points: {}",
                    e
                )))
            })?;

            // Find bounding box
            let min_x = points.iter().map(|p| p.x as i32).min().unwrap_or(0);
            let max_x = points.iter().map(|p| p.x as i32).max().unwrap_or(0);
            let min_y = points.iter().map(|p| p.y as i32).min().unwrap_or(0);
            let max_y = points.iter().map(|p| p.y as i32).max().unwrap_or(0);

            // Clamp confidence to valid range
            let clamped_confidence = confidence.clamp(0.0, 1.0);

            // Create validated text region
            let region = TextRegion::new(
                min_x,
                min_y,
                max_x - min_x,
                max_y - min_y,
                clamped_confidence,
            )?;
            regions.push(region);
        }

        Ok(regions)
    }
}
