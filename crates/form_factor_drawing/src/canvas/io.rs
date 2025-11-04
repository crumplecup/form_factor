//! Canvas I/O operations
//!
//! This module contains all file I/O, image loading, and persistence operations
//! for the drawing canvas, including:
//! - Clearing operations (shapes, detections, images)
//! - Form image loading and management
//! - Project state serialization/deserialization
//! - Recent project tracking
//! - Text detection integration (with feature flag)
//! - OCR text extraction (with feature flag)

use super::core::{CanvasError, CanvasErrorKind, DrawingCanvas};
use crate::{LayerType, RecentProjects};
#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use crate::{Rectangle, Shape};
#[cfg(feature = "text-detection")]
use form_factor_cv::TextDetector;
#[cfg(feature = "logo-detection")]
use form_factor_cv::LogoDetector;
#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use egui::{Color32, Pos2, Stroke};
use std::path::PathBuf;
use tracing::{debug, instrument, warn};
#[cfg(any(feature = "text-detection", feature = "logo-detection"))]
use tracing::trace;

impl DrawingCanvas {
    /// Clear all shapes and detections from the canvas
    pub fn clear(&mut self) {
        debug!("Clearing canvas: shapes={}, detections={}", self.shapes.len(), self.detections.len());
        self.shapes.clear();
        self.detections.clear();
    }

    /// Clear only shapes from the canvas
    pub fn clear_shapes(&mut self) {
        debug!("Clearing shapes: count={}", self.shapes.len());
        self.shapes.clear();
        self.selected_shape = None;
    }

    /// Clear only detections from the canvas
    pub fn clear_detections(&mut self) {
        debug!("Clearing detections: count={}", self.detections.len());
        self.detections.clear();
    }

    /// Clear the canvas image (form image)
    pub fn clear_canvas_image(&mut self) {
        debug!("Clearing canvas image: path={:?}", self.form_image_path);
        self.form_image = None;
        self.form_image_size = None;
        self.form_image_path = None;
        self.pending_image_load = None;
    }

    /// Clear the loaded form image
    pub fn clear_form_image(&mut self) {
        self.form_image = None;
        self.form_image_size = None;
        self.form_image_path = None;
    }

    /// Load a form image from a file path
    pub fn load_form_image(&mut self, path: &str, ctx: &egui::Context) -> Result<(), CanvasError> {
        // Load the image from disk
        let img = image::open(path).map_err(|e| {
            CanvasError::new(CanvasErrorKind::ImageLoad(e.to_string()), line!(), file!())
        })?;

        // Convert to RGBA8
        let size = [img.width() as usize, img.height() as usize];
        let img_rgba = img.to_rgba8();
        let pixels = img_rgba.as_flat_samples();

        // Create egui ColorImage
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        // Create texture
        let texture = ctx.load_texture(
            "form_image",
            color_image,
            egui::TextureOptions::default(),
        );

        // Store the texture and metadata
        self.form_image_size = Some(egui::Vec2::new(img.width() as f32, img.height() as f32));
        self.form_image = Some(texture);
        self.form_image_path = Some(path.to_string());

        // Reset zoom and pan to fit image to window
        self.zoom_level = 1.0;
        self.pan_offset = egui::Vec2::ZERO;

        tracing::info!("Loaded form image: {} ({}x{})", path, img.width(), img.height());
        Ok(())
    }

