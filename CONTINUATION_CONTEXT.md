# Form Factor - Continuation Context

**Date**: November 8, 2025
**Branch**: `plugins`
**Last Commit**: `138fe6f` - "Add plugin system with event bus architecture"

## Project Overview

Form Factor is a GUI application for tagging scanned forms with OCR metadata. Built with Rust and egui, it provides an accessible interface for document annotation with computer vision capabilities.

## Recent Work Completed

### Plugin System Implementation (✅ Completed - Nov 8, 2025)

Successfully implemented a flexible plugin system with event-driven architecture:

#### Core Infrastructure
- **Plugin Trait**: Defines plugin lifecycle (`ui`, `on_event`, `on_load`, `on_save`, `on_shutdown`)
- **Event Bus**: Message passing using `tokio::sync::mpsc` unbounded channels
- **Plugin Manager**: Coordinates plugin lifecycle and event distribution
- **App Events**: 15+ typed events for inter-plugin communication
- **Plugin Context**: Provides plugins access to events and application state

#### Implemented Plugins (Feature-Gated)
1. **Canvas Plugin** (`plugin-canvas`):
   - Tool selection UI (Select, Rectangle, Circle, Freehand, Edit, Rotate)
   - Zoom controls with +/- buttons and reset
   - Pan offset display
   - Emits `ToolSelected`, `CanvasZoomChanged` events

2. **Layers Plugin** (`plugin-layers`):
   - Layer visibility toggles for all 4 layers (Canvas, Detections, Shapes, Grid)
   - Layer selection highlighting
   - Lock status indicators
   - Emits `LayerSelected`, `LayerVisibilityChanged` events

3. **File Plugin** (`plugin-file`):
   - Open/Save/Save As buttons
   - Current file path display
   - Recent files list (max 10, with deduplication)
   - Emits `OpenFileRequested`, `SaveFileRequested`, `SaveAsRequested` events

4. **Detection Plugin** (`plugin-detection`):
   - Text detection trigger button
   - Logo detection trigger button
   - Detection count display (text regions, logos)
   - Emits `TextDetectionRequested`, `LogoDetectionRequested` events

5. **OCR Plugin** (`plugin-ocr`):
   - Extract text button
   - Extracted text display in scrollable area
   - Emits `OcrExtractionRequested` events
   - Handles custom `text_extracted` events

#### Event System
- **Event Types**: Canvas zoom/pan, shape/layer selection, file operations, detections, OCR, tool selection
- **Custom Events**: JSON-encoded data for plugin-specific communication
- **Decoupled Communication**: Plugins don't depend on each other, only on event types
- **Event Distribution**: Manager broadcasts all events to all plugins

#### Testing & Quality
- **20 unit tests** covering all plugin modules
- **100% test pass rate** for plugin system
- **Zero clippy warnings** for plugins crate
- **Test utilities**: `EventSender::new_test()` helper for plugin testing

#### Architecture Decisions
1. **Compile-time plugins**: Feature flags for conditional compilation (no runtime plugin loading)
2. **Message passing**: Tokio MPSC channels for event bus (async-friendly)
3. **No shared state**: Plugins communicate only via events
4. **Trait-based**: `dyn Plugin` trait objects for polymorphism

#### New Crate: `form_factor_plugins`
- **Location**: `crates/form_factor_plugins/`
- **Modules**:
  - `bus.rs` - Event bus implementation (186 lines)
  - `event.rs` - Event types (164 lines)
  - `plugin.rs` - Plugin trait (138 lines)
  - `manager.rs` - Plugin manager (230 lines)
  - `canvas.rs` - Canvas plugin (175 lines)
  - `layers.rs` - Layers plugin (221 lines)
  - `file.rs` - File plugin (229 lines)
  - `detection.rs` - Detection plugin (120 lines)
  - `ocr.rs` - OCR plugin (115 lines)
  - `lib.rs` - Public API (81 lines)
- **Total**: ~1,761 lines of code

#### Dependencies Added
- `tokio = { version = "1.42", features = ["sync"] }` - For MPSC channels
- Added to workspace dependencies

