# Template UI Implementation Plan

**Status**: Not Started
**Created**: 2024-12-05
**Estimated Effort**: Large (1-2 weeks)
**Dependencies**: Template system core (completed in TEMPLATE_SYSTEM_PLAN)

## Overview

This document outlines the implementation plan for visual template creation and editing in the Form Factor application. The core template system (TemplateBuilder, TemplateRegistry, field definitions) is complete. This plan focuses on the UI layer that allows users to create and edit templates visually rather than programmatically.

## Goals

1. **Visual Template Creation**: Create templates through UI instead of code
2. **Drag-and-Drop Field Placement**: Interactively position field bounds on form images
3. **Field Property Editor**: Edit field metadata (name, type, validation, optional flags)
4. **Template Preview**: Real-time preview of field overlays and validation
5. **Template Management**: List, load, save, and delete templates from registry

## Architecture

### Component Structure

```
Template UI
├── Template Manager Panel
│   ├── Template List View
│   ├── New/Edit/Delete Actions
│   └── Import/Export Functionality
│
├── Template Editor View
│   ├── Canvas with Form Image
│   ├── Field Overlay Rendering
│   ├── Field Selection/Dragging
│   └── Field Creation Tools
│
└── Field Properties Panel
    ├── Basic Properties (name, label, type)
    ├── Validation Settings (required, regex)
    ├── Bounds Adjustment (x, y, width, height)
    └── Page Selection (multi-page support)
```

### State Management

```rust
pub struct TemplateEditorState {
    /// Current template being edited
    current_template: Option<TemplateBuilder>,

    /// Selected field index for editing
    selected_field: Option<usize>,

    /// Current page being edited
    current_page: usize,

    /// Editor mode (select, draw, edit)
    mode: EditorMode,

    /// Temporary field being drawn
    drawing_field: Option<FieldDefinitionBuilder>,

    /// Drag state for field movement/resize
    drag_state: Option<DragState>,

    /// Undo/redo stacks
    undo_stack: Vec<TemplateSnapshot>,
    redo_stack: Vec<TemplateSnapshot>,
}

pub enum EditorMode {
    Select,     // Select and move existing fields
    Draw,       // Draw new field bounds
    Edit,       // Edit field properties
}

pub struct DragState {
    field_index: usize,
    drag_type: DragType,
    start_pos: Pos2,
    original_bounds: Rect,
}

pub enum DragType {
    Move,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
}
```

## Implementation Priorities

### ✅ Priority 1: Template Manager Panel (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Small (~1 day)
**Commit**: `66fc6f7`

#### Features

1. **Template List View**
   - Display all templates from registry
   - Show template ID, name, page count
   - Sort by name, creation date, last modified
   - Search/filter templates

2. **Template Actions**
   - New: Create empty template
   - Edit: Load template into editor
   - Delete: Remove from registry (with confirmation)
   - Duplicate: Clone template with new ID

3. **Import/Export**
   - Export template to JSON file
   - Import template from JSON file
   - Validate on import

#### UI Layout

