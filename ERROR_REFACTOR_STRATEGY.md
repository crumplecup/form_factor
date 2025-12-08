# Error Handling Refactor Strategy

## ✅ COMPLETED (2024-12-08)

This refactor has been successfully completed. All error types now follow CLAUDE.md patterns:

### Achievements

- **All crates use derive_more** - No manual Display/Error impls remain
- **Proper error hierarchy** - Module errors → Crate umbrellas → Workspace umbrella
- **#[track_caller] throughout** - Automatic location tracking at every error site
- **External error wrapping** - All external errors (opencv, serde, etc.) properly wrapped
- **Feature-gated properly** - CV and OCR errors only available with their features
- **Zero test breakage** - All existing tests still pass

### Final Architecture

```
form_factor_error (workspace umbrella)
├── FormFactorError
└── FormFactorErrorKind
    ├── Core(CoreError)
    ├── Drawing(FormError)
    ├── Cv(CvError) [feature = "cv"]
    └── Ocr(OCRError) [feature = "ocr"]

form_factor_core
├── CoreError (crate umbrella)
└── IoError (module error)

form_factor_drawing
├── FormError (crate umbrella)
└── Module errors: CanvasError, LayerError, ShapeError, TemplateError, InstanceError

form_factor_cv
├── CvError (crate umbrella)
├── TextDetectionError [feature = "text-detection"]
└── LogoDetectionError [feature = "logo-detection"]

form_factor_ocr
└── OCRError (single module, acts as crate error)
```

### Commits

- `refactor(cv): Convert text detection errors to derive_more` (5d7b540)
- `refactor(errors): Complete derive_more error conversion for CV and OCR` (5139529)

---

## Original State Analysis (Historical)

### Problems

1. **form_factor_error crate is empty** - Just a stub with `add()` function
2. **FormError in wrong place** - Defined in `form_factor_drawing` but should be workspace-wide umbrella
3. **Inconsistent naming** - FormError vs FormFactorError vs specific errors
4. **Missing umbrella pattern** - No crate-level aggregation following derive_more pattern from CLAUDE.md
5. **Poor separation** - Drawing crate has umbrella error but it should be at workspace level

### Current Error Locations

```
form_factor_error/          # EMPTY - just stub code
form_factor_core/error.rs   # IoError only
form_factor_drawing/error.rs # FormError (umbrella) - WRONG PLACE
  ├── canvas/error.rs
  ├── instance/error.rs  
  ├── layer/error.rs
  ├── shape/error.rs
  └── template/error.rs
form_factor/error.rs        # FormFactorError (binary-specific)
```

## Target Architecture (Per CLAUDE.md)

### Pattern from CLAUDE.md Section: "Error Handling"

```rust
// Pattern 1: Simple Error (message + location)
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("HTTP Error: {} at {}:{}", message, file, line)]
pub struct HttpError {
    pub message: String,
    pub line: u32,
    pub file: &'static str,
}

// Pattern 2: ErrorKind Enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum StorageErrorKind {
    #[display("Media not found: {}", _0)]
    NotFound(String),
}

// Pattern 3: Wrapper (ErrorKind + location)
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("Storage: {} at {}:{}", kind, file, line)]
pub struct StorageError {
    pub kind: StorageErrorKind,
    pub line: u32,
    pub file: &'static str,
}

// Pattern 4: Crate-Level Aggregation (UMBRELLA)
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum CrateErrorKind {
    #[from(HttpError)]
    Http(HttpError),
    
    #[from(StorageError)]
    Storage(StorageError),
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Crate Error: {}", _0)]
pub struct CrateError(Box<CrateErrorKind>);

// Generic blanket From
impl<T> From<T> for CrateError
where T: Into<CrateErrorKind>
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}
```

### Reference Implementation

See `crates/botticelli_error` in CLAUDE.md for complete working example.

## Refactor Plan

