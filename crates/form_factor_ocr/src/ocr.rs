//! Optical Character Recognition (OCR) using Tesseract
//!
//! This module provides OCR capabilities for extracting text from images and regions.
//! It wraps Tesseract OCR with a clean, Rust-friendly API.
//!
//! # Features
//!
//! - Extract text from entire images or specific regions
//! - Multi-language support (100+ languages)
//! - Configurable page segmentation modes
//! - Confidence scores for quality assessment
//! - Image preprocessing (deskew, denoise, contrast)
//! - Integration with text detection results
//!
//! # Example
//!
//! ```no_run
//! use form_factor_ocr::{OCREngine, OCRConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an OCR engine
//! let ocr = OCREngine::new(OCRConfig::default())?;
//!
//! // Extract text from an image
//! let result = ocr.extract_text_from_file("document.png")?;
//! println!("Text: {}", result.text());
//! println!("Confidence: {:.1}%", result.confidence());
//!
//! // Extract from a specific region
//! let region = (100, 200, 300, 50); // x, y, width, height
//! let result = ocr.extract_text_from_region_file("document.png", region)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Installation Requirements
//!
//! Tesseract must be installed on your system:
//!
//! ## Linux
//! ```bash
//! # Ubuntu/Debian
//! sudo apt-get install tesseract-ocr libtesseract-dev
//!
//! # For additional languages:
//! sudo apt-get install tesseract-ocr-spa  # Spanish
//! sudo apt-get install tesseract-ocr-fra  # French
//! sudo apt-get install tesseract-ocr-deu  # German
//!
//! # Arch/Manjaro
//! sudo pacman -S tesseract tesseract-data-eng
//! sudo pacman -S tesseract-data-spa  # Spanish
//! ```
//!
//! ## macOS
//! ```bash
//! brew install tesseract
//! brew install tesseract-lang  # All languages
//! ```
//!
//! ## Windows
//! Download and install from: https://github.com/UB-Mannheim/tesseract/wiki

use derive_getters::Getters;
use image::{DynamicImage, GrayImage};
use leptess::{LepTess, Variable};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument, trace, warn};

// ============================================================================
// Constants
// ============================================================================

/// Default minimum confidence threshold (0-100)
const DEFAULT_MIN_CONFIDENCE: i32 = 60;

/// Default language for OCR
const DEFAULT_LANGUAGE: &str = "eng";

/// Maximum pixel value for grayscale images
const MAX_PIXEL_VALUE: u8 = 255;

/// Minimum pixel value for grayscale images
const MIN_PIXEL_VALUE: u8 = 0;

/// Minimum valid confidence value
const MIN_CONFIDENCE: f32 = 0.0;

/// Maximum valid confidence value
const MAX_CONFIDENCE: f32 = 100.0;

// ============================================================================
// Error Types
// ============================================================================

/// Kinds of errors that can occur during OCR
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OCRErrorKind {
    /// Failed to initialize Tesseract engine
    Initialization(String),
    /// Failed to load image file
    ImageLoad(String),
    /// Failed to encode or process image
    ImageProcessing(String),
    /// Text extraction failed
    Extraction(String),
    /// Invalid region specified
    InvalidRegion(String),
    /// Invalid parameter value
    InvalidParameter(String),
}

impl std::fmt::Display for OCRErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OCRErrorKind::Initialization(msg) => {
                write!(f, "Failed to initialize Tesseract: {}", msg)
            }
            OCRErrorKind::ImageLoad(msg) => write!(f, "Failed to load image: {}", msg),
            OCRErrorKind::ImageProcessing(msg) => write!(f, "Image processing error: {}", msg),
            OCRErrorKind::Extraction(msg) => write!(f, "Text extraction failed: {}", msg),
            OCRErrorKind::InvalidRegion(msg) => write!(f, "Invalid region: {}", msg),
            OCRErrorKind::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

/// OCR error with location information
#[derive(Debug, Clone)]
pub struct OCRError {
    /// Error category
    pub kind: OCRErrorKind,
    /// Line number where error occurred
    pub line: u32,
    /// File where error occurred
    pub file: &'static str,
}

