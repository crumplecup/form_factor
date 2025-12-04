//! Detection plugin for computer vision features.
//!
//! This plugin provides UI for:
//! - Text detection
//! - Logo detection
//! - Detection results display

use crate::{
    event::AppEvent,
    plugin::{Plugin, PluginContext},
};
use tracing::{debug, instrument};

/// Plugin for computer vision detection features.
///
/// Provides buttons and status for:
/// - Text detection using OpenCV
/// - Logo detection
/// - Detection count display
pub struct DetectionPlugin {
    /// Number of text detections found
    text_count: usize,
    /// Number of logo detections found
    logo_count: usize,
}

impl DetectionPlugin {
    /// Creates a new detection plugin.
    pub fn new() -> Self {
        Self {
            text_count: 0,
            logo_count: 0,
        }
    }
}

impl Default for DetectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DetectionPlugin {
    fn name(&self) -> &str {
        "detection"
    }

    #[instrument(skip(self, ui, ctx))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext) {
        ui.group(|ui| {
            ui.heading("Detection");

            ui.horizontal(|ui| {
                if ui.button("Detect Text").clicked() {
                    debug!("Text detection requested");
                    ctx.events.emit(AppEvent::TextDetectionRequested);
                }

                if ui.button("Detect Logos").clicked() {
                    debug!("Logo detection requested");
                    ctx.events.emit(AppEvent::LogoDetectionRequested);
                }
            });

            ui.separator();

            ui.label(format!("Text regions: {}", self.text_count));
            ui.label(format!("Logos: {}", self.logo_count));
        });
    }

    #[instrument(skip(self, _ctx), fields(plugin = "detection"))]
    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::DetectionComplete {
                count,
                detection_type,
            } => {
                debug!(count, detection_type, "Detection completed");
                if detection_type.contains("text") || detection_type.contains("Text") {
                    self.text_count = *count;
                } else if detection_type.contains("logo") || detection_type.contains("Logo") {
                    self.logo_count = *count;
                }
                None
            }
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "Computer vision detection features"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_plugin_creation() {
        let plugin = DetectionPlugin::new();
        assert_eq!(plugin.name(), "detection");
        assert_eq!(plugin.text_count, 0);
        assert_eq!(plugin.logo_count, 0);
    }

    #[test]
    fn test_detection_complete_event() {
        let mut plugin = DetectionPlugin::new();
        let (sender, _rx) = crate::EventSender::new_test();
        let ctx = PluginContext::new(sender);

        let event = AppEvent::DetectionComplete {
            count: 5,
            detection_type: "text".to_string(),
        };
        plugin.on_event(&event, &ctx);

        assert_eq!(plugin.text_count, 5);
        assert_eq!(plugin.logo_count, 0);
    }
}
