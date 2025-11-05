# Form Factor - Continuation Context

**Date**: November 4, 2025
**Branch**: `plugins`
**Last Commit**: `aaa3092` - "Fix unused import warnings in canvas/io.rs"

## Project Overview

Form Factor is a GUI application for tagging scanned forms with OCR metadata. Built with Rust and egui, it provides an accessible interface for document annotation with computer vision capabilities.

## Recent Major Work Completed

### 1. Workspace Architecture (Completed ✅)

Successfully refactored the monolithic crate into a workspace with 6 specialized crates:

- **`form_factor_core`** - Core traits and shared types
  - `App`, `Backend`, `AppContext` traits
  - Shared error types: `IoError`, `IoOperation`
  - Minimal dependencies, foundation for all other crates

- **`form_factor_drawing`** - Canvas, shapes, layers, tools
  - `DrawingCanvas` with pan/zoom/selection
  - Shapes: `Rectangle`, `Circle`, `PolygonShape`
  - Layer management: Shapes, Text, Logos, Detections
  - Tool modes: Select, Rectangle, Circle, Freehand
  - Recent projects tracking

- **`form_factor_cv`** - Computer vision (OpenCV)
  - Text detection using DB model
  - Logo detection (template + feature matching)
  - Feature-gated: `text-detection`, `logo-detection`
  - Isolated heavy OpenCV dependency

- **`form_factor_ocr`** - OCR text extraction (Tesseract)
  - Text extraction from images and regions
  - Multi-language support (100+ languages)
  - Configurable page segmentation modes
  - Feature-gated: `ocr`
  - Isolated heavy Tesseract/leptess dependency

- **`form_factor_backends`** - Backend implementations
  - eframe backend (default)
  - miniquad backend (stub for future)
  - Feature-gated: `eframe` (enabled by default)

- **`form_factor`** - Main crate
  - Re-exports all workspace crates
  - Backward-compatible public API
  - Aggregates errors into `FormError`/`FormErrorKind`
  - Contains binary application in `src/main.rs`

### 2. Feature Flag Architecture

Proper feature propagation established:

```toml
[features]
default = ["backend-eframe"]
backend-eframe = ["dep:form_factor_backends"]
text-detection = [
    "dep:form_factor_cv",
    "form_factor_cv/text-detection",
    "form_factor_drawing/text-detection"  # Important: propagates to drawing!
]
logo-detection = [
    "dep:form_factor_cv",
    "form_factor_cv/logo-detection",
    "form_factor_drawing/logo-detection"
]
ocr = [
    "dep:form_factor_ocr",
    "form_factor_drawing/ocr"
]
dev = ["text-detection", "logo-detection", "ocr"]
```

**Critical**: Features must propagate to `form_factor_drawing` so that methods like `detect_text_regions()`, `detect_logos()`, and `extract_text_from_detections()` are available.

### 3. Error Handling Architecture

Following the project's error handling pattern:

```rust
// Module-specific error kind enum
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasErrorKind {
    ImageLoad(String),
    NoFormImageLoaded,
    // ...
}

// Module-specific error struct with location info
#[derive(Debug, Clone)]
pub struct CanvasError {
    pub kind: CanvasErrorKind,
    pub line: u32,
    pub file: &'static str,
}

// Crate-level aggregation
#[derive(Debug, derive_more::From)]
pub enum FormErrorKind {
    Canvas(CanvasError),
    Layer(LayerError),
    Shape(ShapeError),
    // ...
}

// Top-level boxed error
pub struct FormError(Box<FormErrorKind>);
```

- Use `&'static str` for file fields (matches `file!()` macro)
- Include line and file info for debugging
- Aggregate at crate level with `derive_more::From`

### 4. Recent UI Features Added

- **Expandable Detections Layer**: Dropdown showing Logos and Text counts separately
- **Selectable Sub-layers**: Click Logos or Text to filter which detections are visible
- **Logo Detection Button**: Compares document to logos in `logos/` folder
- **Detection Rendering**: Logos in green, text regions in orange

## Current State

### Git Status
- **Main branch**: Contains complete workspace architecture, pushed to GitHub
- **Plugins branch**: Active branch, identical to main, ready for plugin work
- **Workspace branch**: Deleted (merged to main)

### Build Status
- ✅ `cargo check --workspace --all-features`: Clean
- ✅ `cargo test --workspace --all-features`: 107 unit tests + 17 doctests passing
- ✅ No compilation warnings

### Documentation
- `WORKSPACE.md`: Documents workspace architecture and rationale
- `CLAUDE.md`: Project guidelines for code structure, errors, docs, etc.
- All public APIs have documentation (enforced by `#![warn(missing_docs)]`)

## Next Steps: Plugin System

### The Plan (Discussed but Not Implemented)

Implement an egui plugin system on top of the workspace architecture. This was the "hybrid approach" - workspace for major separation, plugins for UI modularity.

### Plugin Design Considerations

1. **Plugin Trait**: Define a trait for UI plugins
   ```rust
   pub trait Plugin {
       fn name(&self) -> &str;
       fn ui(&mut self, ui: &mut egui::Ui, ctx: &AppContext);
       // Optional hooks: on_load, on_save, etc.
   }
   ```

2. **Plugin Registration**: System to register and manage plugins
   - Dynamic plugin loading at runtime
   - Plugin dependencies and ordering
   - Plugin state management

3. **Potential Plugins** (based on current features):
   - Canvas plugin (drawing tools, pan/zoom)
   - Layers plugin (layer management UI)
   - Detection plugin (text/logo detection buttons)
   - OCR plugin (text extraction UI)
   - Properties plugin (shape/object properties)
   - File operations plugin (open/save/export)

