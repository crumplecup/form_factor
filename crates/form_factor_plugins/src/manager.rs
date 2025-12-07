//! Plugin manager for coordinating multiple plugins.

use crate::{
    bus::EventBus,
    plugin::{Plugin, PluginContext},
};
use tracing::{debug, info, instrument, warn};

/// Manages the lifecycle and coordination of all plugins.
///
/// The plugin manager:
/// - Initializes plugins at startup
/// - Distributes events to all plugins
/// - Coordinates plugin rendering
/// - Handles plugin shutdown
pub struct PluginManager {
    /// Registered plugins
    plugins: Vec<Box<dyn Plugin>>,
    /// Event bus for plugin communication
    event_bus: EventBus,
}

impl PluginManager {
    /// Creates a new plugin manager.
    pub fn new() -> Self {
        info!("Initializing plugin manager");
        Self {
            plugins: Vec::new(),
            event_bus: EventBus::new(),
        }
    }

    /// Registers a plugin with the manager.
    ///
    /// The plugin will be initialized via `on_load` callback.
    #[instrument(skip(self, plugin), fields(plugin_name = plugin.name()))]
    pub fn register(&mut self, mut plugin: Box<dyn Plugin>) {
        let plugin_name = plugin.name().to_string();
        info!(plugin = %plugin_name, "Registering plugin");

        // Initialize the plugin
        let ctx = self.create_context();
        plugin.on_load(&ctx);

        self.plugins.push(plugin);
        debug!(plugin = %plugin_name, total = self.plugins.len(), "Plugin registered");
    }

    /// Returns the number of registered plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Returns the names of all registered plugins.
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Renders all enabled plugins.
    ///
    /// This should be called once per frame from the main UI loop.
    #[cfg(feature = "plugin-canvas")]
    #[instrument(skip(self, ui, canvas))]
    pub fn render_plugins(
        &mut self,
        ui: &mut egui::Ui,
        canvas: &form_factor_drawing::DrawingCanvas,
    ) {
        let ctx = PluginContext::with_canvas(self.event_bus.sender(), canvas);

        for plugin in &mut self.plugins {
            if plugin.is_enabled() {
                plugin.ui(ui, &ctx);
            }
        }
    }

    /// Renders all enabled plugins without canvas context.
    ///
    /// This should be called once per frame from the main UI loop.
    #[cfg(not(feature = "plugin-canvas"))]
    #[instrument(skip(self, ui))]
    pub fn render_plugins_no_canvas(&mut self, ui: &mut egui::Ui) {
        let ctx = PluginContext::new(self.event_bus.sender());

        for plugin in &mut self.plugins {
            if plugin.is_enabled() {
                plugin.ui(ui, &ctx);
            }
        }
    }

    /// Processes all pending events and distributes them to plugins.
    ///
    /// This should be called once per frame, typically before rendering.
    /// Plugins can emit new events in response to received events.
    #[instrument(skip(self))]
    pub fn process_events(&mut self) {
        let events = self.event_bus.drain_events();

        if events.is_empty() {
            return;
        }

        debug!(event_count = events.len(), "Processing events");

        // Create context once before loop to avoid borrow checker issues
        let sender = self.event_bus.sender();
        let ctx = PluginContext::new(sender);

        // Distribute each event to all plugins
        for event in &events {
            for plugin in &mut self.plugins {
                if let Some(response) = plugin.on_event(event, &ctx) {
                    debug!(
                        plugin = plugin.name(),
                        ?response,
                        "Plugin emitted response event"
                    );
                    ctx.events.emit(response);
                }
            }
        }
    }

    /// Notifies all plugins that state is being saved.
    #[instrument(skip(self))]
    pub fn save_plugins(&mut self) {
        info!("Saving plugin state");
        let sender = self.event_bus.sender();
        let ctx = PluginContext::new(sender);

        for plugin in &mut self.plugins {
            plugin.on_save(&ctx);
        }
    }

    /// Shuts down all plugins.
    ///
    /// This should be called when the application is closing.
    #[instrument(skip(self))]
    pub fn shutdown(&mut self) {
        info!("Shutting down plugins");
        let sender = self.event_bus.sender();
        let ctx = PluginContext::new(sender);

        for plugin in &mut self.plugins {
            debug!(plugin = plugin.name(), "Shutting down plugin");
            plugin.on_shutdown(&ctx);
        }

        self.plugins.clear();
        info!("All plugins shut down");
    }

    /// Gets a reference to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Gets a mutable reference to the event bus.
    pub fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    /// Creates a plugin context for event handling.
    #[cfg(feature = "plugin-canvas")]
    fn create_context(&self) -> PluginContext<'_> {
        PluginContext::new(self.event_bus.sender())
    }

    /// Creates a plugin context for event handling.
    #[cfg(not(feature = "plugin-canvas"))]
    fn create_context(&self) -> PluginContext {
        PluginContext::new(self.event_bus.sender())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() {
            warn!(
                "PluginManager dropped with {} plugins still registered. \
                 Call shutdown() explicitly to ensure clean plugin shutdown.",
                self.plugins.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AppEvent;

    struct MockPlugin {
        name: String,
        events_received: Vec<AppEvent>,
    }

    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext<'_>) {}

        fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext<'_>) -> Option<AppEvent> {
            self.events_received.push(event.clone());
            None
        }
    }

    #[test]
    fn test_register_plugins() {
        let mut manager = PluginManager::new();

        let plugin1 = Box::new(MockPlugin {
            name: "plugin1".to_string(),
            events_received: Vec::new(),
        });

        let plugin2 = Box::new(MockPlugin {
            name: "plugin2".to_string(),
            events_received: Vec::new(),
        });

        manager.register(plugin1);
        manager.register(plugin2);

        assert_eq!(manager.plugin_count(), 2);
        assert_eq!(manager.plugin_names(), vec!["plugin1", "plugin2"]);
    }

    #[test]
    fn test_event_distribution() {
        let mut manager = PluginManager::new();

        let plugin = Box::new(MockPlugin {
            name: "test".to_string(),
            events_received: Vec::new(),
        });

        manager.register(plugin);

        // Send an event
        let sender = manager.event_bus().sender();
        sender.emit(AppEvent::SelectionCleared);

        // Process events
        manager.process_events();

        // Verify the plugin was registered
        assert_eq!(manager.plugin_count(), 1);
        assert_eq!(manager.plugin_names(), vec!["test"]);
    }
}
