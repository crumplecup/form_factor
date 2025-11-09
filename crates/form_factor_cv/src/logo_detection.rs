//! Logo detection using OpenCV template and feature matching
//!
//! This module provides logo detection capabilities for identifying logos in form images.
//! It supports both template matching (fast, scale-sensitive) and feature matching
//! (slower, scale-invariant) methods.
//!
//! # Example
//!
//! ```no_run
//! use form_factor_cv::LogoDetector;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a detector with template matching
//! let mut detector = LogoDetector::builder()
//!     .template_matching()
//!     .with_confidence_threshold(0.8)
//!     .with_scales(vec![0.75, 1.0, 1.25])
//!     .build();
//!
//! // Add logos to detect
//! detector.add_logo("CompanyLogo", "logos/company.png")?;
//! detector.add_logo("BrandMark", "logos/brand.png")?;
//!
//! // Detect logos in an image
//! let results = detector.detect_logos_from_path("form.png")?;
//!
//! for result in results {
//!     println!("Found {} at ({}, {}) with confidence {:.2}%",
//!              result.logo_name, result.location.x, result.location.y,
//!              result.confidence * 100.0);
//! }
//! # Ok(())
//! # }
//! ```

use opencv::{
    core::{self, Mat, Point, Size, CV_32FC1},
    imgcodecs::{self, IMREAD_COLOR},
    imgproc::{self, TM_CCOEFF_NORMED},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument, trace, warn};

/// Method used for logo detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogoDetectionMethod {
    /// Template matching - fast, scale-sensitive
    /// Best for logos that appear at consistent sizes
    TemplateMatching,

    /// Feature matching - slower, scale-invariant
    /// Best for logos that may appear at different scales or rotations
    FeatureMatching,
}

/// A logo template for detection
#[derive(Debug, Clone)]
pub struct Logo {
    /// Name identifier for this logo
    pub name: String,

    /// Original logo image (color)
    image: Mat,

    /// Grayscale version (cached for performance)
    image_gray: Mat,
}

impl Logo {
    /// Create a new logo from an image file
    ///
    /// # Errors
    ///
    /// Returns an error if the image cannot be read or is invalid
    #[instrument(skip_all, fields(name, path))]
    pub fn from_file(name: impl Into<String>, path: impl AsRef<Path>) -> Result<Self, String> {
        let name = name.into();
        let path = path.as_ref();

        debug!("Loading logo '{}' from {:?}", name, path);

        // Read the logo image
        let image = imgcodecs::imread(
            path.to_str().ok_or("Invalid path encoding")?,
            IMREAD_COLOR,
        )
        .map_err(|e| format!("Failed to read logo image: {}", e))?;

        if image.empty() {
            return Err(format!("Logo image '{}' is empty or invalid", name));
        }

        // Convert to grayscale (cache for performance)
        let mut image_gray = Mat::default();
        imgproc::cvt_color(&image, &mut image_gray, imgproc::COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| format!("Failed to convert logo to grayscale: {}", e))?;

        info!(
            "Loaded logo '{}': {}x{} pixels",
            name,
            image.cols(),
            image.rows()
        );

        Ok(Logo {
            name,
            image,
            image_gray,
        })
    }

    /// Create a new logo from an existing Mat
    ///
    /// # Errors
    ///
    /// Returns an error if the Mat is empty or grayscale conversion fails
    #[instrument(skip(image), fields(name))]
    pub fn from_mat(name: impl Into<String>, image: Mat) -> Result<Self, String> {
        let name = name.into();
        let width = image.cols();
        let height = image.rows();

        if image.empty() {
            return Err(format!("Logo image '{}' is empty", name));
        }

        // Convert to grayscale
        let mut image_gray = Mat::default();
        imgproc::cvt_color(&image, &mut image_gray, imgproc::COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| format!("Failed to convert logo to grayscale: {}", e))?;

        debug!("Created logo '{}' from Mat: {}x{}", name, width, height);

        Ok(Logo {
            name,
            image,
            image_gray,
        })
    }

    /// Get the size of this logo
    pub fn size(&self) -> (i32, i32) {
        (self.image.cols(), self.image.rows())
    }
}

/// Result of logo detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoDetectionResult {
    /// Name of the detected logo
    pub logo_name: String,

    /// Location (top-left corner) in the image
    pub location: LogoLocation,

    /// Size of the detected logo
    pub size: LogoSize,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Scale at which the logo was detected (1.0 = original size)
    pub scale: f64,
}

/// Location of a detected logo
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LogoLocation {
    /// X coordinate of the top-left corner
    pub x: i32,
    /// Y coordinate of the top-left corner
    pub y: i32,
}