impl OCRError {
    /// Create a new OCR error
    pub fn new(kind: OCRErrorKind, line: u32, file: &'static str) -> Self {
        Self { kind, line, file }
    }
}

impl std::fmt::Display for OCRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OCR Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for OCRError {}

// ============================================================================
// Bounding Box
// ============================================================================

/// Bounding box for detected text regions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BoundingBox {
    /// X coordinate of top-left corner
    #[serde(default)]
    pub x: i32,
    /// Y coordinate of top-left corner
    #[serde(default)]
    pub y: i32,
    /// Width in pixels
    #[serde(default)]
    pub width: i32,
    /// Height in pixels
    #[serde(default)]
    pub height: i32,
}

impl BoundingBox {
    /// Create a new bounding box
    ///
    /// # Errors
    ///
    /// Returns error if width or height is not positive
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Result<Self, OCRError> {
        if width <= 0 {
            return Err(OCRError::new(
                OCRErrorKind::InvalidParameter(format!("Width must be positive, got: {}", width)),
                line!(),
                file!(),
            ));
        }
        if height <= 0 {
            return Err(OCRError::new(
                OCRErrorKind::InvalidParameter(format!("Height must be positive, got: {}", height)),
                line!(),
                file!(),
            ));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Convert to tuple (x, y, width, height)
    pub fn to_tuple(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Create from tuple (x, y, width, height)
    pub fn from_tuple(tuple: (i32, i32, i32, i32)) -> Result<Self, OCRError> {
        Self::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// OCR configuration options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OCRConfig {
    /// Language(s) to use for OCR (e.g., "eng", "spa", "eng+spa")
    #[serde(default = "default_language")]
    pub language: String,

    /// Page Segmentation Mode
    #[serde(default)]
    pub page_segmentation_mode: PageSegmentationMode,

    /// OCR Engine Mode
    #[serde(default)]
    pub engine_mode: EngineMode,

    /// Minimum confidence threshold (0-100)
    /// Results below this confidence will be flagged
    #[serde(default = "default_min_confidence")]
    pub min_confidence: i32,

    /// Enable preprocessing (deskew, denoise, etc.)
    #[serde(default = "default_preprocess")]
    pub preprocess: bool,

    /// Tesseract data path (optional)
    /// If None, uses system default
    #[serde(default)]
    pub tessdata_path: Option<String>,
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_string()
}

fn default_min_confidence() -> i32 {
    DEFAULT_MIN_CONFIDENCE
}

fn default_preprocess() -> bool {
    true
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            language: DEFAULT_LANGUAGE.to_string(),
            page_segmentation_mode: PageSegmentationMode::Auto,
            engine_mode: EngineMode::Default,
            min_confidence: DEFAULT_MIN_CONFIDENCE,
            preprocess: true,
            tessdata_path: None,
        }
    }
}

impl OCRConfig {
    /// Create a new configuration with English language
    ///
    /// This is equivalent to `OCRConfig::default()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the language (builder pattern)
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set the page segmentation mode (builder pattern)
    pub fn with_psm(mut self, psm: PageSegmentationMode) -> Self {
        self.page_segmentation_mode = psm;
        self
    }

    /// Set the engine mode (builder pattern)
    pub fn with_engine_mode(mut self, mode: EngineMode) -> Self {
        self.engine_mode = mode;
        self
    }

    /// Set minimum confidence threshold (builder pattern)
    pub fn with_min_confidence(mut self, confidence: i32) -> Self {
        self.min_confidence = confidence.clamp(0, 100);
        self
    }

    /// Enable or disable preprocessing (builder pattern)
    pub fn with_preprocessing(mut self, enable: bool) -> Self {
        self.preprocess = enable;
        self
    }

    /// Set custom tessdata path (builder pattern)
    pub fn with_tessdata_path(mut self, path: impl Into<String>) -> Self {
        self.tessdata_path = Some(path.into());
        self
    }
}

/// Page Segmentation Mode - how Tesseract should segment the page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PageSegmentationMode {
    /// Orientation and script detection (OSD) only
    OsdOnly = 0,
    /// Automatic page segmentation with OSD
    AutoOsd = 1,
    /// Automatic page segmentation, but no OSD or OCR
    AutoOnly = 2,
    /// Fully automatic page segmentation, but no OSD (Default)
    #[default]
    Auto = 3,
    /// Assume a single column of text of variable sizes
    SingleColumn = 4,
    /// Assume a single uniform block of vertically aligned text
    SingleBlockVertText = 5,
    /// Assume a single uniform block of text
    SingleBlock = 6,
    /// Treat the image as a single text line
    SingleLine = 7,
    /// Treat the image as a single word
    SingleWord = 8,
    /// Treat the image as a single word in a circle
    CircleWord = 9,
    /// Treat the image as a single character
    SingleChar = 10,
    /// Sparse text - find as much text as possible in no particular order
    SparseText = 11,
    /// Sparse text with OSD
    SparseTextOsd = 12,
    /// Raw line - treat the image as a single text line, bypassing hacks in Tesseract
    RawLine = 13,
}

/// OCR Engine Mode - which Tesseract engine to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EngineMode {
    /// Legacy engine only
    TesseractOnly = 0,
    /// Neural nets LSTM engine only
    LstmOnly = 1,
    /// Legacy + LSTM engines
    TesseractLstm = 2,
    /// Default, based on what is available (LSTM + Legacy)
    #[default]
    Default = 3,
}