```
┌─────────────────────────────────────┐
│ Template Manager                    │
├─────────────────────────────────────┤
│ [New] [Import]          [Search: ] │
├─────────────────────────────────────┤
│ Templates:                          │
│ ┌─────────────────────────────────┐ │
│ │ ○ W-2 Form (2 pages)            │ │
│ │   [Edit] [Duplicate] [Delete]   │ │
│ │                                 │ │
│ │ ○ 1099-MISC (1 page)            │ │
│ │   [Edit] [Duplicate] [Delete]   │ │
│ │                                 │ │
│ │ ○ Invoice Template (1 page)     │ │
│ │   [Edit] [Duplicate] [Delete]   │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

#### Implementation Tasks

- [x] Add `TemplateManagerState` struct
- [x] Implement template list rendering
- [x] Add search/filter logic
- [x] Implement CRUD operations calling TemplateRegistry
- [ ] Add import/export file dialogs (deferred to separate PR)
- [x] Add confirmation dialogs for destructive actions

#### Files Created

**`crates/form_factor_drawing/src/template_ui/mod.rs`**:
- Module organization and public exports
- Re-exports TemplateManagerPanel, state types

**`crates/form_factor_drawing/src/template_ui/state.rs`** (~290 lines):
- `TemplateManagerState` - Search, selection, delete confirmation state
- `TemplateEditorState` - Editor state with undo/redo stacks (50 limit)
- `TemplateSnapshot` - Snapshot for undo/redo
- `EditorMode` enum - Select/Draw/Edit modes
- `DragState` and `DragType` - Field manipulation state (future use)

**`crates/form_factor_drawing/src/template_ui/manager.rs`** (~195 lines):
- `TemplateManagerPanel` widget for egui
- Template list view with ScrollArea
- Search/filter functionality
- CRUD action buttons per template
- Delete confirmation dialog
- `ManagerAction` enum for event handling

#### Files Modified

**`crates/form_factor_drawing/src/lib.rs`**:
- Added `pub mod template_ui`
- Re-exported UI types at crate level

**`crates/form_factor_drawing/src/template/implementation.rs`**:
- Added `Clone` derive to `DrawingTemplateBuilder`
- Required for undo/redo snapshots

#### Features Implemented

1. **Template List View**: ✅
   - Displays all templates from TemplateRegistry
   - Shows template ID, name, page count, field count
   - Scrollable list with egui ScrollArea
   - Template selection with radio buttons

2. **Search and Filter**: ✅
   - Real-time search by template ID or name
   - Case-insensitive matching
   - Filters list as you type

3. **Template Actions**: ✅
   - New button (returns ManagerAction::New)
   - Edit button per template (returns ManagerAction::Edit)
   - Duplicate button per template (returns ManagerAction::Duplicate)
   - Export button per template (returns ManagerAction::Export)
   - Delete button with confirmation dialog

4. **Delete Confirmation**: ✅
   - Modal dialog for delete operations
   - Shows template ID being deleted
   - "This action cannot be undone" warning
   - Confirm/Cancel buttons

5. **State Management**: ✅
   - TemplateManagerState tracks search query, selection, pending deletes
   - TemplateEditorState with undo/redo stacks ready for editor
   - EditorMode enum for Select/Draw/Edit

#### Integration Notes

- Panel integrates with existing `TemplateRegistry`
- Uses `FormTemplate` trait methods (id, name, page_count, fields)
- Returns `ManagerAction` enum for parent UI to handle
- Parent must call `registry.delete_from_global()` for Delete action
- Parent must implement Import/Export file dialogs
- Parent must create new template or load existing for Edit action

#### Known Limitations

1. **No Import/Export Dialogs**: Action enum values returned, but file dialogs not implemented
2. **No Duplicate Implementation**: Returns action, but cloning logic not in manager
3. **No New Template Flow**: Returns action, parent must create blank template
4. **No Sorting**: Templates displayed in HashMap iteration order
5. **Field Count Uses All Pages**: Shows total across all pages, not per-page

#### Next Steps

- Priority 2: Implement basic template editor view
- Add file dialog support for import/export
- Implement duplicate template logic in parent
- Add template creation wizard for New action

### ✅ Priority 2: Basic Template Editor (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Medium (~1 day)
**Commit**: `d216baf`

#### Features

1. **Template Canvas**
   - Display form image at specified page
   - Render field overlays (reuse existing `render_field_overlays`)
   - Handle zoom/pan (reuse existing canvas transform)

2. **Field Selection**
   - Click to select field
   - Show selection highlight
   - Display field info in properties panel

3. **Page Navigation**
   - Previous/Next page buttons
   - Page selector dropdown
   - Keyboard shortcuts (PgUp/PgDn)

4. **Mode Switching**
   - Select mode (default)
   - Draw mode (create new fields)
   - Edit mode (modify field properties)

#### UI Layout

```
┌───────────────────────────────────────────────────────┐
│ Template Editor: W-2 Form                             │
├───────────────────────────────────────────────────────┤
│ [Select] [Draw] [Edit]    Page: [1] [<] [>] of 2     │
├─────────────────────────┬─────────────────────────────┤
│                         │ Field Properties            │
│                         ├─────────────────────────────┤
│                         │ Name: employee_name         │
│                         │ Label: Employee Name        │
│    Form Image           │ Type: Text                  │
│    with Field           │ Required: ☑                 │
│    Overlays             │ Pattern: [a-zA-Z ]+         │
│                         │                             │
│                         │ Bounds:                     │
│                         │   X: 100   Y: 50            │
│                         │   W: 200   H: 30            │
│                         │                             │
│                         │ [Apply] [Cancel]            │
└─────────────────────────┴─────────────────────────────┘
│ [Save Template] [Cancel] [Undo] [Redo]                │
└───────────────────────────────────────────────────────┘
```

#### Implementation Tasks

- [x] Add `TemplateEditorState` struct (completed in Priority 1)
- [x] Implement template loading from registry (basic version, needs to_builder())
- [x] Render field overlays (simple transform, not full canvas pipeline)
- [x] Implement field selection (click detection)
- [x] Add selection highlight rendering
- [x] Implement page navigation
- [x] Add mode switching UI
- [ ] Integrate with existing canvas zoom/pan (deferred - simple transform for now)

#### Files Created

**`crates/form_factor_drawing/src/template_ui/editor.rs`** (~280 lines):
- `TemplateEditorPanel` widget for egui
- Template canvas with painter allocation
- Field overlay rendering with simple coordinate transform
- Click detection and hit testing for field selection
- Page navigation controls
- Mode switching toolbar
- Undo/Redo button integration
- `EditorAction` enum (None, Save, Cancel)

#### Files Modified

**`crates/form_factor_drawing/src/lib.rs`**:
- Added re-export of `EditorAction`, `TemplateEditorPanel`

**`crates/form_factor_drawing/src/template/implementation.rs`**:
- Added `page_count()` method to `DrawingTemplateBuilder`
- Added `fields()` method to get all fields across pages
- Added `fields_for_page(index)` method to get fields for specific page
- Enables builder to be queried without building

**`crates/form_factor_drawing/src/template_ui/mod.rs`**:
- Added editor module
- Re-exported `EditorAction`, `TemplateEditorPanel`

#### Features Implemented

1. **Template Canvas**: ✅
   - Allocates painter area with click_and_drag sense
   - Gray background (Color32::from_gray(240))
   - Renders all fields for current page
   - Simple coordinate transform (canvas-relative)

2. **Field Overlay Rendering**: ✅
   - Semi-transparent rectangles with rounded corners
   - Blue overlay for selected fields (0, 150, 255, 100)
   - Green overlay for unselected fields (0, 200, 0, 80)
   - Border strokes (2px selected, 1px unselected)
   - Field ID labels at top-left of each field

3. **Field Selection**: ✅
   - Click detection on canvas
   - `find_field_at_position()` hit testing
   - Reverse iteration (top fields selected first)
   - Updates `TemplateEditorState.selected_field`
   - Tracing logs for selection changes
   - Deselect by clicking empty area

4. **Selection Highlighting**: ✅
   - Blue color for selected fields
   - Green color for unselected fields
   - Thicker border (2px) for selected
   - Visual feedback on click

5. **Page Navigation**: ✅
   - Previous/Next buttons in toolbar
   - Current page / total pages display (e.g., "1 / 2")
   - Boundary checking prevents invalid navigation
   - Updates `TemplateEditorState.current_page`
   - Tracing logs for page changes

6. **Mode Switching**: ✅
   - Selectable labels for Select/Draw/Edit modes
   - Visual highlighting of current mode
   - Updates `TemplateEditorState.mode`
   - Tracing logs for mode changes

7. **Undo/Redo Integration**: ✅
   - Buttons enabled/disabled based on stack state
   - Calls `TemplateEditorState.undo()/redo()`
   - Uses snapshot system from Priority 1

8. **Save/Cancel Actions**: ✅
   - Save Template button returns `EditorAction::Save`
   - Cancel button returns `EditorAction::Cancel`
   - Parent UI handles actual save/cancel logic

#### API Integration

**Creating New Template**:
```rust
let mut editor = TemplateEditorPanel::new();
editor.new_template("my_template", "My Template");
```

**Loading Template** (incomplete):
```rust
let loaded = editor.load_template("template_id", &registry);
// TODO: Needs DrawingTemplate.to_builder() method
```

**Rendering Editor**:
```rust
let action = editor.show(ui, &registry);
match action {
    EditorAction::Save => { /* save template */ },
    EditorAction::Cancel => { /* close editor */ },
    EditorAction::None => { /* continue editing */ },
}
```

#### Known Limitations

1. **No Form Image Display**: Canvas shows gray background only, no actual form image
2. **Simple Coordinate Transform**: Direct pixel mapping, not integrated with canvas transform pipeline
3. **No Zoom/Pan**: Canvas is fixed size, no zoom or pan controls
4. **load_template() Incomplete**: Needs `DrawingTemplate.to_builder()` method to edit existing templates
5. **No Field Properties Display**: Selected field info not shown yet (waits for Priority 4)
6. **No Field Editing**: Can only view and select, not modify fields (waits for Priority 3)
7. **No Keyboard Shortcuts**: Page navigation is mouse-only (PgUp/PgDn not implemented)
8. **Field Count Only**: Shows "Fields on this page: N" but no other template metadata

#### Integration Notes

- Panel requires `TemplateRegistry` reference in `show()` method
- Returns `EditorAction` enum for parent to handle
- Parent must handle Save action (validate and persist to registry)
- Parent must handle Cancel action (close editor, discard changes)
- Editor maintains internal state in `TemplateEditorState`
- State includes undo/redo stacks with 50 action limit

#### Next Steps

- Priority 3: Implement field drawing and manipulation
- Priority 4: Add field properties panel
- Implement `DrawingTemplate.to_builder()` for loading existing templates
- Add form image display to canvas
- Integrate with proper canvas transform pipeline for zoom/pan
- Add keyboard shortcuts for page navigation

### ✅ Priority 3: Field Drawing and Manipulation (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Medium (~1 day)
**Commit**: `ecf096e`

#### Features

1. **Field Drawing**
   - Click-and-drag to define field bounds
   - Show preview rectangle while dragging
   - Auto-generate default field name
   - Open properties panel on creation

2. **Field Movement**
   - Drag field to new position
   - Snap to other field edges (optional)
   - Show bounds while dragging

3. **Field Resizing**
   - Drag corners to resize
   - Show resize handles on selection
   - Maintain minimum size (10x10 pixels)
   - Show dimensions while resizing

4. **Field Deletion**
   - Delete key removes selected field
   - Confirmation for deletion
   - Undo/redo support

#### Interaction Patterns

**Draw Mode**:
1. User clicks "Draw" mode button
2. Cursor changes to crosshair
3. User clicks and drags on canvas
4. Preview rectangle shows during drag
5. On release, field is created and properties panel opens

**Move Mode**:
1. User selects field
2. Cursor changes to move icon when over field
3. User drags field
4. Field position updates in real-time
5. On release, field bounds are updated

**Resize Mode**:
1. User selects field
2. Resize handles appear at corners
3. User drags handle
4. Field bounds update in real-time
5. On release, field bounds are saved

#### Implementation Tasks

- [x] Implement field drawing state machine
- [x] Add preview rectangle rendering
- [x] Implement drag detection and tracking
- [x] Add resize handle rendering
- [x] Implement corner resize logic
- [x] Add field deletion (without confirmation - uses undo instead)
- [ ] Implement snap-to-edge functionality (deferred)
- [x] Add minimum size constraints (20x20 pixels)

#### Files Created

**`crates/form_factor_drawing/src/template_ui/manipulation.rs`** (~380 lines):
- `handle_draw_mode()` - Draws new fields with preview rectangle
- `handle_select_mode()` - Handles selection, movement, and resizing
- `render_resize_handles()` - Renders blue corner handles on selected field
- `render_drawing_preview()` - Renders orange preview during drawing
- `create_field_from_drawing()` - Creates FieldDefinition from drawn rectangle
- `delete_field()` - Removes field with Delete key
- `get_resize_handle_at_position()` - Hit testing for corner handles
- `update_field_bounds()` - Updates bounds during drag operations
- Constants: HANDLE_SIZE (8.0px), MIN_FIELD_SIZE (20.0px)

#### Files Modified

**`crates/form_factor_drawing/src/template_ui/editor.rs`**:
- Added `DrawingState` struct for field creation preview
- Added `DragOperation` struct for move/resize tracking
- Added `DragOperationType` enum (Move, ResizeTopLeft, etc.)
- Updated `TemplateEditorPanel` with drawing_state and drag_state fields
- Modified `show()` to dispatch to mode-specific handlers
- Integrated keyboard input for Delete key
- Added resize handle rendering
- Added drawing preview rendering
- Made structs/fields `pub(super)` for manipulation module access
- Made `find_field_at_position()` pub(super)

**`crates/form_factor_drawing/src/template_ui/mod.rs`**:
- Added manipulation module
- Imports field manipulation functionality

**`crates/form_factor_drawing/src/template/implementation.rs`**:
- Made `pages` field `pub(crate)` on DrawingTemplateBuilder
- Enables direct page access for field manipulation

#### Features Implemented

1. **Field Drawing (Draw Mode)**: ✅
   - Click-and-drag creates preview rectangle
   - Orange semi-transparent preview (255, 200, 0, 60)
   - Shows dimensions during drawing (e.g., "100x50")
   - Minimum size constraint (20x20 pixels)
   - Auto-generates field ID (field_1, field_2, etc.)
   - Auto-generates field label (Field 1, Field 2, etc.)
   - Creates FieldDefinition with FreeText type
   - Pushes undo snapshot after creation

2. **Field Movement (Select Mode)**: ✅
   - Click-and-drag on field body moves entire field
   - Real-time position updates during drag
   - Smooth movement with delta tracking
   - Preserves field size during movement
   - Pushes undo snapshot after move completes
   - Tracing logs for debugging

3. **Field Resizing (Select Mode)**: ✅
   - Four corner resize handles (8px radius circles)
   - Blue handles match selected field color
   - Independent corner resize operations:
     - Top-left: Resize from top-left corner
     - Top-right: Resize from top-right corner
     - Bottom-left: Resize from bottom-left corner
     - Bottom-right: Resize from bottom-right corner
   - Minimum size enforcement (20x20 pixels)
   - Real-time size updates during resize
   - Pushes undo snapshot after resize completes

4. **Field Deletion**: ✅
   - Delete key removes selected field
   - Clears selection after deletion
   - Pushes undo snapshot for undo capability
   - Tracing logs for debugging
   - No confirmation dialog (relies on undo)

5. **Interaction State Management**: ✅
   - DrawingState tracks preview rectangle (start_pos, current_pos)
   - DragOperation tracks active drag (field_index, operation_type, start_pos, original_bounds)
   - DragOperationType enum for operation types
   - Proper state cleanup on drag completion
   - Clone-based state handling to avoid borrow conflicts

6. **Visual Feedback**: ✅
   - Orange preview rectangle during field drawing
   - Blue resize handles (8px) on selected field
   - Selection highlighting (blue overlay from Priority 2)
   - Dimension display during drawing
   - Smooth real-time updates

#### API Usage

**Draw Mode - Creating Fields**:
```rust
// Switch to Draw mode
editor.state_mut().set_mode(EditorMode::Draw);

