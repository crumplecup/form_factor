# Main.rs Refactoring Plan

**Status:** ✅ Phase 1 Complete, Phase 2 In Progress
**Created:** 2025-12-07
**Last Updated:** 2025-12-08
**Goal:** Refactor main.rs from a "dumping ground" into a well-organized, maintainable structure while preserving all functionality.

## ✅ Completed Work (2025-12-08)

### Error Handling Refactor
Before starting main.rs refactor, we completed a comprehensive error handling refactor following CLAUDE.md patterns:
- Converted all error types to use `derive_more::Display` and `derive_more::Error`
- Established proper hierarchy: Module errors → Crate umbrellas → Workspace umbrella
- Added `#[track_caller]` for automatic location tracking throughout
- Properly wrapped all external errors (opencv, serde, etc.)
- See [ERROR_REFACTOR_STRATEGY.md](ERROR_REFACTOR_STRATEGY.md) for full details

### Main.rs Module Extraction (Phase 1)
Successfully extracted main.rs logic into focused modules following Rust idioms:
- ✅ Created skeletal module structure (lib.rs with mod declarations)
- ✅ Extracted property rendering (PropertyRenderer helper type)
- ✅ Extracted file dialogs (FileDialogs helper type)
- ✅ Extracted plugin setup (PluginSetup helper type)
- ✅ Extracted detection tasks (TextDetectionTask, LogoDetectionTask, OcrExtractionTask)
- ✅ Extracted type converters (ToolConverter, LayerConverter)
- ✅ Extracted file events (FileEventHandlers)
- ✅ Extracted canvas events (CanvasEventHandlers)
- ✅ Extracted layer events (LayerEventHandlers)
- ✅ Extracted object events (ObjectEventHandlers)
- ✅ Extracted detection result handlers (DetectionResultHandlers)
- ✅ All helpers are public and documented
- ✅ All helpers use `#[instrument]` for tracing
- ✅ Proper feature gating throughout
- ✅ Full workspace compiles with `just check`

**Architecture:**
```
crates/form_factor/src/
├── lib.rs              # Module declarations + pub use exports
├── main.rs             # Slim binary entry point
├── property_rendering.rs
├── file_dialogs.rs
├── plugin_setup.rs
├── detection_tasks.rs
├── converters.rs
├── file_events.rs
├── canvas_events.rs
├── layer_events.rs
├── object_events.rs
└── detection_results.rs
```

**Current main.rs size:** Reduced from ~1265 lines to manageable event loop

**Benefits achieved:**
- Clear separation of concerns
- Testable helper types
- Better documentation
- Easier to add features
- No namespace pollution (all via Type::method() pattern)

## Current State Analysis

### What Works (DO NOT BREAK)
- ✅ Application initialization and setup
- ✅ Plugin system registration and lifecycle
- ✅ Event bus wiring between plugins and canvas
- ✅ Mode switching (Canvas, TemplateManager, InstanceFilling)
- ✅ Background thread spawning for detection (text, logo, OCR)
- ✅ Toast notifications
- ✅ File dialogs (open, save, load image)
- ✅ Property editor for shapes/detections
- ✅ Field type selector dialog
- ✅ Selection change detection and event emission
- ✅ Recent project auto-loading

### Problems

1. **1265 lines of main.rs** - Too large, hard to navigate
2. **Massive event handler** (lines 154-1041) - 887 lines of nested match statements
3. **Inline background thread logic** - Text detection (lines 451-549), Logo detection (lines 550-702), OCR (lines 703-847)
4. **Duplicated file dialog patterns** - Open/Save/SaveAs all similar
5. **String-based type conversion** - Tool names, layer names converted from strings (brittle)
6. **Mixed concerns** - App lifecycle, UI rendering, event handling, background tasks all in one place
7. **Feature-gated code blocks** - Makes it hard to reason about what's compiled
8. **Limited reusability** - Detection logic can't be used elsewhere

### Key Dependencies
- `form_factor` crate (App, AppContext, DrawingCanvas, etc.)
- `form_factor_drawing` (Shape, DetectionType, FieldTypeSelector)
- `form_factor_plugins` (Plugin types, event bus)
- `form_factor_cv` (TextDetector, LogoDetector)
- `egui` and `egui_notify`
- `rfd` for file dialogs

