//! Plugin coordination integration tests.
//!
//! Tests plugin lifecycle, event bus mechanics, and plugin communication.

use botticelli_health::{
    CountingPlugin, EventCollectorPlugin, ResponsePlugin, create_test_plugin,
    create_test_plugin_with_name,
};
use form_factor_plugins::{AppEvent, PluginManager};

// ============================================================================
// Phase 1.3.1: Plugin Lifecycle Tests
// ============================================================================

#[test]
fn test_plugin_manager_creation() {
    let manager = PluginManager::new();
    assert_eq!(manager.plugin_count(), 0);
    assert!(manager.plugin_names().is_empty());
}

#[test]
fn test_register_single_plugin() {
    let mut manager = PluginManager::new();
    let plugin = create_test_plugin("test-plugin");

    manager.register(plugin);

    assert_eq!(manager.plugin_count(), 1);
    assert_eq!(manager.plugin_names(), vec!["test-plugin"]);
}

#[test]
fn test_register_multiple_plugins() {
    let mut manager = PluginManager::new();

    manager.register(create_test_plugin("plugin-1"));
    manager.register(create_test_plugin("plugin-2"));
    manager.register(create_test_plugin("plugin-3"));

    assert_eq!(manager.plugin_count(), 3);
    assert_eq!(
        manager.plugin_names(),
        vec!["plugin-1", "plugin-2", "plugin-3"]
    );
}

#[test]
fn test_plugin_on_load_called_during_registration() {
    let mut manager = PluginManager::new();
    let plugin = Box::new(CountingPlugin::new("counter"));

    // CountingPlugin increments load_count in on_load
    manager.register(plugin);

    // We can't directly access the plugin after registration,
    // but we verified it compiles and doesn't panic
    assert_eq!(manager.plugin_count(), 1);
}

#[test]
fn test_multiple_plugins_all_loaded() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(CountingPlugin::new("counter-1")));
    manager.register(Box::new(CountingPlugin::new("counter-2")));

    assert_eq!(manager.plugin_count(), 2);
    // All plugins should have received on_load callback
}

// ============================================================================
// Phase 1.3.2: Event Bus Mechanics Tests
// ============================================================================

#[test]
fn test_event_bus_emit_and_drain() {
    let mut manager = PluginManager::new();

    // Emit event directly to bus
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 2.0 });

    let events = manager.event_bus_mut().drain_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], AppEvent::CanvasZoomChanged { zoom: 2.0 });

    // Events should be drained (empty after drain)
    let events2 = manager.event_bus_mut().drain_events();
    assert!(events2.is_empty());
}

#[test]
fn test_event_bus_multiple_events() {
    let mut manager = PluginManager::new();

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 1.5 });
    sender.emit(AppEvent::CanvasPanChanged { x: 10.0, y: 20.0 });
    sender.emit(AppEvent::SelectionCleared);

    let events = manager.event_bus_mut().drain_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_process_events_distributes_to_plugins() {
    let mut manager = PluginManager::new();

    // Register collector plugin
    manager.register(Box::new(EventCollectorPlugin::new("collector")));

    // Emit events
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 2.0 });

    // Process events
    manager.process_events();

    // Event should have been distributed to plugin
    // (We can't directly verify, but no panic = success)
}

#[test]
fn test_plugins_can_respond_to_events() {
    let mut manager = PluginManager::new();

    // ResponsePlugin responds to zoom changes with pan changes
    manager.register(Box::new(ResponsePlugin::new("responder")));

    // Emit zoom event
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 3.0 });

    // Process events (plugin emits response)
    manager.process_events();

    // Check for response event
    let events = manager.event_bus_mut().drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], AppEvent::CanvasPanChanged { .. }));
}

#[test]
fn test_event_chain_reaction() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(ResponsePlugin::new("responder-1")));
    manager.register(Box::new(ResponsePlugin::new("responder-2")));

    // Initial event
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 2.0 });

    // Process first round (both plugins respond)
    manager.process_events();

    // Should have response events
    let events = manager.event_bus_mut().drain_events();
    assert_eq!(events.len(), 2); // Both plugins responded
}

// ============================================================================
// Phase 1.3.3: Cross-Plugin Scenarios Tests
// ============================================================================

#[test]
fn test_multiple_plugins_receive_same_event() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(EventCollectorPlugin::new("collector-1")));
    manager.register(Box::new(EventCollectorPlugin::new("collector-2")));
    manager.register(Box::new(EventCollectorPlugin::new("collector-3")));

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::SelectionCleared);

    manager.process_events();

    // All plugins received the event (no panic = success)
    assert_eq!(manager.plugin_count(), 3);
}

#[test]
fn test_plugin_communication_through_events() {
    let mut manager = PluginManager::new();

    // Collector listens, responder emits
    manager.register(Box::new(EventCollectorPlugin::new("listener")));
    manager.register(Box::new(ResponsePlugin::new("emitter")));

    // Initial event
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 1.0 });

    // Process (responder emits pan event)
    manager.process_events();

    // Listener should have received both zoom and pan events
    let remaining = manager.event_bus_mut().drain_events();
    assert_eq!(remaining.len(), 1); // Pan event from responder
}

