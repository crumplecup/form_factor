//! Layer management for canvas rendering
//!
//! Layers are rendered in a fixed z-order from bottom to top:
//! 1. Canvas (background form image) - bottom
//! 2. Detections (automatically detected regions)
//! 3. Shapes (user-drawn annotations)
//! 4. Grid (alignment grid overlay) - top

use derive_getters::Getters;
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

/// Types of layers in the canvas
///
/// Layers are rendered in enum discriminant order (Canvas first, Grid last).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Enum,
    strum::EnumIter,
    derive_more::Display,
)]
pub enum LayerType {
    /// The canvas layer (background form image) - rendered first (bottom)
    #[display("Canvas")]
    Canvas,
    /// The detections layer (automatically detected regions)
    #[display("Detections")]
    Detections,
    /// The template layer (field definitions - blueprint for forms)
    #[display("Template")]
    Template,
    /// The instance layer (filled-in form data)
    #[display("Instance")]
    Instance,
    /// The shapes layer (user-drawn annotations)
    #[display("Shapes")]
    Shapes,
    /// The grid layer (alignment grid overlay) - rendered last (top)
    #[display("Grid")]
    Grid,
}



/// A layer with visibility and lock control
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
pub struct Layer {
    /// Display name of the layer
    name: String,
    /// Type of this layer
    layer_type: LayerType,
    /// Whether the layer is visible
    visible: bool,
    /// Whether the layer is locked (non-editable)
    locked: bool,
}

impl Layer {
    /// Create a new layer with default visibility and lock state
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            name: name.into(),
            layer_type,
            visible: true,
            locked: false,
        }
    }

    /// Create a new hidden layer
    pub fn new_hidden(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            name: name.into(),
            layer_type,
            visible: false,
            locked: false,
        }
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Set locked state
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    /// Toggle locked state
    pub fn toggle_locked(&mut self) {
        self.locked = !self.locked;
    }

    /// Set the display name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}

/// Error type for layer operations
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum LayerError {
    /// Layer with the given type was not found
    #[display("Layer not found: {}", _0)]
    LayerNotFound(LayerType),
}

impl std::error::Error for LayerError {}

/// Manages the collection of layers with type-safe access
///
/// LayerManager guarantees:
/// - Exactly one layer per LayerType exists
/// - Layers are iterated in render order (Canvas to Grid)
/// - Type-safe access via EnumMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerManager {
    layers: EnumMap<LayerType, Layer>,
}

impl LayerManager {
    /// Create a new layer manager with default layers
    ///
    /// Default state:
    /// - Canvas: visible, unlocked
    /// - Detections: visible, unlocked
    /// - Shapes: visible, unlocked
    /// - Grid: hidden, unlocked
    pub fn new() -> Self {
        Self {
            layers: enum_map::enum_map! {
                LayerType::Canvas => Layer::new("Canvas", LayerType::Canvas),
                LayerType::Detections => Layer::new("Detections", LayerType::Detections),
                LayerType::Template => Layer::new_hidden("Template", LayerType::Template),
                LayerType::Instance => Layer::new_hidden("Instance", LayerType::Instance),
                LayerType::Shapes => Layer::new("Shapes", LayerType::Shapes),
                LayerType::Grid => Layer::new_hidden("Grid", LayerType::Grid),
            },
        }
    }

    /// Get a layer by type
    pub fn get_layer(&self, layer_type: LayerType) -> &Layer {
        &self.layers[layer_type]
    }

    /// Get a mutable reference to a layer by type
    pub fn get_layer_mut(&mut self, layer_type: LayerType) -> &mut Layer {
        &mut self.layers[layer_type]
    }

    /// Get all layers in render order (Canvas first, Grid last)
    pub fn layers_in_order(&self) -> impl Iterator<Item = &Layer> {
        LayerType::iter().map(|layer_type| &self.layers[layer_type])
    }

    /// Check if a layer type is visible
    pub fn is_visible(&self, layer_type: LayerType) -> bool {
        self.layers[layer_type].visible
    }

    /// Check if a layer type is locked
    pub fn is_locked(&self, layer_type: LayerType) -> bool {
        self.layers[layer_type].locked
    }

    /// Set layer visibility
    pub fn set_visible(&mut self, layer_type: LayerType, visible: bool) {
        self.layers[layer_type].set_visible(visible);
    }

    /// Toggle layer visibility by type
    pub fn toggle_layer(&mut self, layer_type: LayerType) {
        self.layers[layer_type].toggle_visibility();
    }

    /// Set layer locked state
    pub fn set_locked(&mut self, layer_type: LayerType, locked: bool) {
        self.layers[layer_type].set_locked(locked);
    }

    /// Toggle layer locked state
    pub fn toggle_locked(&mut self, layer_type: LayerType) {
        self.layers[layer_type].toggle_locked();
    }

    /// Validate that all required layers exist and are properly configured
    ///
    /// This is primarily useful after deserialization to ensure the layer
    /// manager is in a valid state.
    pub fn validate(&self) -> Result<(), LayerError> {
        // With EnumMap, we automatically have all layers, so just verify
        // each layer's layer_type matches its key
        for layer_type in LayerType::iter() {
            let layer = &self.layers[layer_type];
            if layer.layer_type != layer_type {
                // This should never happen with properly serialized data
                // but could occur with manual construction or corrupted data
                return Err(LayerError::LayerNotFound(layer_type));
            }
        }
        Ok(())
    }

    /// Get the number of layers (always 4)
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Check if empty (always false, as we always have all LayerTypes)
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}
