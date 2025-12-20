//! Tests for overlay manager functionality.

use form_factor::{Overlay, OverlayManager, OverlayResponse};

// Test overlay implementation
struct TestOverlay {
    title: String,
    modal: bool,
    should_close: bool,
}

impl TestOverlay {
    fn new(title: impl Into<String>, modal: bool) -> Self {
        Self {
            title: title.into(),
            modal,
            should_close: false,
        }
    }
}

impl Overlay for TestOverlay {
    fn show(&mut self, _ctx: &egui::Context) -> OverlayResponse {
        if self.should_close {
            OverlayResponse::Close
        } else {
            OverlayResponse::KeepOpen
        }
    }

    fn is_modal(&self) -> bool {
        self.modal
    }

    fn title(&self) -> &str {
        &self.title
    }
}

#[test]
fn test_overlay_manager_empty() {
    let manager = OverlayManager::new();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
    assert!(!manager.has_modal());
}

#[test]
fn test_overlay_manager_push_pop() {
    let mut manager = OverlayManager::new();

    let overlay1 = Box::new(TestOverlay::new("Test 1", false));
    manager.push(overlay1);
    assert_eq!(manager.len(), 1);
    assert!(!manager.is_empty());

    let overlay2 = Box::new(TestOverlay::new("Test 2", true));
    manager.push(overlay2);
    assert_eq!(manager.len(), 2);
    assert!(manager.has_modal());

    let popped = manager.pop();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().title(), "Test 2");
    assert_eq!(manager.len(), 1);

    manager.clear();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_overlay_manager_modal_detection() {
    let mut manager = OverlayManager::new();

    // Non-modal overlay
    manager.push(Box::new(TestOverlay::new("Non-modal", false)));
    assert!(!manager.has_modal());

    // Add modal overlay
    manager.push(Box::new(TestOverlay::new("Modal", true)));
    assert!(manager.has_modal());

    // Remove modal overlay
    manager.pop();
    assert!(!manager.has_modal());
}

#[test]
fn test_overlay_manager_top_access() {
    let mut manager = OverlayManager::new();

    assert!(manager.top().is_none());
    assert!(manager.top_mut().is_none());

    manager.push(Box::new(TestOverlay::new("Test", false)));

    assert!(manager.top().is_some());
    assert_eq!(manager.top().unwrap().title(), "Test");

    assert!(manager.top_mut().is_some());
    assert_eq!(manager.top_mut().unwrap().title(), "Test");
}
