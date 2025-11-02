# Text Detection Feature

This document explains how to enable and use the text detection feature in Form Factor.

## Overview

The text detection feature uses OpenCV's **DB (Differentiable Binarization)** model to automatically detect text regions in form images. DB is a modern (2019) deep learning model that offers excellent performance on curved text, small text, and complex layouts. Detected regions are automatically converted to rectangle shapes on the canvas.

## Prerequisites

### System Dependencies

The text detection feature requires OpenCV and its dependencies to be installed on your system:

#### Linux (Ubuntu/Debian)
```bash
# Check for missing packages and install only what's needed
PACKAGES="libopencv-dev clang libclang-dev llvm-dev"
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
```

#### Linux (Fedora/RHEL)
```bash
# Check for missing packages and install only what's needed
PACKAGES="opencv-devel clang clang-devel llvm-devel"
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
```

#### Linux (Manjaro/Arch)
```bash
# Check for missing packages and install only what's needed
# Note: opencv-cuda provides the same functionality as opencv with GPU acceleration
PACKAGES="clang llvm"
OPENCV_PKG=""

# Check if either opencv or opencv-cuda is installed
if ! pacman -Q opencv &>/dev/null && ! pacman -Q opencv-cuda &>/dev/null; then
    OPENCV_PKG="opencv"
fi

MISSING="$OPENCV_PKG"
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
```

**Note**: If you have an NVIDIA GPU and want GPU-accelerated text detection, you can install `opencv-cuda` instead of `opencv`:
```bash
sudo pacman -S opencv-cuda clang llvm
```
The `opencv-cuda` package provides the same functionality as `opencv` but with CUDA acceleration for faster inference.

#### macOS
```bash
# Check for missing packages and install only what's needed
PACKAGES="opencv llvm"
MISSING=""
for pkg in $PACKAGES; do
    if ! brew list $pkg &>/dev/null; then
        MISSING="$MISSING $pkg"
    fi
done
if [ -n "$MISSING" ]; then
    echo "Installing missing packages:$MISSING"
    brew install $MISSING
else
    echo "All required packages are already installed"
fi
```

#### Windows
Download and install OpenCV from https://opencv.org/releases/
Set the `OPENCV_LINK_PATHS` and `OPENCV_INCLUDE_PATHS` environment variables.

### DB Model Files

You need to download a pre-trained DB text detection model. OpenCV officially provides several variants:

#### Available Models

1. **DB_IC15_resnet50.onnx** - ResNet-50 backbone, trained on ICDAR2015 (English text)
   - Best for: High accuracy requirements, English documents
   - Size: ~100 MB

2. **DB_IC15_resnet18.onnx** - ResNet-18 backbone, trained on ICDAR2015 (English text)
   - Best for: Faster inference, resource-constrained environments
   - Size: ~50 MB

3. **DB_TD500_resnet50.onnx** - ResNet-50 backbone, trained on MSRA-TD500 (English + Chinese text)
   - Best for: Multi-language documents (default model)
   - Size: ~100 MB

4. **DB_TD500_resnet18.onnx** - ResNet-18 backbone, trained on MSRA-TD500 (English + Chinese text)
   - Best for: Faster multi-language detection
   - Size: ~50 MB

#### Download Instructions

1. Create a `models` directory:
```bash
mkdir -p models
cd models
```

2. Download your preferred model from OpenCV's Google Drive:

**Option 1: TD500 ResNet-50 (recommended default)**
```bash
# Download multi-language model (English + Chinese)
wget --no-check-certificate 'https://drive.google.com/uc?export=download&id=19YWhArrNccaoSza0CfkXlA8im4-lAGsR' -O DB_TD500_resnet50.onnx
```

**Option 2: IC15 ResNet-50 (English only, high accuracy)**
```bash
# Download English-only model
wget --no-check-certificate 'https://drive.google.com/uc?export=download&id=17_ABp79PlFt9yPCxSaarVc_DKTmrSGGf' -O DB_IC15_resnet50.onnx
```

**Option 3: Faster models (ResNet-18)**
```bash
# TD500 ResNet-18 (multi-language, faster)
wget --no-check-certificate 'https://drive.google.com/uc?export=download&id=1vY_KsDZZZb_svLPwybZhHXcYp7zPWPZU' -O DB_TD500_resnet18.onnx

# IC15 ResNet-18 (English only, faster)
wget --no-check-certificate 'https://drive.google.com/uc?export=download&id=1sZszH3pEt8hliyBlTSz2u_v3_d-nnD5e' -O DB_IC15_resnet18.onnx
```

3. Verify the file exists:
```bash
ls models/DB_TD500_resnet50.onnx  # or whichever model you downloaded
```

**Note**: These are ONNX format models officially provided by OpenCV. The links come from OpenCV's official documentation at https://docs.opencv.org/4.x/d4/d43/tutorial_dnn_text_spotting.html