## Refactoring Strategy

### Phase 1: Extract Background Detection Logic
**Goal:** Move detection spawning to separate modules using helper types

#### 1.1 Create `detection_tasks.rs`
- Define helper types: `TextDetectionTask`, `LogoDetectionTask`, `OcrExtractionTask`
- Each type has static/associated methods for spawning background threads
- Pattern: `TextDetectionTask::spawn(form_path, sender)`
- **Add `#[instrument]` to all public methods**
- **Add structured logging at key points** (start, progress, completion, errors)
- **Proper error handling** - no panics, emit error events

**Example:**
```rust
pub struct TextDetectionTask;

impl TextDetectionTask {
    #[cfg(feature = "text-detection")]
    #[instrument(skip(sender), fields(form_path))]
    pub fn spawn(form_path: String, sender: EventSender) {
        tracing::info!("Spawning text detection background task");
        
        std::thread::spawn(move || {
            tracing::debug!("Text detection thread started");
            
            match Self::run_detection(&form_path) {
                Ok(shapes) => {
                    tracing::info!(count = shapes.len(), "Text detection complete");
                    // Emit success events
                }
                Err(e) => {
                    tracing::error!(error = %e, "Text detection failed");
                    // Emit error event
                }
            }
        });
    }
    
    #[instrument(fields(form_path))]
    fn run_detection(form_path: &str) -> Result<Vec<Shape>, String> {
        // Detection logic with proper error handling
    }
}
```

**Benefits:**
- Testable in isolation
- Reusable from other contexts
- Clear namespacing: `TextDetectionTask::spawn()`
- Feature flags isolated to impl blocks
- No namespace pollution

**Risks:**
- Must preserve exact same Shape creation logic
- Event emission timing must stay identical
- Thread spawning behavior must match

**Testing:**
```bash
just check-package form_factor
just test-package form_factor
```

#### 1.2 Create `detection_results.rs`
- Define `DetectionResultHandler` type
- Methods for deserialization and processing results
- Pattern: `DetectionResultHandler::process(event, canvas)`
- **Add `#[instrument]` to all public methods**
- **Return proper Result types** instead of ignoring errors
- **Log deserialization success/failure** with counts

**Example:**
```rust
pub struct DetectionResultHandler;

impl DetectionResultHandler {
    #[instrument(skip(canvas), fields(json_len = shapes_json.len()))]
    pub fn process_text_results(shapes_json: &str, canvas: &mut DrawingCanvas) -> Result<usize, String> {
        tracing::debug!("Processing text detection results");
        
        let shapes: Vec<Shape> = serde_json::from_str(shapes_json)
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to deserialize shapes");
                format!("Deserialization failed: {}", e)
            })?;
        
        let count = shapes.len();
        tracing::info!(count, "Adding text detection shapes to canvas");
        
        for shape in shapes {
            canvas.add_detection(shape);
        }
        
        Ok(count)
    }
    
    #[instrument(skip(canvas), fields(json_len = shapes_json.len()))]
    pub fn process_logo_results(shapes_json: &str, canvas: &mut DrawingCanvas) -> Result<usize, String> {
        // Similar pattern
    }
}
```

**Benefits:**
- Separates result handling from initiation
- Easier to add new detection types
- Clear API: `DetectionResultHandler::process_text_results()`
- **Proper error propagation** for caller handling
- **Observable via tracing** - can debug issues easily

---

### Phase 2: Extract Event Handling
**Goal:** Break up the massive event match statement using handler types

#### 2.1 Create `event_handlers/` module directory
```
src/event_handlers/
├── mod.rs              # Module exports only
├── canvas.rs           # CanvasEventHandler
├── layers.rs           # LayerEventHandler
├── objects.rs          # ObjectEventHandler
├── files.rs            # FileEventHandler
├── detection.rs        # DetectionEventHandler
└── selection.rs        # SelectionEventHandler
```

#### 2.2 Pattern
Each handler module exports a type with methods:
- **Add `#[instrument]` to all public methods**
- **Return Result types** where operations can fail
- **Log state changes** at appropriate levels (debug for minor, info for major)
- **Log errors before returning** them

