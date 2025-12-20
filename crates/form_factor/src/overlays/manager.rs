//! Overlay manager for handling stacked UI overlays.

use egui::{Color32, Sense};
use tracing::{debug, instrument};

/// Response from an overlay's show method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayResponse {
    /// Keep the overlay open
    KeepOpen,
    /// Close this overlay
    Close,
}

/// Trait for UI overlays that can be displayed over the main canvas.
pub trait Overlay: Send {
    /// Renders the overlay and returns whether it should stay open.
    fn show(&mut self, ctx: &egui::Context) -> OverlayResponse;

    /// Returns whether this overlay is modal (blocks interaction with content below).
    fn is_modal(&self) -> bool;

    /// Returns the title of this overlay for debugging/logging.
    fn title(&self) -> &str;
}

/// Manages a stack of overlays with z-ordering and backdrop rendering.
#[derive(Default)]
pub struct OverlayManager {
    /// Stack of active overlays (bottom to top)
    overlays: Vec<Box<dyn Overlay>>,
    /// Whether the ESC key was just pressed
    esc_pressed: bool,
}

impl OverlayManager {
    /// Creates a new empty overlay manager.
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating overlay manager");
        Self {
            overlays: Vec::new(),
            esc_pressed: false,
        }
    }

    /// Pushes a new overlay onto the stack.
    #[instrument(skip(self, overlay), fields(overlay_title = overlay.title()))]
    pub fn push(&mut self, overlay: Box<dyn Overlay>) {
        let title = overlay.title();
        debug!(title, "Pushing overlay onto stack");
        self.overlays.push(overlay);
    }

    /// Returns the number of active overlays.
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// Returns whether the overlay stack is empty.
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }

    /// Returns whether any modal overlay is active.
    pub fn has_modal(&self) -> bool {
        self.overlays.iter().any(|o| o.is_modal())
    }

    /// Pops the top overlay from the stack.
    #[instrument(skip(self))]
    pub fn pop(&mut self) -> Option<Box<dyn Overlay>> {
        let overlay = self.overlays.pop();
        if let Some(ref o) = overlay {
            debug!(title = o.title(), "Popped overlay from stack");
        }
        overlay
    }

    /// Clears all overlays from the stack.
    #[instrument(skip(self))]
    pub fn clear(&mut self) {
        let count = self.overlays.len();
        debug!(count, "Clearing all overlays");
        self.overlays.clear()
    }

    /// Renders all overlays and handles input.
    #[instrument(skip(self, ctx), fields(overlay_count = self.overlays.len()))]
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.overlays.is_empty() {
            return;
        }

        // Check for ESC key press
        self.esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        // Find the topmost modal overlay to determine backdrop rendering
        let topmost_modal_index = self
            .overlays
            .iter()
            .enumerate()
            .rev()
            .find(|(_, o)| o.is_modal())
            .map(|(i, _)| i);

        // Render backdrop if we have a modal overlay
        if let Some(modal_index) = topmost_modal_index {
            self.render_backdrop(ctx);
            debug!(modal_index, "Rendering backdrop for modal overlay");
        }

        // Show all overlays and collect which ones want to close
        let mut indices_to_remove = Vec::new();
        for (index, overlay) in self.overlays.iter_mut().enumerate() {
            let response = overlay.show(ctx);

            if response == OverlayResponse::Close {
                indices_to_remove.push(index);
                debug!(index, title = overlay.title(), "Overlay requested close");
            }
        }

        // Handle ESC key - close topmost overlay
        if self.esc_pressed && !self.overlays.is_empty() {
            let top_index = self.overlays.len() - 1;
            if !indices_to_remove.contains(&top_index) {
                indices_to_remove.push(top_index);
                debug!(top_index, "ESC pressed, closing topmost overlay");
            }
        }

        // Remove overlays that want to close (in reverse order to maintain indices)
        indices_to_remove.sort_unstable();
        for index in indices_to_remove.iter().rev() {
            self.overlays.remove(*index);
        }
    }

    /// Renders a semi-transparent backdrop over the entire screen.
    fn render_backdrop(&self, ctx: &egui::Context) {
        let screen_rect = ctx.viewport_rect();

        // Create a semi-transparent dark overlay
        let backdrop_color = Color32::from_black_alpha(128); // 50% transparent black

        // Allocate a layer for the backdrop
        egui::Area::new(egui::Id::new("overlay_backdrop"))
            .fixed_pos(screen_rect.left_top())
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                // Create an invisible, non-interactive rect that covers the screen
                let response = ui.allocate_rect(screen_rect, Sense::hover());

                // Paint the backdrop
                ui.painter().rect_filled(
                    screen_rect,
                    0.0, // No rounding
                    backdrop_color,
                );

                // Prevent interaction with content below
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                }
            });
    }

    /// Returns a reference to the top overlay, if any.
    pub fn top(&self) -> Option<&Box<dyn Overlay>> {
        self.overlays.last()
    }

    /// Returns a mutable reference to the top overlay, if any.
    pub fn top_mut(&mut self) -> Option<&mut Box<dyn Overlay>> {
        self.overlays.last_mut()
    }
}