### Step 1: Define Workspace Umbrella in form_factor_error

Create the **workspace-wide umbrella error** that all crates can use:

```rust
// crates/form_factor_error/src/lib.rs

use derive_more::{Display, Error, From};

/// Workspace-wide error kind aggregating all crate errors
#[derive(Debug, From, Display, Error)]
pub enum FormFactorErrorKind {
    #[from]
    Core(form_factor_core::CoreError),
    
    #[from]
    Drawing(form_factor_drawing::DrawingError),
    
    #[cfg(feature = "cv")]
    #[from]
    Cv(form_factor_cv::CvError),
    
    #[cfg(feature = "ocr")]
    #[from]
    Ocr(form_factor_ocr::OcrError),
    
    // Add other crate errors as needed
}

/// Workspace-wide umbrella error
#[derive(Debug, Display, Error)]
#[display("FormFactor Error: {}", _0)]
pub struct FormFactorError(Box<FormFactorErrorKind>);

impl<T> From<T> for FormFactorError
where
    T: Into<FormFactorErrorKind>,
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

/// Workspace-wide result type
pub type FormFactorResult<T> = Result<T, FormFactorError>;
```

### Step 2: Fix Each Crate's Error Organization

Each crate follows same pattern:

#### form_factor_core

```rust
// crates/form_factor_core/src/error.rs

// Keep IoError, TemplateError, etc. - module-specific errors

// Add crate-level umbrella:
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum CoreErrorKind {
    #[from]
    Io(IoError),
    
    #[from]
    TemplateBrowser(TemplateBrowserError),
    
    // Other module errors
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Core Error: {}", _0)]
pub struct CoreError(Box<CoreErrorKind>);

impl<T> From<T> for CoreError
where T: Into<CoreErrorKind>
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
```

#### form_factor_drawing

```rust
// crates/form_factor_drawing/src/error.rs

// Rename FormError -> DrawingError
// Keep CanvasError, LayerError, ShapeError, etc.

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum DrawingErrorKind {
    #[from]
    Canvas(CanvasError),
    
    #[from]
    Layer(LayerError),
    
    #[from]
    Shape(ShapeError),
    
    #[from]
    Template(TemplateError),
    
    #[from]
    Instance(InstanceError),
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("Drawing Error: {}", _0)]
pub struct DrawingError(Box<DrawingErrorKind>);

impl<T> From<T> for DrawingError
where T: Into<DrawingErrorKind>
{
    fn from(err: T) -> Self {
        Self(Box::new(err.into()))
    }
}

pub type DrawingResult<T> = Result<T, DrawingError>;
```

### Step 3: Update Dependencies

```toml
# crates/form_factor_error/Cargo.toml
[dependencies]
derive_more = { workspace = true }
form_factor_core = { workspace = true }
form_factor_drawing = { workspace = true }
form_factor_cv = { workspace = true, optional = true }
form_factor_ocr = { workspace = true, optional = true }

[features]
cv = ["dep:form_factor_cv"]
ocr = ["dep:form_factor_ocr"]
```

### Step 4: Update Facade Exports

```rust
// crates/form_factor/src/lib.rs

// Re-export workspace umbrella
pub use form_factor_error::{FormFactorError, FormFactorErrorKind, FormFactorResult};

// Re-export crate-level errors for advanced users
pub use form_factor_core::{CoreError, CoreResult};
pub use form_factor_drawing::{DrawingError, DrawingResult};

// Re-export module-specific errors for fine-grained handling
pub use form_factor_drawing::{
    CanvasError, LayerError, ShapeError, 
    TemplateError, InstanceError
};
```

### Step 5: Migration Path for Tests

```rust
// OLD (in tests)
use form_factor::FormError;

// NEW
use form_factor::FormFactorError;
// OR for crate-specific
use form_factor::DrawingError;
// OR for module-specific
use form_factor::LayerError;
```

## Critical Understanding

