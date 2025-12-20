# UI Roadmap: Overlay-Based Simplified Architecture

**Status:** Planning Phase 📋
**Created:** 2024-12-20
**Goal:** Complete Form Factor UI with canvas overlays instead of complex mode switching

---

## Executive Summary

**New Approach:** Keep the main canvas as the central workspace. Use **overlays and sidebars** for all workflows instead of full-screen mode switches.

**Philosophy:**
- Canvas is always visible (provides context)
- Overlays for temporary workflows (fill form, browse templates)
- Plugins/sidebars for persistent tools (layers, properties)
- No complex mode state machine

**Current State:** 80% complete
- ✅ Canvas with template layer (draw fields, edit fields, save templates)
- ✅ Properties sidebar (edit selected shapes/fields)
- ✅ Layer management (show/hide templates, instances, shapes)
- ✅ Template backend (registry, validation, multi-page)
- ❌ **"Add to Template" workflow (CRITICAL GAP)**
- ❌ Template browser overlay (quick access to templates)
- ❌ Instance filling overlay (data entry panel)
- ❌ Workflow integration (buttons, transitions)

**Critical Discovery:** The template creation workflow is incomplete. Users can run detections and see boxes, but have **no way to convert detections or shapes into template fields** through the UI. This is the highest priority fix.

---

## Architecture: Canvas + Overlays

### Core Principle
**One canvas, multiple overlays**

```
┌─────────────────────────────────────────────────────────┐
│ Top Bar: [File] [Edit] [View] [Tools]                  │
├────────┬────────────────────────────────────────┬───────┤
│ Left   │                                        │ Right │
│ Tool   │         Main Canvas                    │ Plugin│
│ bar    │         (always visible)               │ Panel │
│        │                                        │       │
│ [Rect] │  ┌──────────────────────────────────┐ │ Layers│
│ [Circ] │  │                                  │ │ Props │
│ [Free] │  │  Form content with overlays      │ │ Files │
│ [Edit] │  │                                  │ │       │
│        │  └──────────────────────────────────┘ │       │
└────────┴────────────────────────────────────────┴───────┘
```

### Overlay Types

1. **Modal Overlays** (center screen, blocks canvas interaction)
   - Template Browser
   - Instance Browser
   - Settings dialog

2. **Side Overlays** (slide in from right, semi-transparent backdrop)
   - Data Entry Panel (instance filling)
   - Template Validation Results
   - Export/Import wizard

3. **Inline Overlays** (hover/contextual, no backdrop)
   - Field type picker (when drawing new field)
   - Quick actions menu (right-click on field)

---

## Workflow: Template Creation

**Critical Feature:** Case-by-case field addition from detections and shapes

### Complete User Flow

1. **Load form image** ✅ Works
   - User: File → Open
   - Canvas shows form image

2. **Run detections** ✅ Works
   - User: DetectionPlugin → "Detect Logos" / "Detect Text" / "Extract OCR"
   - Detection boxes appear on canvas

3. **Start template creation** ✅ Works
   - User: Layers panel → Enable "Template" layer
   - Or: "Start New Template" button (to be added)
   - Canvas enters template editing mode

4. **Select and add detection to template** ❌ **MISSING** (Phase 2, Task 2.1-2.2)
   - User: Click on logo detection box
   - Properties panel shows detection metadata
   - User: Edit label to "company_logo"
   - User: Select field type "Logo"
   - User: Click **"Add to Template"** button
   - **Result:** Detection converted to template field

5. **Draw new field** ✅ Works
   - User: Select Rectangle tool
   - User: Draw box over name area
   - Field auto-created when Template layer active

6. **Add canvas shape to template** ❌ **MISSING** (Phase 2, Task 2.2)
   - Alternative flow: Draw on Canvas layer first
   - User: Select shape
   - Properties panel shows shape properties
   - User: Click **"Add to Template"** button
   - **Result:** Shape converted to template field

7. **Edit field properties** ✅ Works
   - User: Select field (detection-based or drawn)
   - Properties panel shows field editor
   - User: Edit name, type, validation, required flag
   - Changes saved automatically