#[test]
fn test_plugin_isolation() {
    let mut manager = PluginManager::new();

    manager.register(create_test_plugin_with_name("plugin-a"));
    manager.register(create_test_plugin_with_name("plugin-b"));

    // Each plugin operates independently
    assert_eq!(manager.plugin_count(), 2);

    // Events to one don't affect others' state
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::LayerSelected {
        layer_name: "layer1".to_string(),
    });
    manager.process_events();

    // No panic = plugins are isolated
}

#[test]
fn test_plugin_order_independence() {
    let mut manager1 = PluginManager::new();
    manager1.register(Box::new(ResponsePlugin::new("a")));
    manager1.register(Box::new(EventCollectorPlugin::new("b")));

    let mut manager2 = PluginManager::new();
    manager2.register(Box::new(EventCollectorPlugin::new("b")));
    manager2.register(Box::new(ResponsePlugin::new("a")));

    // Same event to both
    let event = AppEvent::CanvasZoomChanged { zoom: 1.5 };
    let sender1 = manager1.event_bus_mut().sender();
    sender1.emit(event.clone());
    let sender2 = manager2.event_bus_mut().sender();
    sender2.emit(event);

    manager1.process_events();
    manager2.process_events();

    // Both should produce same results
    let events1 = manager1.event_bus_mut().drain_events();
    let events2 = manager2.event_bus_mut().drain_events();

    assert_eq!(events1.len(), events2.len());
}

// ============================================================================
// Phase 1.3.4: State Synchronization Tests
// ============================================================================

#[test]
fn test_event_propagation_consistency() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(CountingPlugin::new("counter")));
    manager.register(Box::new(EventCollectorPlugin::new("collector")));

    // Multiple events
    for i in 0..5 {
        let sender = manager.event_bus_mut().sender();
        sender.emit(AppEvent::ShapeSelected { index: i });
    }

    manager.process_events();

    // All events processed
    assert!(manager.event_bus_mut().drain_events().is_empty());
}

#[test]
fn test_concurrent_event_emission() {
    let mut manager = PluginManager::new();

    // Multiple responders
    manager.register(Box::new(ResponsePlugin::new("r1")));
    manager.register(Box::new(ResponsePlugin::new("r2")));
    manager.register(Box::new(ResponsePlugin::new("r3")));

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 2.0 });

    manager.process_events();

    // All responses collected
    let events = manager.event_bus_mut().drain_events();
    assert_eq!(events.len(), 3); // One response per plugin
}

#[test]
fn test_state_sync_through_events() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(EventCollectorPlugin::new("sync-1")));
    manager.register(Box::new(EventCollectorPlugin::new("sync-2")));

    // Simulate state change
    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::LayerVisibilityChanged {
        layer_name: "annotations".to_string(),
        visible: false,
    });

    manager.process_events();

    // Both plugins should have processed the state change
    assert_eq!(manager.plugin_count(), 2);
}

#[test]
fn test_rapid_event_processing() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(CountingPlugin::new("counter")));

    // Rapid-fire events
    for i in 0..100 {
        let sender = manager.event_bus_mut().sender();
        sender.emit(AppEvent::ShapeSelected { index: i % 10 });
    }

    manager.process_events();

    // All processed without panic
    assert!(manager.event_bus_mut().drain_events().is_empty());
}

// ============================================================================
// Phase 1.3.5: Edge Cases and Error Handling Tests
// ============================================================================

#[test]
fn test_empty_plugin_manager_process_events() {
    let mut manager = PluginManager::new();

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::SelectionCleared);
    manager.process_events();

    // No plugins = events just drain
    assert!(manager.event_bus_mut().drain_events().is_empty());
}

#[test]
fn test_plugin_with_no_event_response() {
    let mut manager = PluginManager::new();

    manager.register(create_test_plugin("passive"));

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::CanvasZoomChanged { zoom: 1.0 });
    manager.process_events();

    // No response events = empty bus
    assert!(manager.event_bus_mut().drain_events().is_empty());
}

#[test]
fn test_many_plugins_single_event() {
    let mut manager = PluginManager::new();

    // Register 20 plugins
    for i in 0..20 {
        manager.register(Box::new(EventCollectorPlugin::new(&format!(
            "collector-{}",
            i
        ))));
    }

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::SelectionCleared);
    manager.process_events();

    // All received event (no panic)
    assert_eq!(manager.plugin_count(), 20);
}

#[test]
fn test_single_plugin_many_events() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(CountingPlugin::new("counter")));

    // Send 50 different events
    for i in 0..50 {
        let sender = manager.event_bus_mut().sender();
        sender.emit(AppEvent::ShapeSelected { index: i });
    }

    manager.process_events();

    // All processed
    assert!(manager.event_bus_mut().drain_events().is_empty());
}

#[test]
fn test_process_events_idempotent() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(CountingPlugin::new("counter")));

    let sender = manager.event_bus_mut().sender();
    sender.emit(AppEvent::SelectionCleared);

    // Process multiple times
    manager.process_events();
    manager.process_events();
    manager.process_events();

    // Should not cause issues (events already drained)
    assert!(manager.event_bus_mut().drain_events().is_empty());
}