// User clicks and drags on canvas
// - DrawingState captures start and current positions
// - Orange preview shows during drag
// - On release, field is created automatically
```

**Select Mode - Moving Fields**:
```rust
// Switch to Select mode
editor.state_mut().set_mode(EditorMode::Select);

// User clicks on field -> selects it
// User drags field -> DragOperation::Move
// Field position updates in real-time
// On release, undo snapshot is pushed
```

**Select Mode - Resizing Fields**:
```rust
// Field must be selected first
// User clicks on corner handle -> DragOperation::Resize*
// Field bounds update in real-time
// Minimum size enforced (20x20)
// On release, undo snapshot is pushed
```

**Delete Field**:
```rust
// Select field first
// Press Delete key -> field removed
// Selection cleared
// Undo snapshot pushed
```

#### Known Limitations

1. **No Snap-to-Grid**: Fields can be placed at any pixel position
2. **No Snap-to-Field**: No automatic alignment with other field edges
3. **No Multi-Select**: Can only manipulate one field at a time
4. **No Bulk Operations**: No copy/paste or duplicate fields
5. **No Field Validation**: Can create fields with invalid IDs or properties
6. **Default Field Type Only**: All created fields are FreeText type
7. **No Confirmation Dialog**: Delete immediately removes (relies on undo)
8. **Resize Handles Always Visible**: No hover state, always shown on selected
9. **No Cursor Changes**: Cursor doesn't indicate move/resize operations
10. **No Dimension Display During Resize**: Only shows during initial draw

#### Integration Notes

- Field drawing requires Draw mode (click mode button in toolbar)
- Field movement/resizing requires Select mode
- Delete key must be pressed while field is selected
- All operations push undo snapshots automatically
- Minimum field size is 20x20 pixels (enforced during draw and resize)
- Auto-generated field IDs are sequential (field_1, field_2, etc.)
- All created fields default to FreeText type
- Parent UI can query editor state to check current mode

#### Next Steps

- Priority 4: Implement field properties panel for editing metadata
- Priority 5: Undo/Redo system (foundation already complete)
- Priority 6: Template validation and save
- Add snap-to-grid feature
- Add snap-to-field edges feature
- Add field type selector during drawing
- Add cursor changes for move/resize
- Add dimension display during resize
- Add multi-select support

### ✅ Priority 4: Field Properties Panel (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Small (~1 day)
**Commit**: `d774fd9`

#### Features

1. **Basic Properties**
   - Name (unique identifier)
   - Label (display text)
   - Field type (text, number, date, checkbox)

2. **Validation Settings**
   - Required checkbox
   - Regex pattern input
   - Pattern validation preview
   - Common pattern presets

3. **Bounds Adjustment**
   - Numeric inputs for x, y, width, height
   - Apply button to update bounds
   - Visual feedback on canvas

4. **Page Assignment**
   - Page selector for multi-page forms
   - Move field to different page

#### UI Components

```rust
pub struct FieldPropertiesPanel {
    /// Currently edited field index
    field_index: Option<usize>,

