//! Plugin trait and context.

use crate::{bus::EventSender, event::AppEvent};

/// Context provided to plugins during rendering and event handling.
///
/// The plugin context gives plugins access to application state and
/// the ability to send events to other plugins.
#[cfg(feature = "plugin-canvas")]
pub struct PluginContext<'a> {
    /// Event sender for publishing events
    pub events: EventSender,
    /// Optional reference to the drawing canvas (for UI rendering)
    pub canvas: Option<&'a form_factor_drawing::DrawingCanvas>,
}

/// Context provided to plugins during rendering and event handling.
///
/// The plugin context gives plugins access to application state and
/// the ability to send events to other plugins.
#[cfg(not(feature = "plugin-canvas"))]
pub struct PluginContext {
    /// Event sender for publishing events
    pub events: EventSender,
}

#[cfg(feature = "plugin-canvas")]
impl<'a> PluginContext<'a> {
    /// Creates a new plugin context for event handling (without canvas access).
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            canvas: None,
        }
    }

    /// Creates a new plugin context with canvas access for UI rendering.
    pub fn with_canvas(
        events: EventSender,
        canvas: &'a form_factor_drawing::DrawingCanvas,
    ) -> Self {
        Self {
            events,
            canvas: Some(canvas),
        }
    }
}

#[cfg(not(feature = "plugin-canvas"))]
impl PluginContext {
    /// Creates a new plugin context for event handling (without canvas access).
    pub fn new(events: EventSender) -> Self {
        Self { events }
    }
}

/// Trait that all plugins must implement.
///
/// Plugins are modular UI components that can be enabled/disabled at compile time
/// using feature flags. Each plugin has a unique name and can render UI, handle
/// events, and communicate with other plugins through the event bus.
pub trait Plugin: Send {
    /// Returns the unique name of this plugin.
    ///
    /// Plugin names should be lowercase with hyphens (e.g., "canvas-tools").
    fn name(&self) -> &str;

    /// Renders the plugin's UI.
    ///
    /// This method is called every frame to allow the plugin to draw its interface.
    ///
    /// # Arguments
    /// * `ui` - The egui UI context for rendering
    /// * `ctx` - Plugin context with access to events and app state
    #[cfg(feature = "plugin-canvas")]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext);

    /// Renders the plugin's UI.
    ///
    /// This method is called every frame to allow the plugin to draw its interface.
    ///
    /// # Arguments
    /// * `ui` - The egui UI context for rendering
    /// * `ctx` - Plugin context with access to events and app state
    #[cfg(not(feature = "plugin-canvas"))]
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PluginContext);

    /// Handles an event from the event bus.
    ///
    /// Plugins can react to events from other plugins or the application.
    /// They can also emit new events in response.
    ///
    /// # Arguments
    /// * `event` - The event to handle
    /// * `ctx` - Plugin context with access to events
    ///
    /// # Returns
    /// The plugin can optionally return a new event to emit in response.
    #[cfg(feature = "plugin-canvas")]
    fn on_event(&mut self, _event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        None
    }

    /// Handles an event from the event bus.
    ///
    /// Plugins can react to events from other plugins or the application.
    /// They can also emit new events in response.
    ///
    /// # Arguments
    /// * `event` - The event to handle
    /// * `ctx` - Plugin context with access to events
    ///
    /// # Returns
    /// The plugin can optionally return a new event to emit in response.
    #[cfg(not(feature = "plugin-canvas"))]
    fn on_event(&mut self, _event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        None
    }

    /// Called when the plugin is first loaded.
    ///
    /// Use this to initialize plugin state or subscribe to events.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(feature = "plugin-canvas")]
    fn on_load(&mut self, _ctx: &PluginContext) {}

    /// Called when the plugin is first loaded.
    ///
    /// Use this to initialize plugin state or subscribe to events.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(not(feature = "plugin-canvas"))]
    fn on_load(&mut self, _ctx: &PluginContext) {}

    /// Called before the application saves state.
    ///
    /// Plugins can use this to persist their state or clean up resources.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(feature = "plugin-canvas")]
    fn on_save(&mut self, _ctx: &PluginContext) {}

    /// Called before the application saves state.
    ///
    /// Plugins can use this to persist their state or clean up resources.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(not(feature = "plugin-canvas"))]
    fn on_save(&mut self, _ctx: &PluginContext) {}

    /// Called when the application is shutting down.
    ///
    /// Plugins should clean up any resources they hold.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(feature = "plugin-canvas")]
    fn on_shutdown(&mut self, _ctx: &PluginContext) {}

    /// Called when the application is shutting down.
    ///
    /// Plugins should clean up any resources they hold.
    ///
    /// # Arguments
    /// * `ctx` - Plugin context with access to events
    #[cfg(not(feature = "plugin-canvas"))]
    fn on_shutdown(&mut self, _ctx: &PluginContext) {}

    /// Returns whether this plugin should be displayed in the UI.
    ///
    /// By default, all plugins are enabled. Override this to add
    /// conditional visibility logic.
    fn is_enabled(&self) -> bool {
        true
    }

    /// Returns a short description of what this plugin does.
    ///
    /// Used for help text and debugging.
    fn description(&self) -> &str {
        "No description available"
    }
}

/// Helper for creating plugin instances with builder pattern.
pub trait PluginBuilder: Send + Sync {
    /// Builds a new instance of the plugin.
    fn build(&self) -> Box<dyn Plugin>;

    /// Returns the name of the plugin this builder creates.
    fn plugin_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        call_count: usize,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext) {
            self.call_count += 1;
        }

        fn description(&self) -> &str {
            "A test plugin"
        }
    }

    #[test]
    fn test_plugin_basic() {
        let plugin = TestPlugin {
            name: "test".to_string(),
            call_count: 0,
        };

        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.description(), "A test plugin");
        assert!(plugin.is_enabled());
        assert_eq!(plugin.call_count, 0);
    }
}
