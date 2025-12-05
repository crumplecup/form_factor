//! Automatic field extraction from detections using OCR
//!
//! This module provides functionality to automatically extract field values from
//! detected text regions using OCR. It matches template field definitions with
//! detection bounding boxes and extracts text content.

use super::error::{InstanceError, InstanceErrorKind};
use crate::{DrawingInstance, Shape};
use form_factor_core::{FieldBounds, FieldDefinition, FieldValue, FormInstance, FormTemplate};
use tracing::{debug, instrument, warn};

/// Minimum overlap threshold for matching detections to fields (0.0-1.0)
///
/// A detection must overlap at least this much with a field to be considered a match.
const MIN_OVERLAP_THRESHOLD: f32 = 0.3;

/// Result of matching a detection to a field
#[derive(Debug, Clone)]
struct DetectionMatch {
    /// Index of the detection shape in the detections vector
    detection_index: usize,
    /// Overlap score (0.0-1.0), calculated as intersection over union
    overlap_score: f32,
    /// Bounds of the matched detection
    bounds: FieldBounds,
}

impl DrawingInstance {
    /// Extract field values from detections using OCR
    ///
    /// Matches template field definitions with detected text regions and extracts
    /// text content using OCR. Returns the number of fields successfully extracted.
    ///
    /// # Arguments
    ///
    /// * `template` - Template defining the expected fields
    /// * `page_index` - Which page to extract fields from (0-indexed)
    /// * `detections` - Detection shapes (bounding boxes) to match against fields
    /// * `image_path` - Path to the form image for OCR
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Page index is invalid
    /// - OCR extraction fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use form_factor_drawing::DrawingInstance;
    /// # use form_factor_core::FormTemplate;
    /// # fn example(instance: &mut DrawingInstance, template: &dyn FormTemplate, detections: &[form_factor_drawing::Shape]) -> Result<(), Box<dyn std::error::Error>> {
    /// let extracted_count = instance.extract_fields_from_detections(
    ///     template,
    ///     0,
    ///     detections,
    ///     "form.png",
    /// )?;
    /// println!("Extracted {} fields", extracted_count);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ocr")]
    #[instrument(skip(self, template, detections), fields(page_index, image_path))]
    pub fn extract_fields_from_detections(
        &mut self,
        template: &dyn FormTemplate,
        page_index: usize,
        detections: &[Shape],
        image_path: &str,
    ) -> Result<usize, InstanceError> {
        use form_factor_ocr::{OCRConfig, OCREngine};

        // Validate page index
        if page_index >= self.page_count() {
            return Err(InstanceError::new(
                InstanceErrorKind::InvalidPageIndex {
                    index: page_index,
                    max: self.page_count().saturating_sub(1),
                },
                line!(),
                file!(),
            ));
        }

        // Get fields for this page from the template
        let fields = template.fields_for_page(page_index);
        debug!(
            page_index,
            field_count = fields.len(),
            detection_count = detections.len(),
            "Starting field extraction"
        );

        // Initialize OCR engine
        let ocr_engine = OCREngine::new(OCRConfig::default()).map_err(|e| {
            InstanceError::new(
                InstanceErrorKind::OCRFailed(format!("Failed to initialize OCR: {}", e)),
                line!(),
                file!(),
            )
        })?;

        let mut extracted_count = 0;

        // For each field in the template, find matching detections
        for field_def in fields {
            debug!(field_id = %field_def.id, "Processing field");

            // Find best matching detection for this field
            match find_best_detection_match(field_def, detections) {
                Some(detection_match) => {
                    debug!(
                        field_id = %field_def.id,
                        detection_index = detection_match.detection_index,
                        overlap = detection_match.overlap_score,
                        "Found matching detection"
                    );

                    // Extract text from the detection region using OCR
                    match extract_text_from_region(&ocr_engine, image_path, &detection_match.bounds)
                    {
                        Ok((text, confidence)) => {
                            debug!(
                                field_id = %field_def.id,
                                text_length = text.len(),
                                confidence,
                                "Extracted text from region"
                            );

                            // Create field value with extracted text
                            let field_value = FieldValue::new_text(
                                &field_def.id,
                                text,
                                detection_match.bounds,
                                page_index,
                            )
                            .with_confidence(confidence);

                            // Store the field value
                            self.set_field_value(&field_def.id, field_value)
                                .map_err(|e| {
                                    InstanceError::new(
                                        InstanceErrorKind::OCRFailed(e.to_string()),
                                        line!(),
                                        file!(),
                                    )
                                })?;

                            extracted_count += 1;
                        }
                        Err(e) => {
                            warn!(
                                field_id = %field_def.id,
                                error = %e,
                                "Failed to extract text from detection"
                            );
                            // Continue with other fields even if one fails
                        }
                    }
                }
                None => {
                    debug!(
                        field_id = %field_def.id,
                        "No matching detection found for field"
                    );
                    // No detection found for this field - leave it empty
                }
            }
        }

        debug!(extracted_count, "Field extraction complete");
        Ok(extracted_count)
    }
}

/// Find the best matching detection for a field definition
///
/// Returns the detection with the highest overlap score above the threshold.
fn find_best_detection_match(
    field_def: &FieldDefinition,
    detections: &[Shape],
) -> Option<DetectionMatch> {
    let mut best_match: Option<DetectionMatch> = None;
    let mut best_score = MIN_OVERLAP_THRESHOLD;

    for (index, detection) in detections.iter().enumerate() {
        // Get bounds from the detection shape
        let detection_bounds = shape_to_bounds(detection);

        // Calculate overlap score
        let overlap = calculate_overlap(&field_def.bounds, &detection_bounds);

        // Update best match if this is better
        if overlap > best_score {
            best_score = overlap;
            best_match = Some(DetectionMatch {
                detection_index: index,
                overlap_score: overlap,
                bounds: detection_bounds,
            });
        }
    }

    best_match
}