### Previous Work: Workspace Architecture (Completed ✅)

Successfully refactored the monolithic crate into a workspace with 6 specialized crates:

- **`form_factor_core`** - Core traits and shared types
- **`form_factor_drawing`** - Canvas, shapes, layers, tools
- **`form_factor_cv`** - Computer vision (OpenCV)
- **`form_factor_ocr`** - OCR text extraction (Tesseract)
- **`form_factor_backends`** - Backend implementations (eframe, miniquad stub)
- **`form_factor_plugins`** - Plugin system (NEW!)
- **`form_factor`** - Main crate, re-exports, binary

## Current State

### Git Status
- **Plugins branch**: Active, 3 commits ahead of main
  - b7f0215: Remove Podman setup documentation files
  - 138fe6f: Add plugin system with event bus architecture
- **Main branch**: Contains complete workspace architecture

### Build Status
- ✅ `cargo check --workspace --all-features`: Clean
- ✅ `cargo test -p form_factor_plugins --all-features`: 20 tests passing
- ✅ `cargo clippy -p form_factor_plugins --all-features`: No warnings
- ℹ️ Main application not yet updated to use plugins

### Testing
- **Plugin tests**: 20 unit tests + 1 doctest
- **Previous tests**: 107 unit tests + 17 doctests (workspace)
- **Total**: 127 unit tests + 18 doctests

## Next Steps: Plugin System Integration

### What's Left to Do

1. **Integrate plugins into main application** (`crates/form_factor/src/main.rs`):
   - Add `form_factor_plugins` dependency to main crate
   - Create `PluginManager` instance in app state
   - Register plugins based on enabled features
   - Call `manager.process_events()` in update loop
   - Call `manager.render_plugins()` in UI rendering

2. **Connect plugins to existing DrawingCanvas**:
   - Canvas plugin should control the actual `DrawingCanvas` state
   - File plugin should trigger actual file I/O operations
   - Layers plugin should control actual layer visibility
   - Detection plugin should trigger CV operations
   - OCR plugin should trigger text extraction

3. **Update main application features**:
   - Propagate plugin features from main crate Cargo.toml
   - Add `plugins` feature that enables plugin system
   - Update `dev` feature to include `plugins`

4. **Event wiring**:
   - Wire up app events to DrawingCanvas state changes
   - Wire up plugin events to trigger actual operations
   - Ensure bidirectional communication (app → plugins, plugins → app)

5. **UI Layout**:
   - Decide where to render plugins (sidebar, panels, etc.)
   - Consider using egui's `SidePanel`, `TopBottomPanel`, or `Window`
   - Maybe use `egui::containers::CollapsingHeader` for collapsible plugin panels

6. **State Synchronization**:
   - When DrawingCanvas zoom changes, emit `CanvasZoomChanged` event
   - When layer visibility changes, emit `LayerVisibilityChanged` event
   - When tool changes, emit `ToolSelected` event
   - Ensure plugins reflect current app state on initialization

### Integration Example Pattern

```rust
// In main.rs or app struct
struct App {
    canvas: DrawingCanvas,
    plugin_manager: PluginManager,
}

impl App {
    fn new() -> Self {
        let mut manager = PluginManager::new();

        #[cfg(feature = "plugin-canvas")]
        manager.register(Box::new(form_factor_plugins::canvas::CanvasPlugin::new()));

        #[cfg(feature = "plugin-layers")]
        manager.register(Box::new(form_factor_plugins::layers::LayersPlugin::new()));

        #[cfg(feature = "plugin-file")]
        manager.register(Box::new(form_factor_plugins::file::FilePlugin::new()));

        Self {
            canvas: DrawingCanvas::new(),
            plugin_manager: manager,
        }
    }

    fn update(&mut self, ctx: &egui::Context) {
        // Process events first
        self.plugin_manager.process_events();

        // Handle events and update app state
        for event in self.plugin_manager.event_bus_mut().drain_events() {
            match event {
                AppEvent::CanvasZoomChanged { zoom } => {
                    self.canvas.set_zoom(zoom);
                }
                AppEvent::ToolSelected { tool_name } => {
                    // Parse and set tool
                }
                // ... handle other events
            }
        }

        // Render UI
        egui::SidePanel::left("plugins").show(ctx, |ui| {
            self.plugin_manager.render_plugins(ui);
        });
    }
}
```

