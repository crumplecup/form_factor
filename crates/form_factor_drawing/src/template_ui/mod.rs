//! Template UI module for visual template creation and editing.

mod manager;
mod state;

pub use manager::TemplateManagerPanel;
pub use state::{EditorMode, TemplateEditorState, TemplateManagerState};
