- After generating new code and correcting any check or build errors, run cargo clippy and clear all warnings, then commit the changes to git using best practices for code auditing.
- Avoid running cargo clean often, to take advantage of incremental compilation during development.
- In lib.rs, export the 
visibility of all types at the 
root level with pub use 
statements.  Keep the mod 
statements prive so there is only
 one way for users to import the 
type. In modules, import types from the crate level with use crate::{type1, type2} statements.
- When running clippy, rather than deny all warnings, let them complete so you can fix them all in a single pass.