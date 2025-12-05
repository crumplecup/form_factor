# Form Template and Instance System - Strategic Plan

## Executive Summary

This document tracks the implementation of a comprehensive template and instance system for structured forms in the form_factor project. The system provides trait-based abstractions for defining form templates and managing filled form instances with multi-page support.

**Status**: Phase 6 Complete - Full system with OCR, Canvas integration, and Migration
**Last Updated**: 2024-12-05

---

## Architecture Overview

### Core Design

```
form_factor_core (traits)
    ├── FormTemplate trait
    ├── FormInstance trait
    ├── FieldType enum (30+ types)
    ├── FieldDefinition
    ├── FieldValue
    └── ValidationResult

form_factor_drawing (implementations)
    ├── DrawingTemplate (implements FormTemplate)
    ├── DrawingInstance (implements FormInstance)
    ├── FormPage (wraps DrawingCanvas)
    ├── TemplateRegistry (storage)
    └── Builders (fluent API)
```

### Key Concepts

- **Template**: Defines form structure, field types, locations, and validation rules
- **Instance**: A filled form based on a template, with actual field values
- **Multi-Page**: Forms can span multiple pages, each with its own canvas
- **Field Types**: 30+ semantic types (SSN, Email, Date, etc.) with validation patterns
- **Registry**: Global and project-local template storage

---

## Implementation Status

### ✅ Phase 1: Core Traits (COMPLETED)

**Commit**: 3a0e5ba

**Implemented**:
- `FormTemplate` trait in `form_factor_core/src/template.rs`
  - Methods: `id()`, `name()`, `version()`, `page_count()`, `fields()`, `validate_instance()`
- `FormInstance` trait in `form_factor_core/src/instance.rs`
  - Methods: `template_id()`, `page_count()`, `field_values()`, `set_field_value()`
- `FieldType` enum with 30+ types:
  - Personal: FirstName, LastName, FullName, Email, PhoneNumber, DateOfBirth
  - Address: StreetAddress, City, State, ZipCode, Country
  - IDs: SSN, TaxId, DriverLicense, PassportNumber
  - Financial: AccountNumber, RoutingNumber, Currency, Amount
  - Employment: EmployerName, JobTitle, EmployeeId
  - Controls: Checkbox, RadioButton, Signature, Initials
  - Company: CompanyName, CompanyAddress, Logo
  - Generic: TextRegion, NumericField, FreeText, Barcode, QRCode
  - Dates: Date, DateSigned
  - Custom: User-defined types
- `FieldDefinition` with builder pattern
- `FieldBounds` for position/size in image pixels
- `FieldValue` for instance field data
- `FieldContent` enum (Text, Boolean, Number, Signature, Logo, Empty)
- `ValidationResult` with error tracking

**Files Created**:
- `crates/form_factor_core/src/template.rs` (634 lines)
- `crates/form_factor_core/src/instance.rs` (435 lines)

### ✅ Phase 2: Template Implementation (COMPLETED)

**Commit**: 3a0e5ba

**Implemented**:
- `DrawingTemplate` implementing `FormTemplate`
- `TemplatePage` for multi-page templates
- `DrawingTemplateBuilder` with fluent API
- `TemplatePageBuilder` for page construction
- `TemplateRegistry` for storage
  - Global location: `~/.config/form_factor/templates/`
  - Project location: `<project_dir>/templates/`
  - Project templates override global
- Template validation (duplicate IDs, structure)
- Instance validation (required fields, patterns, types)
- JSON serialization

**Files Created**:
- `crates/form_factor_drawing/src/template/error.rs` (94 lines)
- `crates/form_factor_drawing/src/template/implementation.rs` (382 lines)
- `crates/form_factor_drawing/src/template/registry.rs` (277 lines)
- `crates/form_factor_drawing/src/template/mod.rs` (13 lines)

**Dependencies Added**:
- `regex = "1.11"` to form_factor_drawing
- `dirs = "5.0"` to form_factor_drawing

