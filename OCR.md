# OCR (Optical Character Recognition) Feature

This document explains how to enable and use the OCR feature in Form Factor for extracting text from form images and detected regions.

## Overview

The OCR feature uses **Tesseract OCR** to automatically extract text from images and specific regions. It integrates seamlessly with text detection to provide a complete pipeline:
1. Detect text regions (using OpenCV DB model)
2. Extract text from regions (using Tesseract OCR)
3. Tag and export the data

## Prerequisites

### System Dependencies

The OCR feature requires Tesseract OCR and Leptonica to be installed on your system:

#### Linux (Ubuntu/Debian)
```bash
# Check for missing packages and install only what's needed
PACKAGES="tesseract-ocr libtesseract-dev libleptonica-dev"
MISSING=""
for pkg in $PACKAGES; do
    if ! dpkg -l | grep -q "^ii  $pkg "; then
        MISSING="$MISSING $pkg"
    fi
done
if [ -n "$MISSING" ]; then
    echo "Installing missing packages:$MISSING"
    sudo apt-get update && sudo apt-get install -y $MISSING
else
    echo "All required packages are already installed"
fi

# Optional: Install additional language data
sudo apt-get install tesseract-ocr-spa  # Spanish
sudo apt-get install tesseract-ocr-fra  # French
sudo apt-get install tesseract-ocr-deu  # German
sudo apt-get install tesseract-ocr-chi-sim  # Simplified Chinese
```

#### Linux (Fedora/RHEL)
```bash
# Check for missing packages and install only what's needed
PACKAGES="tesseract tesseract-devel leptonica leptonica-devel"
MISSING=""
for pkg in $PACKAGES; do
    if ! rpm -q $pkg &>/dev/null; then
        MISSING="$MISSING $pkg"
    fi
done
if [ -n "$MISSING" ]; then
    echo "Installing missing packages:$MISSING"
    sudo dnf install -y $MISSING
else
    echo "All required packages are already installed"
fi

# Optional: Install additional language data
sudo dnf install tesseract-langpack-spa  # Spanish
sudo dnf install tesseract-langpack-fra  # French
sudo dnf install tesseract-langpack-deu  # German
```

#### Linux (Manjaro/Arch)
```bash
# Check for missing packages and install only what's needed
PACKAGES="tesseract tesseract-data-eng leptonica"
MISSING=""
for pkg in $PACKAGES; do
    if ! pacman -Q $pkg &>/dev/null; then
        MISSING="$MISSING $pkg"
    fi
done

if [ -n "$MISSING" ]; then
    echo "Installing missing packages:$MISSING"
    sudo pacman -S --needed --noconfirm $MISSING
else
    echo "All required packages are already installed"
fi

# Optional: Install additional language data
sudo pacman -S tesseract-data-spa  # Spanish
sudo pacman -S tesseract-data-fra  # French
sudo pacman -S tesseract-data-deu  # German
```

#### macOS
```bash
# Install Tesseract via Homebrew
brew install tesseract
brew install leptonica

# Optional: Install all languages
brew install tesseract-lang
```

#### Windows
1. Download and install Tesseract from: https://github.com/UB-Mannheim/tesseract/wiki
2. During installation, select additional languages if needed
3. Add Tesseract to your PATH or set environment variables:
   ```
   TESSERACT_PATH=C:\Program Files\Tesseract-OCR
   ```

### Verify Installation

After installing, verify Tesseract is available:

```bash
tesseract --version
```

You should see output like:
```
tesseract 5.3.0
 leptonica-1.82.0
  libgif 5.2.1 : libjpeg 8d (libjpeg-turbo 2.1.3) : libpng 1.6.37 : libtiff 4.4.0 : zlib 1.2.11 : libwebp 1.2.4
```

Check available languages:
```bash
tesseract --list-langs
```

## Building with OCR

To enable the OCR feature, build with the `ocr` feature flag:

```bash
cargo build --features ocr
cargo run --features ocr
```

Or enable it with text detection:
```bash
cargo build --features "text-detection,ocr"
cargo run --features "text-detection,ocr"
```

Or use the `dev` feature which includes everything:
```bash
cargo build --features dev
cargo run --features dev
```

## Usage

### Basic Workflow

