//! State management for template UI components.

use crate::DrawingTemplateBuilder;

/// State for the template manager panel.
#[derive(Debug, Default, Clone, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_", borrow_self)]
pub struct TemplateManagerState {
    /// Search query for filtering templates
    search_query: String,

    /// Currently selected template ID
    selected_template: Option<String>,

    /// Whether to show the delete confirmation dialog
    show_delete_confirm: bool,

    /// Template ID pending deletion
    pending_delete: Option<String>,
}

impl TemplateManagerState {
    /// Creates a new template manager state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a mutable reference to the search query for editing.
    pub fn search_query_mut(&mut self) -> &mut String {
        &mut self.search_query
    }

    /// Gets the selected template ID.
    pub fn selected_template_str(&self) -> Option<&str> {
        self.selected_template.as_deref()
    }

    /// Shows delete confirmation dialog.
    pub fn show_delete_confirmation(&mut self, template_id: String) {
        self.with_pending_delete(Some(template_id));
        self.with_show_delete_confirm(true);
    }

    /// Hides delete confirmation dialog.
    pub fn hide_delete_confirmation(&mut self) {
        self.with_show_delete_confirm(false);
        self.with_pending_delete(None);
    }

    /// Gets the template ID pending deletion.
    pub fn pending_delete_str(&self) -> Option<&str> {
        self.pending_delete.as_deref()
    }
}

/// Editor mode for template editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    /// Select and move existing fields
    #[default]
    Select,
    /// Draw new field bounds
    Draw,
    /// Edit field properties
    Edit,
}

/// State for the template editor.
#[derive(Debug, Clone, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_", borrow_self)]
pub struct TemplateEditorState {
    /// Current template being edited
    current_template: Option<DrawingTemplateBuilder>,

    /// Selected field index for editing
    selected_field: Option<usize>,

    /// Current page being edited
    current_page: usize,

    /// Editor mode
    mode: EditorMode,

    /// Undo stack
    undo_stack: Vec<TemplateSnapshot>,

    /// Redo stack
    redo_stack: Vec<TemplateSnapshot>,
}

impl Default for TemplateEditorState {
    fn default() -> Self {
        Self {
            current_template: None,
            selected_field: None,
            current_page: 0,
            mode: EditorMode::Select,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl TemplateEditorState {
    /// Creates a new template editor state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current template mutably.
    pub fn current_template_mut(&mut self) -> Option<&mut DrawingTemplateBuilder> {
        self.current_template.as_mut()
    }

    /// Sets the current template and resets state.
    pub fn set_current_template(&mut self, template: Option<DrawingTemplateBuilder>) {
        self.with_current_template(template);
        self.with_selected_field(None);
        self.with_current_page(0);
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Pushes a snapshot to the undo stack.
    pub fn push_snapshot(&mut self, description: impl Into<String>) {
        if let Some(template) = &self.current_template {
            let snapshot = TemplateSnapshot::new(template.clone(), description);

            self.undo_stack.push(snapshot);
            self.redo_stack.clear();

            // Limit stack size to 50
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
        }
    }

    /// Undoes the last action.
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            if let Some(current) = &self.current_template {
                let redo_snapshot = TemplateSnapshot::new(current.clone(), "Redo point");
                self.redo_stack.push(redo_snapshot);
            }
            self.current_template = Some(snapshot.template);
        }
    }

    /// Redoes the last undone action.
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            if let Some(current) = &self.current_template {
                let undo_snapshot = TemplateSnapshot::new(current.clone(), "Undo point");
                self.undo_stack.push(undo_snapshot);
            }
            self.current_template = Some(snapshot.template);
        }
    }

    /// Gets the description of the last undo action.
    pub fn last_undo_description(&self) -> Option<&str> {
        self.undo_stack
            .last()
            .map(|s| s.action_description().as_str())
    }

    /// Gets the description of the last redo action.
    pub fn last_redo_description(&self) -> Option<&str> {
        self.redo_stack
            .last()
            .map(|s| s.action_description().as_str())
    }

    /// Gets the undo stack for browsing history.
    pub fn undo_history(&self) -> &[TemplateSnapshot] {
        &self.undo_stack
    }

    /// Gets the redo stack for browsing history.
    pub fn redo_history(&self) -> &[TemplateSnapshot] {
        &self.redo_stack
    }

    /// Checks if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Checks if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Snapshot of template state for undo/redo.
#[derive(Debug, Clone, derive_getters::Getters)]
pub struct TemplateSnapshot {
    pub(crate) template: DrawingTemplateBuilder,
    timestamp: std::time::SystemTime,
    action_description: String,
}

impl TemplateSnapshot {
    /// Creates a new snapshot.
    pub fn new(template: DrawingTemplateBuilder, description: impl Into<String>) -> Self {
        Self {
            template,
            timestamp: std::time::SystemTime::now(),
            action_description: description.into(),
        }
    }
}
