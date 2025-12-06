//! Plugin test helpers for event bus and plugin coordination testing
//!
//! This module provides utilities for testing plugin interactions, event delivery,
//! and plugin manager coordination.

use form_factor_plugins::{AppEvent, EventBus, Plugin, PluginManager};

/// Creates a plugin manager with no plugins registered
///
/// Useful as a base for tests that want to register specific plugins.
///
/// # Example
/// ```
/// use botticelli_health::create_empty_plugin_manager;
///
/// let manager = create_empty_plugin_manager();
/// assert_eq!(manager.plugin_count(), 0);
/// ```
pub fn create_empty_plugin_manager() -> PluginManager {
    PluginManager::new()
}

/// Creates a plugin manager with standard plugins registered
///
/// Note: This requires the actual plugin types to be created.
/// Currently returns an empty manager as a placeholder.
///
/// TODO: Implement when plugin instantiation is needed for tests
///
/// # Example
/// ```
/// use botticelli_health::create_test_plugin_manager;
///
/// let manager = create_test_plugin_manager();
/// // Would have plugins registered in full implementation
/// ```
pub fn create_test_plugin_manager() -> PluginManager {
    // TODO: Register standard test plugins when needed
    // For now, return empty manager
    create_empty_plugin_manager()
}

/// Creates a plugin manager with specific plugins
///
/// # Arguments
/// * `plugins` - Vector of boxed plugins to register
///
/// # Example
/// ```
/// use botticelli_health::create_plugin_manager_with;
/// use form_factor_plugins::Plugin;
///
/// let plugins: Vec<Box<dyn Plugin>> = vec![]; // Add test plugins
/// let manager = create_plugin_manager_with(plugins);
/// ```
pub fn create_plugin_manager_with(plugins: Vec<Box<dyn Plugin>>) -> PluginManager {
    let mut manager = PluginManager::new();
    for plugin in plugins {
        manager.register(plugin);
    }
    manager
}

/// Collects all pending events from an event bus
///
/// Drains the event queue and returns all events.
///
/// # Arguments
/// * `event_bus` - The event bus to drain
///
/// # Returns
/// Vector of all events that were in the queue
///
/// # Example
/// ```
/// use botticelli_health::collect_events;
/// use form_factor_plugins::{EventBus, AppEvent};
///
/// let mut bus = EventBus::new();
/// let sender = bus.sender();
/// sender.emit(AppEvent::SelectionCleared);
///
/// let events = collect_events(&mut bus);
/// assert_eq!(events.len(), 1);
/// ```
pub fn collect_events(event_bus: &mut EventBus) -> Vec<AppEvent> {
    event_bus.drain_events()
}

/// Emits an event and processes it through the plugin manager
///
/// Useful for testing event flow through plugins.
///
/// # Arguments
/// * `manager` - The plugin manager to process events through
/// * `event` - The event to emit
///
/// # Example
/// ```
/// use botticelli_health::{create_test_plugin_manager, emit_and_process};
/// use form_factor_plugins::AppEvent;
///
/// let mut manager = create_test_plugin_manager();
/// emit_and_process(&mut manager, AppEvent::SelectionCleared);
/// ```
pub fn emit_and_process(manager: &mut PluginManager, event: AppEvent) {
    let sender = manager.event_bus().sender();
    sender.emit(event);
    manager.process_events();
}

/// Emits multiple events and processes them through the plugin manager
///
/// # Arguments
/// * `manager` - The plugin manager to process events through
/// * `events` - Vector of events to emit
pub fn emit_and_process_all(manager: &mut PluginManager, events: Vec<AppEvent>) {
    let sender = manager.event_bus().sender();
    for event in events {
        sender.emit(event);
    }
    manager.process_events();
}

/// Asserts that a specific event was emitted
///
/// Searches through the event list for an exact match.
///
/// # Panics
/// Panics if the expected event is not found
///
/// # Arguments
/// * `events` - The list of events to search
/// * `expected` - The event to look for
///
/// # Example
/// ```
/// use botticelli_health::{collect_events, assert_event_emitted};
/// use form_factor_plugins::{EventBus, AppEvent};
///
/// let mut bus = EventBus::new();
/// let sender = bus.sender();
/// sender.emit(AppEvent::SelectionCleared);
///
/// let events = collect_events(&mut bus);
/// assert_event_emitted(&events, &AppEvent::SelectionCleared);
/// ```
pub fn assert_event_emitted(events: &[AppEvent], expected: &AppEvent) {
    assert!(
        events.contains(expected),
        "Expected event {:?} not found in events: {:?}",
        expected,
        events
    );
}

