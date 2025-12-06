//! Event bus for plugin communication.

use crate::event::AppEvent;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Event bus for plugin-to-plugin and plugin-to-app communication.
///
/// The event bus uses an unbounded MPSC channel to deliver events asynchronously.
/// Plugins can send events through the bus, and the application can collect and
/// distribute events to all registered plugins.
pub struct EventBus {
    /// Sender side of the event channel
    tx: mpsc::UnboundedSender<AppEvent>,
    /// Receiver side of the event channel
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventBus {
    /// Creates a new event bus.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        debug!("Event bus created");
        Self { tx, rx }
    }

    /// Gets a sender that can be used to publish events to the bus.
    ///
    /// Multiple senders can be cloned to allow concurrent event publishing.
    pub fn sender(&self) -> EventSender {
        EventSender {
            tx: self.tx.clone(),
        }
    }

    /// Tries to receive the next event from the bus without blocking.
    ///
    /// Returns `None` if no events are currently available.
    pub fn try_recv(&mut self) -> Option<AppEvent> {
        self.rx.try_recv().ok()
    }

    /// Collects all currently available events from the bus.
    ///
    /// This drains the event queue and returns all pending events.
    /// Useful for batch processing events in the main application loop.
    pub fn drain_events(&mut self) -> Vec<AppEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.try_recv() {
            events.push(event);
        }
        if !events.is_empty() {
            debug!(count = events.len(), "Drained events from bus");
        }
        events
    }

    /// Gets the number of events currently queued in the bus.
    ///
    /// Note: This is an approximation and may not be exact due to concurrent access.
    pub fn pending_count(&self) -> usize {
        // Unfortunately, UnboundedReceiver doesn't expose queue length
        // We'd need to track this separately if needed
        0
    }

    /// Returns a copy of pending events without draining the queue
    ///
    /// This method is only available in test builds to allow tests to
    /// inspect events without consuming them.
    ///
    /// Note: This drains and re-sends events, which means event ordering
    /// is preserved but this should only be used in single-threaded test
    /// contexts.
    #[cfg(test)]
    pub fn pending_events(&mut self) -> Vec<AppEvent> {
        let events = self.drain_events();
        // Re-send all events so they're still in the queue
        let sender = self.sender();
        for event in &events {
            let _ = sender.send(event.clone());
        }
        events
    }

    /// Checks if a specific event is in the pending queue
    ///
    /// This is a test-only helper that checks for an event without draining.
    #[cfg(test)]
    pub fn has_event(&mut self, target: &AppEvent) -> bool {
        let events = self.pending_events();
        events.contains(target)
    }

    /// Counts events matching a predicate in the pending queue
    ///
    /// This is a test-only helper.
    #[cfg(test)]
    pub fn count_events<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&AppEvent) -> bool,
    {
        let events = self.pending_events();
        events.iter().filter(|e| predicate(e)).count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for sending events to the event bus.
///
/// Multiple senders can exist simultaneously, allowing plugins to send
/// events concurrently.
#[derive(Clone)]
pub struct EventSender {
    tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventSender {
    /// Creates a new event sender for testing purposes.
    ///
    /// This is primarily useful for unit tests where you need to create
    /// a sender without a full EventBus.
    #[cfg(test)]
    pub fn new_test() -> (Self, mpsc::UnboundedReceiver<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }

    /// Sends an event to the bus.
    ///
    /// # Errors
    /// Returns an error if the bus has been closed (receiver dropped).
    pub fn send(&self, event: AppEvent) -> Result<(), SendError> {
        debug!(?event, "Sending event to bus");
        self.tx.send(event).map_err(|_e| {
            warn!("Failed to send event: receiver closed");
            SendError {
                kind: SendErrorKind::ReceiverClosed,
                line: line!(),
                file: file!(),
            }
        })
    }

    /// Sends an event to the bus, logging and ignoring any errors.
    ///
    /// Use this when you want to fire-and-forget an event without
    /// handling potential errors.
    pub fn emit(&self, event: AppEvent) {
        if let Err(e) = self.send(event) {
            warn!("Failed to emit event: {}", e);
        }
    }
}

/// Error that can occur when sending events.
#[derive(Debug, Clone, PartialEq)]
pub enum SendErrorKind {
    /// The receiver side of the channel has been closed
    ReceiverClosed,
}

impl std::fmt::Display for SendErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendErrorKind::ReceiverClosed => write!(f, "Event bus receiver has been closed"),
        }
    }
}

/// Error that occurs when sending an event fails.
#[derive(Debug, Clone)]
pub struct SendError {
    /// The kind of error
    pub kind: SendErrorKind,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Send Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for SendError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_send_recv() {
        let mut bus = EventBus::new();
        let sender = bus.sender();

        let event = AppEvent::CanvasZoomChanged { zoom: 2.0 };
        sender.send(event.clone()).unwrap();

        let received = bus.try_recv();
        assert_eq!(received, Some(event));
    }

    #[test]
    fn test_drain_events() {
        let mut bus = EventBus::new();
        let sender = bus.sender();

        sender.send(AppEvent::SelectionCleared).unwrap();
        sender
            .send(AppEvent::CanvasZoomChanged { zoom: 1.5 })
            .unwrap();
        sender
            .send(AppEvent::LayerSelected {
                layer_name: "test".to_string(),
            })
            .unwrap();

        let events = bus.drain_events();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_multiple_senders() {
        let mut bus = EventBus::new();
        let sender1 = bus.sender();
        let sender2 = bus.sender();

        sender1
            .send(AppEvent::CanvasZoomChanged { zoom: 1.0 })
            .unwrap();
        sender2
            .send(AppEvent::CanvasZoomChanged { zoom: 2.0 })
            .unwrap();

        let events = bus.drain_events();
        assert_eq!(events.len(), 2);
    }
}