    /// Temporary field state (unsaved)
    field_draft: Option<FieldDefinitionBuilder>,

    /// Validation error messages
    validation_errors: Vec<String>,
}

impl FieldPropertiesPanel {
    pub fn show(&mut self, ui: &mut Ui, template: &mut TemplateBuilder) {
        if let Some(index) = self.field_index {
            if let Some(field) = template.get_field(index) {
                // Render property inputs
                self.show_basic_properties(ui, field);
                self.show_validation_settings(ui, field);
                self.show_bounds_adjustment(ui, field);
                self.show_actions(ui);
            }
        }
    }
}
```

#### Implementation Tasks

- [x] Add `FieldPropertiesPanel` struct
- [x] Implement basic property inputs (text, combo boxes)
- [x] Add validation settings UI
- [x] Implement bounds numeric inputs
- [x] Add apply/cancel/delete buttons
- [x] Implement field validation and error display
- [x] Add regex pattern presets (Email, Phone, ZIP)
- [ ] Implement page assignment selector (deferred - low priority)

#### Files Created

**`crates/form_factor_drawing/src/template_ui/properties.rs`** (~275 lines):
- `FieldPropertiesPanel` struct with temp state pattern
- `show()` method - Main UI rendering with validation
- `reset()` method - Clears temp state on selection change
- `PropertiesAction` enum - Apply/Cancelled/Delete(usize)
- Validation: ID/label required, regex pattern syntax checking
- Field type dropdown with 20+ types organized by category
- Pattern presets: Email, Phone (XXX-XXX-XXXX), ZIP code

#### Files Modified

**`crates/form_factor_drawing/src/template_ui/mod.rs`**:
- Added properties module
- Exported FieldPropertiesPanel and PropertiesAction

**`crates/form_factor_drawing/src/template_ui/editor.rs`**:
- Added properties_panel field to TemplateEditorPanel
- Restructured show() to use horizontal layout (70/30 canvas/properties split)
- Properties panel in right sidebar with scrolling
- Delete key now resets properties panel state
- Integrated PropertiesAction handling (Apply/Cancel/Delete)

**`crates/form_factor_drawing/src/lib.rs`**:
- Exported FieldPropertiesPanel and PropertiesAction to crate root

**`crates/form_factor_drawing/src/template_ui/state.rs`**:
- Simplified TemplateSnapshot (removed timestamp/action_description)
- Updated push_snapshot to accept but ignore description parameter
- Added documentation noting fields will be added in Priority 5
- Removed unused DragState and DragType structs

#### Features Implemented

1. **Basic Properties Editing**: ✅
   - ID field with required validation
   - Label field with required validation
   - Field type ComboBox with 20+ types:
     - Common: FreeText, Date, DateOfBirth, Checkbox, Signature, Initials
     - Personal Info: FirstName, LastName, FullName, Email, PhoneNumber
     - Address: StreetAddress, City, State, ZipCode
     - Financial: Currency, Amount
   - Organized by category with separators

2. **Validation Settings**: ✅
   - Required checkbox
   - Regex pattern input with syntax validation
   - Pattern presets: Email, Phone (XXX-XXX-XXXX), ZIP (5 or 5+4)
   - Error messages displayed in red above form
   - Apply button blocked until validation passes

3. **Bounds Adjustment**: ✅
   - DragValue inputs for X, Y position
   - DragValue inputs for Width, Height with 20px minimum
   - Real-time canvas updates (via temp state pattern)

4. **Help Text**: ✅
   - Multiline TextEdit for optional help text
   - Empty becomes None in FieldDefinition

5. **Action Buttons**: ✅
   - Apply: Validates, saves changes, pushes undo snapshot
   - Cancel: Discards changes, resets temp state
   - Delete Field: Removes field, clears selection, pushes undo

6. **State Management**: ✅
   - Temp state pattern prevents unintended changes
   - State initialized from selected field on first show
   - Reset on selection change or deletion
   - Validation errors tracked and displayed

7. **UI Integration**: ✅
   - Right sidebar (30% width, scrollable)
   - Canvas (70% width)
   - "No field selected" message when nothing selected
   - Automatic state sync with selection changes

#### API Usage

**Basic Integration**:
```rust
use form_factor_drawing::{
    TemplateEditorPanel, FieldPropertiesPanel, PropertiesAction
};

