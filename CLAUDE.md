- After generating new code and correcting any check or build errors:
  1. Run cargo clippy and clear all warnings.
  2. Run cargo test and clear all errors. If fixes are applied, go back to step one (clippy).
  3. Commit the changes to git using best practices for code auditing.
- Avoid running cargo clean often, to take advantage of incremental compilation during development.
- In lib.rs, export the visibility of all types at the root level with pub use statements.
  - Keep the mod statements private so there is only one way for users to import the type.
  - In modules, import types from the crate level with use crate::{type1, type2} statements.
- When running clippy, rather than deny all warnings, let them complete so you can fix them all in a single pass.
- After editing a markdown file, run markdownlint and either fix the error or add an exception, as appropriate in the context.
- Data structures should derive Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, and Hash if possible.
  Use derive_more to derive Display, FromStr, From, Deref, DerefMut, AsRef, and AsMut when appropriate.
  For enums with no fields, use strum to derive EnumIter.
- Use unique error types for different sources to create encapsulation around error conditions for easier isolation.
  - For specific errors types capturing initial error condition, wrap enums in a struct that include the line and file where the error occurred using the line! and file! macros.
  - The idiom is to call the enumeration something like MyErrorKind, and the wrapper struct MyError.
  - The idiom for MyError is to have fields kind, line and file.
  - Omit the enum type and kind field when a static message conveys sufficient information, but still include the line and file.
  - Implement a specific error message in the display impl for each variant of the enum, then wrap this msg in the display impl for the wrapper. E.g. If the display for MyErrorKind is e, then MyError displays "My Error: {e} at line {line} in {file}" so the user can see the whole context.
  - Expand and improve error structs and enums as necessary to capture sufficient information about the error conditions to gain insight into the nature of the problem.
- Do not place mod tests in the module next to the code. Place unit tests in the tests directory.

