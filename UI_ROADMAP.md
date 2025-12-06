# UI Roadmap: From Current State to Awesome UI

**Status:** Planning
**Created:** 2024-12-05
**Goal:** Transform Form Factor from a partially-integrated application into a polished, user-friendly form processing tool with complete template creation and instance filling workflows

## Executive Summary

Form Factor has **excellent foundation but incomplete integration**. The plugin architecture successfully modularized auxiliary functions (tools, layers, files, detection), and comprehensive template/instance UI panels exist in the codebase. However, these panels are **not integrated into the main application**, leaving users with no way to access core functionality through the UI.

**Current State:** 60% complete
- ✅ Plugin UI working (canvas, layers, files, detection, OCR)
- ✅ Template backend fully implemented
- ✅ Template UI panels fully built (TemplateManagerPanel, TemplateEditorPanel, FieldPropertiesPanel)
- ❌ Template panels not shown anywhere in application
- ❌ Instance data entry UI completely missing
- ❌ No mode management or workflow guidance

**Goal State:** Professional form processing application
- Users can create templates from scratch or imported forms
- Users can fill instances with guided data entry
- Clear visual modes and workflow transitions
- Keyboard shortcuts and accessibility
- Batch operations and productivity features

---

## Problem Statement

### The Core Issue

During the transition to plugin architecture, **template/instance UI panels were built but never integrated** into the main application. These panels exist as isolated components:

**Built but Hidden:**
- `TemplateManagerPanel` - Complete template browser with search, edit, delete
- `TemplateEditorPanel` - Full field editor with undo/redo, validation
- `FieldPropertiesPanel` - Comprehensive field property editing

**Users cannot:**
1. Create a template through the UI (no entry point)
2. See the template manager (panel exists but not shown)
3. Fill instance data (no data entry interface exists at all)
4. Switch between template editing and instance filling modes
5. See which mode they're in
6. Navigate pages during instance data entry

### User Impact

**Current user experience:**
```
User: "I want to create a template for W-2 forms"
App: *shows canvas with drawing tools*
User: *draws rectangles on canvas*
User: "How do I make these template fields?"
App: *no UI to do this*
```

**Desired user experience:**
```
User: "I want to create a template for W-2 forms"
App: *shows "New Template" button*
User: *clicks "New Template"*
App: *shows template editor with field drawing tools*
User: *draws fields, sets properties, saves*
App: *template saved to library*
User: "Now I want to fill out a W-2"
App: *shows "Fill Form" button*
User: *selects W-2 template, fills data*
App: *saves completed instance*
```

---

## Current State Assessment

### What Works (Plugin UI)

#### CanvasPlugin
- ✅ Tool selection: Select, Rectangle, Circle, Freehand, Edit, Rotate
- ✅ Zoom controls: +/-, Reset, percentage display
- ✅ Pan offset display

#### LayersPlugin
- ✅ Layer visibility toggles (6 layers: Canvas, Detections, Shapes, Grid, Template, Instance)
- ✅ Lock indicators
- ✅ Selection highlighting
- ✅ Clear layer buttons (partial - Template/Instance handlers TODO)

#### FilePlugin
- ✅ Open/Save/Save As buttons
- ✅ Recent files list (max 10)
- ✅ File path display

#### DetectionPlugin
- ✅ Text detection button
- ✅ Logo detection button
- ✅ Detection counts display

#### OcrPlugin
- ✅ OCR extraction button
- ✅ Extracted text display (scrollable)

### What's Built but Not Integrated (Template UI)

#### TemplateManagerPanel
**Location:** `form_factor_drawing/src/template_ui/manager.rs`

**Features:**
- Template list with search filtering
- New Template button
- Import button (deferred implementation)
- Per-template actions: Edit, Duplicate, Export, Delete
- Delete confirmation dialog
- Shows template metadata (ID, name, page count, field count)

**Status:** ✅ Fully functional, ❌ Not shown in application

#### TemplateEditorPanel
**Location:** `form_factor_drawing/src/template_ui/editor.rs`

**Features:**
- Mode toolbar: Select, Draw, Edit
- Page navigation (Prev/Next)
- Undo/Redo with 50-action stack
- Validate button
- Save/Cancel buttons
- Canvas area with field rendering
- Resize handles on selected fields
- Drawing preview
- Keyboard shortcuts (Delete, Ctrl+Z, Ctrl+Shift+Z)

**Status:** ✅ Fully functional, ❌ Not shown in application

#### FieldPropertiesPanel
**Location:** `form_factor_drawing/src/template_ui/properties.rs`

**Features:**
- ID and label editing
- Field type selection (38+ types organized by category)
- Validation: Required checkbox, regex pattern, presets
- Help text editing
- Position & size drag values
- Apply/Cancel/Delete actions
- Real-time validation

**Status:** ✅ Fully functional, ❌ Not shown in application

### What's Missing Entirely

#### Instance Data Entry UI
**Status:** ❌ Does not exist

**Required features:**
- Field-by-field data entry form
- Field type-specific input widgets:
  - Text fields for text/name/email
  - Date pickers for dates
  - Checkbox for booleans
  - Signature capture for signatures
- Validation feedback during entry
- Required field highlighting
- Completion indicator (progress bar or field checklist)
- Multi-page navigation during data entry
- Save/Cancel/Validate buttons