/// Convert a Shape to FieldBounds
///
/// Extracts the bounding box from a Shape (Rectangle, Circle, or Polygon).
fn shape_to_bounds(shape: &Shape) -> FieldBounds {
    match shape {
        Shape::Rectangle(rect) => {
            // Get the corners and find the bounding box
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

            FieldBounds::new(min_x, min_y, max_x - min_x, max_y - min_y)
        }
        Shape::Circle(circle) => {
            let center = circle.center();
            let radius = *circle.radius();
            FieldBounds::new(
                center.x - radius,
                center.y - radius,
                radius * 2.0,
                radius * 2.0,
            )
        }
        Shape::Polygon(poly) => {
            // Get the exterior ring coordinates from the geo polygon
            use geo::algorithm::bounding_rect::BoundingRect;

            if let Some(rect) = poly.polygon().bounding_rect() {
                FieldBounds::new(
                    rect.min().x as f32,
                    rect.min().y as f32,
                    rect.width() as f32,
                    rect.height() as f32,
                )
            } else {
                // Fallback to zero bounds if bounding rect can't be computed
                FieldBounds::new(0.0, 0.0, 0.0, 0.0)
            }
        }
    }
}

/// Calculate overlap between two bounding boxes using IoU (Intersection over Union)
///
/// Returns a score between 0.0 (no overlap) and 1.0 (perfect overlap).
fn calculate_overlap(bounds1: &FieldBounds, bounds2: &FieldBounds) -> f32 {
    // Calculate intersection rectangle
    let x1 = bounds1.x.max(bounds2.x);
    let y1 = bounds1.y.max(bounds2.y);
    let x2 = (bounds1.x + bounds1.width).min(bounds2.x + bounds2.width);
    let y2 = (bounds1.y + bounds1.height).min(bounds2.y + bounds2.height);

    // Check if there's any intersection
    if x2 <= x1 || y2 <= y1 {
        return 0.0;
    }

    // Calculate intersection area
    let intersection_area = (x2 - x1) * (y2 - y1);

    // Calculate union area
    let area1 = bounds1.width * bounds1.height;
    let area2 = bounds2.width * bounds2.height;
    let union_area = area1 + area2 - intersection_area;

    // Calculate IoU
    if union_area > 0.0 {
        intersection_area / union_area
    } else {
        0.0
    }
}

/// Extract text from a specific region using OCR
///
/// Returns the extracted text and confidence score (0.0-1.0).
#[cfg(feature = "ocr")]
fn extract_text_from_region(
    ocr_engine: &form_factor_ocr::OCREngine,
    image_path: &str,
    bounds: &FieldBounds,
) -> Result<(String, f32), InstanceError> {
    use form_factor_ocr::OCRError;

    // Convert bounds to region tuple (x, y, width, height) as unsigned integers
    let region = (
        bounds.x.max(0.0) as u32,
        bounds.y.max(0.0) as u32,
        bounds.width.max(0.0) as u32,
        bounds.height.max(0.0) as u32,
    );

    // Extract text from the region
    let result = ocr_engine
        .extract_text_from_region_file(image_path, region)
        .map_err(|e: OCRError| {
            InstanceError::new(
                InstanceErrorKind::OCRFailed(format!("OCR extraction failed: {}", e)),
                line!(),
                file!(),
            )
        })?;

    Ok((result.text().to_string(), *result.confidence()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overlap_identical() {
        let bounds1 = FieldBounds::new(10.0, 20.0, 100.0, 50.0);
        let bounds2 = FieldBounds::new(10.0, 20.0, 100.0, 50.0);

        let overlap = calculate_overlap(&bounds1, &bounds2);
        assert!(
            (overlap - 1.0).abs() < 0.001,
            "Expected 1.0, got {}",
            overlap
        );
    }

    #[test]
    fn test_calculate_overlap_no_overlap() {
        let bounds1 = FieldBounds::new(10.0, 20.0, 100.0, 50.0);
        let bounds2 = FieldBounds::new(200.0, 200.0, 100.0, 50.0);

        let overlap = calculate_overlap(&bounds1, &bounds2);
        assert_eq!(overlap, 0.0);
    }

    #[test]
    fn test_calculate_overlap_partial() {
        let bounds1 = FieldBounds::new(0.0, 0.0, 100.0, 100.0);
        let bounds2 = FieldBounds::new(50.0, 50.0, 100.0, 100.0);

        let overlap = calculate_overlap(&bounds1, &bounds2);
        // Intersection: 50x50 = 2500
        // Union: 10000 + 10000 - 2500 = 17500
        // IoU: 2500/17500 = 0.142857
        assert!(
            (overlap - 0.142857).abs() < 0.001,
            "Expected ~0.143, got {}",
            overlap
        );
    }

    #[test]
    fn test_calculate_overlap_contained() {
        let bounds1 = FieldBounds::new(0.0, 0.0, 200.0, 200.0);
        let bounds2 = FieldBounds::new(50.0, 50.0, 50.0, 50.0);

        let overlap = calculate_overlap(&bounds1, &bounds2);
        // Intersection: 50x50 = 2500
        // Union: 40000 + 2500 - 2500 = 40000
        // IoU: 2500/40000 = 0.0625
        assert!(
            (overlap - 0.0625).abs() < 0.001,
            "Expected 0.0625, got {}",
            overlap
        );
    }
}
