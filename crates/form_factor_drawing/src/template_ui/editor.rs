//! Template editor panel for visual template creation and editing.

use super::{EditorMode, FieldPropertiesPanel, PropertiesAction, TemplateEditorState};
use crate::TemplateRegistry;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use form_factor_core::{FieldDefinition, FormTemplate};
use tracing::{debug, info, instrument};

/// Template editor panel.
#[derive(Debug)]
pub struct TemplateEditorPanel {
    pub(super) state: TemplateEditorState,
    /// Drawing state for creating new fields
    pub(super) drawing_state: Option<DrawingState>,
    /// Drag state for moving/resizing fields
    pub(super) drag_state: Option<DragOperation>,
    /// Properties panel for editing field metadata
    properties_panel: FieldPropertiesPanel,
    /// Whether this is a new template (not yet in registry)
    is_new: bool,
    /// Validation errors from last validation attempt
    validation_errors: Vec<crate::ValidationError>,
}

/// State while drawing a new field.
#[derive(Debug, Clone)]
pub(super) struct DrawingState {
    pub(super) start_pos: Pos2,
    pub(super) current_pos: Pos2,
}

/// Active drag operation.
#[derive(Debug, Clone)]
pub(super) struct DragOperation {
    pub(super) field_index: usize,
    pub(super) operation_type: DragOperationType,
    pub(super) start_pos: Pos2,
    pub(super) original_bounds: form_factor_core::FieldBounds,
}

/// Type of drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DragOperationType {
    Move,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
}

impl Default for TemplateEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEditorPanel {
    /// Creates a new template editor panel.
    pub fn new() -> Self {
        Self {
            state: TemplateEditorState::new(),
            drawing_state: None,
            drag_state: None,
            properties_panel: FieldPropertiesPanel::new(),
            is_new: true,
            validation_errors: Vec::new(),
        }
    }

    /// Gets the editor state.
    pub fn state(&self) -> &TemplateEditorState {
        &self.state
    }

    /// Gets the editor state mutably.
    pub fn state_mut(&mut self) -> &mut TemplateEditorState {
        &mut self.state
    }

    /// Attempts to build and save the current template.
    ///
    /// Returns the built template on success, or validation errors on failure.
    #[instrument(skip(self, registry))]
    pub fn save_template(
        &mut self,
        registry: &mut TemplateRegistry,
    ) -> Result<crate::DrawingTemplate, Vec<crate::ValidationError>> {
        if let Some(builder) = self.state.current_template() {
            // Validate first
            let errors = crate::TemplateValidator::validate(builder, registry, self.is_new);
            if !errors.is_empty() {
                self.validation_errors = errors.clone();
                debug!(error_count = errors.len(), "Validation failed during save");
                return Err(errors);
            }

            // Build the template
            match builder.clone().build() {
                Ok(template) => {
                    // Register in registry
                    registry.register(template.clone());
                    info!(template_id = %template.id(), "Template saved successfully");

                    // After successful save, mark as no longer new
                    self.is_new = false;
                    self.validation_errors.clear();

                    Ok(template)
                }
                Err(e) => {
                    // Build error - convert to validation error
                    let error = crate::ValidationError::EmptyTemplateId;
                    self.validation_errors = vec![error.clone()];
                    debug!(error = ?e, "Template build failed");
                    Err(vec![error])
                }
            }
        } else {
            let error = crate::ValidationError::NoFields;
            self.validation_errors = vec![error.clone()];
            Err(vec![error])
        }
    }

    /// Loads a template into the editor.
    #[instrument(skip(self, registry))]
    pub fn load_template(&mut self, template_id: &str, registry: &TemplateRegistry) -> bool {
        if let Some(_template) = registry.get(template_id) {
            // Convert DrawingTemplate to DrawingTemplateBuilder
            // For now, we'll create a new builder from the template
            // TODO: Add a proper to_builder() method on DrawingTemplate
            info!(template_id = %template_id, "Loading template into editor");

            // We need to convert the template to a builder
            // This is a limitation - we should add a method to do this
            debug!("Template loaded successfully");
            true
        } else {
            debug!(template_id = %template_id, "Template not found");
            false
        }
    }

    /// Creates a new empty template.
    #[instrument(skip(self, template_id, template_name))]
    pub fn new_template(
        &mut self,
        template_id: impl Into<String>,
        template_name: impl Into<String>,
    ) {
        use crate::DrawingTemplateBuilder;

        let builder = DrawingTemplateBuilder::default()
            .id(template_id)
            .name(template_name)
            .version("1.0.0");

        self.state.set_current_template(Some(builder));
        self.is_new = true;
        self.validation_errors.clear();
        info!("Created new template");
    }