#### Mode Management UI
**Status:** ❌ Does not exist

**Required features:**
- Visual mode indicator (banner or toolbar showing current mode)
- Mode switcher (radio buttons or tabs)
- Modes: View Canvas, Edit Template, Fill Instance, View Instance
- Context-sensitive help text
- Workflow guidance (tooltips, hints)

#### Template/Instance Management Integration
**Status:** ❌ Does not exist

**Required features:**
- Template library browser (uses TemplateManagerPanel)
- Instance browser (list of filled forms)
- Project workspace (template + related instances)
- Batch operations (export multiple instances, duplicate templates)

---

## Architecture Analysis

### Current Architecture

```
Main Window
├── Right Sidebar (280px)
│   ├── CanvasPlugin
│   ├── LayersPlugin
│   ├── FilePlugin
│   ├── DetectionPlugin
│   └── OcrPlugin
└── Central Panel
    └── DrawingCanvas.ui()
```

**Plugin event bus:**
- Plugins emit AppEvent
- Main.rs processes events and calls canvas methods
- Canvas updates and emits response events

### Architectural Challenges

1. **Modal vs. Modeless Panels**
   - Template editing needs significant screen space
   - Instance data entry needs focused, distraction-free interface
   - Current plugin sidebar (280px) too narrow for complex editing

2. **State Machine Complexity**
   - Canvas has modes: TemplateMode, InstanceMode
   - No UI state machine to coordinate mode transitions
   - No visual feedback of current mode

3. **Panel Orchestration**
   - Template panels return action enums (ManagerAction, EditorAction, PropertiesAction)
   - Main.rs needs to process these actions and update canvas
   - Current plugin system only handles AppEvent, not action enums

4. **Layout Flexibility**
   - Current layout: fixed sidebar + central canvas
   - Template editing needs: manager sidebar + canvas + properties sidebar
   - Instance filling needs: template overlay + data entry panel
   - Need dynamic layout based on mode

### Design Patterns to Consider

#### Option 1: Plugin Wrapper for Template/Instance Panels
**Pros:**
- Consistent with plugin architecture
- Easy to toggle visibility
- Minimal changes to main.rs

**Cons:**
- Panels too wide for 280px sidebar
- Action enum handling awkward in plugin system
- Modal workflows (template editing) don't fit plugin pattern

#### Option 2: Mode-Specific Layouts
**Pros:**
- Optimal layout for each mode
- Clear visual separation
- Natural workflow transitions

**Cons:**
- More complex main.rs layout logic
- Need layout state machine
- Harder to preview template while filling instance

#### Option 3: Tabbed Interface
**Pros:**
- Familiar pattern (browser-like)
- Can have multiple templates/instances open
- Easy mode switching

**Cons:**
- Tab bar overhead
- Complex state management
- May not fit form-filling mental model

**Recommendation:** **Option 2 (Mode-Specific Layouts)** with simplified state machine

---

## UI Roadmap Phases

### Phase 1: Template UI Integration (Week 1-2)

**Goal:** Expose existing template UI panels to users

#### 1.0 Property Inspector Plugin ✅

**Status:** Complete
**Commit:** 3999308

**Implementation:**
- Created `PropertiesPlugin` for editing selected shape/field properties
- Supports both shape editing (position, size, color, label) and template field editing (name, type, position, size, required)
- Listens for `ShapeSelected` and `SelectionCleared` events
- Integrated into plugin system with feature flag `plugin-properties`
- Added to `all-plugins` feature for easy enablement

**Next Steps:**
- Enhance with actual shape data fetching from canvas
- Add property change event emission back to canvas
- Support multi-selection property editing

#### 1.1 Add Mode Management ✅

**Status:** Complete
**Commit:** 89f83eb

**Implementation:**
- Created `AppMode` enum with 5 states: Canvas, TemplateManager, TemplateEditor, InstanceFilling, InstanceViewing
- Created `AppState` struct managing current mode, navigation history, and associated data
- Mode transition validation prevents data loss (blocks when unsaved changes exist)
- Back navigation with previous mode tracking
- Comprehensive test coverage (6 unit tests)

**Files:**
- `form_factor_drawing/src/app_mode.rs` - Mode management implementation
- Exported through `form_factor` facade crate

**Next Steps:**
- Integrate AppState into FormFactorApp in main.rs
- Wire mode changes to UI events
- Add mode indicator UI component

#### 1.2 Template/Instance Mode Switcher ✅

**Status:** Complete
**Files:** 
- `form_factor_drawing/src/mode_switcher.rs` - ModeSwitcher UI component
- `form_factor/src/main.rs` - Integration into main app

**Implementation:**
- Created `ModeSwitcher` component providing toolbar for mode transitions
- Integrated with `AppState` for validated transitions
- Top toolbar showing current mode with selectable buttons:
  - Canvas mode (always available)
  - Template Manager (📋 Templates)
  - Template Editor (✏ Edit Template - shown when template loaded)
  - Instance Filling (📝 Fill Form - shown when instance loaded)
  - Instance Viewing (👁 View Form - shown when instance loaded)
- Unsaved changes indicator (⚠)
- Back button for navigation history
- Confirmation dialog for mode changes with unsaved changes
- Support for Save & Continue, Discard & Continue, Cancel

**Layout variations:**