impl From<Point> for LogoLocation {
    fn from(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<LogoLocation> for Point {
    fn from(loc: LogoLocation) -> Self {
        Point::new(loc.x, loc.y)
    }
}

/// Size of a detected logo
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LogoSize {
    /// Width of the logo in pixels
    pub width: i32,
    /// Height of the logo in pixels
    pub height: i32,
}

/// Logo detector with configurable parameters
pub struct LogoDetector {
    logos: Vec<Logo>,
    method: LogoDetectionMethod,
    confidence_threshold: f64,
    scales: Vec<f64>,
}

impl LogoDetector {
    /// Create a new builder for configuring a logo detector
    pub fn builder() -> LogoDetectorBuilder {
        LogoDetectorBuilder::new()
    }

    /// Create a new logo detector with default settings
    ///
    /// Default settings:
    /// - Method: Template matching
    /// - Confidence threshold: 0.8
    /// - Scales: [0.5, 0.75, 1.0, 1.25, 1.5]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Add a logo template from a file path
    ///
    /// # Errors
    ///
    /// Returns an error if the logo image cannot be loaded
    #[instrument(skip(self), fields(name, path))]
    pub fn add_logo(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> Result<(), String> {
        let logo = Logo::from_file(name, path)?;
        info!("Added logo '{}' to detector", logo.name);
        self.logos.push(logo);
        Ok(())
    }

    /// Add a logo template from an existing Mat
    ///
    /// # Errors
    ///
    /// Returns an error if the Mat is invalid
    pub fn add_logo_from_mat(&mut self, name: impl Into<String>, image: Mat) -> Result<(), String> {
        let logo = Logo::from_mat(name, image)?;
        info!("Added logo '{}' to detector", logo.name);
        self.logos.push(logo);
        Ok(())
    }

    /// Remove a logo by name
    ///
    /// Returns true if the logo was found and removed
    pub fn remove_logo(&mut self, name: &str) -> bool {
        if let Some(pos) = self.logos.iter().position(|l| l.name == name) {
            self.logos.remove(pos);
            debug!("Removed logo '{}'", name);
            true
        } else {
            false
        }
    }

    /// Clear all logos from the detector
    pub fn clear_logos(&mut self) {
        let count = self.logos.len();
        self.logos.clear();
        debug!("Cleared {} logos from detector", count);
    }

    /// Get the names of all loaded logos
    pub fn logo_names(&self) -> Vec<&str> {
        self.logos.iter().map(|l| l.name.as_str()).collect()
    }

    /// Get the number of loaded logos
    pub fn logo_count(&self) -> usize {
        self.logos.len()
    }

    /// Detect logos in an image file
    ///
    /// # Errors
    ///
    /// Returns an error if the image cannot be read or detection fails
    #[instrument(skip(self), fields(path, logos = self.logos.len()))]
    pub fn detect_logos_from_path(&self, path: impl AsRef<Path>) -> Result<Vec<LogoDetectionResult>, String> {
        let path = path.as_ref();
        debug!("Loading image from {:?}", path);

        let image = imgcodecs::imread(
            path.to_str().ok_or("Invalid path encoding")?,
            IMREAD_COLOR,
        )
        .map_err(|e| format!("Failed to read input image: {}", e))?;

        if image.empty() {
            return Err("Input image is empty or invalid".to_string());
        }

        self.detect_logos(&image)
    }

    /// Detect logos in an image Mat
    ///
    /// # Errors
    ///
    /// Returns an error if the image is invalid or detection fails
    #[instrument(skip(self, image), fields(width = image.cols(), height = image.rows(), logos = self.logos.len()))]
    pub fn detect_logos(&self, image: &Mat) -> Result<Vec<LogoDetectionResult>, String> {
        if image.empty() {
            return Err("Input image is empty".to_string());
        }

        if self.logos.is_empty() {
            warn!("No logos loaded in detector");
            return Ok(Vec::new());
        }

        info!(
            "Detecting {} logos in {}x{} image using {:?}",
            self.logos.len(),
            image.cols(),
            image.rows(),
            self.method
        );

        // Convert input image to grayscale once (optimization)
        let mut image_gray = Mat::default();
        imgproc::cvt_color(image, &mut image_gray, imgproc::COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)
            .map_err(|e| format!("Failed to convert image to grayscale: {}", e))?;

        // Detect all logos
        let mut results = Vec::new();
        for logo in &self.logos {
            match self.detect_logo(&image_gray, logo) {
                Ok(mut logo_results) => {
                    debug!(
                        "Found {} instances of logo '{}'",
                        logo_results.len(),
                        logo.name
                    );
                    results.append(&mut logo_results);
                }
                Err(e) => {
                    warn!("Failed to detect logo '{}': {}", logo.name, e);
                }
            }
        }

        info!("Total detections: {}", results.len());
        Ok(results)
    }

    /// Detect a single logo in an image (all instances)
    #[instrument(skip(self, image_gray, logo), fields(logo_name = %logo.name))]
    fn detect_logo(&self, image_gray: &Mat, logo: &Logo) -> Result<Vec<LogoDetectionResult>, String> {
        match self.method {
            LogoDetectionMethod::TemplateMatching => {
                self.detect_logo_template_matching(image_gray, logo)
            }
            LogoDetectionMethod::FeatureMatching => {
                self.detect_logo_feature_matching(image_gray, logo)
            }
        }
    }

    /// Detect a logo using multi-scale template matching
    #[instrument(skip(self, image_gray, logo), fields(logo_name = %logo.name, scales = ?self.scales))]
    fn detect_logo_template_matching(
        &self,
        image_gray: &Mat,
        logo: &Logo,
    ) -> Result<Vec<LogoDetectionResult>, String> {
        let mut best_result: Option<LogoDetectionResult> = None;

        // Try each scale
        for &scale in &self.scales {
            trace!("Trying scale {:.2}", scale);

            // Skip if logo would be larger than image
            let scaled_width = (logo.image.cols() as f64 * scale) as i32;
            let scaled_height = (logo.image.rows() as f64 * scale) as i32;

            if scaled_width > image_gray.cols() || scaled_height > image_gray.rows() {
                trace!("Skipping scale {:.2} - logo too large", scale);
                continue;
            }

            // Resize logo template
            let mut logo_scaled = Mat::default();
            if (scale - 1.0).abs() < 0.01 {
                // Use original if scale is ~1.0
                logo_scaled = logo.image_gray.clone();
            } else {
                imgproc::resize(
                    &logo.image_gray,
                    &mut logo_scaled,
                    Size::default(),
                    scale,
                    scale,
                    imgproc::INTER_LINEAR,
                )
                .map_err(|e| format!("Failed to resize logo template: {}", e))?;
            }

            // Perform template matching
            let result_size = Size::new(
                image_gray.cols() - logo_scaled.cols() + 1,
                image_gray.rows() - logo_scaled.rows() + 1,
            );

            if result_size.width <= 0 || result_size.height <= 0 {
                continue;
            }

            let mut result = Mat::new_rows_cols_with_default(
                result_size.height,
                result_size.width,
                CV_32FC1,
                core::Scalar::all(0.0),
            )
            .map_err(|e| format!("Failed to create result matrix: {}", e))?;

            imgproc::match_template(
                image_gray,
                &logo_scaled,
                &mut result,
                TM_CCOEFF_NORMED,
                &core::no_array(),
            )
            .map_err(|e| format!("Failed to perform template matching: {}", e))?;

            // Find the maximum value
            let mut min_val = 0.0;
            let mut max_val = 0.0;
            let mut min_loc = Point::default();
            let mut max_loc = Point::default();

            core::min_max_loc(
                &result,
                Some(&mut min_val),
                Some(&mut max_val),
                Some(&mut min_loc),
                Some(&mut max_loc),
                &core::no_array(),
            )
            .map_err(|e| format!("Failed to find maximum value: {}", e))?;

            trace!(
                "Scale {:.2}: confidence = {:.4} at ({}, {})",
                scale,
                max_val,
                max_loc.x,
                max_loc.y
            );

            // Check if this is the best result so far
            if max_val >= self.confidence_threshold
                && (best_result.is_none() || max_val > best_result.as_ref().unwrap().confidence)
            {
                best_result = Some(LogoDetectionResult {
                    logo_name: logo.name.clone(),
                    location: max_loc.into(),
                    size: LogoSize {
                        width: logo_scaled.cols(),
                        height: logo_scaled.rows(),
                    },
                    confidence: max_val,
                    scale,
                });
            }
        }

        if let Some(result) = best_result {
            debug!(
                "Best match for '{}': confidence={:.4}, scale={:.2}",
                logo.name, result.confidence, result.scale
            );
            Ok(vec![result])
        } else {
            debug!("No match found for '{}' above threshold {}", logo.name, self.confidence_threshold);
            Ok(Vec::new())
        }
    }

    /// Detect a logo using feature matching
    ///
    /// Note: This is a placeholder for future implementation
    fn detect_logo_feature_matching(
        &self,
        _image_gray: &Mat,
        logo: &Logo,
    ) -> Result<Vec<LogoDetectionResult>, String> {
        warn!(
            "Feature matching not yet implemented for logo '{}'",
            logo.name
        );
        Ok(Vec::new())
    }
}

impl Default for LogoDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring a LogoDetector
pub struct LogoDetectorBuilder {
    method: LogoDetectionMethod,
    confidence_threshold: f64,
    scales: Vec<f64>,
}

impl LogoDetectorBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            method: LogoDetectionMethod::TemplateMatching,
            confidence_threshold: 0.8,
            scales: vec![0.5, 0.75, 1.0, 1.25, 1.5],
        }
    }