8. **Multi-page support** ✅ Works (needs UI integration)
   - User: Page navigation controls
   - Add/remove pages
   - Fields associated with specific pages

9. **Save template** ✅ Works (needs dialog)
   - User: "Save Template" button
   - Dialog prompts for name/description
   - Template saved to registry
   - Validation runs before save

### Visual Indicators (Phase 2, Task 2.3)

**Template fields:**
- Green highlight (not blue like regular shapes)
- Checkmark badge ✓
- Included in template field count

**Regular shapes/detections:**
- Blue highlight
- No badge
- Not included in template

**Template Status Widget:**
```
┌─────────────────────────┐
│ Template: W-2 Form      │
│ Fields: 12              │  ← Updates live
│ Pages: 2                │
│ [Save Template]         │
└─────────────────────────┘
```

### Enhancement Needed (Phase 2)

**Current limitations:**
- ✅ Shapes auto-convert when Template layer active (works but inflexible)
- ❌ **No "Add to Template" button** (critical gap)
- ❌ No visual distinction between template fields and shapes
- ❌ No template field count display
- ❌ No "Remove from Template" button

---

## Workflow: Template Browser Overlay

**New Component:** Template Browser Overlay (modal)

### User Flow
1. User clicks "Browse Templates" button (in toolbar or File menu)
2. **Overlay appears** over canvas (semi-transparent backdrop)
3. User sees template grid/list with thumbnails
4. User can:
   - **Load for editing**: Click "Edit" → loads template into canvas Template layer
   - **Create instance**: Click "Fill Form" → opens Data Entry overlay
   - **Delete/Duplicate**: Quick actions on each template
   - **Search/Filter**: By name, tags, field count
5. User clicks outside or "Close" → overlay disappears

### UI Design
```
┌─────────────────────────────────────────────────────────┐
│                   🗂️ Template Library                     │
├─────────────────────────────────────────────────────────┤
│ Search: [_________________]  [New Template] [Import]    │
├─────────────────────────────────────────────────────────┤
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │
│ │📄 W-2 Form   │ │📄 1099-MISC  │ │📄 Invoice    │    │
│ │ 2 pages      │ │ 1 page       │ │ 1 page       │    │
│ │ 18 fields    │ │ 12 fields    │ │ 8 fields     │    │
│ │              │ │              │ │              │    │
│ │[Edit][Fill]  │ │[Edit][Fill]  │ │[Edit][Fill]  │    │
│ └──────────────┘ └──────────────┘ └──────────────┘    │
│                                                         │
│                    [Close]                              │
└─────────────────────────────────────────────────────────┘
```

### Implementation
- **File:** `crates/form_factor/src/overlays/template_browser.rs`
- **State:** `TemplateBrowserOverlay` struct
- **Integration:** Add to main app's overlay stack
- **Rendering:** egui Window with `modal: true`

---

## Workflow: Instance Filling Overlay

**New Component:** Data Entry Side Panel (slide-in)

### User Flow
1. User clicks "Fill Form" from:
   - Template Browser overlay, OR
   - Template menu → "Fill Instance"
2. **Side panel slides in from right** (covers plugin sidebar)
3. Canvas shows template fields as visual guide (read-only)
4. User fills data in panel:
   - Form-style inputs (text, date, checkbox)
   - Field-by-field with labels
   - Real-time validation feedback
   - Progress indicator (e.g., "8 of 12 fields complete")
5. User clicks "Submit" → instance saved, panel slides out
6. Or "Cancel" → confirms discard, panel slides out

### UI Design
```
┌─────────────────────────────────────────────────────────┐
│                                         ┌───────────────┤
│                                         │ 📝 Fill Form  │
│         Canvas shows                    ├───────────────┤
│         template fields                 │ W-2 Form 2024 │
│         (grayed out, read-only)         │ Page 1 of 2   │
│                                         │ Progress: 67% │
│                                         ├───────────────┤
│                                         │ Employee Name*│
│                                         │ [John Doe___] │
│                                         │               │
│                                         │ SSN*          │
│                                         │ [___-__-____] │
│                                         │               │
│                                         │ Wages*        │
│                                         │ $[50,000.00]  │
│                                         │               │
│                                         │ [Prev] [Next] │
│                                         │ [Save Draft]  │
│                                         │ [Submit]      │
│                                         │ [Cancel]      │
└─────────────────────────────────────────┴───────────────┘
```