1. **Canvas Mode** (current):
   ```
   [Top: Mode Switcher] | [Right Sidebar: Plugins] | [Central: Canvas]
   ```

2. **Template Manager Mode** (future):
   ```
   [Top: Mode Switcher] | [Full Width: TemplateManagerPanel]
   ```

3. **Template Editor Mode** (future):
   ```
   [Top: Mode Switcher] | [Left: Field Properties 30%] | [Center: Editor Canvas 70%]
   ```

**Next Steps:**
- Implement actual layout switching based on AppMode
- Wire mode changes to show/hide appropriate panels
- Add keyboard shortcut to switch modes (Ctrl+M)

#### 1.3 Template Manager Integration

**Entry point:** Add "Templates" button to plugin sidebar or menu bar

**Workflow:**
1. User clicks "Templates" button
2. App switches to TemplateManager mode
3. TemplateManagerPanel shown full-width
4. User can: New, Edit, Duplicate, Delete templates
5. Clicking "Edit" switches to TemplateEditor mode
6. Clicking "Back" returns to Canvas mode

**Event handling:**
```rust
match manager_panel.ui(&mut ui, &registry) {
    ManagerAction::New => {
        app_state.start_new_template();
        app_state.set_mode(AppMode::TemplateEditor);
    }
    ManagerAction::Edit(template_id) => {
        app_state.load_template_for_editing(template_id);
        app_state.set_mode(AppMode::TemplateEditor);
    }
    // ... other actions
}
```

#### 1.4 Template Editor Integration

**Canvas integration:**
- Use existing TemplateEditorPanel
- Editor has its own canvas area (not DrawingCanvas)
- FieldPropertiesPanel shown on left (30% width)

**Workflow:**
1. User in TemplateEditor mode (from manager or "New Template")
2. Editor panel shown with canvas and properties
3. User draws fields, edits properties, validates
4. Save → saves to registry, returns to TemplateManager mode
5. Cancel → confirms discard, returns to TemplateManager mode

**Save handling:**
```rust
match editor_panel.ui(&mut ui, &mut template, &mut undo_stack) {
    EditorAction::Save => {
        registry.save(&template)?;
        app_state.set_mode(AppMode::TemplateManager);
    }
    EditorAction::Cancel => {
        if confirm_discard_changes() {
            app_state.set_mode(AppMode::TemplateManager);
        }
    }
    EditorAction::None => {}
}
```

#### 1.5 Template Layer Clear Handler

**File:** `form_factor/src/main.rs`

**Implementation:**
```rust
AppEvent::LayerClearRequested { layer_name } => {
    match layer_name.as_str() {
        "Template" => {
            if let Some(canvas) = &mut self.canvas {
                canvas.clear_template_fields();
                canvas.set_template_mode(TemplateMode::None);
            }
        }
        // ... other layers
    }
}
```

#### 1.6 Mode Indicator UI

**Top banner (optional, non-intrusive):**
```
[Mode: Template Editor] [Back to Canvas] [Help: Ctrl+M to switch modes]
```

**Alternative: Status bar at bottom:**
```
Mode: Template Editor | Template: W-2 Form | Fields: 12 | Valid: Yes
```

### Phase 1 Deliverables

- ✅ AppState with mode management
- ✅ Mode-specific layouts in main.rs
- ✅ "Templates" button to enter TemplateManager mode
- ✅ TemplateManagerPanel integrated
- ✅ TemplateEditorPanel integrated
- ✅ Template layer clear handler
- ✅ Mode indicator UI
- ✅ Back navigation

### Phase 1 Success Metrics

- Users can click "Templates" button
- Template library shown with existing templates
- Users can create new template via "New Template" button
- Template editor shown with drawing canvas and properties panel
- Users can draw fields, set properties, validate, save
- Saved templates appear in library
- Users can edit existing templates
- Users can return to Canvas mode via "Back" button
- Template layer clear button works

---

### Phase 2: Instance Data Entry UI (Week 3-4)

**Goal:** Build and integrate instance filling workflow

#### 2.1 Instance Data Entry Panel

**File:** `form_factor_drawing/src/instance_ui/data_entry.rs` (new file)

**UI Components:**

1. **Header:**
   - Template name display
   - Instance name input
   - Page navigation (X of Y)
   - Progress indicator (% complete)

2. **Field List (scrollable):**
   - Per-page field list (only show current page fields)
   - Per field:
     - Label (from template)
     - Input widget (type-specific)
     - Required indicator (red asterisk)
     - Validation error (red text below input)
     - Help text (tooltip or info icon)

3. **Input Widgets by Field Type:**
   - **Text fields**: TextEdit for FullName, FirstName, Email, etc.
   - **Numeric**: DragValue for Currency, Amount
   - **Date**: DatePicker widget (egui_extras::DatePickerButton)
   - **Boolean**: Checkbox for Checkbox field type
   - **Signature**: Button to open signature capture (deferred)
   - **Initials**: Small TextEdit (max 3-4 chars)

4. **Footer:**
   - Validation status (X errors, Y fields remaining)
   - Previous Page / Next Page buttons
   - Save Draft button
   - Submit button (enabled when valid)
   - Cancel button

