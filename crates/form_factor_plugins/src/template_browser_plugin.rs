use egui::{Context, ScrollArea, Ui};
use form_factor_core::{SortOrder, TemplateBrowser, TemplateEntry};
use tracing::instrument;

/// Plugin for browsing and managing templates.
#[derive(Debug)]
pub struct TemplateBrowserPlugin {
    browser: TemplateBrowser,
}

impl TemplateBrowserPlugin {
    /// Creates a new template browser plugin.
    pub fn new() -> Self {
        Self {
            browser: TemplateBrowser::new(),
        }
    }

    /// Gets a reference to the browser state.
    pub fn browser(&self) -> &TemplateBrowser {
        &self.browser
    }

    /// Gets a mutable reference to the browser state.
    pub fn browser_mut(&mut self) -> &mut TemplateBrowser {
        &mut self.browser
    }

    /// Renders the template browser UI.
    #[instrument(skip(self, ctx))]
    pub fn show(&mut self, ctx: &Context) {
        egui::SidePanel::right("template_browser_panel")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.show_content(ui);
            });
    }

    /// Renders the browser content.
    fn show_content(&mut self, ui: &mut Ui) {
        ui.heading("Templates");

        ui.separator();

        // Search/filter box
        let mut filter_text = self.browser.filter_text().clone();
        ui.horizontal(|ui| {
            ui.label("🔍");
            if ui.text_edit_singleline(&mut filter_text).changed() {
                self.browser.set_filter_text(filter_text);
            }
        });

        ui.add_space(4.0);

        // Sort order selector
        ui.horizontal(|ui| {
            ui.label("Sort:");
            egui::ComboBox::from_id_salt("sort_order")
                .selected_text(self.browser.sort_order().display_name())
                .show_ui(ui, |ui| {
                    for order in SortOrder::all() {
                        if ui
                            .selectable_label(
                                *self.browser.sort_order() == *order,
                                order.display_name(),
                            )
                            .clicked()
                        {
                            self.browser.set_sort_order(*order);
                        }
                    }
                });
        });

        ui.separator();

        // Template list
        let filtered = self.browser.filtered_templates();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if filtered.is_empty() {
                    ui.label("No templates found");
                } else {
                    for (idx, entry) in &filtered {
                        self.show_template_entry(ui, *idx, entry);
                    }
                }
            });

        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ New").clicked() {
                // TODO: Emit event to create new template
            }

            if ui.button("📁 Import").clicked() {
                // TODO: Emit event to import template
            }

            if self.browser.selected_template().is_some()
                && ui.button("🗑 Delete").clicked()
            {
                // TODO: Emit event to delete selected template
            }
        });
    }

    /// Renders a single template entry.
    fn show_template_entry(&self, ui: &mut Ui, index: usize, entry: &TemplateEntry) {
        let is_selected = self.browser.selected_index() == &Some(index);

        let response = ui.selectable_label(is_selected, entry.metadata().name());

        if response.clicked() {
            // Note: This is read-only for now. Selection handled elsewhere.
        }

        if response.double_clicked() {
            // Note: Double-click handled elsewhere.
        }

        // Show metadata on hover
        let response = response.on_hover_ui(|ui| {
            ui.label(format!("Version: {}", entry.metadata().version()));
            ui.label(format!("Pages: {}", entry.metadata().page_count()));
            ui.label(format!("Fields: {}", entry.metadata().field_count()));

            if let Some(desc) = entry.metadata().description() {
                ui.separator();
                ui.label(desc);
            }

            if !entry.metadata().tags().is_empty() {
                ui.separator();
                ui.label(format!("Tags: {}", entry.metadata().tags().join(", ")));
            }
        });

        // Context menu
        response.context_menu(|ui| {
            if ui.button("Open").clicked() {
                ui.close();
            }

            if ui.button("Duplicate").clicked() {
                ui.close();
            }

            if ui.button("Export").clicked() {
                ui.close();
            }

            ui.separator();

            if ui.button("Delete").clicked() {
                ui.close();
            }
        });
    }
}

impl Default for TemplateBrowserPlugin {
    fn default() -> Self {
        Self::new()
    }
}
