//! State management for template UI components.

use crate::DrawingTemplateBuilder;

/// State for the template manager panel.
#[derive(Debug, Default, Clone, derive_getters::Getters)]
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

    /// Sets the search query.
    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    /// Gets the selected template ID.
    pub fn selected_template_str(&self) -> Option<&str> {
        self.selected_template.as_deref()
    }

    /// Sets the selected template ID.
    pub fn set_selected_template(&mut self, id: Option<String>) {
        self.selected_template = id;
    }

    /// Shows delete confirmation dialog.
    pub fn show_delete_confirmation(&mut self, template_id: String) {
        self.pending_delete = Some(template_id);
        self.show_delete_confirm = true;
    }

    /// Hides delete confirmation dialog.
    pub fn hide_delete_confirmation(&mut self) {
        self.show_delete_confirm = false;
        self.pending_delete = None;
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
#[derive(Debug, Clone)]
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

    /// Gets the current template.
    pub fn current_template(&self) -> Option<&DrawingTemplateBuilder> {
        self.current_template.as_ref()
    }

    /// Gets the current template mutably.
    pub fn current_template_mut(&mut self) -> Option<&mut DrawingTemplateBuilder> {
        self.current_template.as_mut()
    }

    /// Sets the current template.
    pub fn set_current_template(&mut self, template: Option<DrawingTemplateBuilder>) {
        self.current_template = template;
        self.selected_field = None;
        self.current_page = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Gets the selected field index.
    pub fn selected_field(&self) -> Option<usize> {
        self.selected_field
    }

    /// Sets the selected field index.
    pub fn set_selected_field(&mut self, index: Option<usize>) {
        self.selected_field = index;
    }

    /// Gets the current page.
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Sets the current page.
    pub fn set_current_page(&mut self, page: usize) {
        self.current_page = page;
    }

    /// Gets the editor mode.
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Sets the editor mode.
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
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
        self.undo_stack.last().map(|s| s.description())
    }

    /// Gets the description of the last redo action.
    pub fn last_redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|s| s.description())
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
#[derive(Debug, Clone)]
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

    /// Gets the action description.
    pub fn description(&self) -> &str {
        &self.action_description
    }

    /// Gets the timestamp.
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
}
