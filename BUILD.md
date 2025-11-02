# Building Form Factor

This document provides instructions for building the Form Factor application on different platforms.

## Prerequisites

### All Platforms

- Rust 1.70+ (install from https://rustup.rs)
- Git

### Feature-Specific Dependencies

Depending on which features you want to enable, you'll need:

#### Text Detection (optional, via `text-detection` feature)
- OpenCV 4.x with DNN module
- See [TEXT_DETECTION.md](TEXT_DETECTION.md) for installation

#### Logo Detection (optional, via `logo-detection` feature)
- OpenCV 4.x (same as text detection)
- See [TEXT_DETECTION.md](TEXT_DETECTION.md) for installation

#### OCR (optional, via `ocr` feature)
- Tesseract OCR 4.0+
- Leptonica library
- See [OCR.md](OCR.md) for installation

## Quick Start

### Build Without Optional Features

```bash
cargo build --release
cargo run --release
```

This builds the core application without text detection, logo detection, or OCR.

### Build With All Features (Recommended for Development)

**On Linux (Arch/Manjaro):**

```bash
# Method 1: Use the build script (handles libclang path automatically)
./build.sh --release --dev

# Method 2: Set environment variables manually
export LIBCLANG_PATH=/usr/lib
export LD_LIBRARY_PATH=/usr/lib
cargo build --release --features dev

# Method 3: Add to your shell profile (~/.bashrc or ~/.zshrc)
echo 'export LIBCLANG_PATH=/usr/lib' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/lib' >> ~/.bashrc
source ~/.bashrc
cargo build --release --features dev
```

**On Linux (Ubuntu/Debian):**

```bash
export LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu
cargo build --release --features dev
```

**On macOS:**

```bash
# Apple Silicon
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib
cargo build --release --features dev

# Intel Mac
export LIBCLANG_PATH=/usr/local/opt/llvm/lib
cargo build --release --features dev
```

**On Windows:**

```cmd
cargo build --release --features dev
```

## Build Script (Linux/macOS)

A convenience script is provided to handle environment setup:

```bash
# Debug build with all features
./build.sh --dev

# Release build with all features
./build.sh --release --dev

# Release build with specific features
./build.sh --release --features "text-detection,ocr"

# Custom build
./build.sh --release --features "backend-eframe,text-detection"
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `backend-eframe` | eframe GUI backend (default) | None |
| `text-detection` | Text region detection with OpenCV | OpenCV 4.x |
| `logo-detection` | Logo detection with OpenCV | OpenCV 4.x |
| `ocr` | Text extraction with Tesseract | Tesseract, Leptonica |
| `dev` | Enable all features for development | All of the above |

### Example Builds

```bash
# Minimal build (GUI only, no CV features)
cargo build --release

# With text detection only
cargo build --release --features text-detection

# With text detection and OCR (complete pipeline)
cargo build --release --features "text-detection,ocr"

# With everything
cargo build --release --features dev
```

## Troubleshooting

### libclang not found (Linux)

**Error:**
```
a `libclang` shared library is not loaded on this thread
```

**Solution:**

Find your libclang location:
```bash
find /usr -name "libclang.so*" 2>/dev/null
```

Then set the path:
```bash
export LIBCLANG_PATH=/path/to/lib/directory
export LD_LIBRARY_PATH=/path/to/lib/directory
```

Common paths:
- Arch/Manjaro: `/usr/lib`
- Ubuntu/Debian: `/usr/lib/x86_64-linux-gnu`
- Fedora: `/usr/lib64`

### OpenCV not found

**Error:**
```
Could not find OpenCV
```

**Solution:**

Install OpenCV development packages:

```bash
# Arch/Manjaro
sudo pacman -S opencv

# Ubuntu/Debian
sudo apt-get install libopencv-dev

# Fedora
sudo dnf install opencv-devel

# macOS
brew install opencv
```

See [TEXT_DETECTION.md](TEXT_DETECTION.md) for detailed instructions.

### Tesseract not found

**Error:**
```
The system library `lept` required by crate `leptonica-sys` was not found
```

**Solution:**

Install Tesseract and Leptonica:

```bash
# Arch/Manjaro
sudo pacman -S tesseract tesseract-data-eng leptonica

# Ubuntu/Debian
sudo apt-get install tesseract-ocr libtesseract-dev libleptonica-dev

# Fedora
sudo dnf install tesseract tesseract-devel leptonica leptonica-devel

# macOS
brew install tesseract leptonica
```

See [OCR.md](OCR.md) for detailed instructions.

### Build is very slow

The first build compiles many dependencies and can take 5-10 minutes. Subsequent builds use cargo's incremental compilation and are much faster (30 seconds to 2 minutes).

To speed up:
1. Use `--release` only for final builds (debug is faster)
2. Enable fewer features during development
3. Use `cargo check` to check for errors without building binaries

### Out of memory during build

OpenCV compilation is memory-intensive. If you run out of memory:

1. Close other applications
2. Add swap space
3. Use fewer parallel jobs:
   ```bash
   cargo build --release --features dev -j 2
   ```

## Cross-Compilation

Cross-compilation is not officially supported due to OpenCV dependencies. Build natively on your target platform or use Docker.

## Docker Build (Advanced)

A Dockerfile is not currently provided, but you can create one using the official Rust image and installing the system dependencies as shown in the respective `.md` files.

## Development Tips

### Fast Iteration

For quick development without OpenCV features:
```bash
cargo check                    # Fast syntax check
cargo build                    # Build without optional features
cargo build --features dev     # Full build (slower)
```

### Clean Builds

To force a clean rebuild:
```bash
cargo clean
cargo build --release --features dev
```

This removes all build artifacts (~13GB) and rebuilds everything.

### Incremental Builds

After the initial build, cargo caches compiled dependencies. Only changed code is recompiled:
```bash
# First build: 5-10 minutes
cargo build --features dev

# After making changes: 30 seconds - 2 minutes
cargo build --features dev
```

## Platform-Specific Notes

### Arch/Manjaro

The `build.sh` script automatically sets `LIBCLANG_PATH` for you. Or add to your shell profile:

```bash
echo 'export LIBCLANG_PATH=/usr/lib' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/lib' >> ~/.bashrc
```

### Ubuntu/Debian

You may need to set a different `LIBCLANG_PATH`:

```bash
export LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu
```

### macOS

Install dependencies via Homebrew first:

```bash
brew install llvm opencv tesseract leptonica
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib  # Apple Silicon
```

### Windows

Windows support is experimental. OpenCV and Tesseract setup is more complex. Consider using WSL2 (Windows Subsystem for Linux) for easier development.

## Getting Help

If you encounter build issues:

1. Check this document first
2. Check the feature-specific docs (TEXT_DETECTION.md, OCR.md)
3. Verify all dependencies are installed
4. Try a clean build: `cargo clean && cargo build`
5. Check the error message carefully - it usually indicates what's missing

## Summary

**Quick start for Arch/Manjaro users:**

```bash
# Install dependencies
sudo pacman -S opencv tesseract tesseract-data-eng leptonica clang llvm

# Build with everything
./build.sh --release --dev

# Run
./target/release/form_factor
```

Done! 🎉