    /// Use template matching method
    pub fn template_matching(mut self) -> Self {
        self.method = LogoDetectionMethod::TemplateMatching;
        self
    }

    /// Use feature matching method
    pub fn feature_matching(mut self) -> Self {
        self.method = LogoDetectionMethod::FeatureMatching;
        self
    }

    /// Set the detection method
    pub fn with_method(mut self, method: LogoDetectionMethod) -> Self {
        self.method = method;
        self
    }

    /// Set the minimum confidence threshold (0.0-1.0)
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set the scales to try for multi-scale detection
    ///
    /// Default: [0.5, 0.75, 1.0, 1.25, 1.5]
    pub fn with_scales(mut self, scales: Vec<f64>) -> Self {
        self.scales = scales;
        self
    }

    /// Add a single scale to the list
    pub fn add_scale(mut self, scale: f64) -> Self {
        self.scales.push(scale);
        self
    }

    /// Build the LogoDetector
    pub fn build(self) -> LogoDetector {
        LogoDetector {
            logos: Vec::new(),
            method: self.method,
            confidence_threshold: self.confidence_threshold,
            scales: self.scales,
        }
    }
}

impl Default for LogoDetectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let detector = LogoDetector::builder()
            .template_matching()
            .with_confidence_threshold(0.9)
            .with_scales(vec![1.0])
            .build();

        assert_eq!(detector.method, LogoDetectionMethod::TemplateMatching);
        assert_eq!(detector.confidence_threshold, 0.9);
        assert_eq!(detector.scales, vec![1.0]);
    }

    #[test]
    fn test_logo_management() {
        let detector = LogoDetector::new();
        assert_eq!(detector.logo_count(), 0);

        // Note: Can't test add_logo without actual image files
        // In real tests, you would use test fixtures

        assert!(detector.logo_names().is_empty());
    }

    #[test]
    #[ignore = "Requires logos directory with actual logo files"]
    fn test_logo_self_detection() {
        use std::fs;

        // Path to the logos directory (relative to workspace root)
        let logos_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("logos");

        // Skip test if logos directory doesn't exist
        if !logos_dir.exists() {
            eprintln!("Skipping test: logos directory not found at {:?}", logos_dir);
            return;
        }

        // Read all logo files from the directory
        let logo_files: Vec<_> = fs::read_dir(&logos_dir)
            .expect("Failed to read logos directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                // Only process image files (png, jpg, jpeg)
                if path.is_file() {
                    let ext = path.extension()?.to_str()?.to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        return Some(path);
                    }
                }
                None
            })
            .collect();

        // Skip test if no logos found
        if logo_files.is_empty() {
            eprintln!("Skipping test: no logo files found in {:?}", logos_dir);
            return;
        }

        eprintln!("Testing {} logo files", logo_files.len());

        // Test each logo
        for logo_path in logo_files {
            let logo_name = logo_path.file_stem()
                .unwrap()
                .to_str()
                .unwrap();

            eprintln!("\nTesting logo: {}", logo_name);

            // Create a new detector for this logo
            let mut detector = LogoDetector::builder()
                .template_matching()  // Use template matching for exact match
                .with_confidence_threshold(0.8)  // High threshold - should be near perfect
                .with_scales(vec![1.0])  // Only test at original scale
                .build();

            // Add the logo as a template
            detector.add_logo(logo_name, &logo_path)
                .expect("Failed to add logo");

            // Try to detect the logo in itself
            let results = detector.detect_logos_from_path(&logo_path)
                .expect("Failed to detect logos");

            // Verify we found at least one detection
            assert!(
                !results.is_empty(),
                "Logo '{}' should be able to detect itself, but found no detections",
                logo_name
            );

            // Verify high confidence
            let best_result = &results[0];
            assert!(
                best_result.confidence >= 0.95,
                "Logo '{}' self-detection confidence ({:.4}) should be >= 0.95 (near perfect match)",
                logo_name,
                best_result.confidence
            );

            // Verify the detected logo name matches
            assert_eq!(
                best_result.logo_name, logo_name,
                "Detected logo name should match"
            );

            eprintln!(
                "  ✓ Self-detection successful: confidence={:.4}, location=({}, {})",
                best_result.confidence,
                best_result.location.x,
                best_result.location.y
            );
        }
    }
}