### ✅ Phase 3: Instance Implementation (COMPLETED)

**Commit**: 3a0e5ba

**Implemented**:
- `DrawingInstance` implementing `FormInstance`
- `FormPage` wrapper around `DrawingCanvas`
  - Each page has its own canvas, shapes, detections, image
- Multi-page support (1 to N pages)
- Field value tracking with metadata:
  - OCR confidence scores
  - Verification flags
  - Page index
- JSON serialization for persistence
- Metadata storage (arbitrary key-value pairs)

**Files Created**:
- `crates/form_factor_drawing/src/instance/error.rs` (78 lines)
- `crates/form_factor_drawing/src/instance/implementation.rs` (248 lines)
- `crates/form_factor_drawing/src/instance/mod.rs` (11 lines)

### ✅ Phase 4: Error Handling (COMPLETED)

**Commit**: 3a0e5ba

**Implemented**:
- `FormError` crate-level error type
- `FormErrorKind` wrapping all specific errors:
  - CanvasError
  - ShapeError
  - LayerError
  - TemplateError
  - InstanceError
- Automatic `From` implementations via `derive_more`

**Files Created**:
- `crates/form_factor_drawing/src/error.rs` (91 lines)

### ✅ Phase 5: Comprehensive Testing (COMPLETED)

**Date**: 2024-12-05

**Implemented**:
- **Template Builder Tests** (16 tests): Builder API, page management, validation, JSON serialization
- **Instance Management Tests** (22 tests): Field CRUD, page access, multi-page support, metadata
- **Registry Tests** (17 tests): Global/project storage, template override, persistence
- **Validation Tests** (15 tests): Pattern matching, required fields, type checking, custom patterns
- **Multi-Page Workflow Tests** (3 tests): End-to-end multi-page form workflows
- **Lifecycle Integration Tests** (3 tests): Complete template-instance lifecycle, versioning, JSON roundtrips

**Test Coverage**:
- Total: 77 tests (76 unit/integration + 1 doctest)
- All tests passing ✓
- Zero clippy warnings ✓
- Comprehensive coverage of all core functionality

**Files Created**:
- `crates/form_factor_drawing/tests/template_builder_tests.rs` (16 tests)
- `crates/form_factor_drawing/tests/instance_tests.rs` (22 tests)
- `crates/form_factor_drawing/tests/registry_tests.rs` (17 tests)
- `crates/form_factor_drawing/tests/validation_tests.rs` (15 tests)
- `crates/form_factor_drawing/tests/multi_page_workflow_test.rs` (3 tests)
- `crates/form_factor_drawing/tests/lifecycle_test.rs` (3 tests)

---

## Usage Examples

### Creating a Template

```rust
use form_factor_core::{FieldDefinitionBuilder, FieldType, FieldBounds};
use form_factor_drawing::{DrawingTemplateBuilder, TemplatePage};

// Define a field
let name_field = FieldDefinitionBuilder::default()
    .id("full_name")
    .field_type(FieldType::FullName)
    .label("Full Name")
    .page_index(0)
    .bounds(FieldBounds {
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 30.0,
    })
    .required(true)
    .build()?;

// Create a page
let mut page = TemplatePage::new(0);
page.add_field(name_field);

// Build template
let template = DrawingTemplateBuilder::default()
    .id("employee_form")
    .name("Employee Information Form")
    .version("1.0.0")
    .add_page(page)
    .build()?;

// Save to registry
let mut registry = TemplateRegistry::new()?;
registry.save(&template)?;
```

### Creating an Instance