let mut editor = TemplateEditorPanel::new();
let action = editor.show(ui, &registry);

// Properties panel is automatically shown in editor.show()
// No separate integration needed
```

**Properties Action Handling** (happens internally in editor):
```rust
match properties_action {
    PropertiesAction::Applied => {
        // Changes saved, undo snapshot pushed
    }
    PropertiesAction::Cancelled => {
        // Changes discarded
    }
    PropertiesAction::Delete(field_idx) => {
        // Field deleted, selection cleared
    }
    PropertiesAction::None => {
        // No action taken
    }
}
```

**Field Type Selection**:
```rust
// User can select from 20+ field types:
FieldType::FreeText       // Default for new fields
FieldType::FirstName      // Personal info
FieldType::Email          // With email pattern preset
FieldType::PhoneNumber    // With phone pattern preset
FieldType::StreetAddress  // Address fields
FieldType::Currency       // Financial fields
// ... and more
```

**Validation Patterns**:
```rust
// Preset patterns available via buttons:
Email:  r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
Phone:  r"^\d{3}-\d{3}-\d{4}$"
ZIP:    r"^\d{5}(-\d{4})?$"

// Or custom regex pattern
```

#### Known Limitations

1. **No Page Assignment**: Fields cannot be moved to different pages (low priority)
2. **No Field Duplication**: Cannot copy field to create similar field
3. **No Keyboard Shortcuts**: No Tab to next field, Enter to apply, etc.
4. **No Field Preview**: Properties panel doesn't show field render preview
5. **No Metadata Editor**: Cannot edit custom key-value pairs in metadata HashMap
6. **Fixed Panel Width**: Properties panel width not adjustable (always 30%)
7. **No Field Ordering**: Cannot change z-order or field tab order
8. **No Pattern Preview**: Regex patterns not tested against sample values in UI
9. **No Field Templates**: Cannot save frequently used field configurations
10. **Single Field Only**: Cannot edit multiple fields at once

#### Code Quality Achievements

- **Zero Clippy Warnings**: 40 automatic fixes applied
  - Collapsed if statements (let-chains)
  - Removed unnecessary casts (f32 -> f32)
  - Removed clone on Copy types (FieldBounds)
- **All Tests Passing**: 93 tests (no new tests added for Priority 4 yet)
- **CLAUDE.md Compliance**:
  - No `#[allow]` directives
  - Removed unused code (DragState, DragType)
  - Simplified TemplateSnapshot to avoid dead_code warnings
  - Proper error handling and validation
