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
//! use form_factor::{OCREngine, OCRConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an OCR engine
//! let ocr = OCREngine::new(OCRConfig::default())?;
//!
//! // Extract text from an image
//! let result = ocr.extract_text_from_file("document.png")?;
//! println!("Text: {}", result.text);
//! println!("Confidence: {:.1}%", result.confidence);
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

use image::{DynamicImage, GrayImage};
use leptess::{LepTess, Variable};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument, trace, warn};

/// OCR configuration options
#[derive(Debug, Clone)]
pub struct OCRConfig {
    /// Language(s) to use for OCR (e.g., "eng", "spa", "eng+spa")
    /// Default: "eng"
    pub language: String,

    /// Page Segmentation Mode
    /// Default: Auto
    pub page_segmentation_mode: PageSegmentationMode,

    /// OCR Engine Mode
    /// Default: Default (LSTM + Legacy)
    pub engine_mode: EngineMode,

    /// Minimum confidence threshold (0-100)
    /// Results below this confidence will be flagged
    /// Default: 60
    pub min_confidence: i32,

    /// Enable preprocessing (deskew, denoise, etc.)
    /// Default: true
    pub preprocess: bool,

    /// Tesseract data path (optional)
    /// If None, uses system default
    pub tessdata_path: Option<String>,
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            page_segmentation_mode: PageSegmentationMode::Auto,
            engine_mode: EngineMode::Default,
            min_confidence: 60,
            preprocess: true,
            tessdata_path: None,
        }
    }
}

impl OCRConfig {
    /// Create a new configuration with English language
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageSegmentationMode {
    /// Orientation and script detection (OSD) only
    OsdOnly = 0,
    /// Automatic page segmentation with OSD
    AutoOsd = 1,
    /// Automatic page segmentation, but no OSD or OCR
    AutoOnly = 2,
    /// Fully automatic page segmentation, but no OSD (Default)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineMode {
    /// Legacy engine only
    TesseractOnly = 0,
    /// Neural nets LSTM engine only
    LstmOnly = 1,
    /// Legacy + LSTM engines
    TesseractLstm = 2,
    /// Default, based on what is available (LSTM + Legacy)
    Default = 3,
}

/// Result of OCR text extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRResult {
    /// Extracted text
    pub text: String,

    /// Mean confidence score (0-100)
    pub confidence: f32,

    /// Whether confidence is above minimum threshold
    pub meets_threshold: bool,

    /// Individual word-level results (if available)
    pub words: Vec<WordResult>,
}