4. **Benefits**:
   - UI components can be developed independently
   - Easy to add/remove features
   - Clear separation of concerns at UI level
   - Testable in isolation

### Open Questions

1. **Plugin Discovery**: Compile-time (feature flags) or runtime (dynamic loading)?
2. **Plugin Communication**: How do plugins interact? Event bus? Shared state?
3. **Plugin Lifecycle**: When are plugins initialized/destroyed?
4. **Plugin Configuration**: Per-plugin settings?

## Key Architecture Patterns

### 1. Module Organization

When a module exceeds ~500-1000 lines, split into subdirectory:
```
canvas/
  ├── core.rs       # Core data structures
  ├── io.rs         # File I/O operations
  ├── rendering.rs  # egui rendering
  ├── tools.rs      # Tool interactions
  └── mod.rs        # Public re-exports
```

### 2. Feature Gating

Feature-gated code uses conditional compilation:
```rust
#[cfg(feature = "text-detection")]
pub fn detect_text_regions(&mut self) -> Result<usize, CanvasError> { ... }
```

Imports also feature-gated to avoid warnings:
```rust
#[cfg(feature = "text-detection")]
use form_factor_cv::TextDetector;
```

### 3. Derive Policies (from CLAUDE.md)

- Data structures derive: `Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash` if possible
- Use `derive_more` for `Display, FromStr, From, Deref, DerefMut, AsRef, AsMut`
- For enums with no fields, use `strum::EnumIter`
- Serialize with `serde`: `Serialize, Deserialize` for persistent types
- Use `#[serde(skip)]` for runtime state, `#[serde(default)]` for optional fields

### 4. Tracing/Logging

- Use `tracing` crate (not `println!` in library code)
- Use `#[instrument]` macro for automatic entry/exit logging
- Log levels: `trace!`, `debug!`, `info!`, `warn!`, `error!`
- Structured logging: `debug!(count = items.len(), "Processing items")`

## Important File Locations

### Core Files
- `/home/erik/repos/form_factor/Cargo.toml` - Workspace root
- `/home/erik/repos/form_factor/WORKSPACE.md` - Architecture docs
- `/home/erik/repos/form_factor/CLAUDE.md` - Project guidelines

### Main Crate
- `crates/form_factor/src/main.rs` - Application entry point
- `crates/form_factor/src/lib.rs` - Public API re-exports
- `crates/form_factor/src/error.rs` - Top-level error aggregation

### Drawing Crate (largest, most complex)
- `crates/form_factor_drawing/src/canvas/` - Canvas implementation
- `crates/form_factor_drawing/src/shape.rs` - Shape primitives
- `crates/form_factor_drawing/src/layer.rs` - Layer management

### Configuration
- `.env` - Environment variables (not committed)
- Models in `models/` - ML models for text detection
- Logos in `logos/` - Logo templates for detection

## Testing

### Test Organization
- Unit tests in `crates/*/tests/` directories
- Doctests in source files (module and function level)
- No `#[cfg(test)] mod tests` in source files (per project guidelines)

### Current Test Count
- 107 unit tests
- 17 doctests
- All passing ✅

### Running Tests
```bash
cargo test --workspace --all-features  # All tests
cargo test -p form_factor_drawing      # Specific crate
cargo test --doc                       # Just doctests
```

## Common Build Commands

```bash
# Default build (just eframe backend)
cargo build

# With all features
cargo build --features dev

# Specific features
cargo build --features text-detection,logo-detection

# Check without building
cargo check --workspace --all-features

# Clean warnings
cargo clippy --workspace --all-features

# Run application with features
cargo run --features dev
```

## Known Issues / Gotchas

1. **Feature Propagation**: Features must be propagated to drawing crate, not just CV/OCR crates
2. **Import Feature Gates**: Imports used only in feature-gated code must also be feature-gated
3. **Circular Dependencies**: CV and OCR crates cannot depend on drawing crate (was fixed)
4. **Heavy Dependencies**: OpenCV and Tesseract are slow to compile - isolated in separate crates

## Development Environment

- **Rust**: Latest stable (as of Nov 2025)
- **IDE**: Likely VS Code with rust-analyzer
- **Platform**: Linux (Manjaro, kernel 6.12.48-1-MANJARO)
- **Container Setup**: Podman setup guides in `podman_setup_*.md`

## Untracked Files (Not Committed)

```
podman_setup_linux.md
podman_setup_windows.md
```

These are local development setup guides, not part of the project.

## Questions to Consider Next Week

1. Should plugins be compile-time (feature flags) or runtime (dynamic loading)?
2. How should plugin state be managed? Global state? Passed through context?
3. What's the plugin lifecycle? When do they initialize?
4. Should there be a plugin registry/discovery mechanism?
5. How do plugins communicate with each other?
6. Should plugins be able to add menu items, keyboard shortcuts, etc.?

## References

- egui plugin examples: Look at other egui applications with plugin systems
- Rust plugin patterns: Consider `dyn Trait` objects vs generics
- Event-driven architecture: For plugin communication
- egui's `Context` and state management patterns

## Getting Back Up to Speed

1. Read `WORKSPACE.md` for architecture overview
2. Read `CLAUDE.md` for project conventions
3. Review this file for recent work and next steps
4. Check `git log` for recent commits
5. Run `cargo test --workspace --all-features` to verify everything works
6. Start exploring plugin system design (the next major task)

---

**Ready to continue**: The workspace architecture is solid, all tests pass, and the codebase is ready for plugin system implementation. The hard part (workspace refactor) is done - now we can build the flexible plugin system on top of this clean foundation.
