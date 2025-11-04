# Workspace Architecture

This document describes the workspace structure for the form_factor project.

## Workspace Structure

```
form_factor/
├── Cargo.toml                      # Workspace root
└── crates/
    ├── form_factor_core/           # Core traits (App, Backend, AppContext)
    ├── form_factor_drawing/        # Canvas, shapes, layers, tools
    ├── form_factor_cv/             # Computer vision (text/logo detection)
    ├── form_factor_ocr/            # OCR using Tesseract
    ├── form_factor_backends/       # Backend implementations (eframe, etc.)
    └── form_factor/                # Main crate (re-exports + demo app)
```

## Dependency Graph

```
form_factor (main crate, re-exports everything)
├── form_factor_core (traits only, minimal deps: egui)
├── form_factor_drawing (depends on: core, egui, serde, geo, image)
├── form_factor_cv (depends on: drawing, opencv)
├── form_factor_ocr (depends on: drawing, leptess)
└── form_factor_backends (depends on: core, eframe)
```

## Rationale

### Why a Workspace?

1. **Isolation of Heavy Dependencies**
   - OpenCV (~500MB) and Tesseract isolated to `form_factor_cv` and `form_factor_ocr`
   - Users can use just `form_factor_drawing` without CV dependencies
   - Faster compilation when not working on CV features

2. **Clear Separation of Concerns**
   - Core traits stable and minimal
   - Drawing functionality independent of detection
   - Backend implementations swappable

3. **Reusability**
   - Each crate independently useful
   - Mix and match as needed

4. **Better Build Times**
   - Incremental compilation per-crate
   - Parallel compilation of independent crates

## Crate Descriptions

### `form_factor_core`
Foundation traits and types.

**Exports:** `App`, `Backend`, `AppContext`
**Dependencies:** egui only

### `form_factor_drawing`
Drawing canvas with interactive annotation tools.

**Exports:** `DrawingCanvas`, `Shape`, `Rectangle`, `Circle`, `PolygonShape`, `LayerManager`, `LayerType`, `ToolMode`, `RecentProjects`
**Dependencies:** core, egui, serde, geo, image

### `form_factor_cv`
Computer vision (text/logo detection).

**Exports:** `TextDetector`, `LogoDetector`, `TextRegion`, `LogoDetectionResult`
**Dependencies:** drawing, opencv
**Features:** `text-detection`, `logo-detection`

### `form_factor_ocr`
OCR using Tesseract.

**Exports:** `OCREngine`, `OCRConfig`, `OCRResult`, `PageSegmentationMode`
**Dependencies:** drawing, leptess

### `form_factor_backends`
Backend implementations.

**Exports:** `EframeBackend`
**Dependencies:** core, eframe
**Features:** `eframe` (default)

### `form_factor`
Main crate that re-exports everything and provides the demo app.

**Dependencies:** All above crates

## Usage Examples

### Just the drawing canvas:
```toml
[dependencies]
form_factor_drawing = "0.1"
```

### With computer vision:
```toml
[dependencies]
form_factor_cv = { version = "0.1", features = ["text-detection", "logo-detection"] }
form_factor_ocr = "0.1"
```

### Full application:
```toml
[dependencies]
form_factor = "0.1"
```

## Status

✅ Workspace structure created
✅ Cargo.toml files configured
✅ Stub lib.rs files created
⏳ Code migration (next phase)
⏳ Testing
⏳ Merge to main

## Next Steps

1. Move code from monolith to appropriate crates
2. Update imports and module paths
3. Verify compilation
4. Run tests
5. Merge workspace branch to main