1. **Load a form image**: Click the "📁 Load Form" button
2. **Detect text regions**: Click the "🔍 Detect Text" button (requires `text-detection` feature)
3. **Extract text with OCR**: Click the "📝 Extract Text (OCR)" button
4. **View results**: Check the terminal/logs for extracted text and confidence scores

### Programmatic Usage

```rust
use form_factor::{OCREngine, OCRConfig, PageSegmentationMode};

// Create an OCR engine
let ocr = OCREngine::new(OCRConfig::new()
    .with_language("eng")
    .with_psm(PageSegmentationMode::Auto)
    .with_min_confidence(60)
)?;

// Extract text from an entire image
let result = ocr.extract_text_from_file("document.png")?;
println!("Text: {}", result.text);
println!("Confidence: {:.1}%", result.confidence);

// Extract text from a specific region (x, y, width, height)
let region = (100, 200, 300, 50);
let result = ocr.extract_text_from_region_file("document.png", region)?;
println!("Regional text: {}", result.text.trim());
```

### Integration with DrawingCanvas

```rust
use form_factor::{DrawingCanvas, OCREngine, OCRConfig};

let mut canvas = DrawingCanvas::new();
canvas.load_form_image("form.png", &ctx)?;

// Detect text regions first
#[cfg(feature = "text-detection")]
canvas.detect_text_regions(0.5)?;

// Extract text from all detections
#[cfg(feature = "ocr")]
{
    let ocr = OCREngine::new(OCRConfig::default())?;
    let results = canvas.extract_text_from_detections(&ocr)?;

    for (idx, result) in results {
        println!("Detection {}: '{}' ({:.1}%)",
                 idx, result.text.trim(), result.confidence);
    }
}
```

## Configuration

### Language Configuration

Tesseract supports 100+ languages. Specify one or combine multiple:

```rust
// Single language
let ocr = OCREngine::new(OCRConfig::new()
    .with_language("eng")  // English
)?;

// Multiple languages (combined with +)
let ocr = OCREngine::new(OCRConfig::new()
    .with_language("eng+spa")  // English and Spanish
)?;

// Other languages:
// "fra" - French
// "deu" - German
// "ita" - Italian
// "por" - Portuguese
// "rus" - Russian
// "chi_sim" - Simplified Chinese
// "chi_tra" - Traditional Chinese
// "jpn" - Japanese
// "ara" - Arabic
```

### Page Segmentation Modes (PSM)

The PSM tells Tesseract how to interpret the image layout:

```rust
use form_factor::PageSegmentationMode;

// Fully automatic (default)
.with_psm(PageSegmentationMode::Auto)

// Single text line (best for single-line fields)
.with_psm(PageSegmentationMode::SingleLine)

// Single word (best for single-word fields)
.with_psm(PageSegmentationMode::SingleWord)

// Single character (best for character-level OCR)
.with_psm(PageSegmentationMode::SingleChar)

// Sparse text (find text anywhere, no order)
.with_psm(PageSegmentationMode::SparseText)
```

**Choose the right PSM:**
- `Auto` - General documents, multi-line text
- `SingleLine` - Form fields, addresses, names
- `SingleWord` - Dates, numbers, single words
- `SingleChar` - Checkbox labels, single letters
- `SparseText` - Scattered text, stamps, watermarks

### Engine Modes

```rust
use form_factor::EngineMode;

// Default (LSTM + Legacy) - best accuracy
.with_engine_mode(EngineMode::Default)

// LSTM only - fast, modern
.with_engine_mode(EngineMode::LstmOnly)

// Legacy only - older Tesseract engine
.with_engine_mode(EngineMode::TesseractOnly)
```

### Confidence Threshold

```rust
// Set minimum confidence threshold (0-100)
.with_min_confidence(70)  // Flag results below 70%
```

Results below this threshold will have `meets_threshold: false` in the OCRResult.

### Preprocessing

```rust
// Enable/disable preprocessing
.with_preprocessing(true)   // Default: enabled
.with_preprocessing(false)  // Disable for pre-processed images
```

Preprocessing includes:
- Grayscale conversion
- Contrast enhancement (histogram stretching)
- Noise reduction (optional)

## Typical Accuracy