```rust
// canvas.rs
pub struct CanvasEventHandler;

impl CanvasEventHandler {
    #[instrument(skip(canvas), fields(zoom))]
    pub fn handle_zoom_changed(zoom: f32, canvas: &mut DrawingCanvas) {
        tracing::debug!(zoom, "Canvas zoom changed");
        canvas.set_zoom(zoom);
    }
    
    #[instrument(skip(canvas), fields(x, y))]
    pub fn handle_pan_changed(x: f32, y: f32, canvas: &mut DrawingCanvas) {
        tracing::debug!(x, y, "Canvas pan changed");
        canvas.set_pan_offset(x, y);
    }
    
    #[instrument(skip(canvas), fields(tool_name))]
    pub fn handle_tool_selected(tool_name: &str, canvas: &mut DrawingCanvas) -> Result<(), String> {
        tracing::info!(tool_name, "Tool selected");
        
        let tool = ToolParser::from_name(tool_name)
            .ok_or_else(|| {
                tracing::warn!(tool_name, "Unknown tool name");
                format!("Unknown tool: {}", tool_name)
            })?;
        
        canvas.set_tool(tool);
        Ok(())
    }
}

// layers.rs
pub struct LayerEventHandler;

impl LayerEventHandler {
    #[instrument(skip(canvas), fields(layer_name, visible))]
    pub fn handle_visibility_changed(layer_name: &str, visible: bool, canvas: &mut DrawingCanvas) -> Result<(), String> {
        tracing::info!(layer_name, visible, "Layer visibility changed");
        
        let layer_type = LayerParser::from_name(layer_name)
            .ok_or_else(|| {
                tracing::warn!(layer_name, "Unknown layer name");
                format!("Unknown layer: {}", layer_name)
            })?;
        
        // Toggle logic with proper state checking
        if canvas.layer_manager().is_visible(layer_type) != visible {
            canvas.layer_manager_mut().toggle_layer(layer_type);
            tracing::debug!(layer_name, visible, "Layer visibility toggled");
        }
        
        Ok(())
    }
    
    #[instrument(skip(canvas, app_state), fields(layer_name))]
    pub fn handle_clear_requested(
        layer_name: &str,
        canvas: &mut DrawingCanvas,
        app_state: &mut AppState,
    ) -> Result<(), String> {
        tracing::info!(layer_name, "Layer clear requested");
        // Layer clearing logic with proper error handling
    }
}
```

**Benefits:**
- Clear separation of concerns
- Easy to find event handling code: `CanvasEventHandler::handle_zoom_changed()`
- Testable handlers
- Reduces main.rs significantly
- No namespace pollution
- Logical grouping of related handlers

**Risks:**
- Must preserve exact event ordering
- Borrowing rules may require careful structuring
- Need to maintain state mutations correctly

---

### Phase 3: Extract File Dialog Operations
**Goal:** Centralize file dialog patterns using helper type

#### 3.1 Create `file_dialogs.rs`
- **Add `#[instrument]` to all public methods**
- **Log dialog open/cancel/success** at appropriate level
- **Return PathBuf directly** (Option is implicit via dialog)

```rust
pub struct FileDialogs;

impl FileDialogs {
    #[instrument]
    pub fn open_project() -> Option<PathBuf> {
        tracing::debug!("Opening project file dialog");
        
        let result = rfd::FileDialog::new()
            .add_filter("Form Factor Project", &["ffp"])
            .pick_file();
        
        match &result {
            Some(path) => tracing::info!(path = ?path, "Project file selected"),
            None => tracing::debug!("Project file dialog cancelled"),
        }
        
        result
    }
    
    #[instrument(fields(default_name))]
    pub fn save_project(default_name: &str) -> Option<PathBuf> {
        tracing::debug!(default_name, "Opening save project dialog");
        
        let result = rfd::FileDialog::new()
            .add_filter("Form Factor Project", &["ffp"])
            .set_file_name(format!("{}.ffp", default_name))
            .save_file();
        
        match &result {
            Some(path) => tracing::info!(path = ?path, "Save location selected"),
            None => tracing::debug!("Save dialog cancelled"),
        }
        
        result
    }
    
    #[instrument]
    pub fn load_image() -> Option<PathBuf> {
        tracing::debug!("Opening image file dialog");
        
        let result = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file();
        
        match &result {
            Some(path) => tracing::info!(path = ?path, "Image file selected"),
            None => tracing::debug!("Image dialog cancelled"),
        }
        
        result
    }
}
```

