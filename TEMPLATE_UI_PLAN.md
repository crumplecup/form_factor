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

### Priority 2: Basic Template Editor

**Estimated Effort**: Medium (3-4 days)
**Goal**: Load template, view fields, basic selection

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

- [ ] Add `TemplateEditorState` struct
- [ ] Implement template loading from registry
- [ ] Render form image and field overlays
- [ ] Implement field selection (click detection)
- [ ] Add selection highlight rendering
- [ ] Implement page navigation
- [ ] Add mode switching UI
- [ ] Integrate with existing canvas zoom/pan

### Priority 3: Field Drawing and Manipulation

**Estimated Effort**: Medium (3-4 days)
**Goal**: Create, move, and resize fields visually

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

- [ ] Implement field drawing state machine
- [ ] Add preview rectangle rendering
- [ ] Implement drag detection and tracking
- [ ] Add resize handle rendering
- [ ] Implement corner resize logic
- [ ] Add field deletion with confirmation
- [ ] Implement snap-to-edge functionality
- [ ] Add minimum size constraints

### Priority 4: Field Properties Panel

**Estimated Effort**: Small (1-2 days)
**Goal**: Edit all field metadata through UI

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

- [ ] Add `FieldPropertiesPanel` struct
- [ ] Implement basic property inputs (text, combo boxes)
- [ ] Add validation settings UI
- [ ] Implement bounds numeric inputs
- [ ] Add apply/cancel buttons
- [ ] Implement field validation and error display
- [ ] Add regex pattern presets dropdown
- [ ] Implement page assignment selector

### Priority 5: Undo/Redo System

**Estimated Effort**: Small (1-2 days)
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

- [ ] Add `TemplateSnapshot` struct
- [ ] Implement undo/redo stacks in `TemplateEditorState`
- [ ] Add snapshot push before each modification
- [ ] Implement undo/redo methods
- [ ] Add keyboard shortcuts
- [ ] Add UI buttons with enable/disable logic
- [ ] Add action descriptions for better UX

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