### Implementation
- **File:** `crates/form_factor/src/overlays/data_entry.rs`
- **State:** `DataEntryOverlay` struct
- **Integration:** Add to main app's overlay stack
- **Rendering:** egui SidePanel (right side, 400px width)
- **Canvas Integration:** 
  - Lock canvas editing when panel open
  - Show template fields as visual guide
  - Highlight current field being edited

---

## Implementation Plan

### Phase 1: Foundation (1-2 days)

**Goal:** Overlay infrastructure

#### Task 1.1: Overlay Manager
- Create `overlay_manager.rs` module
- `OverlayStack` struct manages active overlays
- `Overlay` trait with `show()`, `is_modal()`, `wants_close()`
- z-order management (modals block lower overlays)

#### Task 1.2: Backdrop Rendering
- Semi-transparent backdrop for modal overlays
- Click-outside-to-close for modals
- ESC key to close top overlay

**Files:**
- `crates/form_factor/src/overlays/mod.rs` (new)
- `crates/form_factor/src/overlays/manager.rs` (new)

---

### Phase 2: Template Creation & Browser (4-6 days)

**Goal:** Complete template creation workflow and browser overlay

#### Task 2.1: "Add to Template" Backend (1-2 days)

**Critical Missing Feature:** Users cannot convert detections or shapes into template fields.

##### Subtask 2.1.1: Detection → Field Conversion
```rust
// In crates/form_factor_drawing/src/canvas/field_creator.rs

impl FieldCreator {
    /// Creates a field from a detection
    pub fn create_field_from_detection(
        &mut self,
        detection: &DetectionMetadata,
        detection_bounds: (f32, f32, f32, f32), // x, y, w, h
    ) -> FieldCreatorResult<FieldDefinition> {
        // Map FormFieldType → FieldType
        let field_type = match detection.form_field_type() {
            Some(FormFieldType::Text) => FieldType::FreeText,
            Some(FormFieldType::Date) => FieldType::Date,
            Some(FormFieldType::Number) => FieldType::Currency,
            Some(FormFieldType::Signature) => FieldType::Signature,
            None => FieldType::FreeText,
        };
        
        // Use detection label or generate name
        let field_id = detection.label()
            .clone()
            .unwrap_or_else(|| {
                self.field_counter += 1;
                format!("field_{}", self.field_counter)
            });
        
        // Build field definition
        FieldDefinitionBuilder::default()
            .id(field_id.clone())
            .label(field_id)
            .field_type(field_type)
            .bounds(FieldBounds::new(
                detection_bounds.0,
                detection_bounds.1,
                detection_bounds.2,
                detection_bounds.3
            ))
            .page_index(0) // TODO: Use current page
            .required(false)
            .build()
            .map_err(|e| FieldCreatorError::new(
                FieldCreatorErrorKind::BuilderError(e.to_string())
            ))
    }
}
```

##### Subtask 2.1.2: Add Field to Template Method
```rust
// In crates/form_factor_drawing/src/canvas/core.rs

impl DrawingCanvas {
    /// Adds a field to the current template
    pub fn add_field_to_template(
        &mut self,
        field: FieldDefinition,
    ) -> Result<(), TemplateError> {
        if let Some(template) = &mut self.current_template {
            let page_index = self.current_page;
            
            // Ensure page exists
            while template.pages.len() <= page_index {
                template.pages.push(TemplatePage::default());
            }
            
            // Add field to current page
            template.pages[page_index].fields.push(field);
            
            debug!(
                page = page_index,
                field_count = template.pages[page_index].fields.len(),
                "Added field to template"
            );
            
            Ok(())
        } else {
            Err(TemplateError::new(TemplateErrorKind::NoActiveTemplate))
        }
    }
    
    /// Removes a field from the current template
    pub fn remove_field_from_template(
        &mut self,
        field_id: &str,
    ) -> Result<(), TemplateError> {
        if let Some(template) = &mut self.current_template {
            for page in &mut template.pages {
                page.fields.retain(|f| f.id() != field_id);
            }
            debug!(field_id, "Removed field from template");
            Ok(())
        } else {
            Err(TemplateError::new(TemplateErrorKind::NoActiveTemplate))
        }
    }
}
```