**Benefits:**
- DRY principle
- Consistent file filtering
- Easy to add new dialog types
- Mockable for testing
- Clear API: `FileDialogs::open_project()`
- No namespace pollution

---

### Phase 4: Extract Plugin Setup
**Goal:** Move plugin registration out of FormFactorApp::new()

#### 4.1 Create `plugin_setup.rs`
- **Add `#[instrument]` to all methods**
- **Log each plugin registration** with plugin name
- **Count and report total** registered plugins

```rust
pub struct PluginSetup;

impl PluginSetup {
    #[cfg(feature = "plugins")]
    #[instrument]
    pub fn create_manager() -> PluginManager {
        tracing::info!("Creating plugin manager");
        let mut manager = PluginManager::new();
        let mut count = 0;
        
        #[cfg(feature = "plugin-canvas")]
        {
            Self::register_canvas_plugin(&mut manager);
            count += 1;
        }
        
        #[cfg(feature = "plugin-layers")]
        {
            Self::register_layers_plugin(&mut manager);
            count += 1;
        }
        
        #[cfg(feature = "plugin-file")]
        {
            Self::register_file_plugin(&mut manager);
            count += 1;
        }
        
        #[cfg(feature = "plugin-detection")]
        {
            Self::register_detection_plugin(&mut manager);
            count += 1;
        }
        
        #[cfg(feature = "plugin-properties")]
        {
            Self::register_properties_plugin(&mut manager);
            count += 1;
        }
        
        tracing::info!(count, "Plugin manager created with {} plugin(s)", count);
        manager
    }
    
    #[cfg(all(feature = "plugins", feature = "plugin-canvas"))]
    #[instrument(skip(manager))]
    fn register_canvas_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(CanvasPlugin::new()));
        tracing::info!("Registered canvas plugin");
    }
    
    #[cfg(all(feature = "plugins", feature = "plugin-layers"))]
    #[instrument(skip(manager))]
    fn register_layers_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(LayersPlugin::new()));
        tracing::info!("Registered layers plugin");
    }
    
    // ... other registration methods with same pattern
}
```

**Benefits:**
- FormFactorApp::new() becomes cleaner
- Plugin registration is testable
- Feature flags isolated to one place
- Clear API: `PluginSetup::create_manager()`
- Easy to see all plugins at a glance

---

### Phase 5: Extract UI Rendering
**Goal:** Separate UI rendering from application logic using renderer types

#### 5.1 Create `ui_properties.rs`
- **Add `#[instrument]` to all public methods**
- **Log rendering operations** at debug level
- **Return Result** if rendering can fail
- **Handle missing data gracefully** with proper error messages

```rust
pub struct PropertyRenderer;

impl PropertyRenderer {
    #[instrument(skip(ui, canvas), fields(shape_idx))]
    pub fn render_shape_properties(
        ui: &mut egui::Ui,
        canvas: &DrawingCanvas,
        shape_idx: usize,
    ) -> Result<(), String> {
        tracing::debug!(shape_idx, "Rendering shape properties");
        
        let shape = canvas.shapes().get(shape_idx)
            .ok_or_else(|| {
                tracing::warn!(shape_idx, "Shape index out of bounds");
                format!("Shape {} not found", shape_idx)
            })?;
        
        // Render properties with proper error handling
        Ok(())
    }
    
    #[instrument(skip(ui, canvas), fields(det_type = ?det_type, det_idx))]
    pub fn render_detection_properties(
        ui: &mut egui::Ui,
        canvas: &DrawingCanvas,
        det_type: DetectionType,
        det_idx: usize,
    ) -> Result<(), String> {
        tracing::debug!(det_type = ?det_type, det_idx, "Rendering detection properties");
        
        // Render with proper bounds checking and error handling
        Ok(())
    }
}
```