/// Asserts that a specific event was NOT emitted
///
/// # Panics
/// Panics if the event is found
pub fn assert_event_not_emitted(events: &[AppEvent], unexpected: &AppEvent) {
    assert!(
        !events.contains(unexpected),
        "Unexpected event {:?} found in events: {:?}",
        unexpected,
        events
    );
}

/// Asserts that exactly N events were emitted
///
/// # Panics
/// Panics if the event count doesn't match
pub fn assert_event_count(events: &[AppEvent], expected: usize) {
    let actual = events.len();
    assert_eq!(
        actual, expected,
        "Expected {} events, but found {}",
        expected, actual
    );
}

/// Filters events by type (using pattern matching helper)
///
/// Returns events that match a specific discriminant.
/// This is useful when you want to find all ZoomChanged events, for example.
///
/// # Example
/// ```
/// use botticelli_health::filter_events_by_discriminant;
/// use form_factor_plugins::AppEvent;
///
/// let events = vec![
///     AppEvent::SelectionCleared,
///     AppEvent::CanvasZoomChanged { zoom: 1.0 },
///     AppEvent::CanvasZoomChanged { zoom: 2.0 },
/// ];
///
/// let zoom_events = filter_events_by_discriminant(&events, |e| {
///     matches!(e, AppEvent::CanvasZoomChanged { .. })
/// });
/// assert_eq!(zoom_events.len(), 2);
/// ```
pub fn filter_events_by_discriminant<F>(events: &[AppEvent], predicate: F) -> Vec<&AppEvent>
where
    F: Fn(&AppEvent) -> bool,
{
    events.iter().filter(|e| predicate(e)).collect()
}

/// Counts events matching a predicate
///
/// # Example
/// ```
/// use botticelli_health::count_events_matching;
/// use form_factor_plugins::AppEvent;
///
/// let events = vec![
///     AppEvent::SelectionCleared,
///     AppEvent::CanvasZoomChanged { zoom: 1.0 },
/// ];
///
/// let count = count_events_matching(&events, |e| {
///     matches!(e, AppEvent::CanvasZoomChanged { .. })
/// });
/// assert_eq!(count, 1);
/// ```
pub fn count_events_matching<F>(events: &[AppEvent], predicate: F) -> usize
where
    F: Fn(&AppEvent) -> bool,
{
    filter_events_by_discriminant(events, predicate).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_plugin_manager() {
        let manager = create_empty_plugin_manager();
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_collect_events() {
        let mut bus = EventBus::new();
        let sender = bus.sender();

        sender.emit(AppEvent::SelectionCleared);
        sender.emit(AppEvent::CanvasZoomChanged { zoom: 2.0 });

        let events = collect_events(&mut bus);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_assert_event_emitted() {
        let events = vec![
            AppEvent::SelectionCleared,
            AppEvent::CanvasZoomChanged { zoom: 1.0 },
        ];

        assert_event_emitted(&events, &AppEvent::SelectionCleared);
    }

    #[test]
    #[should_panic(expected = "Expected event")]
    fn test_assert_event_emitted_fails() {
        let events = vec![AppEvent::SelectionCleared];
        assert_event_emitted(&events, &AppEvent::CanvasZoomChanged { zoom: 1.0 });
    }

    #[test]
    fn test_assert_event_not_emitted() {
        let events = vec![AppEvent::SelectionCleared];
        assert_event_not_emitted(&events, &AppEvent::CanvasZoomChanged { zoom: 1.0 });
    }

    #[test]
    fn test_assert_event_count() {
        let events = vec![AppEvent::SelectionCleared, AppEvent::SelectionCleared];
        assert_event_count(&events, 2);
    }

    #[test]
    fn test_filter_events_by_discriminant() {
        let events = vec![
            AppEvent::SelectionCleared,
            AppEvent::CanvasZoomChanged { zoom: 1.0 },
            AppEvent::CanvasZoomChanged { zoom: 2.0 },
        ];

        let zoom_events = filter_events_by_discriminant(&events, |e| {
            matches!(e, AppEvent::CanvasZoomChanged { .. })
        });
        assert_eq!(zoom_events.len(), 2);
    }

    #[test]
    fn test_count_events_matching() {
        let events = vec![
            AppEvent::SelectionCleared,
            AppEvent::CanvasZoomChanged { zoom: 1.0 },
        ];

        let count =
            count_events_matching(&events, |e| matches!(e, AppEvent::CanvasZoomChanged { .. }));
        assert_eq!(count, 1);
    }
}