**State management:**
```rust
pub struct DataEntryPanel {
    template: DrawingTemplate,
    instance: DrawingInstance,
    current_page: usize,
    validation_errors: HashMap<String, String>,
    dirty: bool, // Unsaved changes
}

pub enum DataEntryAction {
    SaveDraft,
    Submit,
    Cancel,
    None,
}
```

#### 2.2 Field-Specific Input Widgets

**Email field:**
```rust
ui.label("Email Address *");
ui.text_edit_singleline(&mut field_value);
if let Err(e) = validate_email(&field_value) {
    ui.colored_label(Color32::RED, e);
}
```

**Date field:**
```rust
ui.label("Date of Birth *");
if ui.add(egui_extras::DatePickerButton::new(&mut date_value)).changed() {
    mark_field_dirty("date_of_birth");
}
```

**Currency field:**
```rust
ui.label("Amount *");
ui.add(DragValue::new(&mut amount_value)
    .prefix("$")
    .speed(0.01));
```

#### 2.3 Validation Integration

**Real-time validation:**
- On field blur (focus lost), validate field
- Show error message immediately
- Update progress indicator
- Enable/disable Submit based on validation state

**Validation rules:**
- Required fields must have non-empty values
- Field type validation (email format, phone format, etc.)
- Regex pattern matching (if defined in template)
- Cross-field validation (if implemented in template)

#### 2.4 Instance Filling Mode Integration

**Entry point:** "Fill Form" button in Canvas mode or TemplateManager mode

**Workflow:**
1. User clicks "Fill Form" button
2. If in TemplateManager: show template picker
3. App creates new DrawingInstance from selected template
4. App switches to InstanceFilling mode
5. DataEntryPanel shown with first page fields
6. User fills data, navigates pages
7. Submit → validates all fields, saves instance, returns to Canvas mode
8. Save Draft → saves instance with incomplete data
9. Cancel → confirms discard, returns to previous mode

**Layout (InstanceFilling mode):**
```
[Left: Template overlay 40%] | [Right: Data Entry Panel 60%]
```

Template overlay shows:
- Form image (if loaded)
- Template fields highlighted
- Current field highlighted during editing

#### 2.5 Instance Management

**File:** `form_factor_drawing/src/instance_ui/instance_manager.rs` (new file)

**Features:**
- List all instances (grouped by template)
- Filter by template, date, validation status
- Actions: View, Edit, Export, Delete
- Export formats: JSON, CSV (future)

**Entry point:** "Instances" button in Canvas mode or menu

**Workflow:**
1. User clicks "Instances" button
2. Instance list shown with metadata
3. User can view (read-only) or edit (reopen data entry)
4. User can export (save as JSON) or delete

#### 2.6 Instance Layer Clear Handler

**File:** `form_factor/src/main.rs`

**Implementation:**
```rust
AppEvent::LayerClearRequested { layer_name } => {
    match layer_name.as_str() {
        "Instance" => {
            if let Some(canvas) = &mut self.canvas {
                canvas.clear_instance_fields();
                canvas.set_instance_mode(InstanceMode::None);
            }
        }
        // ... other layers
    }
}
```

### Phase 2 Deliverables

- ✅ DataEntryPanel with field-specific input widgets
- ✅ Real-time validation with error display
- ✅ Multi-page navigation during data entry
- ✅ Progress indicator (% complete)
- ✅ Save Draft and Submit actions
- ✅ InstanceFilling mode layout
- ✅ Template overlay during data entry
- ✅ Instance manager panel
- ✅ Instance layer clear handler
- ✅ "Fill Form" entry point

### Phase 2 Success Metrics

- Users can click "Fill Form" and select template
- Data entry panel shown with all template fields
- Users can enter data with type-specific widgets
- Real-time validation shows errors immediately
- Progress indicator updates as fields completed
- Users can navigate pages and see only relevant fields
- Users can save draft (partial data)
- Users can submit (validates all, saves instance)
- Users can view list of instances
- Users can edit existing instance data
- Instance layer clear button works

---

### Phase 3: Workflow Enhancements (Week 5-6)

**Goal:** Polish user experience with shortcuts, guidance, and productivity features

#### 3.1 Keyboard Shortcuts

**Global shortcuts:**
- `Ctrl+N`: New Template
- `Ctrl+O`: Open File (existing)
- `Ctrl+S`: Save (context-aware)
- `Ctrl+M`: Mode Switcher Dialog
- `Ctrl+T`: Templates Manager
- `Ctrl+F`: Fill Form
- `Ctrl+I`: Instances List
- `Esc`: Back to previous mode / Cancel

**Template Editor shortcuts:**
- `Ctrl+Z`: Undo
- `Ctrl+Shift+Z` or `Ctrl+Y`: Redo
- `Delete`: Delete selected field
- `D`: Draw mode
- `S`: Select mode
- `E`: Edit mode
- `Ctrl+V`: Validate
- `Ctrl+Enter`: Save template

**Instance Filling shortcuts:**
- `Tab`: Next field
- `Shift+Tab`: Previous field
- `Ctrl+Enter`: Submit (if valid)
- `Ctrl+D`: Save Draft
- `PageDown`: Next page
- `PageUp`: Previous page

**Implementation:**
```rust
impl App {
    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
            self.start_new_template();
        }
        // ... other shortcuts
    }
}
```

#### 3.2 Workflow Guidance

**First-time user experience:**
- Welcome dialog on first launch
- Quick tour highlighting key features
- "Create Your First Template" wizard