- **Comprehensive Tracing**: All public methods use `#[instrument]`
- **Documentation**: Inline comments for complex logic, public API docs

#### Integration Notes

- Properties panel automatically integrated into editor sidebar
- Parent UI doesn't need to handle properties panel separately
- Panel state automatically synchronized with field selection
- Delete from properties panel equivalent to Delete key
- All property changes push undo snapshots via `state.push_snapshot()`
- Temp state pattern prevents accidental changes:
  - Changes only applied when "Apply" clicked
  - Selecting different field resets temp state
  - Cancel button discards all changes
- Validation runs before applying changes
- Error messages displayed inline above form

#### Performance Notes

- Temp state cloning minimal impact (only field metadata, not canvas)
- Properties panel only renders when field selected
- Validation runs on demand (Apply button click)
- No polling or continuous validation
- Undo snapshots only pushed on Apply, not during editing

#### Next Steps

- Priority 5: Undo/Redo System (foundation already complete)
  - Add keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z)
  - Add undo/redo history browser (needs timestamp/action_description)
  - Visual feedback for undo/redo actions
- Priority 6: Template validation and save
- Future enhancements:
  - Page assignment selector
  - Field duplication
  - Metadata editor
  - Field templates/presets
  - Multi-field editing

### ✅ Priority 5: Undo/Redo System (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Small (~0.5 days)
**Commit**: Pending
**Goal**: Allow users to undo/redo template changes

#### Features

1. **Template Snapshots**
   - Capture template state before changes
   - Store in undo stack
   - Limit stack size (e.g., 50 actions)

2. **Undo/Redo Actions**
   - Ctrl+Z / Cmd+Z for undo
   - Ctrl+Shift+Z / Cmd+Shift+Z for redo
   - UI buttons for undo/redo
   - Disable when stack is empty

3. **Tracked Changes**
   - Field creation
   - Field deletion
   - Field movement
   - Field resize
   - Property changes

#### Implementation

```rust
pub struct TemplateSnapshot {
    template: TemplateBuilder,
    timestamp: SystemTime,
    action_description: String,
}

impl TemplateEditorState {
    pub fn push_snapshot(&mut self, description: impl Into<String>) {
        if let Some(template) = &self.current_template {
            let snapshot = TemplateSnapshot {
                template: template.clone(),
                timestamp: SystemTime::now(),
                action_description: description.into(),
            };

            self.undo_stack.push(snapshot);
            self.redo_stack.clear(); // Clear redo on new action

            // Limit stack size
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
        }
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            if let Some(current) = &self.current_template {
                self.redo_stack.push(TemplateSnapshot {
                    template: current.clone(),
                    timestamp: SystemTime::now(),
                    action_description: "Redo point".to_string(),
                });
            }
            self.current_template = Some(snapshot.template);
        }
    }

    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            if let Some(current) = &self.current_template {
                self.undo_stack.push(TemplateSnapshot {
                    template: current.clone(),
                    timestamp: SystemTime::now(),
                    action_description: "Undo point".to_string(),
                });
            }
            self.current_template = Some(snapshot.template);
        }
    }
}
```

#### Implementation Tasks

- [x] Add `TemplateSnapshot` struct
- [x] Implement undo/redo stacks in `TemplateEditorState`
- [x] Add snapshot push before each modification
- [x] Implement undo/redo methods
- [x] Add keyboard shortcuts
- [x] Add UI buttons with enable/disable logic
- [x] Add action descriptions for better UX

#### Files Modified

**`crates/form_factor_drawing/src/template_ui/state.rs`**:
- Added `timestamp` and `action_description` fields to `TemplateSnapshot`
- Added `TemplateSnapshot::new()` constructor
- Added accessor methods: `description()`, `timestamp()`
- Updated `push_snapshot()` to use `TemplateSnapshot::new()`
- Updated `undo()` and `redo()` to create snapshots with descriptions
- Added `last_undo_description()` and `last_redo_description()` methods
- Added `undo_history()` and `redo_history()` methods for browsing