##### Subtask 2.1.3: New AppEvents
```rust
// In crates/form_factor_plugins/src/event.rs

pub enum AppEvent {
    // ... existing events
    
    /// User wants to add detection to template
    AddDetectionToTemplate {
        detection_id: String,
    },
    
    /// User wants to add shape to template
    AddShapeToTemplate {
        shape_id: usize,
    },
    
    /// Field was successfully added to template
    FieldAddedToTemplate {
        field_id: String,
    },
    
    /// User wants to remove field from template
    RemoveFieldFromTemplate {
        field_id: String,
    },
}
```

**Files Modified:**
- `crates/form_factor_drawing/src/canvas/field_creator.rs`
- `crates/form_factor_drawing/src/canvas/core.rs`
- `crates/form_factor_plugins/src/event.rs`

---

#### Task 2.2: "Add to Template" UI Integration (1-2 days)

##### Subtask 2.2.1: Detection Properties Panel Button
```rust
// In crates/form_factor_plugins/src/detection_properties.rs

impl DetectionPropertiesPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, bus: &EventBus) -> Option<DetectionAction> {
        // ... existing UI code for label, field type, etc.
        
        ui.separator();
        
        // Add to Template button (only if template is active)
        if self.template_active {
            if ui.button("➕ Add to Template").clicked() {
                bus.emit(AppEvent::AddDetectionToTemplate {
                    detection_id: self.metadata.id().clone(),
                });
                return Some(DetectionAction::AddedToTemplate);
            }
        } else {
            ui.label("Start template creation to add fields");
        }
        
        None
    }
}

pub enum DetectionAction {
    AddedToTemplate,
    PropertiesChanged,
    Removed,
}
```

##### Subtask 2.2.2: Shape Properties Panel Button
```rust
// In crates/form_factor_plugins/src/properties.rs

fn render_shape_properties(
    ui: &mut Ui,
    shape_id: usize,
    template_active: bool,
    bus: &EventBus,
    // ... other params
) {
    // ... existing shape properties
    
    ui.separator();
    
    if template_active {
        if ui.button("➕ Add to Template").clicked() {
            bus.emit(AppEvent::AddShapeToTemplate { shape_id });
        }
    } else {
        ui.label("Start template creation to add fields");
    }
}
```

##### Subtask 2.2.3: Main App Event Handlers
```rust
// In crates/form_factor/src/event_handlers/objects.rs (or new template_handler.rs)

impl FormFactorApp {
    fn handle_add_detection_to_template(&mut self, detection_id: String) -> Result<()> {
        let canvas = self.canvas.as_mut().ok_or(AppError::NoCanvas)?;
        
        // Get detection metadata and bounds
        let detection = self.get_detection_metadata(&detection_id)?;
        let bounds = self.get_detection_bounds(&detection_id)?;
        
        // Convert to field
        let field = self.field_creator.create_field_from_detection(&detection, bounds)?;
        let field_id = field.id().to_string();
        
        // Add to template
        canvas.add_field_to_template(field)?;
        
        // Visual feedback
        self.show_toast(format!("Field '{}' added to template", field_id));
        self.event_bus.emit(AppEvent::FieldAddedToTemplate { field_id });
        
        Ok(())
    }
    
    fn handle_add_shape_to_template(&mut self, shape_id: usize) -> Result<()> {
        let canvas = self.canvas.as_mut().ok_or(AppError::NoCanvas)?;
        
        // Get shape
        let shape = canvas.get_shape(shape_id)?;
        
        // Convert to field using existing FieldCreator
        let field = self.field_creator.create_field(&shape, None, None)?;
        let field_id = field.id().to_string();
        
        // Add to template
        canvas.add_field_to_template(field)?;
        
        // Visual feedback
        self.show_toast(format!("Shape added to template as '{}'", field_id));
        self.event_bus.emit(AppEvent::FieldAddedToTemplate { field_id });
        
        Ok(())
    }
}
```