/// Result of OCR text extraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct OCRResult {
    /// Extracted text
    #[serde(default)]
    text: String,

    /// Mean confidence score (0-100)
    #[serde(default)]
    confidence: f32,

    /// Whether confidence is above minimum threshold
    #[serde(default)]
    meets_threshold: bool,

    /// Individual word-level results (currently not implemented)
    ///
    /// This field is reserved for future word-level confidence and bounding box data.
    /// Currently always None.
    #[serde(default)]
    words: Option<Vec<WordResult>>,
}

impl OCRResult {
    /// Create a new OCR result
    pub(crate) fn new(text: String, confidence: f32, meets_threshold: bool) -> Self {
        Self {
            text,
            confidence,
            meets_threshold,
            words: None,
        }
    }
}

/// Word-level OCR result with position and confidence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct WordResult {
    /// The recognized word
    #[serde(default)]
    text: String,

    /// Confidence for this word (0-100)
    #[serde(default)]
    confidence: f32,

    /// Bounding box
    #[serde(default)]
    bbox: BoundingBox,
}

/// OCR Engine for text extraction
pub struct OCREngine {
    config: OCRConfig,
}

impl OCREngine {
    /// Create a new OCR engine with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if Tesseract cannot be initialized or if the specified
    /// language data is not available.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use form_factor_ocr::{OCREngine, OCRConfig};
    ///
    /// let config = OCRConfig::new()
    ///     .with_language("eng")
    ///     .with_psm(form_factor_ocr::PageSegmentationMode::Auto);
    ///
    /// let ocr = OCREngine::new(config)?;
    /// # Ok::<(), form_factor_ocr::OCRError>(())
    /// ```
    #[instrument(skip_all, fields(language = %config.language, psm = ?config.page_segmentation_mode))]
    pub fn new(config: OCRConfig) -> Result<Self, OCRError> {
        // Test that Tesseract can be initialized with this config
        Self::test_tesseract(&config)?;

        info!(
            "Initialized OCR engine: language={}, psm={:?}",
            config.language, config.page_segmentation_mode
        );

        Ok(Self { config })
    }

