//! State management for template UI components.

use crate::DrawingTemplateBuilder;
use egui::{Pos2, Rect};
use std::time::SystemTime;

/// State for the template manager panel.
#[derive(Debug, Default, Clone)]
pub struct TemplateManagerState {
    /// Search query for filtering templates
    pub(crate) search_query: String,

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

    /// Gets the search query.
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Sets the search query.
    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    /// Gets the selected template ID.
    pub fn selected_template(&self) -> Option<&str> {
        self.selected_template.as_deref()
    }

    /// Sets the selected template ID.
    pub fn set_selected_template(&mut self, id: Option<String>) {
        self.selected_template = id;
    }

    /// Checks if delete confirmation is showing.
    pub fn is_showing_delete_confirm(&self) -> bool {
        self.show_delete_confirm
    }

    /// Shows delete confirmation dialog.
    pub fn show_delete_confirm(&mut self, template_id: String) {
        self.pending_delete = Some(template_id);
        self.show_delete_confirm = true;
    }

    /// Hides delete confirmation dialog.
    pub fn hide_delete_confirm(&mut self) {
        self.show_delete_confirm = false;
        self.pending_delete = None;
    }

    /// Gets the template pending deletion.
    pub fn pending_delete(&self) -> Option<&str> {
        self.pending_delete.as_deref()
    }
}

/// Editor mode for template editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Select and move existing fields
    Select,
    /// Draw new field bounds
    Draw,
    /// Edit field properties
    Edit,
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Select
    }
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
            let snapshot = TemplateSnapshot {
                template: template.clone(),
                timestamp: SystemTime::now(),
                action_description: description.into(),
            };

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
                self.redo_stack.push(TemplateSnapshot {
                    template: current.clone(),
                    timestamp: SystemTime::now(),
                    action_description: "Redo point".to_string(),
                });
            }
            self.current_template = Some(snapshot.template);
        }
    }

    /// Redoes the last undone action.
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            if let Some(current) = &self.current_template {
                self.undo_stack.push(TemplateSnapshot {
                    template: current.clone(),
                    timestamp: SystemTime::now(),
                    action_description: "Undo point".to_string(),
                });
            }
            self.current_template = Some(snapshot.template);
        }
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
    template: DrawingTemplateBuilder,
    timestamp: SystemTime,
    action_description: String,
}

/// Drag state for field manipulation.
#[derive(Debug, Clone)]
pub struct DragState {
    field_index: usize,
    drag_type: DragType,
    start_pos: Pos2,
    original_bounds: Rect,
}

impl DragState {
    /// Creates a new drag state.
    pub fn new(field_index: usize, drag_type: DragType, start_pos: Pos2, original_bounds: Rect) -> Self {
        Self {
            field_index,
            drag_type,
            start_pos,
            original_bounds,
        }
    }

    /// Gets the field index being dragged.
    pub fn field_index(&self) -> usize {
        self.field_index
    }

    /// Gets the drag type.
    pub fn drag_type(&self) -> DragType {
        self.drag_type
    }

    /// Gets the starting position.
    pub fn start_pos(&self) -> Pos2 {
        self.start_pos
    }

    /// Gets the original bounds.
    pub fn original_bounds(&self) -> &Rect {
        &self.original_bounds
    }
}

/// Type of drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragType {
    /// Moving the entire field
    Move,
    /// Resizing from top-left corner
    ResizeTopLeft,
    /// Resizing from top-right corner
    ResizeTopRight,
    /// Resizing from bottom-left corner
    ResizeBottomLeft,
    /// Resizing from bottom-right corner
    ResizeBottomRight,
}