**Files Modified:**
- `crates/form_factor_plugins/src/detection_properties.rs`
- `crates/form_factor_plugins/src/properties.rs`
- `crates/form_factor/src/event_handlers/template.rs` (new)
- `crates/form_factor/src/main.rs` (wire event handlers)

---

#### Task 2.3: Visual Feedback & Template Status (0.5-1 day)

##### Subtask 2.3.1: Template Field Count Display
Add to layers panel or create dedicated template status widget:
```rust
ui.group(|ui| {
    ui.heading("Template Status");
    if let Some(template) = &canvas.current_template() {
        ui.label(format!("Name: {}", template.name));
        let field_count: usize = template.pages.iter()
            .map(|p| p.fields.len())
            .sum();
        ui.label(format!("Fields: {}", field_count));
        ui.label(format!("Pages: {}", template.pages.len()));
    } else {
        ui.label("No active template");
        if ui.button("Start New Template").clicked() {
            // Open template naming dialog
        }
    }
});
```

##### Subtask 2.3.2: Visual Field Highlighting
Differentiate template fields from regular shapes:
```rust
// In canvas rendering
fn render_field_overlay(&self, painter: &Painter, field: &FieldDefinition, in_template: bool) {
    let color = if in_template {
        Color32::from_rgb(0, 200, 0) // Green for template fields
    } else {
        Color32::from_rgb(0, 150, 255) // Blue for regular shapes
    };
    
    let badge = if in_template { "✓" } else { "" };
    
    // Render with appropriate color and badge
}
```

##### Subtask 2.3.3: Toast Notifications
```rust
// Add egui_toast dependency to Cargo.toml
// In FormFactorApp
fn show_toast(&mut self, message: String) {
    self.toasts.success(message);
}
```

**Files Modified:**
- `crates/form_factor_plugins/src/layers.rs` (add template status)
- `crates/form_factor_drawing/src/canvas/rendering.rs` (visual highlighting)
- `crates/form_factor/Cargo.toml` (add egui_toast)
- `crates/form_factor/src/main.rs` (toast manager)

---

#### Task 2.4: Template Browser Overlay (2-3 days)

**Goal:** Replace TemplateManagerPanel with overlay

##### Subtask 2.4.1: Template Browser UI
- Port `TemplateManagerPanel` logic to overlay format
- Grid/list view of templates
- Thumbnail generation (first page preview)
- Search and filtering

##### Subtask 2.4.2: Actions Integration
- "Edit" button → loads template to canvas Template layer
- "Fill Form" button → opens Data Entry overlay
- "Delete" button → confirmation dialog, removes from registry
- "Duplicate" button → clones template with new ID

##### Subtask 2.4.3: Entry Points
- Add "Browse Templates" button to:
  - Top menu bar (File → Templates)
  - Plugin sidebar (new TemplatesPlugin)
  - Keyboard shortcut (Ctrl+T)

**Files:**
- `crates/form_factor/src/overlays/template_browser.rs` (new)
- `crates/form_factor_plugins/src/templates.rs` (new plugin)

---

## Phase 3: Data Entry Overlay (3-4 days) ⏭️ NEXT

**Goal:** Complete instance filling workflow

**Status**: Ready to start

### Overview

Port the DataEntryPanel to a slide-in overlay that appears when user clicks "Fill Form" in template browser. The overlay shows all template fields in a scrollable list with appropriate input widgets for each field type.

### Tasks

#### Task 3.1: Data Entry Overlay UI (2 days)

**Goal:** Create slide-in overlay for filling out template instances

