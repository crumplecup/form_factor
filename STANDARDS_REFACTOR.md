# Standards Refactor - Work in Progress

## Status
Mid-refactor to align codebase with CLAUDE.md standards. Code does not currently compile.

## Completed
1. ✅ Added derive_setters to workspace dependencies
2. ✅ Fixed Shape types (Rectangle, Circle, PolygonShape) to use:
   - Private fields with derive_getters
   - derive_setters for mutation
   - Getter calls (`field()`) instead of direct field access
3. ✅ Fixed most rendering.rs field access violations
4. ✅ Renamed botticelli_health → form_factor_health

## Remaining Work

### Field Access Violations (112 errors)
All template_ui and instance_ui code needs systematic fixes:

1. **FieldDefinition field access** - Use getters:
   - `field.id` → `field.id()`
   - `field.page_index` → `field.page_index()`
   - `field.field_type` → `field.field_type().clone()` (not Copy)
   - `field.validation_pattern` → `field.validation_pattern()`
   - `field.required` → `field.required()`

2. **FieldBounds field access** - Use getters (return &f32, need deref):
   - `bounds.x` → `*bounds.x()`
   - `bounds.y` → `*bounds.y()`
   - `bounds.width` → `*bounds.width()`
   - `bounds.height` → `*bounds.height()`

3. **FieldValue field access** - Use getters:
   - `value.content` → `value.content()`
   - `value.field_id` → `value.field_id()`

4. **TemplateManagerState setters** - Fix derive_setters usage:
   - Currently trying to call `.set_selected_template()` but setter not working
   - Need to verify derive_setters is correctly applied

### Files Needing Fixes

Priority order (most errors first):

1. `crates/form_factor_drawing/src/template_ui/validation.rs` - ~20 errors
   - Field access to id, page_index, validation_pattern
   - Bounds comparisons need deref
   
2. `crates/form_factor_drawing/src/template_ui/properties.rs` - ~15 errors
   - Field access violations
   - Bounds field access

3. `crates/form_factor_drawing/src/template_ui/editor.rs` - ~10 errors
   - Bounds field access in rendering code
   
4. `crates/form_factor_drawing/src/template_ui/manager.rs` - setter issues

5. `crates/form_factor_drawing/src/canvas/tools.rs` - field_type clone issue

6. `crates/form_factor_drawing/src/instance_ui/data_entry.rs` - Option unwrap issue

### Pattern to Follow

**Before:**
```rust
if field.page_index >= page_count {
    let x = bounds.x;
    let area = bounds.width * bounds.height;
}
```

**After:**
```rust
if *field.page_index() >= page_count {
    let x = *bounds.x();
    let area = *bounds.width() * *bounds.height();
}
```

**For non-Copy types:**
```rust
// Before
.field_type(*field.field_type())

// After  
.field_type(field.field_type().clone())
```

## Testing Strategy
1. Fix compilation errors file by file
2. Run `cargo check` after each file
3. Once compiling, run `just check-all`
4. Fix any clippy warnings
5. Run test suite

## Notes
- Do NOT use `#[allow]` directives
- All fields must remain private with getter/setter access
- Bounds getters return `&f32`, need dereference for arithmetic
- FieldType is not Copy, must clone when needed
- Follow CLAUDE.md strictly - no exceptions or workarounds