**`crates/form_factor_drawing/src/template_ui/editor.rs`**:
- Added keyboard shortcuts in input handler:
  - Ctrl+Z / Cmd+Z for undo
  - Ctrl+Shift+Z / Cmd+Shift+Z for redo
  - Ctrl+Y for redo (Windows alternative)
- Added tooltips to Undo/Redo buttons showing:
  - Action description
  - Keyboard shortcut
- Buttons now show "Undo: <action>" / "Redo: <action>" on hover

**`crates/form_factor_drawing/src/template_ui/manipulation.rs`**:
- Updated field drawing to use descriptive snapshot: `"Create field 'field_1'"`
- Updated field deletion to use descriptive snapshot: `"Delete field 'field_1'"`
- Updated drag operations to use specific descriptions:
  - "Move field"
  - "Resize field (top-left)"
  - "Resize field (top-right)"
  - "Resize field (bottom-left)"
  - "Resize field (bottom-right)"

**`crates/form_factor_drawing/src/template_ui/properties.rs`**:
- Updated property application to use descriptive snapshot: `"Edit properties of 'field_1'"`

#### Features Implemented

1. **Enhanced TemplateSnapshot**: ✅
   - Now includes timestamp (SystemTime)
   - Now includes action description (String)
   - Constructor for consistent creation
   - Accessor methods for external use

2. **Keyboard Shortcuts**: ✅
   - Ctrl+Z / Cmd+Z: Undo last action
   - Ctrl+Shift+Z / Cmd+Shift+Z: Redo last undone action
   - Ctrl+Y: Alternative redo (Windows convention)
   - Cross-platform support (Ctrl on Windows/Linux, Cmd on Mac)
   - Checks `can_undo()` / `can_redo()` before executing

3. **Button Tooltips**: ✅
   - Show action description on hover
   - Display keyboard shortcut
   - Dynamic text based on last action
   - Example: "Undo: Create field 'field_1' (Ctrl+Z)"

4. **Descriptive Action Names**: ✅
   - Field creation: "Create field 'field_1'"
   - Field deletion: "Delete field 'field_1'"
   - Field movement: "Move field"
   - Field resizing: "Resize field (top-left)" etc.
   - Property editing: "Edit properties of 'field_1'"

5. **History Browsing API**: ✅
   - `undo_history()` returns full undo stack
   - `redo_history()` returns full redo stack
   - `last_undo_description()` gets most recent undo action
   - `last_redo_description()` gets most recent redo action
   - Foundation for future history browser UI

#### API Usage

**Keyboard Shortcuts** (automatic in editor):
```rust
// User presses Ctrl+Z or Cmd+Z
// -> Undo is triggered automatically

// User presses Ctrl+Shift+Z or Cmd+Shift+Z
// -> Redo is triggered automatically

// User presses Ctrl+Y (Windows/Linux)
// -> Redo is triggered automatically
```

**Button Tooltips** (automatic):
```rust
// Buttons automatically show descriptive tooltips:
// - "Undo: Create field 'field_1' (Ctrl+Z)"
// - "Redo: Delete field 'field_2' (Ctrl+Shift+Z)"
```

**Using Descriptive Actions** (when pushing snapshots):
```rust
// Simple description
state.push_snapshot("Create field");

// Field-specific description
state.push_snapshot(format!("Create field '{}'", field_id));

// Operation-specific description
let action_desc = match operation_type {
    DragOperationType::Move => "Move field",
    DragOperationType::ResizeTopLeft => "Resize field (top-left)",
    // ...
};
state.push_snapshot(action_desc);
```

**Browsing History** (for future UI):
```rust
// Get all undo actions
let undo_actions = state.undo_history();
for snapshot in undo_actions {
    println!("{}: {}", 
        format_timestamp(snapshot.timestamp()), 
        snapshot.description()
    );
}

// Get most recent action descriptions
if let Some(desc) = state.last_undo_description() {
    println!("Can undo: {}", desc);
}
if let Some(desc) = state.last_redo_description() {
    println!("Can redo: {}", desc);
}
```

#### Code Quality Achievements

- **Zero Clippy Warnings**: Fixed 3 collapsible_if warnings
- **All Tests Passing**: 93 tests, no failures
- **CLAUDE.md Compliance**:
  - No `#[allow]` directives used
  - Proper let-chains for condition collapsing
  - Comprehensive tracing with `#[instrument]`
  - Descriptive action names for better debugging
- **Cross-Platform Support**: Works on Windows, Linux, and macOS
- **User Experience**:
  - Standard keyboard shortcuts users expect
  - Tooltips provide discovery and guidance
  - Action descriptions make undo/redo predictable

#### Known Limitations

1. **No History Browser UI**: History API exists but no visual browser yet
2. **No Action Timestamps in UI**: Timestamps captured but not displayed
3. **No Multi-Level Undo Preview**: Can't preview state before undoing
4. **No Undo Groups**: Each action is individual, no batching
5. **No Selective Undo**: Can't undo specific actions out of order
6. **Fixed Stack Size**: Limited to 50 actions (hardcoded)
7. **No Persistent History**: Undo/redo cleared when template closed
8. **No Undo Shortcuts in Documentation**: No in-app help text

#### Integration Notes

- Keyboard shortcuts work anywhere in the editor canvas area
- Shortcuts check `can_undo()` / `can_redo()` before executing
- Button tooltips update automatically when snapshots change
- All existing operations (field creation, deletion, movement, resize, property editing) now use descriptive action names
- Undo/redo operations are logged at debug level
- Stack size limit (50) prevents unbounded memory growth
- Snapshots include full template state (not deltas)

#### Performance Notes

