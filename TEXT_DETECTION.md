# Text Detection Feature

This document explains how to enable and use the text detection feature in Form Factor.

## Overview

The text detection feature uses OpenCV's EAST (Efficient and Accurate Scene Text) model to automatically detect text regions in form images. Detected regions are automatically converted to rectangle shapes on the canvas.

## Prerequisites

### System Dependencies

The text detection feature requires OpenCV and its dependencies to be installed on your system:

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
    libopencv-dev \
    clang \
    libclang-dev \
    llvm-dev
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install opencv-devel clang clang-devel llvm-devel
```

#### macOS
```bash
brew install opencv llvm
```

#### Windows
Download and install OpenCV from https://opencv.org/releases/
Set the `OPENCV_LINK_PATHS` and `OPENCV_INCLUDE_PATHS` environment variables.

### EAST Model Files

You need to download the pre-trained EAST text detection model:

1. Create a `models` directory in the project root:
```bash
mkdir -p models
```

2. Download the EAST model files:
   - Model weights: [frozen_east_text_detection.pb](https://github.com/oyyd/frozen_east_text_detection.pb/raw/master/frozen_east_text_detection.pb)
   - Model config: You can create a basic config file or download from the OpenCV repository

3. Place the files in the `models` directory:
```
models/
  ├── frozen_east_text_detection.pb
  └── frozen_east_text_detection.pb.prototxt
```

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

The text detector uses the following default settings:

- **Confidence threshold**: 0.5 (50%)
- **Model input size**: 320x320 pixels
- **Non-maximum suppression IoU threshold**: 0.4

These settings are optimized for general document text detection but can be adjusted by modifying the `detect_text_regions` method in `src/drawing/canvas.rs`.

## Troubleshooting

### Build Errors

**Error**: `couldn't find any valid shared libraries matching: ['libclang.so']`

**Solution**: Install clang and llvm development libraries:
```bash
# Ubuntu/Debian
sudo apt-get install clang libclang-dev llvm-dev

# macOS
brew install llvm
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib  # For Apple Silicon
```

**Error**: `opencv not found`

**Solution**: Install OpenCV development libraries:
```bash
# Ubuntu/Debian
sudo apt-get install libopencv-dev

# macOS
brew install opencv
```

### Runtime Errors

**Error**: `Failed to load EAST model`

**Solution**: Ensure the model files are in the correct location:
- Default path: `models/frozen_east_text_detection.pb`
- Check that both `.pb` and `.prototxt` files exist

**Error**: `No form image loaded`

**Solution**: Load a form image first using the "📁 Load Form" button before attempting text detection.

## Performance

Text detection performance depends on:
- Image size (larger images take longer)
- CPU/GPU capabilities
- Number of text regions in the image

Typical processing time for a standard form (A4 size, 300 DPI):
- With GPU: 1-3 seconds
- Without GPU: 3-10 seconds

## Limitations

- The EAST model is optimized for horizontal and near-horizontal text
- Very small or very large text may not be detected reliably
- Handwritten text detection is limited
- Best results with high-contrast, clear text

## Alternative Models

While this implementation uses the EAST model, you can adapt the `TextDetector` to use other models:
- CRAFT (Character Region Awareness for Text detection)
- DB (Differentiable Binarization)
- PSENet (Progressive Scale Expansion Network)

Modify `src/text_detection.rs` to integrate alternative models.