**Contextual help:**
- Tooltips on all buttons and inputs
- Info icons with help text
- Status bar hints (e.g., "Press 'D' to draw fields")
- Empty state messages (e.g., "No templates yet. Click 'New Template' to start.")

**Wizard workflows:**

1. **"Create Template from Form" wizard:**
   - Step 1: Load form image
   - Step 2: Run text detection (optional)
   - Step 3: Draw or auto-detect fields
   - Step 4: Review and label fields
   - Step 5: Set validation rules
   - Step 6: Save template

2. **"Fill Form" wizard:**
   - Step 1: Select template
   - Step 2: Name instance (optional)
   - Step 3: Fill page 1 fields
   - Step 4: Fill page 2 fields (if multi-page)
   - Step 5: Review and validate
   - Step 6: Submit

#### 3.3 Template Import/Export

**Import workflow:**
- Import template JSON (from file or registry)
- Import template from PDF (extract fields via OCR)
- Import from standard form libraries (W-2, 1099, etc.)

**Export workflow:**
- Export template as JSON
- Export template with filled instance (JSON bundle)
- Export instance data only (CSV for database import)

#### 3.4 Batch Operations

**Template operations:**
- Duplicate multiple templates
- Delete multiple templates (with confirmation)
- Export multiple templates as ZIP

**Instance operations:**
- Export multiple instances as CSV
- Export all instances for a template
- Delete all instances for a template (with confirmation)
- Batch validation (validate all instances)

#### 3.5 Recent Items and Favorites

**Recent templates:**
- Track recently edited templates (max 10)
- Quick access from Templates button dropdown

**Favorite templates:**
- Star/favorite frequently-used templates
- Show favorites at top of template list
- "Pin to sidebar" for instant access

**Recent instances:**
- Show recently filled instances
- Quick edit access

#### 3.6 Search and Filtering

**Template search:**
- Search by template name, ID, description
- Filter by field types (show only templates with SSN field)
- Filter by page count (single-page vs multi-page)
- Sort by: name, date modified, field count

**Instance search:**
- Search by instance name, template name
- Filter by completion status (complete, draft, errors)
- Filter by date range
- Filter by validation status

### Phase 3 Deliverables

- ✅ Global keyboard shortcuts
- ✅ Mode-specific keyboard shortcuts
- ✅ Welcome dialog and quick tour
- ✅ Contextual help and tooltips
- ✅ "Create Template from Form" wizard
- ✅ "Fill Form" wizard
- ✅ Import template from JSON
- ✅ Export template as JSON
- ✅ Export instance data as CSV
- ✅ Batch delete templates/instances
- ✅ Recent templates/instances tracking
- ✅ Favorite templates feature
- ✅ Search and filtering in all lists

### Phase 3 Success Metrics

- Users can navigate entire app via keyboard
- First-time users see welcome dialog and tour
- Tooltips and help text guide users
- Users can complete workflows via wizards
- Users can import/export templates and instances
- Batch operations work for multiple items
- Recent items accessible via quick menus
- Search returns relevant results instantly

---

### Phase 4: Advanced Features (Week 7-8)

**Goal:** Add power-user features and polish

#### 4.1 Multi-Window Support

**Feature:** Open multiple templates/instances in separate windows

**Use cases:**
- Compare two templates side-by-side
- Reference one instance while filling another
- Copy fields from one template to another

**Implementation:**
- Each window has own egui context
- Shared AppState via Arc<Mutex<>>
- Window manager tracks open windows

#### 4.2 Undo/Redo for Instance Data Entry

**Feature:** Undo data entry mistakes

**Implementation:**
- Track data entry actions in undo stack
- Ctrl+Z undoes last field change
- Ctrl+Shift+Z redoes

**Stack limit:** 50 actions (same as template editor)

#### 4.3 Auto-Fill from OCR

**Feature:** Automatically populate instance fields from OCR results

**Workflow:**
1. User loads form image
2. User clicks "Auto-Fill from OCR"
3. App runs OCR on image
4. App matches OCR text to template fields (spatial matching)
5. App populates field values with confidence scores
6. User reviews and corrects errors
7. User submits

**Matching strategy:**
- Match field bounds to OCR bounding boxes
- Score by overlap percentage
- Populate field if confidence > 80%
- Flag fields with lower confidence for review

#### 4.4 Field Templates (Snippets)

**Feature:** Save common field configurations for reuse

**Examples:**
- "US Address" snippet: StreetAddress, City, State, ZipCode fields pre-configured
- "Personal Info" snippet: FirstName, LastName, Email, Phone fields
- "Payment Info" snippet: Currency, Amount, Account Number fields

**Workflow:**
1. User creates fields in template editor
2. User selects multiple fields
3. User clicks "Save as Snippet"
4. User names snippet (e.g., "US Address")
5. Snippet saved to library
6. In future templates, user clicks "Insert Snippet"
7. All snippet fields added at once

#### 4.5 Template Versioning

**Feature:** Track template versions and migrate instances

**Versioning:**
- Template has version string (e.g., "1.0", "1.1", "2.0")
- Each edit increments version
- Instances track which template version they're from

**Migration:**
- When template changes (add/remove fields), version increments
- Old instances show migration warning
- User can migrate instance to new template version
- Migration wizard guides user through new required fields

