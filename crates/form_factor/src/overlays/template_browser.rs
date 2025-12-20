//! Template browser overlay for managing form templates.

use crate::overlays::{Overlay, OverlayResponse};
use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};
use form_factor_drawing::{DrawingTemplate, TemplateRegistry};
use tracing::{debug, instrument};

/// Template browser overlay for browsing and managing templates.
///
/// Provides UI for:
/// - Grid/list view of available templates
/// - Search and filtering
/// - Template actions (edit, fill, delete, duplicate)
pub struct TemplateBrowserOverlay {
    /// Template registry for loading templates
    registry: TemplateRegistry,
    /// Search query
    search_query: String,
    /// Selected template ID
    selected_template: Option<String>,
    /// Whether to show confirmation dialog
    show_delete_confirm: bool,
    /// Template to delete (for confirmation)
    delete_template_id: Option<String>,
}

impl TemplateBrowserOverlay {
    /// Creates a new template browser overlay.
    #[instrument]
    pub fn new() -> Result<Self, String> {
        debug!("Creating template browser overlay");
        let registry = TemplateRegistry::new()
            .map_err(|e| format!("Failed to create template registry: {}", e))?;
        
        Ok(Self {
            registry,
            search_query: String::new(),
            selected_template: None,
            show_delete_confirm: false,
            delete_template_id: None,
        })
    }

    /// Renders a single template card in the grid.
    fn render_template_card(
        &mut self,
        ui: &mut Ui,
        template: &DrawingTemplate,
    ) -> Response {
        let card_size = Vec2::new(200.0, 150.0);
        let (rect, response) = ui.allocate_exact_size(card_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let is_selected = self.selected_template.as_ref() == Some(template.id());

            // Background
            let bg_color = if is_selected {
                Color32::from_rgb(50, 100, 150)
            } else if response.hovered() {
                Color32::from_rgb(40, 40, 45)
            } else {
                Color32::from_rgb(30, 30, 35)
            };
            ui.painter().rect_filled(rect, 5.0, bg_color);

            // Border
            let border_color = if is_selected {
                Color32::from_rgb(100, 150, 255)
            } else {
                visuals.bg_stroke.color
            };
            let stroke = egui::Stroke::new(2.0, border_color);
            ui.painter().rect_stroke(rect, 5.0, stroke, egui::StrokeKind::Outside);

            // Template info
            let text_pos = rect.min + Vec2::new(10.0, 10.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_TOP,
                template.name(),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );

            // Field count
            let field_count: usize = template.fields().len();
            let info_text = format!("{} fields, {} pages", field_count, template.page_count());
            let info_pos = rect.min + Vec2::new(10.0, 30.0);
            ui.painter().text(
                info_pos,
                egui::Align2::LEFT_TOP,
                info_text,
                egui::FontId::proportional(12.0),
                Color32::GRAY,
            );

            // Placeholder for thumbnail (future enhancement)
            let thumbnail_rect = Rect::from_min_size(
                rect.min + Vec2::new(10.0, 50.0),
                Vec2::new(180.0, 80.0),
            );
            ui.painter().rect_filled(
                thumbnail_rect,
                3.0,
                Color32::from_rgb(20, 20, 25),
            );
            ui.painter().text(
                thumbnail_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Preview",
                egui::FontId::proportional(10.0),
                Color32::DARK_GRAY,
            );
        }

        if response.clicked() {
            self.selected_template = Some(template.id().to_string());
            debug!(template_id = template.id(), "Template selected");
        }

        response
    }

    /// Renders the action buttons for the selected template.
    fn render_actions(&mut self, ui: &mut Ui) -> Option<TemplateBrowserAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            if ui.button("✏ Edit").clicked() {
                action = Some(TemplateBrowserAction::Edit);
            }
            if ui.button("📝 Fill Form").clicked() {
                action = Some(TemplateBrowserAction::Fill);
            }
            if ui.button("🗑 Delete").clicked() {
                self.show_delete_confirm = true;
                self.delete_template_id = self.selected_template.clone();
            }
            if ui.button("📋 Duplicate").clicked() {
                action = Some(TemplateBrowserAction::Duplicate);
            }
        });

        action
    }

    /// Renders the delete confirmation dialog.
    fn render_delete_confirmation(&mut self, ui: &mut Ui) -> Option<TemplateBrowserAction> {
        let mut action = None;

        if self.show_delete_confirm {
            egui::Window::new("Confirm Delete")
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Are you sure you want to delete this template?");
                    ui.label("This action cannot be undone.");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_delete_confirm = false;
                            self.delete_template_id = None;
                        }
                        if ui.button("Delete").clicked() {
                            action = Some(TemplateBrowserAction::Delete);
                            self.show_delete_confirm = false;
                        }
                    });
                });
        }

        action
    }
}

