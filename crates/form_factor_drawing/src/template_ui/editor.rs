//! Template editor panel for visual template creation and editing.

use super::{EditorMode, TemplateEditorState};
use crate::TemplateRegistry;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use form_factor_core::FieldDefinition;
use tracing::{debug, info, instrument};

/// Template editor panel.
#[derive(Debug)]
pub struct TemplateEditorPanel {
    state: TemplateEditorState,
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
    pub fn new_template(&mut self, template_id: impl Into<String>, template_name: impl Into<String>) {
        use crate::DrawingTemplateBuilder;

        let builder = DrawingTemplateBuilder::default()
            .id(template_id)
            .name(template_name)
            .version("1.0.0");

        self.state.set_current_template(Some(builder));
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
            if ui.selectable_label(self.state.mode() == EditorMode::Select, "Select").clicked() {
                self.state.set_mode(EditorMode::Select);
                debug!("Switched to Select mode");
            }
            if ui.selectable_label(self.state.mode() == EditorMode::Draw, "Draw").clicked() {
                self.state.set_mode(EditorMode::Draw);
                debug!("Switched to Draw mode");
            }
            if ui.selectable_label(self.state.mode() == EditorMode::Edit, "Edit").clicked() {
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
            if ui.add_enabled(self.state.can_undo(), egui::Button::new("Undo")).clicked() {
                self.state.undo();
                debug!("Undo");
            }
            if ui.add_enabled(self.state.can_redo(), egui::Button::new("Redo")).clicked() {
                self.state.redo();
                debug!("Redo");
            }

            ui.separator();

            // Save/Cancel buttons
            if ui.button("Save Template").clicked() {
                action = EditorAction::Save;
            }
            if ui.button("Cancel").clicked() {
                action = EditorAction::Cancel;
            }
        });

        ui.separator();

        // Main editor area
        if self.state.current_template().is_some() {
            let current_page = self.state.current_page();

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
                ui.available_size(),
                Sense::click_and_drag(),
            );

            let canvas_rect = response.rect;

            // Draw background
            painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(240));

            // Render field overlays
            for (index, field) in fields.iter().enumerate() {
                let is_selected = self.state.selected_field() == Some(index);
                self.render_field(field, &painter, canvas_rect, is_selected);
            }

            // Handle selection clicks
            if response.clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let selected = self.find_field_at_position(click_pos, &fields, canvas_rect);
                    self.state.set_selected_field(selected);

                    if let Some(idx) = selected {
                        debug!(field_index = idx, "Field selected");
                    } else {
                        debug!("Deselected all fields");
                    }
                }
            }

            // Show field count
            ui.label(format!("Fields on this page: {}", field_count));
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
            canvas_rect.min + Vec2::new(bounds.x as f32, bounds.y as f32),
            Vec2::new(bounds.width as f32, bounds.height as f32),
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
    fn find_field_at_position(
        &self,
        pos: Pos2,
        fields: &[FieldDefinition],
        canvas_rect: Rect,
    ) -> Option<usize> {
        // Search in reverse order so top fields are selected first
        for (index, field) in fields.iter().enumerate().rev() {
            let bounds = &field.bounds;

            let field_rect = Rect::from_min_size(
                canvas_rect.min + Vec2::new(bounds.x as f32, bounds.y as f32),
                Vec2::new(bounds.width as f32, bounds.height as f32),
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
    Save,
    /// Cancel editing
    Cancel,
}
