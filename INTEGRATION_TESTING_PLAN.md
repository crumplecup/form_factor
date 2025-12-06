# Integration Testing Implementation Plan

**Status:** In Progress - Phase 1 Partial Complete
**Created:** 2024-12-05
**Updated:** 2024-12-06
**Goal:** Implement comprehensive integration tests to catch regressions in plugin coordination, canvas workflows, and feature interactions

## Progress Summary

**Completed:**
- ✅ Phase 1.1: Test helper infrastructure (canvas_helpers.rs, mod.rs)
- ✅ Phase 1.2: Basic canvas integration tests (15 tests passing)
- ✅ Test introspection APIs added to DrawingCanvas
- ✅ Clippy fixes applied to source code

**Current Status:** 15/15 tests passing, 0 clippy warnings

**Remaining in Phase 1:**
- ⏳ Phase 1.2: Tool workflow tests (requires egui interaction simulation)
- ⏳ Phase 1.3: Plugin coordination tests (requires form_factor crate)
- ⏳ Phase 1.2: Complex state machine tests (requires interaction simulation)

## Phase 1 Implementation Notes

### What Was Actually Built

**Test Infrastructure Created:**
```
crates/form_factor_drawing/tests/
├── helpers/
│   ├── mod.rs                    # ✅ Module organization and re-exports
│   ├── canvas_helpers.rs         # ✅ Canvas utilities (7 helpers)
│   └── plugin_helpers.rs         # ✅ Created but unused (requires form_factor crate)
└── canvas_integration_test.rs    # ✅ 15 passing tests
```

**Tests Implemented (15 total):**

1. **Helper Verification (1 test)**
   - ✅ `test_helpers_create_test_canvas` - Validates test infrastructure

2. **Canvas State & Initialization (2 tests)**
   - ✅ `test_canvas_default_state` - Initial state verification
   - ✅ `test_project_name_modification` - Metadata management

3. **Tool Management (1 test)**
   - ✅ `test_tool_mode_switching` - Tool changes (Select, Rectangle, Circle, etc.)

4. **Shape & Layer Integration (2 tests)**
   - ✅ `test_shape_addition` - Adding shapes to canvas
   - ✅ `test_shapes_on_shapes_layer` - Layer filtering
   - ✅ `test_detections_separate_from_shapes` - Layer isolation

5. **State Machine (3 tests)**
   - ✅ `test_initial_state_is_idle` - Default state
   - ✅ `test_state_persists_across_tool_changes` - State stability
   - ✅ `test_state_remains_idle_with_shapes` - State with data

6. **Zoom & Pan (4 tests)**
   - ✅ `test_zoom_level_modification` - Zoom state changes
   - ✅ `test_pan_offset_state` - Pan state management
   - ✅ `test_zoom_and_pan_persistence` - State persistence
   - ✅ `test_zoom_with_shapes` - Zoom doesn't affect shape storage
   - ✅ `test_pan_with_shapes` - Pan doesn't affect shape storage

**Helper Functions Implemented:**
- `create_test_canvas()` - Canvas with defaults
- `create_canvas_with_shapes(count)` - Pre-populated canvas
- `assert_active_tool()` - Tool verification
- `assert_shape_count()` - Shape count verification
- `assert_zoom_level()` - Zoom verification
- `assert_pan_offset()` - Pan verification
- `get_shapes_on_layer()` - Layer introspection

**Source Code Enhancements:**
- Added `current_state()` - Expose CanvasState for testing
- Added `shapes_on_layer()` - Filter shapes by LayerType
- Added `selected_shape_index()`, `selected_field_index()` - Selection introspection
- Added `current_template_mode()`, `current_instance_mode()` - Mode introspection
- Exported `CanvasState` from root module
- **Note:** Removed `#[cfg(test)]` guards - they don't work for integration tests in `tests/` directory (separate crate)

### Deviations from Original Plan

**Not Implemented (Deferred):**
1. **Tool Workflow Tests** - Planned 6 tests
   - Requires egui interaction simulation (mouse events, drag gestures)
   - Blocked on: Need to understand how to simulate egui::Response in tests
   - Examples: `test_rectangle_tool_workflow`, `test_circle_tool_workflow`

2. **Complex State Machine Tests** - Planned 4 tests
   - Requires triggering Drawing, DraggingVertex, Rotating states
   - Blocked on: Same as tool workflows - need interaction simulation
   - Examples: `test_idle_to_drawing_transition`, `test_drawing_to_idle_transition`

3. **Plugin Coordination Tests** - Planned 12 tests (Phase 1.3)
   - Requires form_factor crate (PluginManager lives there)
   - Would need separate test file in form_factor crate
   - Deferred to later phase

4. **Test Infrastructure Change**
   - Plan called for `#[cfg(test)]` on introspection APIs
   - **Reality:** Integration tests are in separate `tests/` crate, so `#[cfg(test)]` doesn't apply
   - **Solution:** Made APIs always-public with `#[doc(hidden)]` or documentation noting test usage

### Key Learnings

1. **Integration Tests are a Separate Crate**
   - `tests/` directory compiles as separate crate from `src/`
   - `#[cfg(test)]` only applies within the same crate
   - Test-only APIs must be always-public or use `pub(crate)` + unit tests