```rust
use form_factor_core::{FieldValue, FieldBounds};
use form_factor_drawing::DrawingInstance;

// Create instance from template
let mut instance = DrawingInstance::from_template("employee_form", 1);
instance.set_instance_name("John Doe - 2024-01-15");

// Add field value
let bounds = FieldBounds { x: 100.0, y: 200.0, width: 300.0, height: 30.0 };
let value = FieldValue::new_text("full_name", "John Doe", bounds, 0);
instance.set_field_value("full_name", value)?;

// Validate against template
let validation = template.validate_instance(&instance);
if !validation.is_valid() {
    println!("Validation errors: {:?}", validation.field_errors());
}

// Save to JSON
let json = instance.to_json()?;
std::fs::write("instance.json", json)?;
```

### Loading Templates

```rust
use form_factor_drawing::TemplateRegistry;

// Load all templates
let registry = TemplateRegistry::new()?;
let template = registry.get("employee_form").unwrap();

// List available templates
for id in registry.list_templates() {
    println!("Template: {}", id);
}
```

---

## Future Work

### ✅ Priority 1: Essential Testing (COMPLETED)

**Status**: Completed
**Completed Date**: 2024-12-05
**Actual Effort**: 1 day

#### Unit Tests Implemented ✓

1. **Template Builder Tests** (`tests/template_builder_tests.rs`) - 16 tests
   - ✅ Builder fluent API with all parameters
   - ✅ Page addition and management
   - ✅ Metadata handling (multiple entries)
   - ✅ Validation of template structure (duplicate IDs, invalid pages)
   - ✅ Default values (version)
   - ✅ JSON serialization/deserialization
   - ✅ Page dimensions and field lookup

2. **Instance Management Tests** (`tests/instance_tests.rs`) - 22 tests
   - ✅ Field value CRUD operations (create, read, update)
   - ✅ Page access and manipulation (immutable & mutable)
   - ✅ Multi-page workflows with field distribution
   - ✅ Serialization roundtrips
   - ✅ Metadata management
   - ✅ Confidence and verification tracking
   - ✅ Boolean and empty field values
   - ✅ Validation state management

3. **Template Registry Tests** (`tests/registry_tests.rs`) - 17 tests
   - ✅ Save/load from global location
   - ✅ Save/load from project location
   - ✅ Override behavior (project overrides global)
   - ✅ Template listing and deletion
   - ✅ Register/get/contains operations
   - ✅ Error handling (missing project dir)

4. **Validation Tests** (`tests/validation_tests.rs`) - 15 tests
   - ✅ Pattern matching for each FieldType (Email, SSN, Phone, ZIP, Date, State)
   - ✅ Required field checking
   - ✅ Type mismatch detection
   - ✅ Custom validation patterns
   - ✅ Empty field validation
   - ✅ Template ID mismatch
   - ✅ Multiple simultaneous errors

#### Integration Tests Implemented ✓

1. **Multi-Page Workflow** (`tests/multi_page_workflow_test.rs`) - 3 tests
   - ✅ Create template with multiple pages (3-page employee form)
   - ✅ Create instance and fill each page with different field types
   - ✅ Navigate between pages
   - ✅ Validate complete instance
   - ✅ Test incomplete form validation
   - ✅ Field distribution across pages
   - ✅ Page navigation edge cases

2. **Template-Instance Lifecycle** (`tests/lifecycle_test.rs`) - 3 tests
   - ✅ Create and save template to registry
   - ✅ Load template from registry
   - ✅ Create instance from template
   - ✅ Validate and save instance
   - ✅ Load instance from JSON
   - ✅ Template versioning scenarios
   - ✅ Complete JSON roundtrip persistence

### ✅ Priority 2: OCR Integration (COMPLETED)

**Status**: Completed
**Completed Date**: 2024-12-05
**Actual Effort**: < 1 day
**Feature Gate**: `ocr`

#### Requirements Implemented ✓

- ✅ Automatic field extraction from detections
- ✅ Map detected text regions to template fields using IoU overlap
- ✅ Populate field values with OCR results
- ✅ Set confidence scores from OCR
- ✅ Handle ambiguous mappings (highest overlap wins)
- ✅ Support for Rectangle, Circle, and Polygon detection shapes
- ✅ Configurable overlap threshold (30% minimum)
- ✅ Graceful error handling (continues on individual field failures)