    /// Test that Tesseract can be initialized with the given config
    fn test_tesseract(config: &OCRConfig) -> Result<(), OCRError> {
        let mut lt = if let Some(ref path) = config.tessdata_path {
            LepTess::new(Some(path), &config.language).map_err(|e| {
                OCRError::new(
                    OCRErrorKind::Initialization(format!("Failed with custom path: {}", e)),
                    line!(),
                    file!(),
                )
            })?
        } else {
            LepTess::new(None, &config.language).map_err(|e| {
                OCRError::new(
                    OCRErrorKind::Initialization(format!("Is Tesseract installed? {}", e)),
                    line!(),
                    file!(),
                )
            })?
        };

        // Test setting PSM
        lt.set_variable(
            Variable::TesseditPagesegMode,
            &(config.page_segmentation_mode as i32).to_string(),
        )
        .map_err(|e| {
            OCRError::new(
                OCRErrorKind::Initialization(format!("Failed to set PSM: {}", e)),
                line!(),
                file!(),
            )
        })?;

        debug!("Tesseract initialized successfully");
        Ok(())
    }

    /// Extract text from an image file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or OCR fails.
    #[instrument(skip(self), fields(path))]
    pub fn extract_text_from_file(&self, path: impl AsRef<Path>) -> Result<OCRResult, OCRError> {
        let path = path.as_ref();
        debug!(path = ?path, "Loading image");

        let img = image::open(path).map_err(|e| {
            OCRError::new(OCRErrorKind::ImageLoad(format!("{}", e)), line!(), file!())
        })?;

        self.extract_text(&img)
    }

    /// Extract text from an image
    ///
    /// # Errors
    ///
    /// Returns an error if OCR fails.
    #[instrument(skip(self, image), fields(width = image.width(), height = image.height()))]
    pub fn extract_text(&self, image: &DynamicImage) -> Result<OCRResult, OCRError> {
        let processed = if self.config.preprocess {
            trace!("Preprocessing image");
            Self::preprocess_image(image)
        } else {
            image.to_luma8()
        };

        self.extract_text_from_gray(&processed)
    }

    /// Extract text from a grayscale image
    #[instrument(skip(self, image), fields(width = image.width(), height = image.height()))]
    fn extract_text_from_gray(&self, image: &GrayImage) -> Result<OCRResult, OCRError> {
        // Initialize Tesseract for this operation
        let mut lt = if let Some(ref path) = self.config.tessdata_path {
            LepTess::new(Some(path), &self.config.language)
        } else {
            LepTess::new(None, &self.config.language)
        }
        .map_err(|e| {
            OCRError::new(
                OCRErrorKind::Initialization(format!("{}", e)),
                line!(),
                file!(),
            )
        })?;

        // Configure Tesseract
        lt.set_variable(
            Variable::TesseditPagesegMode,
            &(self.config.page_segmentation_mode as i32).to_string(),
        )
        .map_err(|e| {
            OCRError::new(
                OCRErrorKind::Initialization(format!("Failed to set PSM: {}", e)),
                line!(),
                file!(),
            )
        })?;

        // Encode image as PNG for leptess (new API requires encoded image data)
        let mut png_data = Vec::new();
        {
            use image::ImageEncoder;
            use image::codecs::png::PngEncoder;

            let encoder = PngEncoder::new(&mut png_data);
            encoder
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    image::ExtendedColorType::L8,
                )
                .map_err(|e| {
                    OCRError::new(
                        OCRErrorKind::ImageProcessing(format!("Failed to encode image: {}", e)),
                        line!(),
                        file!(),
                    )
                })?;
        }

        // Set image from encoded PNG data
        lt.set_image_from_mem(&png_data).map_err(|e| {
            OCRError::new(
                OCRErrorKind::ImageProcessing(format!("Failed to set image: {}", e)),
                line!(),
                file!(),
            )
        })?;

        // Get text
        let text = lt.get_utf8_text().map_err(|e| {
            OCRError::new(OCRErrorKind::Extraction(format!("{}", e)), line!(), file!())
        })?;

        // Get confidence and clamp to valid range
        let confidence = (lt.mean_text_conf() as f32).clamp(MIN_CONFIDENCE, MAX_CONFIDENCE);

        debug!(chars = text.len(), confidence = %confidence, "Text extraction complete");

        let meets_threshold = confidence >= self.config.min_confidence as f32;