    /// Save the project state to a file
    #[instrument(skip(self), fields(path, shapes = self.shapes.len(), detections = self.detections.len()))]
    pub fn save_to_file(&self, path: &str) -> Result<(), CanvasError> {
        debug!("Saving project: shapes={}, detections={}", self.shapes.len(), self.detections.len());

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            CanvasError::new(CanvasErrorKind::Serialization(e.to_string()), line!(), file!())
        })?;

        std::fs::write(path, json).map_err(|e| {
            CanvasError::new(CanvasErrorKind::FileWrite(e.to_string()), line!(), file!())
        })?;

        // Add to recent projects
        let mut recent = RecentProjects::load();
        recent.add(PathBuf::from(path));
        if let Err(e) = recent.save() {
            tracing::warn!("Failed to save recent projects: {}", e);
        }

        tracing::info!("Saved project to: {}", path);
        Ok(())
    }

    /// Load the project state from a file
    pub fn load_from_file(&mut self, path: &str, ctx: &egui::Context) -> Result<(), CanvasError> {
        self.load_from_file_impl(path, ctx, false)
    }

    /// Load the project state from a file (internal implementation)
    /// If defer_image_load is true, the image will be loaded on the next update() call
    #[instrument(skip(self, ctx), fields(path, defer_image_load))]
    fn load_from_file_impl(&mut self, path: &str, ctx: &egui::Context, defer_image_load: bool) -> Result<(), CanvasError> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            CanvasError::new(CanvasErrorKind::FileRead(e.to_string()), line!(), file!())
        })?;

        let loaded: DrawingCanvas = serde_json::from_str(&json).map_err(|e| {
            CanvasError::new(CanvasErrorKind::Deserialization(e.to_string()), line!(), file!())
        })?;

        debug!("Deserialized project state: shapes={}, detections={}",
               loaded.shapes.len(), loaded.detections.len());

        // Copy all the serialized state
        self.project_name = loaded.project_name;
        self.shapes = loaded.shapes;
        self.detections = loaded.detections;
        self.current_tool = loaded.current_tool;
        self.layer_manager = loaded.layer_manager;
        self.stroke = loaded.stroke;
        self.fill_color = loaded.fill_color;
        self.zoom_level = loaded.zoom_level;
        self.pan_offset = loaded.pan_offset;
        self.grid_rotation_angle = loaded.grid_rotation_angle;
        self.form_image_rotation = loaded.form_image_rotation;

        debug!("Loaded project state: shapes={}, detections={}, detections_layer_visible={}",
               self.shapes.len(),
               self.detections.len(),
               self.layer_manager.is_visible(LayerType::Detections));

        // If there was a form image saved, try to reload it
        if let Some(form_path) = &loaded.form_image_path {
            if defer_image_load {
                // Defer image loading until the first update() call
                self.pending_image_load = Some(form_path.clone());
                self.form_image_path = Some(form_path.clone());
                tracing::debug!("Deferred loading of form image: {}", form_path);
            } else {
                // Load image immediately
                if let Err(e) = self.load_form_image(form_path, ctx) {
                    tracing::warn!("Could not reload form image from {}: {}", form_path, e);
                    // Don't fail the entire load if the image is missing
                    self.form_image_path = loaded.form_image_path;
                }
            }
        } else {
            self.form_image_path = None;
            self.form_image = None;
            self.form_image_size = None;
        }

        // Add to recent projects
        let mut recent = RecentProjects::load();
        recent.add(PathBuf::from(path));
        if let Err(e) = recent.save() {
            tracing::warn!("Failed to save recent projects: {}", e);
        }

        tracing::info!("Loaded project from: {}", path);
        Ok(())
    }

    /// Load the most recent project on startup (defers image loading)
    pub fn load_recent_on_startup(&mut self, ctx: &egui::Context) -> Result<(), CanvasError> {
        let recent = RecentProjects::load();
        if let Some(recent_path) = recent.most_recent()
            && let Some(path_str) = recent_path.to_str()
        {
            return self.load_from_file_impl(path_str, ctx, true);
        }
        Err(CanvasError::new(CanvasErrorKind::NoRecentProjects, line!(), file!()))
    }

    /// Detect text regions in the loaded form image
    #[cfg(feature = "text-detection")]
    #[instrument(skip(self), fields(confidence_threshold, existing_detections = self.detections.len()))]
    pub fn detect_text_regions(&mut self, confidence_threshold: f32) -> Result<usize, CanvasError> {
        // Check if we have a form image loaded
        let form_path = self.form_image_path.as_ref()
            .ok_or_else(|| CanvasError::new(CanvasErrorKind::NoFormImageLoaded, line!(), file!()))?;

        tracing::info!("Detecting text regions in: {}", form_path);

        // Create text detector with default model path
        let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string()).map_err(|e| {
            CanvasError::new(CanvasErrorKind::TextDetection(e.to_string()), line!(), file!())
        })?;

        // Detect text regions
        let regions = detector.detect_from_file(form_path.as_str(), confidence_threshold).map_err(|e| {
            CanvasError::new(CanvasErrorKind::TextDetection(e.to_string()), line!(), file!())
        })?;

        let count = regions.len();
        tracing::info!("Detected {} text regions", count);

        // Create rectangle shapes for each detected region
        for (i, region) in regions.iter().enumerate() {
            let top_left = Pos2::new(*region.x() as f32, *region.y() as f32);
            let bottom_right = Pos2::new(
                (*region.x() + *region.width()) as f32,
                (*region.y() + *region.height()) as f32,
            );

            // Create a rectangle shape with a distinctive color for text regions
            let stroke = Stroke::new(2.0, Color32::from_rgb(255, 165, 0)); // Orange
            let fill = Color32::TRANSPARENT; // No fill, outline only

            match Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                Ok(mut rect) => {
                    rect.name = format!("Text Region {} ({:.2}%)", i + 1, *region.confidence() * 100.0);
                    self.detections.push(Shape::Rectangle(rect));
                }
                Err(e) => {
                    warn!("Failed to create detection rectangle for region {}: {}", i, e);
                }
            }
        }

        debug!("Added {} detections, total now: {}", count, self.detections.len());

        Ok(count)
    }

    /// Extract text from all detections using OCR
    ///
    /// Returns a vector of (detection_index, OCR_result) pairs
    #[cfg(feature = "ocr")]
    #[instrument(skip(self, ocr), fields(detections = self.detections.len()))]
    pub fn extract_text_from_detections(
        &self,
        ocr: &form_factor_ocr::OCREngine,
    ) -> Result<Vec<(usize, form_factor_ocr::OCRResult)>, CanvasError> {
        let form_path = self.form_image_path.as_ref()
            .ok_or_else(|| CanvasError::new(CanvasErrorKind::NoFormImageLoaded, line!(), file!()))?;

        tracing::info!("Extracting text from {} detections", self.detections.len());

        let mut results = Vec::new();

        for (idx, detection) in self.detections.iter().enumerate() {
            match self.extract_text_from_shape(ocr, form_path, detection) {
                Ok(result) => {
                    debug!(
                        "Detection {}: extracted {} chars with {:.1}% confidence",
                        idx,
                        result.text().len(),
                        result.confidence()
                    );
                    results.push((idx, result));
                }
                Err(e) => {
                    warn!("Failed to extract text from detection {}: {}", idx, e);
                }
            }
        }

        tracing::info!("Extracted text from {}/{} detections", results.len(), self.detections.len());
        Ok(results)
    }

    /// Extract text from a specific shape using OCR
    #[cfg(feature = "ocr")]
    fn extract_text_from_shape(
        &self,
        ocr: &form_factor_ocr::OCREngine,
        image_path: &str,
        shape: &Shape,
    ) -> Result<form_factor_ocr::OCRResult, CanvasError> {
        use crate::Shape;

        // Get bounding box of the shape in image pixel coordinates
        let bbox = match shape {
            Shape::Rectangle(rect) => {
                // Find min/max coords
                let xs: Vec<f32> = rect.corners().iter().map(|p| p.x).collect();
                let ys: Vec<f32> = rect.corners().iter().map(|p| p.y).collect();

                let x_min = xs.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let y_min = ys.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let x_max = xs.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;
                let y_max = ys.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;

                let width = x_max.saturating_sub(x_min);
                let height = y_max.saturating_sub(y_min);

                (x_min, y_min, width, height)
            }
            Shape::Circle(circle) => {
                let x = (circle.center.x - circle.radius).max(0.0) as u32;
                let y = (circle.center.y - circle.radius).max(0.0) as u32;
                let diameter = (circle.radius * 2.0) as u32;
                (x, y, diameter, diameter)
            }
            Shape::Polygon(poly) => {
                // Get bounding box from polygon points
                let points = poly.to_egui_points();
                let xs: Vec<f32> = points.iter().map(|p| p.x).collect();
                let ys: Vec<f32> = points.iter().map(|p| p.y).collect();

                let x_min = xs.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let y_min = ys.iter().fold(f32::INFINITY, |a, &b| a.min(b)) as u32;
                let x_max = xs.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;
                let y_max = ys.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)) as u32;

                let width = x_max.saturating_sub(x_min);
                let height = y_max.saturating_sub(y_min);

                (x_min, y_min, width, height)
            }
        };

        trace!("Shape bbox in image coords: {:?}", bbox);

        // Extract text from this region
        ocr.extract_text_from_region_file(image_path, bbox).map_err(|e| {
            CanvasError::new(CanvasErrorKind::OCRFailed(e.to_string()), line!(), file!())
        })
    }

    /// Detect logos in the loaded form image
    ///
    /// Loads all logo templates from the "logos" directory and detects them in the form image.
    /// Detected logos are added as rectangles to the Detections layer.
    ///
    /// # Returns
    ///
    /// Returns the number of logos detected.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No form image is loaded
    /// - Logo templates cannot be loaded
    /// - Logo detection fails
    #[cfg(feature = "logo-detection")]
    #[instrument(skip(self), fields(existing_detections = self.detections.len()))]
    pub fn detect_logos(&mut self) -> Result<usize, CanvasError> {
        // Check if we have a form image loaded
        let form_path = self.form_image_path.as_ref()
            .ok_or_else(|| CanvasError::new(CanvasErrorKind::NoFormImageLoaded, line!(), file!()))?;

        tracing::info!("Detecting logos in: {}", form_path);

        // Create logo detector with default settings
        let mut detector = LogoDetector::builder()
            .template_matching()
            .with_confidence_threshold(0.7)
            .with_scales(vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0])
            .build();

        // Load all logo templates from the logos directory
        let logos_dir = std::path::Path::new("logos");
        if !logos_dir.exists() {
            return Err(CanvasError::new(
                CanvasErrorKind::LogoDetection("logos directory does not exist".to_string()),
                line!(),
                file!(),
            ));
        }

        let mut logo_count = 0;
        for entry in std::fs::read_dir(logos_dir).map_err(|e| {
            CanvasError::new(
                CanvasErrorKind::LogoDetection(format!("Failed to read logos directory: {}", e)),
                line!(),
                file!(),
            )
        })? {
            let entry = entry.map_err(|e| {
                CanvasError::new(
                    CanvasErrorKind::LogoDetection(format!("Failed to read directory entry: {}", e)),
                    line!(),
                    file!(),
                )
            })?;

            let path = entry.path();
            if path.is_file() {
                // Check if it's an image file
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" || ext_str == "webp" {
                        // Get the logo name from the filename (without extension)
                        let logo_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown");

                        debug!("Loading logo: {} from {:?}", logo_name, path);
                        if let Err(e) = detector.add_logo(logo_name, &path) {
                            warn!("Failed to load logo {}: {}", logo_name, e);
                        } else {
                            logo_count += 1;
                        }
                    }
                }
            }
        }

        if logo_count == 0 {
            return Err(CanvasError::new(
                CanvasErrorKind::LogoDetection("No logo templates found in logos directory".to_string()),
                line!(),
                file!(),
            ));
        }

        tracing::info!("Loaded {} logo templates", logo_count);

        // Detect logos in the form image
        let results = detector.detect_logos_from_path(form_path.as_str()).map_err(|e| {
            CanvasError::new(CanvasErrorKind::LogoDetection(e), line!(), file!())
        })?;

        let detection_count = results.len();
        tracing::info!("Detected {} logo instances", detection_count);

        // Create rectangle shapes for each detected logo
        for (i, result) in results.iter().enumerate() {
            let top_left = Pos2::new(result.location.x as f32, result.location.y as f32);
            let bottom_right = Pos2::new(
                (result.location.x + result.size.width) as f32,
                (result.location.y + result.size.height) as f32,
            );

            // Create a rectangle shape with a distinctive color for logo detections
            let stroke = Stroke::new(3.0, Color32::from_rgb(0, 255, 0)); // Green
            let fill = Color32::TRANSPARENT; // No fill, outline only

            match Rectangle::from_corners(top_left, bottom_right, stroke, fill) {
                Ok(mut rect) => {
                    rect.name = format!(
                        "Logo: {} ({:.1}%, scale={:.2}x)",
                        result.logo_name,
                        result.confidence * 100.0,
                        result.scale
                    );
                    self.detections.push(Shape::Rectangle(rect));
                }
                Err(e) => {
                    warn!("Failed to create detection rectangle for logo {}: {}", i, e);
                }
            }
        }

        debug!("Added {} logo detections, total detections now: {}", detection_count, self.detections.len());

        Ok(detection_count)
    }
}