##### Subtask 3.1.1: Basic Overlay Structure
- Create `DataEntryOverlay` struct
- Slide-in from right side (not full-screen modal)
- Header: Template name, close button
- Body: Scrollable field list
- Footer: Save/Cancel buttons

##### Subtask 3.1.2: Field Input Widgets
Port field rendering from DataEntryPanel:
- **Text fields**: `TextEdit::singleline()`
- **Date fields**: `DatePickerButton` (egui_extras)
- **Currency fields**: `DragValue` with "$" prefix
- **Boolean fields**: `Checkbox`
- **Signature fields**: Placeholder button (future: drawing widget)

##### Subtask 3.1.3: Field Navigation
- Auto-focus on first field
- Tab/Enter to next field
- Visual indicator for current field
- Progress bar: "Field 3 of 12"

**Files:**
- `crates/form_factor/src/overlays/data_entry.rs` (new)
- Port logic from `crates/form_factor_plugins/src/data_entry_panel.rs`

---

#### Task 3.2: Instance Management (1-2 days)

**Goal:** Create and save form instances

##### Subtask 3.2.1: Instance Creation
- User clicks "Fill Form" in template browser
- Create new `FormInstance` from template
- Open DataEntryOverlay with empty instance
- As user fills fields, update instance in real-time

##### Subtask 3.2.2: Save/Export
- Save button validates all required fields
- Save to instance registry (JSON file)
- Export to PDF/CSV (future enhancement)
- Toast notification: "Instance saved successfully"

##### Subtask 3.2.3: Entry Points
- "Fill Form" button in template browser → opens overlay
- "Fill Template" button in Templates plugin (new)
- Ctrl+F keyboard shortcut (future)

**Files Modified:**
- `crates/form_factor/src/overlays/template_browser.rs` - Wire Fill action
- `crates/form_factor_plugins/src/templates.rs` - Add Fill button
- `crates/form_factor/src/main.rs` - Event handling

**New Events:**
- `AppEvent::FillTemplateRequested { template_id }`
- `AppEvent::InstanceSaved { instance_id }`

---

- Page navigation (Prev/Next)

#### Task 3.2: Validation Integration
- Real-time validation on field blur
- Show error messages inline (red text below input)
- Enable/disable Submit based on validation
- Highlight incomplete required fields

#### Task 3.3: Instance Creation
- Create `DrawingInstance` from template
- Populate instance with user data
- Save to instance registry (new `InstanceRegistry`)
- Handle page navigation (show only current page fields)

#### Task 3.4: Canvas Integration
- Lock canvas editing when panel open
- Show template fields as visual guide
- Highlight current field on canvas while editing in panel
- Gray out completed fields

**Files:**
- `crates/form_factor/src/overlays/data_entry.rs` (new)
- `crates/form_factor_drawing/src/instance/registry.rs` (new)

---

### Phase 4: Polish & Integration (2-3 days)

**Goal:** Complete workflows and UX refinements

#### Task 4.1: Keyboard Shortcuts
- `Ctrl+T`: Open template browser
- `Ctrl+Shift+F`: Fill form (if template loaded)
- `ESC`: Close active overlay
- `Tab/Shift+Tab`: Navigate fields in data entry
- `Ctrl+Enter`: Submit in data entry

#### Task 4.2: Visual Feedback
- Smooth slide-in/out animations for side panel
- Fade-in backdrop for modals
- Loading spinners during save/load
- Success/error toasts (egui_toast)

#### Task 4.3: Instance Browser Overlay
- Similar to Template Browser
- List instances grouped by template
- Actions: View (read-only), Edit, Export, Delete
- Filter by date, completion status

#### Task 4.4: Workflow Integration
- "Save as Template" button in Template layer header
- "Fill Form" option when template layer active
- Recent templates quick access
- Template validation before save

**Files:**
- `crates/form_factor/src/overlays/instance_browser.rs` (new)
- `crates/form_factor/src/shortcuts.rs` (new)

---

## Comparison: Old vs New Approach

### Old Approach (Complex Mode System)

**Pros:**
- Full-screen real estate for each task
- Clear separation of concerns