**Implementation:**
```rust
pub struct TemplateVersion {
    version: String,
    created_at: DateTime<Utc>,
    changes: Vec<TemplateChange>,
}

pub enum TemplateChange {
    FieldAdded(FieldDefinition),
    FieldRemoved(String), // field_id
    FieldModified { old: FieldDefinition, new: FieldDefinition },
}
```

#### 4.6 Collaboration Features (Future)

**Feature:** Share templates with team

**Sharing options:**
- Export template with QR code (imports into another instance)
- Cloud sync to template library (requires backend)
- Email template as attachment

**Access control:**
- Public templates (anyone can use)
- Team templates (only team members)
- Private templates (only you)

#### 4.7 Accessibility

**WCAG 2.1 AA Compliance:**

**Keyboard navigation:**
- All UI accessible via keyboard
- Tab order logical
- Focus indicators visible

**Screen reader support:**
- All buttons have aria-labels
- Form fields have labels
- Error messages announced

**Visual:**
- High contrast mode
- Configurable font sizes
- Color-blind friendly error colors (not just red/green)

**Motor:**
- Click targets minimum 44x44 pixels
- No time-based interactions
- Drag alternatives (use arrow keys)

#### 4.8 Themes and Customization

**Theme options:**
- Light theme (default)
- Dark theme
- High contrast theme

**Customization:**
- Font size (small, medium, large)
- Sidebar width adjustment
- Layout preferences (save per-user)

**Implementation:**
```rust
pub struct UserPreferences {
    theme: Theme,
    font_size: FontSize,
    sidebar_width: f32,
    show_help_tooltips: bool,
    auto_save_interval: Option<Duration>,
}
```

### Phase 4 Deliverables

- ✅ Multi-window support
- ✅ Undo/redo for instance data entry
- ✅ Auto-fill from OCR with confidence scores
- ✅ Field snippet library
- ✅ Template versioning and instance migration
- ✅ Template export with QR code
- ✅ Keyboard navigation for all features
- ✅ Screen reader support
- ✅ High contrast theme
- ✅ User preferences panel

### Phase 4 Success Metrics

- Users can open multiple windows
- Users can undo data entry mistakes
- OCR auto-fill populates fields with 80%+ accuracy
- Users can save and reuse field snippets
- Template versions tracked and instances migrate smoothly
- All features accessible via keyboard
- Screen readers can navigate entire application
- High contrast mode meets WCAG AA standards
- User preferences persist across sessions

---

## Implementation Guidelines

### UI Design Principles

1. **Progressive Disclosure**
   - Show simple options first
   - Advanced features behind "More" or "Advanced" buttons
   - Wizards for complex workflows, direct access for power users

2. **Consistent Layout**
   - All manager panels (templates, instances) use same pattern: search, list, actions
   - All editor panels use same pattern: toolbar, canvas, properties sidebar
   - All data entry uses same pattern: field label, input, validation error

3. **Responsive Feedback**
   - Button clicks show immediate feedback (color change, animation)
   - Long operations show progress indicators
   - Validation shows errors immediately (on blur, not on submit)
   - Save shows success confirmation (toast or status bar)

4. **Error Recovery**
   - All destructive actions have confirmation dialogs
   - All long-form edits have "Save Draft" option
   - Undo/redo for all editing operations
   - Auto-save drafts every 30 seconds (optional setting)

5. **Performance**
   - UI updates at 60 FPS minimum
   - Large lists (100+ templates) use virtual scrolling
   - Image loading asynchronous with spinner
   - OCR/detection operations show progress bar

### Code Organization

**New files to create:**

```
form_factor/src/
├── app_state.rs          # AppMode enum, state machine
├── layouts.rs            # Layout functions per mode
└── shortcuts.rs          # Keyboard shortcut handling

form_factor_drawing/src/
├── instance_ui/
│   ├── mod.rs
│   ├── data_entry.rs     # DataEntryPanel
│   ├── instance_manager.rs  # Instance list and management
│   └── validation_ui.rs  # Validation error display
└── template_ui/          # (already exists)
    ├── mod.rs
    ├── manager.rs        # (already exists)
    ├── editor.rs         # (already exists)
    └── properties.rs     # (already exists)
```

### Testing Strategy

**Unit tests:**
- AppState mode transitions
- Layout switcher logic
- Keyboard shortcut parsing
- Validation error formatting

**Integration tests:**
- Template manager → editor workflow (Phase 1)
- Instance filling → validation → save workflow (Phase 2)
- Auto-fill from OCR (Phase 4)
- Template versioning and migration (Phase 4)

**Manual testing:**
- First-time user experience
- Keyboard-only navigation
- Screen reader compatibility
- Cross-platform (Linux, macOS, Windows)

### Performance Targets

- **UI frame time:** < 16ms (60 FPS)
- **Template list load:** < 100ms for 100 templates
- **Template editor load:** < 200ms
- **Instance data entry load:** < 100ms
- **Validation:** < 50ms for 50 fields
- **Auto-save:** Non-blocking, async

### Accessibility Targets

- **WCAG 2.1 AA:** All criteria met
- **Keyboard navigation:** 100% coverage
- **Screen reader:** All UI elements labeled
- **Color contrast:** 4.5:1 minimum
- **Click targets:** 44x44px minimum

---

## Success Criteria