#### 5.2 Consolidate ui_template.rs and ui_update.rs
- Already use function-based approach (OK to keep as is)
- Or refactor to `TemplateRenderer` and `InstanceRenderer` types for consistency

**Benefits:**
- UI code separated from business logic
- Easier to refactor UI independently
- Better feature flag organization
- Consistent API: `PropertyRenderer::render_shape_properties()`

---

### Phase 6: Type Safety Improvements
**Goal:** Replace string-based conversions with type-safe alternatives

#### 6.1 Create `type_conversions.rs` with helper types
- **Add `#[instrument]` to all public methods**
- **Log unknown names** at warn level
- **Return Option** for clean error handling

```rust
pub struct ToolParser;

impl ToolParser {
    #[instrument(fields(name))]
    pub fn from_name(name: &str) -> Option<ToolMode> {
        let result = match name {
            "Select" => Some(ToolMode::Select),
            "Rectangle" => Some(ToolMode::Rectangle),
            "Circle" => Some(ToolMode::Circle),
            "Freehand" => Some(ToolMode::Freehand),
            "Edit" => Some(ToolMode::Edit),
            "Rotate" => Some(ToolMode::Rotate),
            _ => None,
        };
        
        if result.is_none() {
            tracing::warn!(name, "Unknown tool name");
        } else {
            tracing::debug!(name, tool = ?result, "Tool parsed");
        }
        
        result
    }
}

pub struct LayerParser;

impl LayerParser {
    #[instrument(fields(name))]
    pub fn from_name(name: &str) -> Option<LayerType> {
        let result = match name {
            "Canvas" => Some(LayerType::Canvas),
            "Detections" => Some(LayerType::Detections),
            "Shapes" => Some(LayerType::Shapes),
            "Grid" => Some(LayerType::Grid),
            "Template" => Some(LayerType::Template),
            "Instance" => Some(LayerType::Instance),
            _ => None,
        };
        
        if result.is_none() {
            tracing::warn!(name, "Unknown layer name");
        } else {
            tracing::debug!(name, layer = ?result, "Layer parsed");
        }
        
        result
    }
}
```

**Benefits:**
- Centralized conversion logic
- Testable
- Clear API: `ToolParser::from_name("Select")`
- No namespace pollution

**Alternative:** Implement `FromStr` on the types themselves (requires changes to form_factor crate)

**Note:** May require changes to form_factor crate for FromStr approach

---

## Module Structure (Target)

```
crates/form_factor/src/
├── main.rs                       # ~150 lines: bootstrap, App impl skeleton
├── ui_template.rs                # Template mode UI (function-based, OK)
├── ui_update.rs                  # Instance filling UI (function-based, OK)
├── ui_properties.rs              # NEW: PropertyRenderer type
├── detection_tasks.rs            # NEW: TextDetectionTask, LogoDetectionTask, OcrExtractionTask
├── detection_results.rs          # NEW: DetectionResultHandler type
├── file_dialogs.rs               # NEW: FileDialogs type
├── plugin_setup.rs               # NEW: PluginSetup type
├── type_conversions.rs           # NEW: ToolParser, LayerParser types
└── event_handlers/
    ├── mod.rs                    # Module exports only
    ├── canvas.rs                 # CanvasEventHandler type
    ├── layers.rs                 # LayerEventHandler type
    ├── objects.rs                # ObjectEventHandler type
    ├── files.rs                  # FileEventHandler type
    ├── detection.rs              # DetectionEventHandler type
    └── selection.rs              # SelectionEventHandler type
```

**Pattern:** Every module exports helper types with associated methods, not free functions
**Usage:** `HelperType::method()` instead of `module::function()`
**Benefits:** No namespace pollution, clear grouping, testable, mockable

---

## Implementation Order

### Step 1: Create Skeletal Modules (No-op phase)
- Create all new files with empty/stub implementations
- Add module declarations to main.rs
- Verify compilation: `just check-package form_factor`
- **Commit:** "feat(form_factor): Add skeletal modules for refactoring"