| Content Type | Expected Accuracy | Notes |
|-------------|------------------|-------|
| Printed text (clean) | 95-99% | Best results |
| Printed text (scanned) | 85-95% | Good quality scans |
| Printed text (poor quality) | 70-85% | Low resolution, noise |
| Handwritten text | 40-70% | Highly variable |
| Numbers | 90-98% | Generally reliable |
| Mixed fonts | 80-90% | May struggle with unusual fonts |

## Tips for Better Accuracy

1. **Image Quality**: Higher resolution = better accuracy (300 DPI recommended)
2. **Contrast**: Ensure good contrast between text and background
3. **Alignment**: Straighten rotated images before OCR
4. **Noise**: Clean images work better (remove artifacts, spots)
5. **PSM Selection**: Choose the right PSM for your layout
6. **Language Data**: Install language packs for non-English text
7. **Preprocessing**: Enable for low-quality images

## Troubleshooting

### Build Errors

**Error**: `The system library 'lept' required by crate 'leptonica-sys' was not found`

**Solution**: Install Leptonica development libraries (see Prerequisites section above)

**Error**: `The system library 'tesseract' required by crate 'tesseract-sys' was not found`

**Solution**: Install Tesseract development libraries (see Prerequisites section above)

### Runtime Errors

**Error**: `Failed to initialize Tesseract (is it installed?)`

**Solution**:
1. Verify Tesseract is installed: `tesseract --version`
2. Check that language data is available: `tesseract --list-langs`
3. On Linux, ensure `TESSDATA_PREFIX` points to tessdata directory

**Error**: `Failed to initialize Tesseract with custom path`

**Solution**: Check that the tessdata path is correct and contains `.traineddata` files

**Error**: Language not found (e.g., `eng.traineddata`)

**Solution**: Install the language pack:
```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr-eng

# Arch/Manjaro
sudo pacman -S tesseract-data-eng

# macOS
brew install tesseract-lang
```

### Poor OCR Results

**Problem**: Low confidence scores or incorrect text

**Solutions**:
1. Check image quality (resolution, contrast, noise)
2. Try different PSM modes (especially `SingleLine` for form fields)
3. Enable preprocessing if not already enabled
4. Ensure correct language is selected
5. Check that text is not rotated or skewed
6. Verify the image is clear (not blurry, pixelated, or faded)

## Performance

OCR performance depends on:
- **Image size**: Larger images take longer
- **Engine mode**: LSTM is faster than Legacy
- **Number of regions**: More regions = more time
- **Preprocessing**: Adds some overhead

### Typical Processing Time (single text region)

| Image Size | PSM Mode | Typical Time |
|-----------|----------|--------------|
| Small (100x50px) | SingleLine | 50-100ms |
| Medium (300x100px) | SingleLine | 100-200ms |
| Large (800x200px) | Auto | 200-500ms |
| Full page (2480x3508) | Auto | 1-3 seconds |

## Integration with Other Features

### Complete Pipeline Example

```rust
use form_factor::{
    DrawingCanvas,
    TextDetector,
    OCREngine, OCRConfig,
    PageSegmentationMode
};

// 1. Load form
let mut canvas = DrawingCanvas::new();
canvas.load_form_image("invoice.png", &ctx)?;

// 2. Detect text regions
let detector = TextDetector::default();
canvas.detect_text_regions(0.5)?;

// 3. Extract text from detections
let ocr = OCREngine::new(OCRConfig::new()
    .with_language("eng")
    .with_psm(PageSegmentationMode::SingleLine)
    .with_min_confidence(70)
)?;

let results = canvas.extract_text_from_detections(&ocr)?;

// 4. Process results
for (idx, result) in results {
    if result.meets_threshold {
        println!("✓ Detection {}: '{}'", idx, result.text.trim());
    } else {
        println!("⚠ Detection {}: '{}' (low confidence: {:.1}%)",
                 idx, result.text.trim(), result.confidence);
    }
}
```

## Future Enhancements

Planned improvements for the OCR feature:
- Word-level bounding boxes
- Character-level confidence scores
- Table detection and extraction
- Layout analysis
- PDF text extraction
- Batch processing multiple images
- OCR result caching
- Custom Tesseract training data support