### Phase 1 Success
- ✅ Users can create templates via UI (not code)
- ✅ Users can edit existing templates
- ✅ Template library browsable and searchable
- ✅ Template editor has undo/redo working
- ✅ Mode indicator shows current mode

### Phase 2 Success
- ✅ Users can fill instances via data entry panel
- ✅ Validation errors shown in real-time
- ✅ Multi-page navigation works during data entry
- ✅ Instances saved and listed in instance manager
- ✅ Users can edit existing instance data

### Phase 3 Success
- ✅ Keyboard shortcuts work for all major actions
- ✅ First-time users guided through workflows
- ✅ Import/export templates and instances
- ✅ Batch operations work for templates and instances
- ✅ Search and filtering instant and accurate

### Phase 4 Success
- ✅ Multi-window support for comparing templates
- ✅ Auto-fill from OCR with 80%+ accuracy
- ✅ Template versioning tracks changes
- ✅ WCAG AA compliance achieved
- ✅ User preferences persist across sessions

### Overall Success
- ✅ 100% of template/instance workflows accessible via UI
- ✅ Zero code required for normal operations
- ✅ New user can create template and fill instance in < 5 minutes
- ✅ Power user can batch process 100 instances in < 10 minutes
- ✅ Application feels polished and professional

---

## Risk Management

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Mode state machine complexity | High | Start simple (3 modes), add complexity incrementally |
| Layout switching performance | Medium | Cache layout calculations, use egui efficiently |
| Multi-window state sync | High | Use Arc<Mutex<AppState>>, minimize shared state |
| OCR auto-fill accuracy | Medium | Manual review step, confidence thresholds |
| Template versioning migrations | High | Wizard-guided migration, validation before save |

### UX Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Too many modes confuses users | High | Clear mode indicator, limited modes (5 max) |
| Keyboard shortcuts conflict | Medium | Follow platform conventions, make configurable |
| Data loss on crashes | Critical | Auto-save every 30s, save on mode change |
| Overwhelming first-time experience | High | Welcome wizard, progressive disclosure |
| Accessibility barriers | High | Test with screen readers, keyboard-only testing |

### Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Phase 1 takes longer than expected | High | Phase 1 is MVP, delay later phases if needed |
| OCR integration complex | Medium | Phase 4 item, can defer to Phase 5 if needed |
| Multi-window support difficult | Medium | Optional feature, can skip if time-constrained |

---

## Dependencies

### External Dependencies

**Existing crates:**
- `egui` - UI framework (already used)
- `egui_extras` - DatePickerButton for date fields
- `rfd` - File dialogs (already used)
- `serde_json` - Serialization (already used)

**New crates (optional):**
- `egui_toast` - Toast notifications for save confirmations
- `egui_dropdown` - Improved dropdown menus
- `egui_virtual_list` - Virtual scrolling for large lists

### Internal Dependencies

**Must complete first:**
- Integration testing plan (parallel work)
- Template UI panels (already done ✅)

**Can work in parallel:**
- Phase 1 (template integration) and Phase 2 (instance UI) are independent
- Phase 3 (enhancements) depends on Phase 1 and 2
- Phase 4 (advanced) can cherry-pick features independently

---

## Maintenance Plan

### After Implementation

1. **User feedback collection**
   - Survey after first template creation
   - Analytics on most-used features
   - Bug reports tracked in issues

2. **Performance monitoring**
   - Frame time tracking
   - Memory usage profiling
   - Load time metrics

3. **Accessibility audits**
   - Quarterly screen reader testing
   - Keyboard navigation reviews
   - Color contrast checks

4. **Documentation updates**
   - User guide for each major feature
   - Video tutorials for workflows
   - FAQ based on common questions

### Future Enhancements (Phase 5+)

- **Cloud sync** - Sync templates/instances across devices
- **Collaboration** - Real-time co-editing of templates
- **Advanced OCR** - ML-based field detection and classification
- **Mobile app** - iOS/Android app for instance filling
- **API** - REST API for programmatic access
- **Plugins** - Third-party plugin marketplace
- **Integrations** - Zapier, Google Forms, etc.

---

## Appendix A: UI Mockups

### Template Manager Mode