### Questions to Consider

1. **Plugin Layout**: Should plugins render in a single sidebar or separate panels?
2. **State Ownership**: Should plugins own state or just reflect app state?
3. **Event Loop**: Should we process events before or after rendering?
4. **Plugin Enable/Disable**: Should users be able to toggle plugins at runtime?
5. **Plugin Persistence**: Should plugin states be saved/restored?

## Important File Locations

### Plugin System Files
- `/home/erik/repos/form_factor/crates/form_factor_plugins/` - Plugin crate
- `/home/erik/repos/form_factor/crates/form_factor_plugins/src/lib.rs` - Public API
- `/home/erik/repos/form_factor/Cargo.toml` - Workspace with plugins member

### Main Application (Integration Target)
- `crates/form_factor/src/main.rs` - Application entry point
- `crates/form_factor/Cargo.toml` - Main crate dependencies (needs plugin dep)

### Core Files
- `/home/erik/repos/form_factor/WORKSPACE.md` - Architecture docs
- `/home/erik/repos/form_factor/CLAUDE.md` - Project guidelines

## Key Architecture Patterns (Updated)

### Plugin Event Flow
1. User interacts with plugin UI
2. Plugin emits event via `ctx.events.emit()`
3. Event queued in MPSC channel
4. Manager drains events in next update
5. Events distributed to all plugins (including emitter)
6. Plugins can respond with new events

### Plugin Lifecycle
1. **Registration**: `PluginManager::register()` + `on_load()` callback
2. **Event Processing**: `manager.process_events()` each frame
3. **Rendering**: `manager.render_plugins()` each frame
4. **Saving**: `manager.save_plugins()` before state save
5. **Shutdown**: `manager.shutdown()` on app close

### Feature Flag Pattern
```toml
[features]
plugin-canvas = ["dep:form_factor_plugins", "form_factor_plugins/plugin-canvas"]
```

## Testing Commands

```bash
# Test just the plugin system
cargo test -p form_factor_plugins --all-features

# Test entire workspace
cargo test --workspace --all-features

# Check plugins compile
cargo check -p form_factor_plugins --all-features

# Clippy for plugins
cargo clippy -p form_factor_plugins --all-features

# Build with all plugins enabled
cargo build --features all-plugins
```

## Common Build Commands

```bash
# Default build (eframe backend, no plugins)
cargo build

# With all features (including plugins)
cargo build --features dev

# Specific plugin features
cargo build --features plugin-canvas,plugin-layers

# Check workspace
cargo check --workspace --all-features

# Run with plugins
cargo run --features all-plugins
```

## Known Issues / Gotchas

1. **Plugins not yet integrated**: Plugin system exists but isn't wired to main app
2. **State sync**: No automatic state synchronization between app and plugins
3. **Event handling**: App doesn't yet listen to or emit plugin events
4. **Feature propagation**: Main crate needs to propagate plugin features

## Recent Commits

```
138fe6f (HEAD -> plugins) Add plugin system with event bus architecture
b7f0215 Remove Podman setup documentation files
d32b0e9 Add continuation context for next session
aaa3092 Fix unused import warnings in canvas/io.rs
f6c177d Fix feature propagation to drawing crate
```

## Getting Back Up to Speed

1. ✅ Plugin system is implemented and tested
2. ⏭️ Next: Integrate plugins into main application
3. Read this file for context on what's been built
4. Check `crates/form_factor_plugins/src/lib.rs` for plugin API
5. Look at individual plugin implementations for examples
6. Review `AppEvent` enum to understand available events
7. Start integration by adding plugin dependency to main crate

---

**Ready to continue**: The plugin system is complete, tested, and committed. Next step is to integrate it into the main application and wire up the events to actual functionality.