2. **egui Interaction Simulation is Complex**
   - Tool workflows require `egui::Response` with pointer events
   - May need mock responses or test-only interaction APIs
   - Consider: Add `test_handle_input()` that takes position + event type

3. **Plugin Tests Need Different Location**
   - PluginManager is in `form_factor` crate
   - Plugin coordination tests should go in `crates/form_factor/tests/`
   - Can still use pattern established here

4. **Builder Pattern Discovery**
   - `Rectangle` uses `Rectangle::from_corners()`, not builder
   - `Circle` uses either `Circle::new()` or `CircleBuilder`
   - Tests revealed inconsistency in shape construction APIs

### Commit: fdde4da

**Files Changed:** 10 files, +852 insertions, -45 deletions

**Source Changes:**
- `crates/form_factor_drawing/src/canvas/core.rs` - Added introspection APIs
- `crates/form_factor_drawing/src/canvas/mod.rs` - Exported CanvasState
- `crates/form_factor_drawing/src/lib.rs` - Re-exported CanvasState
- `crates/form_factor_drawing/src/canvas/rendering.rs` - Clippy fixes
- `crates/form_factor_drawing/src/canvas/tools.rs` - Clippy fixes
- `crates/form_factor_drawing/src/instance/migration.rs` - Boxed large enum variants

**Test Files Created:**
- `crates/form_factor_drawing/tests/canvas_integration_test.rs` - 15 tests
- `crates/form_factor_drawing/tests/helpers/mod.rs` - Module organization
- `crates/form_factor_drawing/tests/helpers/canvas_helpers.rs` - 7 helpers
- `crates/form_factor_drawing/tests/helpers/plugin_helpers.rs` - Placeholder

## Executive Summary

Current unit tests cover individual components well (templates, instances, enums, individual plugins), but lack integration tests for:
- Multi-component workflows (canvas + plugins + layers)
- Event bus coordination between plugins
- Template/instance editing via canvas interactions
- File operations with full application state
- Feature-gated functionality integration (OCR, detection)

This plan addresses these gaps through 7 test suites organized in 4 implementation phases.

---

## Problem Statement

Recent implementation (Priorities 1-5) added significant canvas and template functionality:
- Field dragging/repositioning (Priority 3 Part 2)
- Field visualization and selection (Priority 3 Part 1)
- Tool mode integration for template fields (Priority 2)
- Multi-page template support (Priority 4)

**Risk:** These features have no integration tests. Changes to event bus, layer system, or canvas state machine could break functionality without test failures.

**Evidence:**
- Zero DrawingCanvas integration tests (only migration tests exist)
- Zero plugin coordination tests (plugins tested in isolation)
- Zero template-canvas interaction tests
- Zero end-to-end file operation tests

---

## Architecture Context

### Key Components

1. **DrawingCanvas** (form_factor_drawing::canvas)
   - State machine: Idle, Drawing, DraggingVertex, DraggingField, Rotating
   - Tool modes: Select, Rectangle, Circle, Freehand, Edit, Rotate
   - Four modules: core, tools, rendering, io

2. **Plugin System** (form_factor_plugins)
   - Event bus (tokio mpsc channels)
   - 5 plugins: Canvas, Layers, File, Detection, OCR
   - AppEvent enum (28+ event types)
   - PluginManager coordinates lifecycle

3. **Layer System** (form_factor_drawing)
   - 4 layer types: Canvas, Detections, Shapes, Grid
   - Z-order rendering
   - Visibility and lock states

4. **Template/Instance System** (form_factor_core, form_factor_drawing)
   - DrawingTemplate with FieldDefinitions
   - DrawingInstance with FieldValues
   - Multi-page support
   - Validation framework

### Critical Integration Points

- Canvas state changes → Plugin events → UI updates
- Tool selection → Canvas behavior → Layer assignment → File state
- Template field editing → Canvas rendering → Event propagation
- File load → Canvas population → Plugin initialization
- Detection/OCR → Canvas detections → Instance population

---

## Phase 1: Foundation (Week 1)

**Goal:** Establish testing infrastructure and core canvas/plugin integration tests

### Files to Create

```
crates/form_factor_drawing/tests/
├── helpers/
│   ├── mod.rs              # Test utilities and mocks
│   ├── canvas_helpers.rs   # Canvas creation, simulation
│   └── plugin_helpers.rs   # Plugin setup, event collection
├── canvas_integration_test.rs
└── plugin_coordination_test.rs
```

### Implementation Steps

#### 1.1 Test Helper Infrastructure ✅ COMPLETE

**Status:** ✅ Implemented and working

**File:** `crates/form_factor_drawing/tests/helpers/mod.rs` ✅
- Module organization and re-exports

**File:** `crates/form_factor_drawing/tests/helpers/canvas_helpers.rs` ✅
- ✅ `create_test_canvas()` - Canvas with default settings
- ✅ `create_canvas_with_shapes(count: usize)` - Pre-populated canvas
- ⏸️ `simulate_click(canvas, pos)` - Deferred (requires egui interaction)
- ⏸️ `simulate_drag(canvas, from, to)` - Deferred (requires egui interaction)
- ⏸️ `simulate_tool_action(canvas, tool, action)` - Deferred (requires egui interaction)
- ✅ `assert_active_tool()` - Tool verification (adapted from planned assert_canvas_state)
- ✅ `assert_shape_count()` - Shape count verification
- ✅ `assert_zoom_level()` - Zoom verification
- ✅ `assert_pan_offset()` - Pan verification
- ✅ `get_shapes_on_layer(canvas, layer)` - Layer inspection