**Never attach external errors directly to umbrella variants!**

Every error must have location tracking. External errors (serde, opencv, etc.) don't have our tracking, so:

1. Create site-specific wrapper at call site
2. Wrapper includes location tracking via `#[track_caller]`
3. Use wrapper in umbrella, not raw external error

```rust
// ❌ WRONG
pub enum ErrorKind {
    Json(serde_json::Error),  // No location tracking!
}

// ✅ RIGHT
pub struct SerializationError {
    pub message: String,
    pub line: u32,
    pub file: &'static str,
}

pub enum ErrorKind {
    Serialization(SerializationError),  // Has location!
}
```

## Implementation Order

1. ✅ **Create strategy document** (this file)
2. ✅ **Catalog all error sites** - Found external errors in all crates
3. ✅ **Create site-specific wrappers** - Using derive_more patterns throughout
4. ✅ **Refactor form_factor_core errors** - Added CoreError umbrella with IoError
5. ✅ **Refactor form_factor_drawing errors** - Created FormError umbrella with all module errors
6. ✅ **Update form_factor_cv errors** - Added CvError umbrella with TextDetection and LogoDetection
7. ✅ **Update form_factor_ocr errors** - Converted OCRError to derive_more patterns
8. ✅ **Implement form_factor_error umbrella** - Aggregates Core, Drawing, CV, OCR umbrellas
9. ✅ **Update facade exports** - form_factor/src/lib.rs exports all error types
10. ✅ **Fix all call sites** - All external errors wrapped with #[track_caller]
11. ✅ **Fix all tests** - No test breakage (tests still pass)
12. ✅ **Run full test suite** - All packages compile and test successfully
13. ⬜ **Update documentation** - Error handling examples (future work)

## Naming Conventions

| Scope | Error Name | Kind Enum | Result Type |
|-------|-----------|-----------|-------------|
| Workspace | `FormFactorError` | `FormFactorErrorKind` | `FormFactorResult<T>` |
| Crate | `{Crate}Error` | `{Crate}ErrorKind` | `{Crate}Result<T>` |
| Module | `{Module}Error` | `{Module}ErrorKind` | N/A (use crate Result) |

Examples:
- Workspace: `FormFactorError`, `FormFactorErrorKind`, `FormFactorResult<T>`
- Crate: `DrawingError`, `DrawingErrorKind`, `DrawingResult<T>`
- Module: `LayerError`, `LayerErrorKind` (uses `DrawingResult<T>`)

## Benefits

1. **Clear hierarchy** - Workspace → Crate → Module
2. **Proper architecture** - Umbrella in dedicated `_error` crate
3. **CLAUDE.md compliant** - Follows Pattern 4 exactly
4. **Easy error handling** - Use workspace error for cross-crate, crate error for single-crate
5. **Automatic conversions** - derive_more::From handles all conversions
6. **Better ergonomics** - Users can `?` operator across crate boundaries

## Non-Goals

- ❌ **Don't remove module-specific errors** - They stay for fine-grained handling
- ❌ **Don't change error tracking** - Keep `#[track_caller]` and location fields
- ❌ **Don't break API** - Add new types, deprecate old ones gracefully
- ❌ **Don't add complexity** - Keep simple Pattern 4 from CLAUDE.md

## Testing Strategy

After each step:
1. `just check` - Basic compilation
2. `just test-package [crate]` - Test affected crate
3. `just check-features` - All feature combinations
4. `just check-all` - Full suite with clippy

## Documentation Updates

Update after completion:
- Error handling examples in each crate's README
- Add error conversion examples to facade docs
- Update CLAUDE.md reference to point to this implementation
- Add error handling guide to main README

## References

- **CLAUDE.md** Section: "Error Handling" (Pattern 4: Crate-Level Aggregation)
- **form_factor_error crate** - Currently empty, will hold workspace umbrella
- **derive_more crate** - Used for Display, Error, From derives