- Each snapshot clones entire `DrawingTemplateBuilder`
- Stack limited to 50 snapshots to control memory usage
- Clone cost is acceptable for template editing workload
- No observable lag during undo/redo operations
- Future optimization: Delta-based snapshots for large templates

#### Next Steps

- Priority 6: Template validation and save
- Future enhancements:
  - History browser UI (visual list of past actions)
  - Display timestamps in history browser
  - Undo grouping (batch related actions)
  - Configurable stack size limit
  - Persistent undo history across sessions
  - Keyboard shortcut help overlay

### Priority 6: Template Validation and Save

**Estimated Effort**: Small (1-2 days)
**Goal**: Validate templates before saving, persist to registry

#### Features

1. **Validation Rules**
   - Template ID is unique
   - Template has at least one field
   - All field names are unique within template
   - All field bounds are valid (positive width/height)
   - All regex patterns are valid
   - All field bounds are within image dimensions

2. **Validation UI**
   - Show validation errors in panel
   - Highlight invalid fields
   - Prevent save until valid

3. **Save Operations**
   - Save to registry
   - Auto-generate template ID if new
   - Confirm overwrite if ID exists
   - Show success/failure notification

#### Implementation

```rust
pub struct TemplateValidator;

impl TemplateValidator {
    pub fn validate(template: &TemplateBuilder, registry: &TemplateRegistry) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check template ID
        if template.id().is_empty() {
            errors.push(ValidationError::EmptyTemplateId);
        } else if registry.contains(template.id()) {
            errors.push(ValidationError::DuplicateTemplateId(template.id().to_string()));
        }

        // Check fields exist
        if template.field_count() == 0 {
            errors.push(ValidationError::NoFields);
        }

        // Check field uniqueness
        let mut field_names = HashSet::new();
        for field in template.all_fields() {
            if !field_names.insert(field.name()) {
                errors.push(ValidationError::DuplicateFieldName(field.name().to_string()));
            }
        }

        // Check field bounds
        for (index, field) in template.all_fields().enumerate() {
            if field.bounds().width <= 0.0 || field.bounds().height <= 0.0 {
                errors.push(ValidationError::InvalidFieldBounds(index, field.name().to_string()));
            }
        }

        // Check regex patterns
        for (index, field) in template.all_fields().enumerate() {
            if let Some(pattern) = field.pattern() {
                if Regex::new(pattern).is_err() {
                    errors.push(ValidationError::InvalidRegex(index, pattern.to_string()));
                }
            }
        }

        errors
    }
}

pub enum ValidationError {
    EmptyTemplateId,
    DuplicateTemplateId(String),
    NoFields,
    DuplicateFieldName(String),
    InvalidFieldBounds(usize, String),
    InvalidRegex(usize, String),
}
```

#### Implementation Tasks

- [ ] Add `TemplateValidator` struct
- [ ] Implement validation rules
- [ ] Add `ValidationError` enum
- [ ] Implement validation UI panel
- [ ] Highlight invalid fields on canvas
- [ ] Add save confirmation dialog
- [ ] Implement save-to-registry operation
- [ ] Add success/failure notifications
- [ ] Disable save button when invalid

## File Organization

```
crates/form_factor_drawing/src/
├── template_ui/
│   ├── mod.rs                      # Module exports
│   ├── manager.rs                  # Template list and CRUD
│   ├── editor.rs                   # Main editor view
│   ├── properties_panel.rs         # Field properties UI
│   ├── validation.rs               # Template validation
│   └── state.rs                    # Editor state management
```

## Testing Strategy

### Unit Tests

- `TemplateEditorState` undo/redo logic
- `TemplateValidator` validation rules
- Field bounds calculations
- Drag detection and tracking

### Integration Tests

- Create template end-to-end
- Save and load template
- Undo/redo workflow
- Field drawing and editing

### Manual Testing

- Multi-page template creation
- Field overlay rendering accuracy
- Zoom/pan interaction
- Keyboard shortcuts

## Dependencies

**Existing Systems**:
- ✅ `TemplateBuilder` (from core)
- ✅ `TemplateRegistry` (from core)
- ✅ `FieldDefinition` (from core)
- ✅ `DrawingCanvas` field overlay rendering
- ✅ Canvas zoom/pan transformations

**New Dependencies**:
- `egui` widgets (already in use)
- File dialogs (already available via `rfd`)

## Migration Path

**Phase 1**: Template Manager Panel (Priority 1)
- Users can browse and manage templates
- Import/export for sharing

**Phase 2**: Basic Editor (Priority 2)
- Users can view templates visually
- Select and inspect fields

**Phase 3**: Field Manipulation (Priority 3)
- Users can create and edit fields
- Full CRUD operations

**Phase 4**: Properties and Validation (Priorities 4-6)
- Complete field editing
- Validation and save

## Future Enhancements

1. **Template Inheritance**: Base templates that can be extended
2. **Field Grouping**: Logical groups for complex forms
3. **Validation Preview**: Test regex patterns with sample data
4. **Keyboard Shortcuts**: Power user features
5. **Grid and Guides**: Alignment aids
6. **Multi-select**: Edit multiple fields at once
7. **Copy/Paste**: Duplicate fields across pages
8. **Export Templates**: Share as standalone files

## Success Criteria

- [ ] User can create template without writing code
- [ ] User can define fields by clicking and dragging
- [ ] User can edit all field properties through UI
- [ ] User can save template to registry
- [ ] User can load saved templates into instances
- [ ] All validation errors are clearly displayed
- [ ] Undo/redo works reliably
- [ ] Multi-page templates are fully supported

---

*This document supersedes Priority 5 from TEMPLATE_SYSTEM_PLAN.md*
