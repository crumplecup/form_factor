use form_factor_core::PageNavigation;
use tracing::instrument;

/// Renders page navigation controls for multi-page templates.
#[instrument(skip(ui, navigation))]
pub fn render_page_navigation(ui: &mut egui::Ui, navigation: &mut PageNavigation) {
    ui.horizontal(|ui| {
        ui.label(format!(
            "Page {} of {}",
            navigation.current_page() + 1,
            navigation.total_pages()
        ));

        ui.add_space(8.0);

        if ui
            .add_enabled(navigation.has_previous(), egui::Button::new("◀ Previous"))
            .clicked()
        {
            navigation.previous_page();
        }

        if ui
            .add_enabled(navigation.has_next(), egui::Button::new("Next ▶"))
            .clicked()
        {
            navigation.next_page();
        }

        ui.add_space(16.0);

        if ui.button("➕ Add Page").clicked() {
            navigation.add_page();
        }

        if ui
            .add_enabled(
                *navigation.total_pages() > 1,
                egui::Button::new("🗑 Remove Page"),
            )
            .clicked()
        {
            navigation.remove_current_page();
        }
    });
}

/// Renders page thumbnails for quick navigation.
#[instrument(skip(ui, navigation))]
pub fn render_page_thumbnails(ui: &mut egui::Ui, navigation: &mut PageNavigation) {
    ui.horizontal(|ui| {
        for page_idx in 0..*navigation.total_pages() {
            let is_current = page_idx == *navigation.current_page();
            let button_text = format!("{}", page_idx + 1);

            let button = if is_current {
                egui::Button::new(button_text).fill(ui.visuals().selection.bg_fill)
            } else {
                egui::Button::new(button_text)
            };

            if ui.add(button).clicked() {
                navigation.goto_page(page_idx);
            }
        }
    });
}
