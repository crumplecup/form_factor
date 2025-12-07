//! Canvas field interaction for template/form editing.
//!
//! Handles field creation, selection, and manipulation on the canvas.

use derive_getters::Getters;
use derive_setters::Setters;
use egui::{Color32, Pos2, Stroke};
use form_factor_core::FieldType;
use form_factor_drawing::{Rectangle, Shape};
use tracing::{debug, instrument};

/// State for canvas field interactions.
#[derive(Debug, Clone, Getters, Setters)]
#[setters(prefix = "with_")]
pub struct CanvasFieldInteraction {
    /// Currently selected field ID
    selected_field: Option<String>,

    /// Field being created (incomplete)
    creating_field: Option<FieldType>,

    /// Start point of field creation drag
    creation_start: Option<Pos2>,

    /// Field being resized
    resizing_field: Option<String>,

    /// Field being moved
    moving_field: Option<String>,
}

impl CanvasFieldInteraction {
    /// Creates a new canvas field interaction state.
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating new canvas field interaction state");
        Self {
            selected_field: None,
            creating_field: None,
            creation_start: None,
            resizing_field: None,
            moving_field: None,
        }
    }
    
    /// Starts creating a new field of the given type.
    #[instrument(skip(self))]
    pub fn start_field_creation(&mut self, field_type: FieldType, start: Pos2) {
        debug!(?field_type, ?start, "Starting field creation");
        self.creating_field = Some(field_type);
        self.creation_start = Some(start);
    }

    /// Completes field creation and returns the created shape.
    #[instrument(skip(self))]
    pub fn complete_field_creation(&mut self, end: Pos2) -> Option<Shape> {
        if let (Some(field_type), Some(start)) = (self.creating_field.clone(), self.creation_start) {
            debug!(?field_type, ?start, ?end, "Completing field creation");

            // Create rectangle with default stroke and transparent fill
            let stroke = Stroke::new(2.0, Color32::from_rgb(100, 100, 255));
            let fill = Color32::TRANSPARENT;

            let rect = Rectangle::from_corners(start, end, stroke, fill)
                .unwrap_or_else(|e| {
                    debug!(?e, "Failed to create rectangle, using default");
                    // Return a minimal valid rectangle as fallback
                    Rectangle::from_corners(
                        Pos2::new(0.0, 0.0),
                        Pos2::new(10.0, 10.0),
                        stroke,
                        fill,
                    )
                    .expect("Default rectangle should be valid")
                });

            let shape = Shape::Rectangle(rect);

            self.creating_field = None;
            self.creation_start = None;

            Some(shape)
        } else {
            debug!("No field creation in progress");
            None
        }
    }
    
    /// Cancels ongoing field creation.
    #[instrument(skip(self))]
    pub fn cancel_field_creation(&mut self) {
        debug!("Cancelling field creation");
        self.creating_field = None;
        self.creation_start = None;
    }
    
    /// Selects a field by ID.
    #[instrument(skip(self))]
    pub fn select_field(&mut self, field_id: String) {
        debug!(?field_id, "Selecting field");
        self.selected_field = Some(field_id);
    }

    /// Clears field selection.
    #[instrument(skip(self))]
    pub fn clear_selection(&mut self) {
        debug!("Clearing field selection");
        self.selected_field = None;
    }

    /// Starts resizing a field.
    #[instrument(skip(self))]
    pub fn start_resize(&mut self, field_id: String) {
        debug!(?field_id, "Starting field resize");
        self.resizing_field = Some(field_id);
    }

    /// Completes field resizing.
    #[instrument(skip(self))]
    pub fn complete_resize(&mut self) {
        debug!("Completing field resize");
        self.resizing_field = None;
    }

    /// Starts moving a field.
    #[instrument(skip(self))]
    pub fn start_move(&mut self, field_id: String) {
        debug!(?field_id, "Starting field move");
        self.moving_field = Some(field_id);
    }

    /// Completes field moving.
    #[instrument(skip(self))]
    pub fn complete_move(&mut self) {
        debug!("Completing field move");
        self.moving_field = None;
    }
}

impl Default for CanvasFieldInteraction {
    fn default() -> Self {
        Self::new()
    }
}
