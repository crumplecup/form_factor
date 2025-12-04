//! OCR plugin for text extraction.
//!
//! This plugin provides UI for:
//! - OCR text extraction
//! - Extracted text display
//! - Language selection

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use tracing::{debug, instrument};

/// Plugin for OCR text extraction.
///
/// Provides controls for:
/// - Running OCR on detected regions
/// - Displaying extracted text
/// - Configuring OCR settings
pub struct OcrPlugin {
    /// Extracted text content
    extracted_text: Vec<String>,
}

impl OcrPlugin {
    /// Creates a new OCR plugin.
    pub fn new() -> Self {
        Self {
            extracted_text: Vec::new(),
        }
    }
}

impl Default for OcrPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for OcrPlugin {
    fn name(&self) -> &str {
        "ocr"
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("OCR");

            if ui.button("Extract Text").clicked() {
                debug!("OCR extraction requested");
                ctx.events.emit(AppEvent::OcrExtractionRequested);
            }

            ui.separator();

            if !self.extracted_text.is_empty() {
                ui.label("Extracted text:");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, text) in self.extracted_text.iter().enumerate() {
                            ui.label(format!("{}: {}", i + 1, text));
                        }
                    });
            } else {
                ui.label("No text extracted yet");
            }
        });
    }

    #[instrument(skip(self, _ctx), fields(plugin = "ocr"))]
    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::Custom {
                plugin,
                event_type,
                data,
            } if plugin == "ocr" && event_type == "text_extracted" => {
                debug!("OCR text extracted");
                if let Ok(text) = serde_json::from_str::<Vec<String>>(data) {
                    self.extracted_text = text;
                }
                None
            }
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "OCR text extraction from images"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_plugin_creation() {
        let plugin = OcrPlugin::new();
        assert_eq!(plugin.name(), "ocr");
        assert!(plugin.extracted_text.is_empty());
    }
}