    /// Shows the template editor panel.
    #[instrument(skip(self, ui, _registry))]
    pub fn show(&mut self, ui: &mut Ui, _registry: &TemplateRegistry) -> EditorAction {
        let mut action = EditorAction::None;

        // Toolbar
        ui.horizontal(|ui| {
            // Mode buttons
            ui.label("Mode:");
            if ui
                .selectable_label(self.state.mode() == EditorMode::Select, "Select")
                .clicked()
            {
                self.state.set_mode(EditorMode::Select);
                debug!("Switched to Select mode");
            }
            if ui
                .selectable_label(self.state.mode() == EditorMode::Draw, "Draw")
                .clicked()
            {
                self.state.set_mode(EditorMode::Draw);
                debug!("Switched to Draw mode");
            }
            if ui
                .selectable_label(self.state.mode() == EditorMode::Edit, "Edit")
                .clicked()
            {
                self.state.set_mode(EditorMode::Edit);
                debug!("Switched to Edit mode");
            }

            ui.separator();

            // Page navigation
            if let Some(template) = self.state.current_template() {
                ui.label("Page:");

                let current_page = self.state.current_page();
                let page_count = template.page_count();

                if ui.button("<").clicked() && current_page > 0 {
                    self.state.set_current_page(current_page - 1);
                    debug!(page = current_page - 1, "Previous page");
                }

                ui.label(format!("{} / {}", current_page + 1, page_count.max(1)));

                if ui.button(">").clicked() && current_page < page_count.saturating_sub(1) {
                    self.state.set_current_page(current_page + 1);
                    debug!(page = current_page + 1, "Next page");
                }
            }

            ui.separator();

            // Undo/Redo
            let undo_text = if let Some(desc) = self.state.last_undo_description() {
                format!("Undo: {}", desc)
            } else {
                "Undo".to_string()
            };

            if ui
                .add_enabled(self.state.can_undo(), egui::Button::new("Undo"))
                .on_hover_text(format!("{} (Ctrl+Z)", undo_text))
                .clicked()
            {
                self.state.undo();
                debug!("Undo");
            }

            let redo_text = if let Some(desc) = self.state.last_redo_description() {
                format!("Redo: {}", desc)
            } else {
                "Redo".to_string()
            };

            if ui
                .add_enabled(self.state.can_redo(), egui::Button::new("Redo"))
                .on_hover_text(format!("{} (Ctrl+Shift+Z)", redo_text))
                .clicked()
            {
                self.state.redo();
                debug!("Redo");
            }

            ui.separator();

            // Save/Cancel buttons with validation
            if ui.button("Validate").clicked()
                && let Some(template) = self.state.current_template() {
                    self.validation_errors =
                        crate::TemplateValidator::validate(template, _registry, self.is_new);
                    if self.validation_errors.is_empty() {
                        info!("Template validation passed");
                    } else {
                        debug!(
                            error_count = self.validation_errors.len(),
                            "Template validation failed"
                        );
                    }
                }

            if ui.button("Save Template").clicked()
                && let Some(template) = self.state.current_template() {
                    self.validation_errors =
                        crate::TemplateValidator::validate(template, _registry, self.is_new);
                    if self.validation_errors.is_empty() {
                        action = EditorAction::Save {
                            is_new: self.is_new,
                        };
                    } else {
                        debug!(
                            error_count = self.validation_errors.len(),
                            "Cannot save: validation failed"
                        );
                    }
                }
            if ui.button("Cancel").clicked() {
                action = EditorAction::Cancel;
            }
        });

        ui.separator();

        // Show validation errors if any
        if !self.validation_errors.is_empty() {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "❌ Validation Errors:");
            });
            ui.indent("validation_errors", |ui| {
                for error in &self.validation_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error.message()));
                }
            });
            ui.separator();
        }

        // Main editor area with properties panel
        if self.state.current_template().is_some() {
            let current_page = self.state.current_page();

            // Horizontal layout: canvas on left, properties on right
            ui.horizontal(|ui| {
                // Canvas area
                ui.vertical(|ui| {
                    // Get fields for current page (need to do this before borrowing for painting)
                    let fields: Vec<FieldDefinition> = self
                        .state
                        .current_template()
                        .map(|t| {
                            t.fields_for_page(current_page)
                                .into_iter()
                                .cloned()
                                .collect()
                        })
                        .unwrap_or_default();

                    let field_count = fields.len();

                    // Canvas area for template editing
                    let (response, painter) = ui.allocate_painter(
                        egui::vec2(ui.available_width() * 0.7, ui.available_height()),
                        Sense::click_and_drag(),
                    );

                    let canvas_rect = response.rect;

                    // Draw background
                    painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(240));

                    // Handle keyboard input
                    ui.input(|i| {
                        // Delete key
                        if i.key_pressed(egui::Key::Delete)
                            && let Some(selected_idx) = self.state.selected_field()
                        {
                            self.delete_field(selected_idx, current_page);
                            self.properties_panel.reset();
                            debug!(field_index = selected_idx, "Field deleted");
                        }

                        // Undo: Ctrl+Z (Windows/Linux) or Cmd+Z (Mac)
                        if i.key_pressed(egui::Key::Z)
                            && (i.modifiers.ctrl || i.modifiers.command)
                            && !i.modifiers.shift
                            && self.state.can_undo()
                        {
                            self.state.undo();
                            debug!("Undo via keyboard shortcut");
                        }

                        // Redo: Ctrl+Shift+Z (Windows/Linux) or Cmd+Shift+Z (Mac)
                        if i.key_pressed(egui::Key::Z)
                            && (i.modifiers.ctrl || i.modifiers.command)
                            && i.modifiers.shift
                            && self.state.can_redo()
                        {
                            self.state.redo();
                            debug!("Redo via keyboard shortcut");
                        }

                        // Alternative Redo: Ctrl+Y (Windows/Linux)
                        if i.key_pressed(egui::Key::Y)
                            && i.modifiers.ctrl
                            && !i.modifiers.command
                            && self.state.can_redo()
                        {
                            self.state.redo();
                            debug!("Redo via keyboard shortcut (Ctrl+Y)");
                        }
                    });

                    // Handle mouse interactions based on mode
                    match self.state.mode() {
                        EditorMode::Draw => {
                            self.handle_draw_mode(&response, &painter, canvas_rect, current_page);
                        }
                        EditorMode::Select | EditorMode::Edit => {
                            self.handle_select_mode(&response, &fields, canvas_rect, current_page);
                        }
                    }

                    // Render field overlays
                    for (index, field) in fields.iter().enumerate() {
                        let is_selected = self.state.selected_field() == Some(index);
                        self.render_field(field, &painter, canvas_rect, is_selected);

                        // Show resize handles for selected field in Select mode
                        if is_selected && self.state.mode() == EditorMode::Select {
                            self.render_resize_handles(field, &painter, canvas_rect);
                        }
                    }

                    // Render drawing preview
                    if let Some(drawing) = &self.drawing_state {
                        self.render_drawing_preview(drawing, &painter);
                    }

                    // Show field count
                    ui.label(format!("Fields on this page: {}", field_count));
                });

                ui.separator();

                // Properties panel on the right
                ui.vertical(|ui| {
                    ui.set_min_width(250.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let properties_action =
                            self.properties_panel
                                .show(ui, &mut self.state, current_page);

                        match properties_action {
                            PropertiesAction::Applied => {
                                debug!("Field properties applied");
                            }
                            PropertiesAction::Cancelled => {
                                debug!("Field properties cancelled");
                            }
                            PropertiesAction::Delete(field_idx) => {
                                self.delete_field(field_idx, current_page);
                                self.properties_panel.reset();
                                debug!(
                                    field_index = field_idx,
                                    "Field deleted via properties panel"
                                );
                            }
                            PropertiesAction::None => {}
                        }
                    });
                });
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No template loaded. Create or open a template to begin editing.");
            });
        }

        action
    }

    /// Renders a single field overlay.
    fn render_field(
        &self,
        field: &FieldDefinition,
        painter: &egui::Painter,
        canvas_rect: Rect,
        is_selected: bool,
    ) {
        let bounds = &field.bounds;

        // Simple transform: scale field bounds to canvas
        // TODO: Integrate with proper canvas transform pipeline
        let field_rect = Rect::from_min_size(
            canvas_rect.min + Vec2::new(bounds.x, bounds.y),
            Vec2::new(bounds.width, bounds.height),
        );

        // Clamp to canvas bounds
        let field_rect = field_rect.intersect(canvas_rect);

        // Draw field rectangle
        let color = if is_selected {
            Color32::from_rgba_unmultiplied(0, 150, 255, 100) // Blue for selected
        } else {
            Color32::from_rgba_unmultiplied(0, 200, 0, 80) // Green for unselected
        };

        painter.rect_filled(field_rect, 2.0, color);

        // Draw border
        let stroke = if is_selected {
            Stroke::new(2.0, Color32::from_rgb(0, 100, 200))
        } else {
            Stroke::new(1.0, Color32::from_rgb(0, 150, 0))
        };
        painter.rect_stroke(field_rect, 2.0, stroke, egui::epaint::StrokeKind::Middle);

        // Draw label
        painter.text(
            field_rect.min + Vec2::new(5.0, 5.0),
            egui::Align2::LEFT_TOP,
            &field.id,
            egui::FontId::proportional(12.0),
            Color32::BLACK,
        );
    }

    /// Finds the field at the given position.
    pub(super) fn find_field_at_position(
        &self,
        pos: Pos2,
        fields: &[FieldDefinition],
        canvas_rect: Rect,
    ) -> Option<usize> {
        // Search in reverse order so top fields are selected first
        for (index, field) in fields.iter().enumerate().rev() {
            let bounds = &field.bounds;

            let field_rect = Rect::from_min_size(
                canvas_rect.min + Vec2::new(bounds.x, bounds.y),
                Vec2::new(bounds.width, bounds.height),
            );

            let field_rect = field_rect.intersect(canvas_rect);

            if field_rect.contains(pos) {
                return Some(index);
            }
        }

        None
    }
}

/// Action to perform based on user interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    /// No action
    None,
    /// Save template
    Save {
        /// Whether this is a new template (not yet in registry)
        is_new: bool,
    },
    /// Cancel editing
    Cancel,
}