### Step 2: Extract Property Rendering
- Create `ui_properties.rs` with `PropertyRenderer` type
- Move `render_shape_properties` and `render_detection_properties` as methods
- Update main.rs to use `PropertyRenderer::render_shape_properties()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract property rendering to PropertyRenderer"

### Step 3: Extract File Dialogs
- Create `file_dialogs.rs` with `FileDialogs` type
- Implement methods: `open_project()`, `save_project()`, `load_image()`
- Replace inline dialogs in main.rs with `FileDialogs::open_project()` etc.
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract file dialogs to FileDialogs helper"

### Step 4: Extract Plugin Setup
- Create `plugin_setup.rs` with `PluginSetup` type
- Implement `create_manager()` and private registration methods
- Update FormFactorApp::new() to use `PluginSetup::create_manager()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract plugin setup to PluginSetup helper"

### Step 5: Extract Detection Tasks (Text)
- Create `detection_tasks.rs` with `TextDetectionTask` type
- Implement `spawn(form_path, sender)` method with exact thread logic
- Replace inline text detection spawn with `TextDetectionTask::spawn()`
- Test: `just check-package form_factor` (no API test yet)
- **Commit:** "refactor(form_factor): Extract text detection to TextDetectionTask"

### Step 6: Extract Detection Tasks (Logo)
- Add `LogoDetectionTask` type to `detection_tasks.rs`
- Implement `spawn(form_path, sender)` method with exact thread logic
- Replace inline logo detection spawn with `LogoDetectionTask::spawn()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract logo detection to LogoDetectionTask"

### Step 7: Extract Detection Tasks (OCR)
- Add `OcrExtractionTask` type to `detection_tasks.rs`
- Implement `spawn(form_path, detections, sender)` method with exact thread logic
- Replace inline OCR spawn with `OcrExtractionTask::spawn()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract OCR extraction to OcrExtractionTask"

### Step 8: Extract Type Conversions
- Create `type_conversions.rs` with `ToolParser` and `LayerParser` types
- Implement `from_name()` methods for each
- Replace inline string matching with parser calls
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract type conversions to parser helpers"

### Step 9: Extract Event Handlers (Files)
- Create `event_handlers/files.rs` with `FileEventHandler` type
- Implement methods: `handle_open_requested()`, `handle_save_requested()`, etc.
- Move file event handling from main.rs to use `FileEventHandler::handle_*()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract file events to FileEventHandler"

### Step 10: Extract Event Handlers (Canvas)
- Create `event_handlers/canvas.rs` with `CanvasEventHandler` type
- Implement methods: `handle_zoom_changed()`, `handle_pan_changed()`, `handle_tool_selected()`
- Move canvas event handling from main.rs to use `CanvasEventHandler::handle_*()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract canvas events to CanvasEventHandler"

### Step 11: Extract Event Handlers (Layers)
- Create `event_handlers/layers.rs` with `LayerEventHandler` type
- Implement methods: `handle_visibility_changed()`, `handle_clear_requested()`, etc.
- Move layer event handling from main.rs to use `LayerEventHandler::handle_*()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract layer events to LayerEventHandler"

### Step 12: Extract Event Handlers (Objects)
- Create `event_handlers/objects.rs` with `ObjectEventHandler` type
- Implement methods: `handle_delete_requested()`, `handle_visibility_changed()`
- Move object event handling from main.rs to use `ObjectEventHandler::handle_*()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract object events to ObjectEventHandler"