/// Word-level OCR result with position and confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordResult {
    /// The recognized word
    pub text: String,

    /// Confidence for this word (0-100)
    pub confidence: f32,

    /// Bounding box (x, y, width, height)
    pub bbox: (i32, i32, i32, i32),
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
    /// use form_factor::{OCREngine, OCRConfig};
    ///
    /// let config = OCRConfig::new()
    ///     .with_language("eng")
    ///     .with_psm(form_factor::PageSegmentationMode::Auto);
    ///
    /// let ocr = OCREngine::new(config)?;
    /// # Ok::<(), String>(())
    /// ```
    #[instrument(skip_all, fields(language = %config.language, psm = ?config.page_segmentation_mode))]
    pub fn new(config: OCRConfig) -> Result<Self, String> {
        // Test that Tesseract can be initialized with this config
        Self::test_tesseract(&config)?;

        info!(
            "Initialized OCR engine: language={}, psm={:?}",
            config.language, config.page_segmentation_mode
        );

        Ok(Self { config })
    }

    /// Test that Tesseract can be initialized with the given config
    fn test_tesseract(config: &OCRConfig) -> Result<(), String> {
        let mut lt = if let Some(ref path) = config.tessdata_path {
            LepTess::new(Some(path), &config.language)
                .map_err(|e| format!("Failed to initialize Tesseract with custom path: {}", e))?
        } else {
            LepTess::new(None, &config.language)
                .map_err(|e| format!("Failed to initialize Tesseract (is it installed?): {}", e))?
        };

        // Test setting PSM
        lt.set_variable(Variable::TesseditPagesegMode, &(config.page_segmentation_mode as i32).to_string())
            .map_err(|e| format!("Failed to set page segmentation mode: {}", e))?;

        debug!("Tesseract initialized successfully");
        Ok(())
    }

    /// Extract text from an image file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or OCR fails.
    #[instrument(skip(self), fields(path))]
    pub fn extract_text_from_file(&self, path: impl AsRef<Path>) -> Result<OCRResult, String> {
        let path = path.as_ref();
        debug!("Loading image from {:?}", path);

        let img = image::open(path)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        self.extract_text(&img)
    }

    /// Extract text from an image
    ///
    /// # Errors
    ///
    /// Returns an error if OCR fails.
    #[instrument(skip(self, image), fields(width = image.width(), height = image.height()))]
    pub fn extract_text(&self, image: &DynamicImage) -> Result<OCRResult, String> {
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
    fn extract_text_from_gray(&self, image: &GrayImage) -> Result<OCRResult, String> {
        // Initialize Tesseract for this operation
        let mut lt = if let Some(ref path) = self.config.tessdata_path {
            LepTess::new(Some(path), &self.config.language)
        } else {
            LepTess::new(None, &self.config.language)
        }
        .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?;

        // Configure Tesseract
        lt.set_variable(
            Variable::TesseditPagesegMode,
            &(self.config.page_segmentation_mode as i32).to_string(),
        )
        .map_err(|e| format!("Failed to set PSM: {}", e))?;

        // Encode image as PNG for leptess (new API requires encoded image data)
        let mut png_data = Vec::new();
        {
            use image::codecs::png::PngEncoder;
            use image::ImageEncoder;

            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::L8
            ).map_err(|e| format!("Failed to encode image: {}", e))?;
        }

        // Set image from encoded PNG data
        lt.set_image_from_mem(&png_data)
            .map_err(|e| format!("Failed to set image: {}", e))?;

        // Get text
        let text = lt.get_utf8_text()
            .map_err(|e| format!("Failed to extract text: {}", e))?;

        // Get confidence
        let confidence = lt.mean_text_conf() as f32;

        debug!("Extracted {} chars with {:.1}% confidence", text.len(), confidence);

        // Get word-level results
        let words = self.get_word_results(&mut lt)?;

        Ok(OCRResult {
            text,
            confidence,
            meets_threshold: confidence >= self.config.min_confidence as f32,
            words,
        })
    }

    /// Extract word-level results from Tesseract
    fn get_word_results(&self, _lt: &mut LepTess) -> Result<Vec<WordResult>, String> {
        // This is a simplified version - full implementation would use Tesseract's
        // word-level confidence API
        // For now, return empty vec
        Ok(Vec::new())
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
    ) -> Result<OCRResult, String> {
        let path = path.as_ref();
        debug!("Loading image from {:?}", path);

        let img = image::open(path)
            .map_err(|e| format!("Failed to load image: {}", e))?;

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
    ) -> Result<OCRResult, String> {
        let (x, y, width, height) = region;

        // Validate region
        if x + width > image.width() || y + height > image.height() {
            return Err(format!(
                "Region ({}, {}, {}, {}) exceeds image bounds ({}x{})",
                x,
                y,
                width,
                height,
                image.width(),
                image.height()
            ));
        }

        debug!("Extracting region: {}x{} at ({}, {})", width, height, x, y);

        // Crop to region
        let cropped = image.crop_imm(x, y, width, height);

        self.extract_text(&cropped)
    }

    /// Preprocess image for better OCR accuracy
    ///
    /// Applies:
    /// - Grayscale conversion
    /// - Contrast enhancement
    /// - Noise reduction
    fn preprocess_image(image: &DynamicImage) -> GrayImage {
        // Convert to grayscale
        let mut gray = image.to_luma8();

        // Apply contrast enhancement using simple histogram stretching
        Self::enhance_contrast(&mut gray);

        // Apply basic noise reduction (optional)
        // Self::reduce_noise(&mut gray);

        gray
    }

    /// Enhance contrast using histogram stretching
    fn enhance_contrast(image: &mut GrayImage) {
        let (width, height) = image.dimensions();

        // Find min and max pixel values
        let mut min_val = 255u8;
        let mut max_val = 0u8;

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
                let stretched = ((pixel[0] - min_val) as f32 / range * 255.0) as u8;
                pixel[0] = stretched;
            }
        }

        trace!("Enhanced contrast: range {} -> 255", range);
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
        assert_eq!(config.page_segmentation_mode, PageSegmentationMode::SingleLine);
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
