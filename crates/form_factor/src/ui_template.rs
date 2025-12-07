use crate::AppContext;

#[cfg(feature = "plugins")]
use form_factor_drawing::AppState;
#[cfg(feature = "plugins")]
use form_factor_plugins::TemplateBrowserPlugin;

/// Handles UI updates for template manager mode
#[tracing::instrument(skip(app_state, template_browser, canvas, ctx))]
#[cfg(feature = "plugins")]
pub fn update_template_manager_mode(
    app_state: &AppState,
    template_browser: &mut Option<TemplateBrowserPlugin>,
    canvas: &mut form_factor::DrawingCanvas,
    ctx: &AppContext,
) {
    // Create template browser if it doesn't exist
    if template_browser.is_none() {
        *template_browser = Some(TemplateBrowserPlugin::new());
        tracing::info!("Created template browser");
    }

    // Render template browser in right panel
    if let Some(browser) = template_browser {
        browser.show(ctx.egui_ctx());
    }

    // TODO: Render page navigation at the top if template is loaded
    if let Some(_template) = app_state.current_template() {
        // egui::TopBottomPanel::top("page_navigation_panel").show(ctx.egui_ctx(), |_ui| {
        //     use form_factor_plugins::template_ui;
        //     let mut navigation = form_factor_core::PageNavigation::new(template.page_count());
        //     template_ui::render_page_navigation(ui, &mut navigation);
        //     template_ui::render_page_thumbnails(ui, &mut navigation);
        // });
    }

    // Render canvas in central panel for template viewing/editing
    egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
        canvas.ui(ui);
    });
}

/// Handles UI updates for template manager mode (no-plugins fallback)
#[tracing::instrument(skip(ctx))]
#[cfg(not(feature = "plugins"))]
pub fn update_template_manager_mode(ctx: &AppContext) {
    egui::CentralPanel::default().show(ctx.egui_ctx(), |ui| {
        ui.label("Template manager requires plugins feature");
    });
}