#### Implementation Details

**Core Algorithm**:
1. For each field in template, find overlapping detections
2. Calculate IoU (Intersection over Union) overlap scores
3. Select best matching detection (highest overlap > 30%)
4. Extract OCR text from matched region
5. Create `FieldValue` with text and confidence
6. Store field value in instance

**Files Created**:
- `crates/form_factor_drawing/src/instance/extraction.rs` (~370 lines)
  - `extract_fields_from_detections()` method
  - IoU overlap calculation
  - Shape-to-bounds conversion
  - OCR text extraction with error handling
  - 4 unit tests for overlap calculation

**Error Handling**:
- Added `OCRFailed` error variant to `InstanceErrorKind`
- Page index validation
- Per-field error handling (continues with remaining fields)
- Comprehensive tracing instrumentation

**API Example**:
```rust
// Extract fields from detections on page 0
let extracted_count = instance.extract_fields_from_detections(
    &template,
    0,                  // page index
    &detections,        // detection shapes
    "form.png",         // image path for OCR
)?;
println!("Extracted {} fields", extracted_count);
```

**Test Coverage**:
- ✅ IoU overlap calculation (identical, no overlap, partial, contained)
- ⏳ Integration tests with real images (deferred - requires image fixtures)

### ✅ Priority 3: Canvas Integration (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Small (half day)

#### Implemented Features

1. **Template Reference Support**
   - Added `template_id: Option<String>` field to `DrawingCanvas`
   - Added `set_template_id()` method for associating templates with canvas
   - Template ID is serialized with canvas state

2. **Field Overlay Rendering**
   - Implemented `render_field_overlays()` method
   - Renders field bounds as semi-transparent green rectangles
   - Displays field labels on overlay
   - Properly transforms field coordinates (image → canvas → screen)
   - Respects zoom/pan transformations

3. **Snap-to-Field Functionality**
   - Implemented `snap_to_field()` method
   - Snaps drawing positions to nearest field edges
   - Configurable snap threshold (default: 10 pixels)
   - Checks all four edges (top, bottom, left, right)
   - Independent X and Y snapping

#### Files Modified

**`crates/form_factor_drawing/src/canvas/core.rs`**:
- Added `template_id` field to `DrawingCanvas` struct (line 144)
- Added `set_template_id()` method (line 289)
- Added `snap_to_field()` method (line 422-460)
- Imported `FieldDefinition` from `form_factor_core`

**`crates/form_factor_drawing/src/canvas/rendering.rs`**:
- Added `render_field_overlays()` method (line 1130-1193)
- Imported `FieldDefinition` from `form_factor_core`
- Field rendering with proper coordinate transformations
- Visual styling: green semi-transparent overlay with labels

#### API Examples

**Setting Template**:
```rust
canvas.set_template_id(Some("w2_form".to_string()));
```

**Rendering Field Overlays**:
```rust
// In render loop, after form image is drawn
if let Some(template_id) = canvas.template_id() {
    if let Some(template) = registry.get(template_id) {
        let fields = template.fields_for_page(page_index);
        canvas.render_field_overlays(
            fields,
            &painter,
            canvas_rect,
            &transform,
        );
    }
}
```

**Snap-to-Field**:
```rust
// Before passing position to drawing tools
let snapped_pos = canvas.snap_to_field(raw_pos, fields, 10.0);
```

#### Integration Notes

- Higher-level code (UI layer) needs to:
  - Look up template from registry using `template_id`
  - Get fields for current page
  - Pass fields to `render_field_overlays()` and `snap_to_field()`
- Canvas doesn't directly depend on `TemplateRegistry` (loose coupling)
- All coordinate transformations handle zoom/pan correctly
- Field overlays respect the canvas transformation pipeline

#### Testing