### Step 13: Extract Event Handlers (Detection)
- Create `event_handlers/detection.rs` with `DetectionEventHandler` type
- Implement methods: `handle_complete()`, `handle_failed()`, `handle_results_ready()`
- Move detection event handling from main.rs to use `DetectionEventHandler::handle_*()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract detection events to DetectionEventHandler"

### Step 14: Extract Event Handlers (Selection)
- Create `event_handlers/selection.rs` with `SelectionEventHandler` type
- Implement selection change detection logic as methods
- Move selection handling from main.rs to use `SelectionEventHandler::check_changes()`
- Test: `just check-package form_factor`
- **Commit:** "refactor(form_factor): Extract selection tracking to SelectionEventHandler"

### Step 15: Final Cleanup
- Review main.rs for any remaining inline code
- Add module-level documentation to all new files
- Verify all helpers use type-based pattern consistently
- Run full test suite: `just check-all form_factor`
- **Commit:** "refactor(form_factor): Complete main.rs refactoring"

### Step 16: Integration Testing
- Manual testing of all features
- API tests if needed: `just test-api`
- Final verification: `just check-all`

---

## Testing Strategy

After each step:
```bash
just check-package form_factor       # Basic compilation
```

After completing phases:
```bash
just test-package form_factor        # Unit/integration tests
just check-all form_factor           # Full checks (clippy, fmt, test)
```

Before merge:
```bash
just check-all                       # Entire workspace
# Manual testing of:
# - Canvas mode: drawing shapes, tools
# - Template mode: template management
# - Instance mode: data entry
# - File operations: open, save, load image
# - Detection: text, logo, OCR (if safe to use API)
# - Plugin interactions: layers, properties, detection
# - Selection: shapes, detections, property display
```

---

## Success Criteria

### Functional
- ✅ All existing features work identically
- ✅ No regressions in UI behavior
- ✅ Detection threads spawn and complete correctly
- ✅ Events flow through system as before
- ✅ File operations work
- ✅ Mode switching works
- ✅ Selection tracking works

### Code Quality
- ✅ main.rs under 300 lines
- ✅ No function over 100 lines
- ✅ Clear module boundaries
- ✅ Each module has single responsibility
- ✅ Feature flags properly isolated
- ✅ No clippy warnings
- ✅ All tests passing

### Maintainability
- ✅ Easy to find code for specific features
- ✅ New detection types easy to add
- ✅ New event handlers easy to add
- ✅ UI changes isolated from business logic
- ✅ Testable components

---

## Risks and Mitigations

### Risk: Breaking Event Ordering
**Mitigation:** Extract event handlers one at a time, test after each

### Risk: Borrowing Issues When Splitting Code
**Mitigation:** May need to pass specific fields instead of whole structs; use `&mut` carefully

### Risk: Feature Flag Combinations
**Mitigation:** Run `just check-features` before final merge

### Risk: Background Thread Timing Changes
**Mitigation:** Preserve exact spawn/emit patterns; add integration test if needed

### Risk: Breaking Plugin System
**Mitigation:** Test plugin registration thoroughly; verify event bus still works

---

## Tracing and Error Handling Guidelines

### Instrumentation Requirements
- ✅ **All public methods** have `#[instrument]`
- ✅ **Skip large params**: `skip(canvas, ui, manager)`
- ✅ **Include context fields**: `fields(shape_idx, tool_name)`
- ✅ **Log at appropriate levels**:
  - `debug!()` - Minor state changes, successful parsing
  - `info!()` - Major operations, plugin registration, file operations
  - `warn!()` - Unknown names, invalid input, recoverable errors
  - `error!()` - Operation failures, exceptions

### Error Handling Requirements
- ✅ **Return Result types** where operations can fail
- ✅ **No panics** - all errors returned or logged
- ✅ **Log before returning errors** for observability
- ✅ **Structured error context**: `error = %e, path = ?path`
- ✅ **Meaningful error messages** for user feedback

### Testing Observability
With proper instrumentation, debugging becomes:
```
RUST_LOG=form_factor=debug cargo run
```

Shows:
- Which handler processed which event
- Tool/layer parsing success/failure
- File dialog interactions
- Detection task lifecycle
- Plugin registration

## Notes

- **Go slow and steady** - Small commits, test after each
- **Preserve git history** - Don't squash commits until certain everything works
- **Document changes** - Update CLAUDE.md if new patterns emerge
- **Communicate** - Update this document as we learn from implementation
- **Add tracing everywhere** - This is a refactoring opportunity to improve observability
- **No shortcuts on errors** - Proper Result types and error logging throughout

---

## Future Improvements (Not in this refactor)

- Move type-unsafe string parsing to form_factor crate with FromStr impls
- Consider extracting FormFactorApp to separate module
- Add unit tests for event handlers
- Consider state machine for app modes
- Add builder pattern for FormFactorApp if it gets more complex