        Ok(OCRResult::new(text, confidence, meets_threshold))
    }

    /// Extract text from a specific region of an image file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    /// * `region` - (x, y, width, height) in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, region is invalid, or OCR fails.
    #[instrument(skip(self), fields(path, region = ?region))]
    pub fn extract_text_from_region_file(
        &self,
        path: impl AsRef<Path>,
        region: (u32, u32, u32, u32),
    ) -> Result<OCRResult, OCRError> {
        let path = path.as_ref();
        debug!(path = ?path, "Loading image");

        let img = image::open(path).map_err(|e| {
            OCRError::new(OCRErrorKind::ImageLoad(format!("{}", e)), line!(), file!())
        })?;

        self.extract_text_from_region(&img, region)
    }

    /// Extract text from a specific region of an image
    ///
    /// # Arguments
    ///
    /// * `image` - The source image
    /// * `region` - (x, y, width, height) in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if region is invalid or OCR fails.
    #[instrument(skip(self, image), fields(region = ?region))]
    pub fn extract_text_from_region(
        &self,
        image: &DynamicImage,
        region: (u32, u32, u32, u32),
    ) -> Result<OCRResult, OCRError> {
        let (x, y, width, height) = region;

        // Validate region bounds
        if x + width > image.width() || y + height > image.height() {
            return Err(OCRError::new(
                OCRErrorKind::InvalidRegion(format!(
                    "Region ({}, {}, {}, {}) exceeds image bounds ({}x{})",
                    x,
                    y,
                    width,
                    height,
                    image.width(),
                    image.height()
                )),
                line!(),
                file!(),
            ));
        }

        // Validate region has positive dimensions
        if width == 0 || height == 0 {
            return Err(OCRError::new(
                OCRErrorKind::InvalidRegion(format!(
                    "Region must have positive width and height, got: {}x{}",
                    width, height
                )),
                line!(),
                file!(),
            ));
        }

        debug!(width, height, x, y, "Extracting region");

        // Crop to region
        let cropped = image.crop_imm(x, y, width, height);

        self.extract_text(&cropped)
    }

    /// Preprocess image for better OCR accuracy
    ///
    /// Applies:
    /// - Grayscale conversion
    /// - Contrast enhancement
    fn preprocess_image(image: &DynamicImage) -> GrayImage {
        // Convert to grayscale
        let mut gray = image.to_luma8();

        // Apply contrast enhancement using simple histogram stretching
        Self::enhance_contrast(&mut gray);

        gray
    }

    /// Enhance contrast using histogram stretching
    fn enhance_contrast(image: &mut GrayImage) {
        let (width, height) = image.dimensions();

        // Find min and max pixel values
        let mut min_val = MAX_PIXEL_VALUE;
        let mut max_val = MIN_PIXEL_VALUE;

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0];
                min_val = min_val.min(pixel);
                max_val = max_val.max(pixel);
            }
        }

        // Avoid division by zero
        if max_val == min_val {
            return;
        }

        // Stretch histogram
        let range = (max_val - min_val) as f32;
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel_mut(x, y);
                let stretched =
                    ((pixel[0] - min_val) as f32 / range * MAX_PIXEL_VALUE as f32) as u8;
                pixel[0] = stretched;
            }
        }

        trace!(range, "Contrast enhancement complete");
    }

    /// Get the current configuration
    pub fn config(&self) -> &OCRConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = OCRConfig::new()
            .with_language("eng+spa")
            .with_psm(PageSegmentationMode::SingleLine)
            .with_min_confidence(70);

        assert_eq!(config.language, "eng+spa");
        assert_eq!(
            config.page_segmentation_mode,
            PageSegmentationMode::SingleLine
        );
        assert_eq!(config.min_confidence, 70);
    }

    #[test]
    fn test_confidence_clamping() {
        let config = OCRConfig::new().with_min_confidence(150);
        assert_eq!(config.min_confidence, 100);

        let config = OCRConfig::new().with_min_confidence(-10);
        assert_eq!(config.min_confidence, 0);
    }
}
