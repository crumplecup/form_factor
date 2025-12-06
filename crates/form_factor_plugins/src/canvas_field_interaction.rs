//! Canvas field interaction for template/form editing.
//!
//! Handles field creation, selection, and manipulation on the canvas.


use derive_getters::Getters;
use derive_setters::Setters;
use form_factor_core::{FieldId, FieldType, Point, Rect};
use form_factor_drawing::Shape;
use tracing::{debug, instrument};

/// State for canvas field interactions.
#[derive(Debug, Clone, Getters, Setters)]
#[setters(prefix = "with_")]
pub struct CanvasFieldInteraction {
    /// Currently selected field ID
    selected_field: Option<FieldId>,
    
    /// Field being created (incomplete)
    creating_field: Option<FieldType>,
    
    /// Start point of field creation drag
    creation_start: Option<Point>,
    
    /// Field being resized
    resizing_field: Option<FieldId>,
    
    /// Field being moved
    moving_field: Option<FieldId>,
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
    pub fn start_field_creation(&mut self, field_type: FieldType, start: Point) {
        debug!(?field_type, ?start, "Starting field creation");
        self.creating_field = Some(field_type);
        self.creation_start = Some(start);
    }
    
    /// Completes field creation and returns the created shape.
    #[instrument(skip(self))]
    pub fn complete_field_creation(&mut self, end: Point) -> Option<Shape> {
        if let (Some(field_type), Some(start)) = (self.creating_field, self.creation_start) {
            debug!(?field_type, ?start, ?end, "Completing field creation");
            
            let rect = Rect::from_points(start, end);
            let shape = Shape::Rectangle(rect.into());
            
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
    pub fn select_field(&mut self, field_id: FieldId) {
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
    pub fn start_resize(&mut self, field_id: FieldId) {
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
    pub fn start_move(&mut self, field_id: FieldId) {
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
