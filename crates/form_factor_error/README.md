# form_factor_error

Workspace-wide error types for the form_factor project.

This crate provides the umbrella error type that aggregates errors from all workspace crates,
enabling seamless error propagation across crate boundaries.

## Architecture

Follows CLAUDE.md Pattern 4: Crate-Level Aggregation

- **Module errors** - Specific error types (e.g., `LayerError`, `CanvasError`)
- **Crate errors** - Aggregate module errors (e.g., `DrawingError` in future)
- **Workspace error** - This crate, aggregates all crate errors (`FormFactorError`)

## Usage

```rust
use form_factor_error::{FormFactorError, FormFactorResult};

fn my_operation() -> FormFactorResult<()> {
    // Automatically converts from any crate error
    let data = load_data()?;
    process_data(data)?;
    Ok(())
}
```