All existing tests pass (77 tests total):
- Template builder tests: 16 passed
- Instance tests: 22 passed
- Registry tests: 17 passed
- Validation tests: 15 passed
- Multi-page workflow: 3 passed
- Lifecycle tests: 3 passed
- Doc tests: 1 passed

### ✅ Priority 4: Legacy Migration (COMPLETED)

**Status**: Completed
**Completion Date**: 2024-12-05
**Actual Effort**: Small (half day)

#### Implemented Features

1. **Version Field Addition**
   - Added `version` field to `DrawingCanvas` (default: 1 for legacy)
   - Added `version` field to `DrawingInstance` (default: 2 for multi-page)
   - Version 0 or missing: Legacy format (implicit)
   - Version 1: Legacy format (explicit)
   - Version 2: Multi-page instance format

2. **Automatic Format Detection**
   - Implemented `ProjectFormat::detect_version()` - inspects JSON to determine version
   - Implemented `ProjectFormat::from_json()` - parses as legacy or instance format
   - Supports both legacy (v0/v1) and instance (v2) formats

3. **Legacy-to-Instance Migration**
   - Implemented `migrate_canvas_to_instance()` function
   - Creates single-page `DrawingInstance` from legacy `DrawingCanvas`
   - Uses default template ID: "legacy"
   - Preserves project name as instance name
   - Preserves all shapes, detections, and canvas settings
   - Adds migration metadata (timestamp, source format)

4. **ProjectFormat Enum**
   - Wrapper enum for either Legacy or Instance format
   - `into_instance()` method converts either format to `DrawingInstance`
   - Seamless handling of both formats

#### Files Created

**`crates/form_factor_drawing/src/instance/migration.rs`** (~240 lines):
- `ProjectFormat` enum with version detection
- `detect_version()` - inspects JSON for version field
- `from_json()` - parses legacy or instance format
- `into_instance()` - converts to DrawingInstance
- `migrate_canvas_to_instance()` - performs migration
- 4 unit tests for version detection

#### Files Modified

**`crates/form_factor_drawing/src/canvas/core.rs`**:
- Added `version: u32` field to `DrawingCanvas` struct (line 133)
- Updated `Default` implementation to set version = 1 (line 220)

**`crates/form_factor_drawing/src/instance/implementation.rs`**:
- Added `version: u32` field to `DrawingInstance` struct (line 16-17)
- Added `default_instance_version()` helper function (line 40-42)
- Added `from_single_page()` constructor for migration (line 66-76)

**`crates/form_factor_drawing/src/instance/mod.rs`**:
- Added `pub mod migration` export
- Re-exported migration types at crate level

**`crates/form_factor_drawing/src/lib.rs`**:
- Re-exported `ProjectFormat`, `migrate_canvas_to_instance`, `LEGACY_TEMPLATE_ID`

#### API Examples

**Automatic Migration**:
```rust
// Load any project file (legacy or new)
let json = std::fs::read_to_string("project.json")?;

// Detect format and convert to instance
let format = ProjectFormat::from_json(&json)?;
let instance = format.into_instance();

// Instance is now always DrawingInstance (version 2)
match format {
    ProjectFormat::Legacy(_) => println!("Migrated from legacy format"),
    ProjectFormat::Instance(_) => println!("Loaded as instance format"),
}
```

**Manual Migration**:
```rust
// Manually migrate a canvas
let canvas = DrawingCanvas::new();
let instance = migrate_canvas_to_instance(canvas);

assert_eq!(instance.template_id(), LEGACY_TEMPLATE_ID);
assert_eq!(instance.page_count(), 1);
```

**Version Detection**:
```rust
let json = std::fs::read_to_string("project.json")?;
let version = ProjectFormat::detect_version(&json)?;

match version {
    0 | 1 => println!("Legacy format"),
    2 => println!("Instance format"),
    _ => println!("Unknown version"),
}
```

