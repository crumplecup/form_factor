# Claude Project Instructions

## Workflow

- After generating new code and correcting any cargo check errors and warnings:
  1. Run cargo test and clear all errors.
  2. Run cargo clippy and clear all warnings.
  3. Commit the changes to git using best practices for code auditing.
- Avoid running cargo clean often, to take advantage of incremental compilation during development.

## Linting

- When running any linter (e.g. clippy or markdownlint), rather than deny all warnings, let them complete so you can fix them all in a single pass.
- After editing a markdown file, run markdownlint and either fix the error or add an exception, as appropriate in the context.

## API structure

- In lib.rs, export the visibility of all types at the root level with pub use statements.
  - Keep the mod statements private so there is only one way for users to import the type.
  - In modules, import types from the crate level with use crate::{type1, type2} statements.

## Derive Policies

- Data structures should derive Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, and Hash if possible.
  Use derive_more to derive Display, FromStr, From, Deref, DerefMut, AsRef, and AsMut when appropriate.
  For enums with no fields, use strum to derive EnumIter.

## Testing

- Do not place mod tests in the module next to the code. Place unit tests in the tests directory.

## Error Handling

- Use unique error types for different sources to create encapsulation around error conditions for easier isolation.
  - For specific errors types capturing initial error condition, wrap enums in a struct that include the line and file where the error occurred using the line! and file! macros.
  - The idiom is to call the enumeration something like MyErrorKind, and the wrapper struct MyError.
  - The idiom for MyError is to have fields kind, line and file.
  - Omit the enum type and kind field when a static message conveys sufficient information, but still include the line and file.
  - Implement a specific error message in the display impl for each variant of the enum, then wrap this msg in the display impl for the wrapper. E.g. If the display for MyErrorKind is e, then MyError displays "My Error: {e} at line {line} in {file}" so the user can see the whole context.
  - Use the derive_more crate to implement Display and Error when convenient.
  - Expand and improve error structs and enums as necessary to capture sufficient information about the error conditions to gain insight into the nature of the problem.
- After creating a new unique error type, add a variant to the crate level error enum using the new error name as a variant type, including the new error type as a field (i.e. CrateErrorKind { ... NewError(NewError)})
  - The display impl for the crate-level enum should forward the impl from the original error (e.g. If the display value of NewError is e, then the display for CrateErrorKind is "{e}").
  - The display impl for the wrapper struct around the crate-level enum should include the display value of its kind field (e.g. If the display value of CrateErrorKind is e, then CrateError displays "Form Error: {e}").
  - Use the derive_more crate to implement Display and Error when convenient.
  - Use derive_more to derive From to streamline conversion from a new error to the crate level error (e.g. use `#[from(NewError)]` above the variant `NewError(NewError)`)
- If a function or method returns a single unique error type, use that type. If the body contains more than one error type in its result types, convert the unique error types to the crate level type, and use the crate level error in the return type of the function or method signature.

## Module Organization

- When a module file exceeds ~500-1000 lines, consider splitting it into a module directory with focused submodules organized by responsibility (e.g., core, io, tools, rendering).
- Use pub(super) visibility for fields and methods that need to be accessible across submodules within the same module directory.
- Create a mod.rs file to re-export the public API and keep internal organization private.
- When using derive_getters on structs in module directories, mark fields as pub(super) to allow cross-submodule access while maintaining encapsulation from external modules.
- Add helper methods (setters, mut accessors) to the core struct for clean cross-submodule communication instead of directly accessing pub(super) fields.

## Error Field Types

- Error struct `file` fields should use `&'static str` (not `String`) to match the return type of the `file!()` macro, reducing allocations.
- Use `#[derive(Debug, derive_more::From)]` on crate-level error enums to automatically generate From implementations for all error variants.

## Common Refactoring Patterns

- **State Machine Extraction**: When multiple boolean flags represent mutually exclusive states, extract them into an enum state machine to prevent invalid state combinations.
- **Borrow Checker**: When encountering borrow checker errors with simultaneous immutable and mutable borrows, extract needed values before taking mutable references (e.g., `let value = *self.field(); /* then mutably borrow */`).

## Unsafe

- Use the forbid unsafe lint at the top level of lib.rs to prevent unsafe code.

