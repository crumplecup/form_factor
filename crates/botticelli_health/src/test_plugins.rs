//! Test plugin implementations for testing plugin coordination.

use form_factor_plugins::{AppEvent, Plugin, PluginContext};

// ============================================================================
// Basic Test Plugin
// ============================================================================

/// Simple plugin for basic lifecycle testing.
pub struct TestPlugin {
    name: String,
    enabled: bool,
}

impl TestPlugin {
    /// Creates a new test plugin.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
        }
    }
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext) {
        // No-op for tests
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Helper to create a test plugin boxed for registration.
pub fn create_test_plugin(name: &str) -> Box<dyn Plugin> {
    Box::new(TestPlugin::new(name))
}

/// Helper to create a test plugin with a specific name.
pub fn create_test_plugin_with_name(name: &str) -> Box<dyn Plugin> {
    Box::new(TestPlugin::new(name))
}

// ============================================================================
// Counting Plugin (tracks lifecycle callbacks)
// ============================================================================

/// Plugin that counts how many times each callback is invoked.
pub struct CountingPlugin {
    name: String,
    enabled: bool,
    load_count: usize,
    event_count: usize,
}

impl CountingPlugin {
    /// Creates a new counting plugin.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            load_count: 0,
            event_count: 0,
        }
    }


}

impl Plugin for CountingPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext) {
        // No-op for tests
    }

    fn on_load(&mut self, _ctx: &PluginContext) {
        self.load_count += 1;
    }

    fn on_event(&mut self, _event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        self.event_count += 1;
        None
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ============================================================================
// Event Collector Plugin (collects events for inspection)
// ============================================================================

/// Plugin that collects all events it receives.
pub struct EventCollectorPlugin {
    name: String,
    enabled: bool,
    events: Vec<AppEvent>,
}

impl EventCollectorPlugin {
    /// Creates a new event collector plugin.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            events: Vec::new(),
        }
    }


}

impl Plugin for EventCollectorPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext) {
        // No-op for tests
    }

    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        self.events.push(event.clone());
        None
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ============================================================================
// Response Plugin (emits events in response to other events)
// ============================================================================

/// Plugin that responds to specific events with other events.
///
/// Responds to CanvasZoomChanged with CanvasPanChanged.
pub struct ResponsePlugin {
    name: String,
    enabled: bool,
}

impl ResponsePlugin {
    /// Creates a new response plugin.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
        }
    }
}

impl Plugin for ResponsePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &PluginContext) {
        // No-op for tests
    }

    fn on_event(&mut self, event: &AppEvent, _ctx: &PluginContext) -> Option<AppEvent> {
        match event {
            AppEvent::CanvasZoomChanged { zoom } => {
                // Respond with pan change
                Some(AppEvent::CanvasPanChanged {
                    x: *zoom * 10.0,
                    y: *zoom * 10.0,
                })
            }
            _ => None,
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