#### Migration Strategy

1. **On Load**:
   - Try to parse JSON
   - Inspect `version` field
   - Version 0/1 → parse as `DrawingCanvas`
   - Version 2 → parse as `DrawingInstance`

2. **On Save**:
   - Always save as `DrawingInstance` (version 2)
   - Automatically migrated on first save after loading legacy format

3. **Backwards Compatibility**:
   - `#[serde(default)]` ensures old files without `version` work
   - Version defaults to 0 for old files
   - Migration is transparent to user

#### Testing

**New Tests**:
- Created `tests/migration_tests.rs` with 12 comprehensive tests:
  - Version detection (legacy with/without version, instance, invalid)
  - Format parsing (legacy and instance)
  - Migration (preserves data, adds metadata)
  - Roundtrip (legacy → instance → save → load)
  - Error handling (invalid JSON, unsupported versions)

**Test Coverage**:
- Total tests: 93 (77 existing + 4 unit + 12 integration)
- All tests passing with zero clippy warnings
- Migration tests: 16 total (12 integration + 4 unit)

#### Metadata Tracking

Migrated instances include metadata:
- `migrated_from`: "legacy_canvas"
- `migration_date`: Unix timestamp
- `original_project_name`: Preserved project name

This allows UI to show migration status and provide upgrade prompts.

### Priority 5: Template Editor UI

**Status**: Not Started
**Estimated Effort**: Large (1-2 weeks)

#### Requirements

- Visual template creation (not just programmatic)
- Drag-and-drop field placement
- Field property editor
- Template preview

#### Implementation Plan

This would be a significant UI addition and should be planned separately once the core system is proven in use.

---

## Technical Debt

### Known Limitations

1. **No Template Versioning**: Templates have a version string but no migration logic between versions
2. **No Field Dependencies**: Can't express "if field A is checked, require field B"
3. **No Composite Fields**: Can't group related fields (e.g., full address)
4. **No Field Defaults from Template**: FieldDefinition has no default_value field
5. **Limited Validation**: Only regex patterns, no custom validation functions

### Future Enhancements

1. **Template Inheritance**: Allow templates to extend other templates
2. **Conditional Fields**: Show/hide fields based on other field values
3. **Computed Fields**: Auto-calculate field values from other fields
4. **Field Groups**: Logical grouping of related fields
5. **Template Marketplace**: Share templates between users
6. **Version Migration**: Automatic migration between template versions

---

## File Structure

```
crates/
├── form_factor_core/
│   └── src/
│       ├── template.rs          (634 lines) - Core traits
│       ├── instance.rs          (435 lines) - Instance traits
│       └── lib.rs               - Exports
│
└── form_factor_drawing/
    └── src/
        ├── error.rs             (91 lines)  - Crate-level errors
        ├── template/
        │   ├── error.rs         (94 lines)  - Template errors
        │   ├── implementation.rs (382 lines) - Template impl
        │   ├── registry.rs      (277 lines) - Storage
        │   └── mod.rs           (13 lines)  - Module exports
        ├── instance/
        │   ├── error.rs         (78 lines)  - Instance errors
        │   ├── implementation.rs (248 lines) - Instance impl
        │   └── mod.rs           (11 lines)  - Module exports
        └── lib.rs               - Exports
```

**Total Lines of Code**: ~2,297 lines

---

## Verification

### Current Status

- ✅ All 187 tests pass (110 existing + 77 new template/instance tests)
- ✅ No clippy warnings
- ✅ Compiles with all feature combinations
- ✅ All code documented (missing_docs lint enabled)
- ✅ JSON serialization fully tested
- ✅ Template registry fully tested
- ✅ Comprehensive test coverage for all template/instance functionality

### Test Coverage (Updated 2024-12-05)

