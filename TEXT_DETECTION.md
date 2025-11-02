# Text Detection Feature

This document explains how to enable and use the text detection feature in Form Factor.

## Overview

The text detection feature uses OpenCV's EAST (Efficient and Accurate Scene Text) model to automatically detect text regions in form images. Detected regions are automatically converted to rectangle shapes on the canvas.

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
PACKAGES="opencv clang llvm"
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
```

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

# macOS
brew install opencv
```

**Note**: The installation scripts in the Prerequisites section automatically detect and install only missing packages, avoiding unnecessary sudo calls.

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