**File:** `crates/form_factor_drawing/tests/helpers/plugin_helpers.rs` ⏸️
- ⏸️ `create_test_plugin_manager()` - Deferred (needs form_factor crate)
- ⏸️ `create_plugin_manager_with(plugins)` - Deferred
- ⏸️ `collect_events(event_bus)` - Deferred
- ⏸️ `emit_and_process(manager, event)` - Deferred
- ⏸️ `assert_event_emitted(events, expected)` - Deferred
- ⏸️ `wait_for_event(bus, predicate)` - Deferred

**Success Criteria:**
- ✅ Helpers compile and pass basic tests
- ✅ Can create canvas (without interaction simulation)
- ⏸️ Can create plugin manager and track events (deferred to form_factor crate)
- ✅ Helper test demonstrates usage

#### 1.2 Canvas Integration Tests ⏸️ PARTIAL

**Status:** ⏸️ 15 tests implemented, 10+ deferred

**File:** `crates/form_factor_drawing/tests/canvas_integration_test.rs` ✅

**Tests Implemented (15 total):**

1. **Helper Verification (1 test)** ✅
   - ✅ `test_helpers_create_test_canvas` - Validates test infrastructure

2. **Canvas State & Initialization (2 tests)** ✅
   - ✅ `test_canvas_default_state` - Initial state verification
   - ✅ `test_project_name_modification` - Metadata management

3. **Tool Management (1 test)** ✅
   - ✅ `test_tool_mode_switching` - Tool changes without interaction

4. **Shape & Layer Integration (3 tests)** ✅
   - ✅ `test_shape_addition` - Adding shapes to canvas
   - ✅ `test_shapes_on_shapes_layer` - Layer filtering
   - ✅ `test_detections_separate_from_shapes` - Layer isolation

5. **State Machine (3 tests)** ✅
   - ✅ `test_initial_state_is_idle` - Default state
   - ✅ `test_state_persists_across_tool_changes` - State stability
   - ✅ `test_state_remains_idle_with_shapes` - State with data

6. **Zoom & Pan (4 tests)** ✅
   - ✅ `test_zoom_level_modification` - Zoom state changes
   - ✅ `test_pan_offset_state` - Pan state management
   - ✅ `test_zoom_and_pan_persistence` - State persistence
   - ✅ `test_zoom_with_shapes` - Zoom doesn't affect shape storage
   - ✅ `test_pan_with_shapes` - Pan doesn't affect shape storage

**Tests Deferred (require egui interaction simulation):**

1. **Tool Workflows** ⏸️ (6 tests planned)
   - ⏸️ `test_rectangle_tool_workflow()` - Complete draw rectangle flow
   - ⏸️ `test_circle_tool_workflow()` - Complete draw circle flow
   - ⏸️ `test_freehand_tool_workflow()` - Multi-point polygon drawing
   - ⏸️ `test_select_tool_selects_shapes()` - Click to select shapes
   - ⏸️ `test_edit_tool_vertex_dragging()` - Drag vertex to modify shape
   - ⏸️ `test_rotate_tool_rotates_selection()` - Rotate selected shapes

2. **State Machine Transitions** ⏸️ (4 tests planned)
   - ⏸️ `test_idle_to_drawing_transition()` - Mouse down starts drawing
   - ⏸️ `test_drawing_to_idle_transition()` - Mouse up completes shape
   - ⏸️ `test_dragging_vertex_workflow()` - Edit mode vertex manipulation
   - ⏸️ `test_invalid_state_transitions_prevented()` - Can't draw while rotating

**Success Criteria:**
- ✅ 15 tests pass (state, layers, zoom/pan)
- ✅ Tests use helpers (no duplication)
- ⏸️ Tool workflow tests (blocked on egui interaction)
- ⏸️ State machine coverage > 80% (partial - Idle state covered, Drawing/Dragging states need interaction)

#### 1.3 Plugin Coordination Tests ⏸️ DEFERRED

**Status:** ⏸️ Deferred to form_factor crate

**Reason:** PluginManager and EventBus live in `form_factor` crate, not `form_factor_drawing`.
These tests should be implemented in `crates/form_factor/tests/plugin_coordination_test.rs`.

**File:** `crates/form_factor/tests/plugin_coordination_test.rs` (not yet created)

**Tests Planned (12+ total):**

1. **Event Bus Mechanics** ⏸️
   - ⏸️ `test_event_delivered_to_all_plugins()` - Broadcast behavior
   - ⏸️ `test_plugin_can_emit_response_event()` - Event chains
   - ⏸️ `test_event_ordering_preserved()` - FIFO semantics
   - ⏸️ `test_multiple_events_in_single_frame()` - Batch processing

2. **Plugin Lifecycle** ⏸️
   - ⏸️ `test_on_load_called_during_registration()` - Initialization
   - ⏸️ `test_plugin_receives_events_after_load()` - Event subscription
   - ⏸️ `test_on_save_preserves_plugin_state()` - State persistence
   - ⏸️ `test_on_shutdown_cleans_up_resources()` - Cleanup