- **Template Implementation**: 16 dedicated tests (builder, validation, serialization)
- **Instance Implementation**: 22 dedicated tests (CRUD, pages, metadata, validation state)
- **Registry**: 17 dedicated tests (global/project storage, override behavior)
- **Validation**: 15 dedicated tests (patterns, required fields, type checking)
- **Multi-Page Workflows**: 3 integration tests (complete workflows, navigation)
- **Lifecycle**: 3 integration tests (end-to-end, versioning, persistence)
- **Core Traits**: Tested via implementations (all trait methods exercised)

**Total Template/Instance Tests**: 77 tests (76 unit/integration + 1 doctest)
**Status**: Production-ready with comprehensive test coverage

---

## API Stability

### Public API Surface

All types are exported at crate root level:

```rust
// form_factor_core
pub use template::{
    FieldBounds, FieldDefinition, FieldDefinitionBuilder,
    FieldType, FormTemplate
};
pub use instance::{
    FieldContent, FieldValidationError, FieldValue,
    FormInstance, ValidationErrorType, ValidationResult
};

// form_factor_drawing
pub use template::{
    DrawingTemplate, DrawingTemplateBuilder,
    TemplateError, TemplateErrorKind,
    TemplatePage, TemplatePageBuilder, TemplateRegistry
};
pub use instance::{
    DrawingInstance, FormPage,
    InstanceError, InstanceErrorKind
};
pub use error::{FormError, FormErrorKind};
```

### Stability Guarantees

- **Core Traits**: Stable (FormTemplate, FormInstance)
- **Field Types**: Stable (FieldType enum with 30+ types)
- **Builders**: Stable (FieldDefinitionBuilder, DrawingTemplateBuilder)
- **Error Types**: Stable (all error enums and structs)

### Breaking Changes Planned

None currently identified. The API is considered stable for the implemented features.

---

## Dependencies

### Added Dependencies

```toml
# form_factor_core
serde = { workspace = true }  # For serialization

# form_factor_drawing
regex = "1.11"               # For validation patterns
dirs = "5.0"                 # For config directory paths
```

### Dependency Rationale

- **serde**: Required for JSON serialization of templates and instances
- **regex**: Required for field validation patterns (email, SSN, etc.)
- **dirs**: Required for cross-platform config directory location

---

## Performance Considerations

### Current Implementation

- Template validation: O(n) where n = number of fields
- Instance validation: O(n × m) where n = fields, m = pattern complexity
- Registry operations: File I/O bound
- Field lookup: O(n) linear search

### Future Optimizations

- Add field index (HashMap<String, usize>) for O(1) field lookup
- Cache compiled regex patterns
- Lazy-load templates from registry
- Add template versioning with content hashing for cache invalidation

---

## References

### Related Documents

- `CLAUDE.md`: Project coding standards and patterns
- `README.md`: Project overview
- `Cargo.toml`: Workspace dependencies

### Key Commits

- `3a0e5ba`: Add template and instance trait system with multi-page support

### Discussion History

This system was designed based on user requirements:
- Templates represent form structure/layout
- Instances are specific filled forms
- Strong reference model (instance stores template ID)
- Position override (detected positions override template)
- Registry with user override capability
- Full operations support (enumeration, queries, validation)

---

## Contact & Maintenance

**Last Updated**: 2024-12-05
**Next Review**: After Priority 2 (OCR Integration) or when planning Priority 3 (Canvas Integration)
**Maintainer**: Claude (AI Assistant)

**Recent Milestones**:
- 2024-12-05: Completed Priority 4 - Legacy Migration (automatic format migration with 93 total tests)
- 2024-12-05: Completed Priority 3 - Canvas Integration (template overlays, snap-to-field)
- 2024-12-05: Completed Priority 2 - OCR Integration (automatic field extraction)
- 2024-12-05: Completed Priority 1 - Essential Testing (77 tests implemented)
- 2024-12-04: Completed Phases 1-4 - Core system implementation

**To update this document**: Edit `TEMPLATE_SYSTEM_PLAN.md` in the project root.