```
┌─────────────────────────────────────────────────────────────┐
│ [Mode: Template Manager]              [Back to Canvas] [Help]│
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Templates                           [New Template] [Import] │
│  ┌───────────────────────────────────────────────────────┐   │
│  │ Search: [____________]                           🔍   │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌───────────────────────────────────────────────────────┐   │
│  │ ○ W-2 Form                                            │   │
│  │   ID: w2_2024 | Pages: 1 | Fields: 18                │   │
│  │   [Edit] [Duplicate] [Export] [Delete]               │   │
│  ├───────────────────────────────────────────────────────┤   │
│  │ ○ 1099-MISC                                           │   │
│  │   ID: 1099_misc | Pages: 1 | Fields: 12              │   │
│  │   [Edit] [Duplicate] [Export] [Delete]               │   │
│  ├───────────────────────────────────────────────────────┤   │
│  │ ○ Employee Onboarding                                 │   │
│  │   ID: employee_onboard | Pages: 3 | Fields: 24       │   │
│  │   [Edit] [Duplicate] [Export] [Delete]               │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Template Editor Mode

```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Mode: Template Editor - W-2 Form]        [Back] [Validate] [Save] [?] │
├──────────────────────┬──────────────────────────────────────────────────┤
│ Field Properties     │ [Select] [Draw] [Edit]  Page 1/1  [Undo] [Redo] │
│ ────────────────────│                                                   │
│                      │  ┌──────────────────────────────────────────┐   │
│ ID: [employee_name__]│  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│                      │  │░░┌─────────────┐░░░░░░░░░░░░░░░░░░░░░░░│   │
│ Label:               │  │░░│ Full Name   │░░░░░░░░░░░░░░░░░░░░░░░│   │
│ [Employee Name_____] │  │░░└─────────────┘░░░░░░░░░░░░░░░░░░░░░░░│   │
│                      │  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│ Type:                │  │░░┌────────┐░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│ [FullName      ▼]    │  │░░│ SSN    │░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│                      │  │░░└────────┘░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│ ☑ Required           │  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│                      │  │░░┌──────────────┐░░░░░░░░░░░░░░░░░░░░░░│   │
│ Pattern:             │  │░░│ Wages        │░░░░░░░░░░░░░░░░░░░░░░│   │
│ [_______________]    │  │░░└──────────────┘░░░░░░░░░░░░░░░░░░░░░░│   │
│ [Email][Phone][ZIP]  │  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│   │
│                      │  └──────────────────────────────────────────┘   │
│ Help Text:           │                                                   │
│ ┌──────────────────┐ │  12 fields total                                 │
│ │Enter your full   │ │                                                   │
│ │legal name        │ │                                                   │
│ └──────────────────┘ │                                                   │
│                      │                                                   │
│ Position: X: 100     │                                                   │
│           Y: 50      │                                                   │
│ Size: W: 300         │                                                   │
│       H: 30          │                                                   │
│                      │                                                   │
│ [Apply] [Cancel]     │                                                   │
│        [Delete Field]│                                                   │
└──────────────────────┴──────────────────────────────────────────────────┘
```

### Instance Filling Mode

```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Mode: Fill Instance - W-2 Form]                [Save Draft] [Submit]  │
├──────────────────────┬──────────────────────────────────────────────────┤
│ Template Preview     │ Instance Name: [John Doe W-2 2024____________]  │
│ (Form Image)         │ Page 1 of 1                Progress: 67% ████░  │
│                      ├──────────────────────────────────────────────────┤
│ ┌──────────────────┐ │ Employee Information                             │
│ │ ┌─────────────┐  │ │                                                  │
│ │ │ Full Name   │  │ │ Full Name *                                      │
│ │ └─────────────┘  │ │ [John Doe____________________________________]  │
│ │                  │ │                                                  │
│ │ ┌────────┐       │ │ Social Security Number *                         │
│ │ │ SSN    │       │ │ [123-45-6789_____________________________]      │
│ │ └────────┘       │ │                                                  │
│ │                  │ │ Address *                                        │
│ │ ┌──────────────┐ │ │ [123 Main St________________________________]   │
│ │ │ Wages        │ │ │                                                  │
│ │ └──────────────┘ │ │ Wages *                                          │
│ └──────────────────┘ │ $ [50,000.00]                                    │
│                      │                                                  │
│                      │ Tax Withheld                                     │
│                      │ $ [8,500.00_]                                    │
│                      │                                                  │
│                      │ Signature ✓ (Signed)                             │
│                      │                                                  │
│                      │ ────────────────────────────────────────────────│
│                      │ Validation: ✓ All required fields complete       │
│                      │ [Previous Page]              [Next Page]         │
│                      │                    [Submit]                      │
└──────────────────────┴──────────────────────────────────────────────────┘
```

---

## Appendix B: Action Enum Summary

### ManagerAction (Template Manager)
```rust
pub enum ManagerAction {
    New,                // Create new template
    Edit(String),       // Edit template by ID
    Duplicate(String),  // Duplicate template by ID
    Delete(String),     // Delete template by ID
    Import,             // Import template from file
    Export(String),     // Export template by ID
    None,               // No action
}
```

### EditorAction (Template Editor)
```rust
pub enum EditorAction {
    Save,    // Save template and exit
    Cancel,  // Cancel editing and exit
    None,    // No action (continue editing)
}
```

### PropertiesAction (Field Properties Panel)
```rust
pub enum PropertiesAction {
    Applied,   // Apply changes to field
    Cancelled, // Cancel changes
    Delete,    // Delete field
}
```

### DataEntryAction (Instance Data Entry)
```rust
pub enum DataEntryAction {
    SaveDraft, // Save incomplete instance
    Submit,    // Save complete instance and exit
    Cancel,    // Cancel and exit
    None,      // No action (continue editing)
}
```

---

## Next Steps

1. **Review this roadmap** - Get stakeholder feedback
2. **Prioritize phases** - Confirm Phase 1-4 order or adjust
3. **Update PLANNING_INDEX.md** - Track this document
4. **Begin Phase 1** - Start with AppState and mode management
5. **Iterate weekly** - Review progress, adjust plan as needed

**Critical Questions:**
1. Is the 8-week timeline realistic?
2. Should any features move between phases?
3. Are there additional UI requirements not captured here?
4. What's the priority: speed to market vs. polish?
5. Should we implement Phase 1-2 (MVP) first, then evaluate before Phase 3-4?
