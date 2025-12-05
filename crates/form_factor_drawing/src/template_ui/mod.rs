//! Template UI module for visual template creation and editing.

mod editor;
mod manager;
mod manipulation;
mod properties;
mod state;

pub use editor::{EditorAction, TemplateEditorPanel};
pub use manager::TemplateManagerPanel;
pub use properties::{FieldPropertiesPanel, PropertiesAction};
pub use state::{EditorMode, TemplateEditorState, TemplateManagerState};
