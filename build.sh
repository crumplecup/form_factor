#!/usr/bin/env bash
# Build script for form_factor
# This ensures libclang is properly found during OpenCV compilation

set -e

# Set libclang path for opencv-rust build (required on Arch/Manjaro)
export LIBCLANG_PATH=/usr/lib
export LD_LIBRARY_PATH=/usr/lib

# Parse arguments
BUILD_TYPE="debug"
FEATURES=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --features)
            FEATURES="--features $2"
            shift 2
            ;;
        --dev)
            FEATURES="--features dev"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--release] [--dev] [--features <features>]"
            exit 1
            ;;
    esac
done

# Build
if [ "$BUILD_TYPE" = "release" ]; then
    echo "Building in release mode with features: $FEATURES"
    cargo build --release $FEATURES
else
    echo "Building in debug mode with features: $FEATURES"
    cargo build $FEATURES
fi

echo "Build complete!"