**Model Source**: Official OpenCV pre-trained models based on the DB paper (https://arxiv.org/abs/1911.08947)

## Building with Text Detection

To enable the text detection feature, build with the `text-detection` feature flag:

```bash
cargo build --features text-detection
cargo run --features text-detection
```

## Usage

1. **Load a form image**: Click the "📁 Load Form" button to load a scanned form or document image.

2. **Detect text regions**: Click the "🔍 Detect Text" button to run text detection.

3. **Review results**: Detected text regions will appear as orange rectangles on the Shapes layer. Each rectangle is labeled with its confidence score.

4. **Adjust and refine**: You can select, move, resize, or delete the detected regions as needed.

## Configuration

The DB text detector uses the following default settings:

- **Binary threshold**: 0.3 - Threshold for binarization (lower = more sensitive)
- **Polygon threshold**: 0.5 - Threshold for text region filtering (0.5-0.7 typical)
- **Unclip ratio**: 2.0 - Expansion ratio for detected text regions
- **Max candidates**: 200 - Maximum number of text regions to detect
- **Confidence threshold**: 0.5 (50%) - Minimum confidence for shape creation

These settings are optimized for general document text detection and can be adjusted using the builder pattern:

```rust
let detector = TextDetector::new("models/DB_TD500_resnet50.onnx".to_string())
    .with_binary_threshold(0.3)
    .with_polygon_threshold(0.6)
    .with_unclip_ratio(2.0)
    .with_max_candidates(200);
```

## Troubleshooting

### Build Errors

**Error**: `couldn't find any valid shared libraries matching: ['libclang.so']`

**Solution**: Use the installation script from the Prerequisites section for your platform, or manually install clang and llvm:
```bash
# Ubuntu/Debian
sudo apt-get install clang libclang-dev llvm-dev

# Fedora/RHEL
sudo dnf install clang clang-devel llvm-devel

# Manjaro/Arch
sudo pacman -S clang llvm

# macOS
brew install llvm
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib  # For Apple Silicon
```

**Error**: `opencv not found`

**Solution**: Use the installation script from the Prerequisites section for your platform, or manually install OpenCV:
```bash
# Ubuntu/Debian
sudo apt-get install libopencv-dev

# Fedora/RHEL
sudo dnf install opencv-devel

# Manjaro/Arch
sudo pacman -S opencv
# Or for GPU acceleration (if you have NVIDIA GPU):
sudo pacman -S opencv-cuda

# macOS
brew install opencv
```

**Note**:
- The installation scripts in the Prerequisites section automatically detect and install only missing packages, avoiding unnecessary sudo calls.
- On Manjaro/Arch, `opencv-cuda` is a drop-in replacement for `opencv` that provides GPU acceleration via CUDA. Both packages provide the same functionality for this crate.

### Runtime Errors

**Error**: `Failed to load DB model`

**Solution**: Ensure the model file is in the correct location:
- Default path: `models/DB_TD500_resnet50.onnx`
- Download from OpenCV's Google Drive (see DB Model Files section above)
- The model must be in ONNX format (`.onnx` extension)
- Verify file is not corrupted: `file models/DB_TD500_resnet50.onnx` should show "data"

**Error**: `No form image loaded`

**Solution**: Load a form image first using the "📁 Load Form" button before attempting text detection.

## Performance

Text detection performance depends on:
- **Model size**: ResNet-50 (~100 MB) vs ResNet-18 (~50 MB)
- **Image size**: Larger images take longer to process
- **CPU/GPU**: GPU acceleration (via opencv-cuda) significantly faster
- **Number of text regions**: More text = longer processing time

### Typical Processing Time (A4 size, 300 DPI):

**ResNet-50 models:**
- With GPU (CUDA): 0.5-2 seconds
- Without GPU: 3-8 seconds

**ResNet-18 models (faster):**
- With GPU (CUDA): 0.3-1 second
- Without GPU: 1-4 seconds

### Model Selection Guide:

- **High accuracy needed**: Use ResNet-50 (DB_TD500_resnet50.onnx or DB_IC15_resnet50.onnx)
- **Speed critical**: Use ResNet-18 (DB_TD500_resnet18.onnx or DB_IC15_resnet18.onnx)
- **Multi-language**: Use TD500 models (English + Chinese)
- **English only**: Use IC15 models

## Strengths & Limitations

### Strengths:
- ✅ Excellent performance on curved and rotated text
- ✅ Handles small text well
- ✅ Works with complex layouts
- ✅ Supports multi-language detection (TD500 models)
- ✅ Modern architecture (2019)

### Limitations:
- ⚠️ Handwritten text detection is limited (trained on printed text)
- ⚠️ Best results with high-contrast text
- ⚠️ Very low-resolution images may not work well
- ⚠️ Requires ~100 MB model file for ResNet-50 variants