**Cons:**
- Complex mode state machine (5 modes)
- Lose canvas context during workflows
- Mode transitions require unsaved changes handling
- More code to maintain

### New Approach (Canvas + Overlays)

**Pros:**
- Canvas always visible (context preserved)
- Simpler state management (no modes)
- Familiar pattern (like Blender, Photoshop)
- Less code, easier to maintain
- Overlays are optional/dismissible

**Cons:**
- Less screen space for data entry
- Need careful z-order management

---

## Success Criteria

### Must Have (MVP)
- ✅ User can create template on canvas (already works)
- ✅ User can save template to library (already works)
- ✅ **User can select detection → click "Add to Template" → field added**
- ✅ **User can select shape → click "Add to Template" → field added**
- ✅ **Visual distinction: template fields show green, shapes show blue**
- ✅ **Template field count updates in real-time**
- ✅ User can browse templates in overlay
- ✅ User can load template for editing
- ✅ User can fill instance via data entry overlay
- ✅ User can save instance
- ✅ Instance validation prevents bad data

### Should Have (Polish)
- ✅ Keyboard shortcuts for common actions
- ✅ Visual feedback (animations, toasts)
- ✅ Instance browser to view/edit saved instances
- ✅ Template thumbnails in browser
- ✅ Search/filter in browsers
- ✅ "Remove from Template" button
- ✅ Checkmark badges on fields already in template

### Nice to Have (Future)
- Multi-window support (separate windows for templates)
- Template versioning and migration
- Export instances to PDF/CSV
- Auto-fill from OCR
- Template inheritance

---

## Implementation Timeline

**Total Estimate:** 10-15 days (2-3 weeks)

- Phase 1: Overlay Foundation (1-2 days)
- **Phase 2: Template Creation & Browser (4-6 days)** ← Includes "Add to Template" workflow
  - Task 2.1: Backend (1-2 days)
  - Task 2.2: UI Integration (1-2 days)
  - Task 2.3: Visual Feedback (0.5-1 day)
  - Task 2.4: Template Browser (2-3 days)
- Phase 3: Data Entry (3-4 days)
- Phase 4: Polish (2-3 days)

**Advantages over old plan:**
- 50% less code than mode-switching approach
- Simpler architecture
- Faster to implement
- Easier to maintain
- **Includes critical "Add to Template" workflow** identified in gap analysis

---

## Technical Details

### Overlay Stack Management

```rust
pub struct OverlayManager {
    overlays: Vec<Box<dyn Overlay>>,
}

pub trait Overlay {
    fn show(&mut self, ctx: &egui::Context) -> OverlayResponse;
    fn is_modal(&self) -> bool;
    fn title(&self) -> &str;
}

pub enum OverlayResponse {
    KeepOpen,
    Close,
    OpenAnother(Box<dyn Overlay>),
}

impl OverlayManager {
    pub fn push(&mut self, overlay: Box<dyn Overlay>) {
        self.overlays.push(overlay);
    }
    
    pub fn show(&mut self, ctx: &egui::Context) {
        // Render backdrop for modals
        if self.top().map(|o| o.is_modal()).unwrap_or(false) {
            self.render_backdrop(ctx);
        }
        
        // Show overlays bottom-to-top
        self.overlays.retain_mut(|overlay| {
            matches!(overlay.show(ctx), OverlayResponse::KeepOpen)
        });
        
        // Handle ESC key for top overlay
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.overlays.pop();
        }
    }
}
```

### Canvas Lock During Data Entry

```rust
// In DrawingCanvas
pub fn lock_editing(&mut self, reason: &str) {
    self.editing_locked = true;
    self.lock_reason = reason.to_string();
}

pub fn unlock_editing(&mut self) {
    self.editing_locked = false;
}

pub fn is_editing_locked(&self) -> bool {
    self.editing_locked
}

// In data_entry overlay
impl DataEntryOverlay {
    pub fn show(&mut self, ctx: &egui::Context, canvas: &mut DrawingCanvas) -> OverlayResponse {
        // Lock canvas when panel opens
        canvas.lock_editing("Instance data entry in progress");
        
        // Show side panel
        egui::SidePanel::right("data_entry_panel")
            .default_width(400.0)
            .show(ctx, |ui| {
                // ... render form fields
            });
        
        // If closing, unlock canvas
        if should_close {
            canvas.unlock_editing();
            return OverlayResponse::Close;
        }
        
        OverlayResponse::KeepOpen
    }
}
```