impl Default for TemplateBrowserOverlay {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            debug!(error = %e, "Failed to create template browser, using empty registry");
            Self {
                registry: TemplateRegistry::new().unwrap_or_else(|_| {
                    // This should never fail the second time, but if it does, panic
                    panic!("Failed to create template registry twice")
                }),
                search_query: String::new(),
                selected_template: None,
                show_delete_confirm: false,
                delete_template_id: None,
            }
        })
    }
}

/// Actions that can be taken from the template browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateBrowserAction {
    /// Edit the selected template
    Edit,
    /// Fill the selected template (create instance)
    Fill,
    /// Delete the selected template
    Delete,
    /// Duplicate the selected template
    Duplicate,
    /// Close the overlay
    Close,
}

impl Overlay for TemplateBrowserOverlay {
    fn title(&self) -> &str {
        "Template Browser"
    }

    fn show(&mut self, ctx: &egui::Context) -> OverlayResponse {
        let mut should_close = false;
        let mut action_taken: Option<TemplateBrowserAction> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("📚 Template Browser");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖ Close").clicked() {
                        should_close = true;
                    }
                });
            });

            ui.separator();

            // Search bar
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.search_query);
            });

            ui.add_space(10.0);

            // Template grid
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Load templates from registry
                let templates: Vec<_> = self.registry.list().iter().map(|t| (*t).clone()).collect();

                if templates.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label("No templates found");
                        ui.add_space(10.0);
                        ui.colored_label(
                            Color32::GRAY,
                            "Create a template by drawing shapes and adding them to a template",
                        );
                    });
                    return;
                }

                // Filter templates by search query
                let filtered_templates: Vec<_> = templates
                    .into_iter()
                    .filter(|t| {
                        if self.search_query.is_empty() {
                            true
                        } else {
                            t.name()
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                                || t.id()
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                        }
                    })
                    .collect();

                // Render template cards in grid
                egui::Grid::new("template_grid")
                    .spacing(Vec2::new(20.0, 20.0))
                    .show(ui, |ui| {
                        for (i, template) in filtered_templates.iter().enumerate() {
                            self.render_template_card(ui, template);

                            // 3 columns
                            if (i + 1) % 3 == 0 {
                                ui.end_row();
                            }
                        }
                    });

                ui.add_space(20.0);

                // Action buttons (only show if template selected)
                if self.selected_template.is_some() {
                    ui.separator();
                    ui.add_space(10.0);
                    ui.heading("Actions");
                    if let Some(action) = self.render_actions(ui) {
                        action_taken = Some(action);
                    }
                }
            });

            // Delete confirmation dialog
            if let Some(action) = self.render_delete_confirmation(ui) {
                action_taken = Some(action);
            }
        });

        // Handle actions
        if let Some(action) = action_taken {
            should_close = self.handle_action(action);
        }

        if should_close {
            OverlayResponse::Close
        } else {
            OverlayResponse::KeepOpen
        }
    }

    fn is_modal(&self) -> bool {
        true
    }
}

impl TemplateBrowserOverlay {
    /// Handles an action from the UI and returns whether the overlay should close.
    fn handle_action(&mut self, action: TemplateBrowserAction) -> bool {
        match action {
            TemplateBrowserAction::Close => true,
            TemplateBrowserAction::Edit => {
                debug!(template_id = ?self.selected_template, "Edit action triggered");
                // TODO: Load template to canvas for editing
                true
            }
            TemplateBrowserAction::Fill => {
                debug!(template_id = ?self.selected_template, "Fill action triggered");
                // TODO: Open data entry overlay
                true
            }
            TemplateBrowserAction::Delete => {
                if let Some(template_id) = &self.delete_template_id {
                    debug!(template_id, "Delete action triggered");
                    if let Err(e) = self.registry.delete_from_global(template_id) {
                        debug!(error = %e, "Failed to delete template");
                    } else {
                        self.selected_template = None;
                        self.delete_template_id = None;
                    }
                }
                false // Stay open after delete
            }
            TemplateBrowserAction::Duplicate => {
                debug!(template_id = ?self.selected_template, "Duplicate action triggered");
                // TODO: Duplicate template with new ID
                false // Stay open after duplicate
            }
        }
    }
}