3. **Cross-Plugin Scenarios** ⏸️
   - ⏸️ `test_canvas_zoom_propagates_to_plugins()` - Zoom sync
   - ⏸️ `test_layer_visibility_affects_plugins()` - Layer state sync
   - ⏸️ `test_tool_selection_updates_all_plugins()` - Tool sync
   - ⏸️ `test_file_open_initializes_all_plugins()` - Coordinated init

4. **Plugin State Synchronization** ⏸️
   - ⏸️ `test_canvas_plugin_tracks_zoom_changes()` - State tracking
   - ⏸️ `test_layers_plugin_tracks_visibility()` - State tracking
   - ⏸️ `test_file_plugin_tracks_recent_projects()` - State tracking

**Success Criteria:**
- ⏸️ All 12+ tests pass (deferred)
- ⏸️ Event bus behavior fully specified (deferred)
- ⏸️ Plugin lifecycle documented via tests (deferred)
- ⏸️ Cross-plugin coordination verified (deferred)

#### 1.4 Test Infrastructure Enhancements ✅ COMPLETE (Modified)

**Status:** ✅ Implemented (without #[cfg(test)])

**DrawingCanvas additions:** ✅
```rust
// Note: Always public (not #[cfg(test)]) because integration tests
// are in separate crate where #[cfg(test)] doesn't apply
impl DrawingCanvas {
    pub fn current_state(&self) -> &CanvasState { &self.state }
    pub fn shapes_on_layer(&self, layer: LayerType) -> Vec<&Shape> { ... }
    pub fn selected_shape_index(&self) -> Option<usize> { ... }
    pub fn current_template_mode(&self) -> &TemplateMode { ... }
    pub fn current_instance_mode(&self) -> &InstanceMode { ... }
    pub fn selected_field_index(&self) -> Option<usize> { ... }
}
```

**EventBus additions:** ⏸️ (deferred to form_factor crate)
```rust
#[cfg(test)]
impl EventBus {
    pub fn pending_events(&self) -> Vec<AppEvent> { ... }
    pub fn event_count(&self) -> usize { ... }
}
```

**Success Criteria:**
- ✅ Test APIs implemented (always-public, not #[cfg(test)])
- ✅ APIs documented as test utilities
- ✅ Helpers use new APIs for cleaner tests
- ⚠️ APIs are in production builds (acceptable - low risk, clear documentation)

### Phase 1 Deliverables (Revised)

- ✅ Test helper infrastructure (helpers/) - **COMPLETE**
- ✅ 15 canvas integration tests (basic workflows) - **COMPLETE**
- ⏸️ 10+ tool workflow tests - **DEFERRED** (egui interaction needed)
- ⏸️ 12+ plugin coordination tests - **DEFERRED** (form_factor crate)
- ✅ Test introspection APIs - **COMPLETE** (always-public)
- ✅ All implemented tests passing - **COMPLETE** (15/15)
- ✅ Documentation in test files - **COMPLETE**

### Phase 1 Success Metrics (Revised)

- ✅ Zero test failures (15/15 passing)
- ⏸️ Test coverage for canvas state machine > 80% (Idle state covered, Drawing/Dragging deferred)
- ⏸️ Test coverage for plugin event handling > 75% (deferred to form_factor crate)
- ✅ All helper functions documented
- ✅ Tests run in < 5 seconds (15 tests in <1 second)

**Achieved:**
- 15 integration tests passing
- Test helper infrastructure working
- Zero clippy warnings
- Canvas state, zoom/pan, layers validated
- Foundation for future test expansion

**Deferred:**
- Tool interaction workflows (need egui simulation)
- Plugin coordination (different crate)
- Complex state transitions (need interaction)

---

## Phase 2: User Workflows (Week 2)

**Goal:** Test template editing and file operations end-to-end

### Files to Create

```
crates/form_factor_drawing/tests/
├── template_canvas_test.rs
└── file_operations_test.rs
```

### Implementation Steps

#### 2.1 Template Canvas Integration Tests

**File:** `crates/form_factor_drawing/tests/template_canvas_test.rs`

Tests covering Priorities 1-5 functionality:

1. **Template Field Creation**
   - `test_create_field_from_rectangle()` - Draw → convert to field
   - `test_create_field_from_circle()` - Circle → field conversion
   - `test_create_field_from_polygon()` - Freehand → field conversion
   - `test_field_type_assignment()` - Set FieldType after creation
   - `test_field_properties_editing()` - Edit label, required, validation

2. **Field Visualization (Priority 3 Part 1)**
   - `test_fields_render_on_canvas()` - Template fields visible
   - `test_field_selection_via_click()` - Click selects field
   - `test_selected_field_highlighted()` - Visual feedback
   - `test_field_properties_panel_updates()` - Panel shows selection
   - `test_multiple_fields_distinct_rendering()` - No visual overlap

3. **Field Dragging/Repositioning (Priority 3 Part 2)**
   - `test_drag_field_updates_bounds()` - Bounds update in template
   - `test_field_snap_to_grid()` - Optional grid snapping
   - `test_drag_field_across_page()` - Can't drag to different page
   - `test_multiple_field_selection_drag()` - Drag group of fields
   - `test_field_drag_respects_constraints()` - Stay within page bounds

4. **Tool Mode Integration (Priority 2)**
   - `test_tool_mode_affects_field_creation()` - Rectangle tool → rect field
   - `test_edit_mode_edits_fields()` - Edit tool modifies field bounds
   - `test_select_mode_selects_fields()` - Select tool picks fields
   - `test_tool_mode_persistence()` - Tool remembered across pages

5. **Multi-Page Template Editing (Priority 4)**
   - `test_create_fields_on_multiple_pages()` - Each page independent
   - `test_page_navigation_preserves_fields()` - Switch pages safely
   - `test_fields_only_visible_on_own_page()` - Page isolation
   - `test_template_validation_across_pages()` - Multi-page validation

6. **Template Persistence**
   - `test_save_template_with_canvas_state()` - Full state save
   - `test_load_template_restores_fields()` - Full state restore
   - `test_template_registry_integration()` - Save to global registry

**Success Criteria:**
- ✅ All 20+ tests pass
- ✅ Priorities 1-5 functionality verified
- ✅ Template-canvas integration fully tested
- ✅ Multi-page workflows validated

#### 2.2 File Operations Integration Tests

**File:** `crates/form_factor_drawing/tests/file_operations_test.rs`

Tests for save/load workflows:

1. **Basic Save/Load**
   - `test_save_canvas_to_file()` - Serialize full state
   - `test_load_canvas_from_file()` - Deserialize full state
   - `test_save_load_roundtrip_preserves_shapes()` - Shapes preserved
   - `test_save_load_roundtrip_preserves_zoom_pan()` - View state preserved
   - `test_save_load_roundtrip_preserves_layers()` - Layer state preserved

2. **Complex State Persistence**
   - `test_save_with_template_and_instance()` - Template + instance
   - `test_save_with_multiple_pages()` - Multi-page projects
   - `test_save_with_detections()` - Detection results preserved
   - `test_save_with_tool_mode()` - Active tool preserved
   - `test_save_with_selection()` - Selected shapes preserved

3. **Recent Projects Tracking**
   - `test_recent_projects_updated_on_save()` - List updated
   - `test_recent_projects_max_limit()` - Max 10 projects
   - `test_recent_projects_move_to_top_on_load()` - MRU order
   - `test_recent_projects_persistence()` - Survives app restart

4. **Error Handling**
   - `test_load_nonexistent_file_returns_error()` - File not found
   - `test_load_corrupted_json_returns_error()` - Invalid JSON
   - `test_load_invalid_schema_returns_error()` - Schema mismatch
   - `test_save_to_readonly_location_returns_error()` - Permission error
   - `test_error_leaves_canvas_unchanged()` - Error safety

5. **Migration Support**
   - `test_load_legacy_format()` - Old format compatibility
   - `test_migration_preserves_data()` - No data loss
   - `test_save_uses_current_format()` - Always save as latest

**Success Criteria:**
- ✅ All 18+ tests pass
- ✅ File I/O error handling verified
- ✅ Recent projects tracking tested
- ✅ Migration path validated

### Phase 2 Deliverables

- ✅ 20+ template-canvas integration tests
- ✅ 18+ file operation tests
- ✅ All Phase 1 + Phase 2 tests passing
- ✅ Recent feature work (Priorities 1-5) has test coverage

### Phase 2 Success Metrics

- Zero test failures
- Template editing workflows covered > 85%
- File I/O error paths tested
- Save/load roundtrip verified for all state
- Tests document template feature usage

---

## Phase 3: Domain Features (Week 3)

**Goal:** Test instance filling workflows and layer system integration

### Files to Create

```
crates/form_factor_drawing/tests/
├── instance_canvas_test.rs
└── layer_integration_test.rs
```

### Implementation Steps

#### 3.1 Instance Canvas Integration Tests

**File:** `crates/form_factor_drawing/tests/instance_canvas_test.rs`

Tests for form filling workflows:

1. **Instance Creation and Rendering**
   - `test_create_instance_from_template()` - Instance initialization
   - `test_instance_fields_render_on_canvas()` - Field overlays
   - `test_empty_fields_highlighted()` - Visual cues for unfilled
   - `test_filled_fields_show_values()` - Value display

2. **Field Value Entry**
   - `test_click_field_to_edit()` - Field selection for editing
   - `test_enter_text_value()` - Text field population
   - `test_enter_numeric_value()` - Numeric field validation
   - `test_enter_date_value()` - Date field formatting
   - `test_checkbox_toggle()` - Boolean field interaction

3. **Validation Integration**
   - `test_required_field_validation()` - Required checking
   - `test_field_type_validation()` - Type checking (email, SSN, etc.)
   - `test_validation_pattern_matching()` - Regex validation
   - `test_validation_visual_feedback()` - Error highlighting
   - `test_instance_validation_state()` - Overall validation state

4. **Multi-Page Instance Filling**
   - `test_fill_fields_across_pages()` - Multi-page workflow
   - `test_page_navigation_with_instance()` - Navigation preserves values
   - `test_incomplete_page_indication()` - Visual progress tracking
   - `test_instance_completion_check()` - All required fields filled

5. **Instance Persistence**
   - `test_save_instance_with_values()` - Save filled form
   - `test_load_instance_restores_values()` - Load filled form
   - `test_instance_validation_results_saved()` - Validation state saved

**Success Criteria:**
- ✅ All 17+ tests pass
- ✅ Instance filling workflow tested
- ✅ Validation integration verified
- ✅ Multi-page instance workflows validated

#### 3.2 Layer Integration Tests

**File:** `crates/form_factor_drawing/tests/layer_integration_test.rs`

Tests for layer system integration:

1. **Layer Rendering and Z-Order**
   - `test_layer_z_order_rendering()` - Canvas → Detections → Shapes → Grid
   - `test_shapes_render_on_correct_layer()` - Layer assignment
   - `test_overlapping_shapes_respect_z_order()` - Overlap handling
   - `test_grid_always_renders_on_top()` - Grid layer priority

2. **Layer Visibility**
   - `test_hide_layer_hides_shapes()` - Visibility affects rendering
   - `test_hidden_layer_shapes_not_selectable()` - Can't interact with hidden
   - `test_show_hidden_layer_restores_shapes()` - Toggle visibility
   - `test_visibility_persists_across_saves()` - State preservation

3. **Active Layer Selection**
   - `test_active_layer_receives_new_shapes()` - Drawing target
   - `test_change_active_layer_changes_target()` - Layer switching
   - `test_active_layer_highlighted_in_ui()` - Visual feedback
   - `test_template_fields_on_separate_layer()` - Template layer isolation

4. **Layer Clearing**
   - `test_clear_shapes_layer()` - Remove all shapes
   - `test_clear_detections_layer()` - Remove all detections
   - `test_clear_preserves_other_layers()` - Selective clearing
   - `test_cannot_clear_grid_layer()` - Grid protection

5. **Layer Lock State**
   - `test_locked_layer_prevents_modifications()` - Lock enforcement
   - `test_locked_layer_allows_visibility_toggle()` - Visibility still works
   - `test_unlock_layer_allows_edits()` - Unlock restores edits
   - `test_lock_state_persists()` - Lock saved with project

**Success Criteria:**
- ✅ All 16+ tests pass
- ✅ Layer system fully integrated
- ✅ Z-order rendering verified
- ✅ Lock and visibility tested

### Phase 3 Deliverables

- ✅ 17+ instance-canvas integration tests
- ✅ 16+ layer integration tests
- ✅ All Phase 1 + 2 + 3 tests passing
- ✅ Form filling workflow documented via tests

### Phase 3 Success Metrics

- Zero test failures
- Instance workflow coverage > 80%
- Layer system coverage > 85%
- All layer interactions tested
- Validation integration verified

---

## Phase 4: Optional Features (Week 4)

**Goal:** Test feature-gated functionality integration

### Files to Create

```
crates/form_factor_drawing/tests/
└── detection_integration_test.rs
```

### Implementation Steps

#### 4.1 Detection Integration Tests

**File:** `crates/form_factor_drawing/tests/detection_integration_test.rs`

Feature-gated tests:

1. **Text Detection Integration**
   ```rust
   #[test]
   #[cfg(feature = "text-detection")]
   fn test_text_detection_creates_detections()
   ```
   - `test_text_detection_creates_detections()` - Detection results
   - `test_text_detections_appear_on_detections_layer()` - Layer assignment
   - `test_detection_count_event_emitted()` - Event emission
   - `test_multiple_text_regions_detected()` - Multi-region support

2. **Logo Detection Integration**
   ```rust
   #[test]
   #[cfg(feature = "logo-detection")]
   fn test_logo_detection_workflow()
   ```
   - `test_logo_detection_creates_detections()` - Detection results
   - `test_logo_template_matching()` - Template matching path
   - `test_logo_feature_matching()` - Feature matching path
   - `test_logo_confidence_scores()` - Score validation

3. **OCR Integration**
   ```rust
   #[test]
   #[cfg(feature = "ocr")]
   fn test_ocr_extraction_workflow()
   ```
   - `test_ocr_extraction_from_image()` - Text extraction
   - `test_ocr_results_with_confidence()` - Confidence scores
   - `test_ocr_bounding_boxes()` - Word-level boxes
   - `test_ocr_event_emission()` - Event bus integration

4. **Combined Detection + OCR**
   ```rust
   #[test]
   #[cfg(all(feature = "text-detection", feature = "ocr"))]
   fn test_detection_then_ocr_workflow()
   ```
   - `test_detection_then_ocr_workflow()` - Two-step process
   - `test_ocr_on_detected_region()` - Target specific region
   - `test_auto_populate_instance_from_ocr()` - Instance filling

5. **Detection + Instance Integration**
   ```rust
   #[test]
   #[cfg(all(feature = "text-detection", feature = "ocr"))]
   fn test_detection_populates_template_fields()
   ```
   - `test_detection_matches_to_fields()` - Spatial matching
   - `test_ocr_populates_field_values()` - Auto-fill
   - `test_confidence_scores_tracked()` - Confidence in instance

**Success Criteria:**
- ✅ All feature-gated tests pass when features enabled
- ✅ Tests skipped when features disabled
- ✅ Detection → OCR → Instance pipeline tested
- ✅ Event bus integration verified

### Phase 4 Deliverables

- ✅ 15+ detection/OCR integration tests
- ✅ Feature flag combinations tested
- ✅ All Phase 1 + 2 + 3 + 4 tests passing
- ✅ Optional features documented

### Phase 4 Success Metrics

- Zero test failures with all features enabled
- Tests properly skipped when features disabled
- OCR → instance integration verified
- Detection event bus flow tested

---

## Testing Strategy

### Test Organization Principles

1. **One file per integration domain**
   - canvas_integration_test.rs - Canvas workflows
   - plugin_coordination_test.rs - Plugin interaction
   - template_canvas_test.rs - Template editing
   - etc.

2. **Helpers in separate module**
   - tests/helpers/ for all test utilities
   - Reusable across all test files
   - Well-documented examples

3. **Feature-gated tests marked clearly**
   ```rust
   #[test]
   #[cfg(feature = "text-detection")]
   fn test_name() { ... }
   ```

4. **Test naming convention**
   - `test_<action>_<expected_result>()`
   - Example: `test_drag_field_updates_bounds()`
   - Clear, descriptive, actionable

### Test Execution

**Run all tests:**
```bash
just test-package form_factor_drawing
```

**Run specific test file:**
```bash
cargo test --package form_factor_drawing --test canvas_integration_test
```

**Run with all features:**
```bash
cargo test --package form_factor_drawing --all-features
```

**Run without optional features:**
```bash
cargo test --package form_factor_drawing --no-default-features
```

### Test Speed Targets

- Phase 1 tests: < 2 seconds total
- Phase 2 tests: < 3 seconds total
- Phase 3 tests: < 3 seconds total
- Phase 4 tests (with features): < 5 seconds total
- **Total suite: < 15 seconds**

Integration tests should be fast. If tests are slow:
- Mock expensive operations (image loading, CV processing)
- Use small test images
- Cache setup where possible

---

## Success Criteria

### Overall Project Success

- ✅ All 90+ integration tests passing
- ✅ Test suite runs in < 15 seconds
- ✅ Zero regressions in existing functionality
- ✅ All Priorities 1-5 features have test coverage
- ✅ Plugin coordination fully specified via tests
- ✅ Canvas state machine coverage > 80%
- ✅ Template/instance workflows documented via tests

### Quality Gates

**Before merging each phase:**
1. All new tests passing
2. All existing tests still passing
3. `just check-all` passes (clippy, fmt, test)
4. Documentation updated if public APIs added
5. Test helpers documented with examples

### Regression Prevention

Integration tests should catch:
- Event bus ordering changes breaking plugin coordination
- Canvas state machine changes breaking workflows
- Layer system changes affecting rendering
- Tool mode changes breaking drawing
- Template field changes breaking instance filling
- File format changes breaking save/load

---

## Dependencies and Risks

### Technical Dependencies

- **egui interaction simulation** - May need mock input events
- **EventBus introspection** - Requires test-only APIs
- **Canvas state access** - Requires test-only accessors
- **Plugin state verification** - May need response events

### Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tests too slow (> 15s) | CI/CD friction | Mock expensive ops, use small fixtures |
| egui UI testing complexity | Can't test rendering | Focus on state, not pixels |
| Feature flag combinations | Exponential test matrix | Test critical combinations only |
| Event bus timing issues | Flaky tests | Use synchronous processing, no delays |
| Mock complexity | Unmaintainable tests | Keep helpers simple, well-documented |

### Assumptions

- egui UI can be tested via state inspection (not pixel-perfect rendering)
- Event bus is synchronous within test context
- File I/O can use temp directories
- Test images < 100KB for speed
- Helpers can be reused across 90+ tests

---

## Implementation Guidelines

### Code Quality Standards

1. **No test duplication** - Use helpers
2. **Descriptive test names** - `test_<action>_<result>()`
3. **Arrange-Act-Assert** pattern:
   ```rust
   // Arrange
   let mut canvas = create_test_canvas();

   // Act
   simulate_click(&mut canvas, pos);

   // Assert
   assert_eq!(canvas.shapes().len(), 1);
   ```
4. **Document complex tests** - Why this test exists, what it prevents
5. **One assertion focus per test** - Test one thing well

### Helper Design Principles

1. **Composable** - Small functions that combine
2. **Documented** - Doc comments with examples
3. **Type-safe** - Use strong types, not stringly-typed
4. **Fail-fast** - Use `expect()` with clear messages
5. **Reusable** - Generic where appropriate

### Test Data Strategy

**Small test fixtures:**
- 10x10 pixel test image for OCR/detection
- 1-3 shapes per test
- 1-2 pages per test
- Minimal field counts

**Shared fixtures** in `tests/fixtures/`:
- `test_form.png` - Small form image
- `test_template.json` - Basic template
- `test_instance.json` - Filled instance

### Error Message Standards

```rust
// ❌ BAD
assert!(result.is_ok());

// ✅ GOOD
assert!(result.is_ok(), "Failed to create canvas: {:?}", result.err());

// ✅ BETTER
let canvas = result.expect("Failed to create canvas with default settings");
```

---

## Maintenance Plan

### After Implementation

1. **Update this document** - Mark completed phases
2. **Add to CI/CD** - Ensure tests run on every commit
3. **Monitor test speed** - Alert if > 15 seconds
4. **Review coverage** - Quarterly coverage reports
5. **Update on refactors** - Keep tests in sync with code

### When Adding New Features

**New feature checklist:**
1. Does it involve canvas interaction? → Add to canvas_integration_test.rs
2. Does it involve plugins? → Add to plugin_coordination_test.rs
3. Does it involve templates? → Add to template_canvas_test.rs
4. Does it involve file I/O? → Add to file_operations_test.rs
5. Is it feature-gated? → Add to detection_integration_test.rs

### Test Debt Prevention

- No feature ships without integration test
- No bug fix without regression test
- Quarterly test review and pruning
- Keep test suite fast (< 15s)

---

## Appendix A: Test File Templates

### Canvas Integration Test Template

```rust
//! Integration tests for DrawingCanvas workflows
//!
//! Tests cover:
//! - Tool mode workflows (Rectangle, Circle, Freehand, Select, Edit, Rotate)
//! - Canvas state machine transitions
//! - Zoom and pan integration
//! - Layer integration with drawing

use form_factor_drawing::{DrawingCanvas, ToolMode, Shape};
use helpers::{create_test_canvas, simulate_click, simulate_drag};

mod helpers;

#[test]
fn test_rectangle_tool_workflow() {
    // Arrange
    let mut canvas = create_test_canvas();
    canvas.set_tool(ToolMode::Rectangle);

    // Act
    simulate_drag(&mut canvas, (10.0, 10.0), (50.0, 50.0));

    // Assert
    assert_eq!(canvas.shapes().len(), 1);
    let shape = &canvas.shapes()[0];
    assert!(matches!(shape, Shape::Rectangle { .. }));
}
```

### Plugin Coordination Test Template

```rust
//! Integration tests for plugin coordination via event bus
//!
//! Tests cover:
//! - Event delivery to all plugins
//! - Plugin lifecycle (load, event, save, shutdown)
//! - Cross-plugin state synchronization
//! - Event chains and responses

use form_factor_plugins::{PluginManager, AppEvent};
use helpers::{create_test_plugin_manager, emit_and_process, collect_events};

mod helpers;

#[test]
fn test_event_delivered_to_all_plugins() {
    // Arrange
    let mut manager = create_test_plugin_manager();

    // Act
    let sender = manager.event_bus().sender();
    sender.emit(AppEvent::SelectionCleared);
    manager.process_events();

    // Assert
    // (Requires plugins to emit response events or expose state)
}
```

---

## Appendix B: Helper Function Signatures

### Canvas Helpers

```rust
/// Creates a canvas with default settings (1024x768, 100% zoom)
pub fn create_test_canvas() -> DrawingCanvas;

/// Creates a canvas with pre-populated shapes
pub fn create_canvas_with_shapes(count: usize) -> DrawingCanvas;

/// Simulates a mouse click at the given position
pub fn simulate_click(canvas: &mut DrawingCanvas, pos: (f32, f32));

/// Simulates a drag gesture from start to end
pub fn simulate_drag(canvas: &mut DrawingCanvas, from: (f32, f32), to: (f32, f32));

/// Simulates a complete tool action (draw rectangle, circle, etc.)
pub fn simulate_tool_action(canvas: &mut DrawingCanvas, tool: ToolMode, action: ToolAction);

/// Asserts canvas is in expected state
pub fn assert_canvas_state(canvas: &DrawingCanvas, expected: CanvasState);

/// Returns all shapes on the given layer
pub fn get_shapes_on_layer(canvas: &DrawingCanvas, layer: LayerType) -> Vec<&Shape>;
```

### Plugin Helpers

```rust
/// Creates plugin manager with all standard plugins registered
pub fn create_test_plugin_manager() -> PluginManager;

/// Creates plugin manager with specific plugins
pub fn create_plugin_manager_with(plugins: Vec<Box<dyn Plugin>>) -> PluginManager;

/// Drains event bus and returns all events
pub fn collect_events(event_bus: &EventBus) -> Vec<AppEvent>;

/// Emits event and processes it through all plugins
pub fn emit_and_process(manager: &mut PluginManager, event: AppEvent);

/// Asserts that the expected event was emitted
pub fn assert_event_emitted(events: &[AppEvent], expected: &AppEvent);
```

---

## Appendix C: Feature Flag Test Matrix

| Feature Combination | Tests Affected | Priority |
|---------------------|----------------|----------|
| (none) | All core tests | High |
| `text-detection` | detection_integration_test.rs | Medium |
| `logo-detection` | detection_integration_test.rs | Medium |
| `ocr` | detection_integration_test.rs | Medium |
| `text-detection` + `ocr` | detection_integration_test.rs | High |
| `text-detection` + `logo-detection` | detection_integration_test.rs | Low |
| `all-features` | All tests | High |

**Test strategy:**
- CI runs with `--no-default-features` (core only)
- CI runs with `--all-features` (everything)
- Individual feature tests run on demand

---

## Next Steps

1. **Review this plan** - Ensure all stakeholders agree
2. **Add to PLANNING_INDEX.md** - Track in project planning
3. **Begin Phase 1** - Start with test helper infrastructure
4. **Iterate weekly** - Complete one phase per week
5. **Update plan** - Mark completed items, adjust as needed

**Questions to resolve before starting:**
1. Are test speed targets reasonable (< 15s total)?
2. Do we need pixel-perfect rendering tests or is state testing sufficient?
3. Should detection tests use real images or mocked detections?
4. What's the process for updating this plan during implementation?