---

## Migration from Old Plan

### Reuse Existing Code

**Keep:**
- ✅ `DataEntryPanel` logic → adapt to overlay
- ✅ `TemplateManagerPanel` logic → adapt to overlay
- ✅ Field validation logic
- ✅ Template/instance data structures

**Remove:**
- ❌ `AppMode` enum (no longer needed)
- ❌ `AppState` mode transitions
- ❌ `ModeSwitcher` component
- ❌ Mode-specific layouts

**Simplify:**
- `TemplateEditorPanel` → not needed (canvas Template layer handles this)
- `FieldPropertiesPanel` → already integrated in Properties plugin

### Migration Steps

1. Create overlay infrastructure (Phase 1)
2. Port TemplateManagerPanel to TemplateBrowserOverlay (Phase 2)
3. Port DataEntryPanel to DataEntryOverlay (Phase 3)
4. Remove old mode system code (Phase 4)
5. Update documentation

---

## Questions to Validate

1. **Is 400px wide enough for data entry panel?**
   - If not, make it resizable or configurable

2. **Should template browser be modal or modeless?**
   - Modal = blocks canvas (recommended)
   - Modeless = can click canvas while open

3. **Where should instances be saved?**
   - Same registry as templates? (simpler)
   - Separate InstanceRegistry? (cleaner separation) ← **Recommended**

4. **What happens to unsaved template when user opens browser?**
   - Prompt to save? (annoying)
   - Auto-save draft? (hidden complexity)
   - Allow multiple drafts? (most flexible) ← **Recommended**

5. **Should "Add to Template" convert detection immediately?**
   - Yes, add to template with current properties ← **Recommended**
   - Alternative: Open field editor first (extra click, but more control)

6. **How to handle detection → field type mapping?**
   - Use current FormFieldType from detection metadata
   - Or: Always prompt user to select from 40+ FieldType options
   - Recommendation: Use detection's type, allow editing after

---

## Gap Analysis Integration

This roadmap incorporates critical findings from the template creation workflow analysis:

### Original Gap Identified
**Problem:** No way to convert detections or shapes into template fields through UI.

**What was missing:**
- "Add to Template" button in Detection Properties panel
- "Add to Template" button in Shape Properties panel
- Backend: `create_field_from_detection()` method
- Backend: `add_field_to_template()` method
- Event handlers: `AddDetectionToTemplate`, `AddShapeToTemplate`
- Visual feedback: field count, green highlighting, toasts

### How This Plan Addresses It
All missing features integrated into **Phase 2: Template Creation & Browser**:
- **Task 2.1:** Backend implementation (detection → field conversion, add/remove field methods)
- **Task 2.2:** UI integration (buttons, event handlers)
- **Task 2.3:** Visual feedback (highlighting, field count, toasts)
- **Task 2.4:** Template browser overlay

**Timeline Impact:** +2-3 days to Phase 2 (now 4-6 days instead of 2-3 days)

### User Stories Now Supported
1. ✅ Load form → detect logos → click logo → "Add to Template" → save
2. ✅ Draw shape on Canvas → select → "Add to Template" → edit properties → save
3. ✅ OCR text → select region → label → "Add to Template" → save
4. ✅ Visual feedback shows which items are in template (green vs. blue)
5. ✅ Template field count updates in real-time

---

## Next Steps

1. **Review this plan** - Confirm approach before implementation
2. **Decide on instance storage** - Same registry or separate?
3. **Start Phase 1** - Build overlay infrastructure
4. **Iterate weekly** - Review progress, adjust as needed

---

**This plan is 50% smaller than the original and achieves the same user goals with less complexity.**
