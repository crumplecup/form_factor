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
use std::fmt;
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
)]
pub enum LayerType {
    /// The canvas layer (background form image) - rendered first (bottom)
    Canvas,
    /// The detections layer (automatically detected regions)
    Detections,
    /// The shapes layer (user-drawn annotations)
    Shapes,
    /// The grid layer (alignment grid overlay) - rendered last (top)
    Grid,
}

impl fmt::Display for LayerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayerType::Canvas => write!(f, "Canvas"),
            LayerType::Detections => write!(f, "Detections"),
            LayerType::Shapes => write!(f, "Shapes"),
            LayerType::Grid => write!(f, "Grid"),
        }
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerError {
    /// Layer with the given type was not found
    LayerNotFound(LayerType),
}

impl fmt::Display for LayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayerError::LayerNotFound(layer_type) => {
                write!(f, "Layer not found: {}", layer_type)
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_manager_creates_all_layers() {
        let manager = LayerManager::new();
        assert_eq!(manager.len(), 4);

        // Verify all layer types exist
        for layer_type in LayerType::iter() {
            let layer = manager.get_layer(layer_type);
            assert_eq!(layer.layer_type(), &layer_type);
        }
    }

    #[test]
    fn default_visibility_state() {
        let manager = LayerManager::new();

        assert!(manager.is_visible(LayerType::Canvas));
        assert!(manager.is_visible(LayerType::Detections));
        assert!(manager.is_visible(LayerType::Shapes));
        assert!(!manager.is_visible(LayerType::Grid)); // Grid hidden by default
    }

    #[test]
    fn toggle_visibility() {
        let mut manager = LayerManager::new();

        assert!(manager.is_visible(LayerType::Shapes));
        manager.toggle_layer(LayerType::Shapes);
        assert!(!manager.is_visible(LayerType::Shapes));
        manager.toggle_layer(LayerType::Shapes);
        assert!(manager.is_visible(LayerType::Shapes));
    }

    #[test]
    fn set_visibility() {
        let mut manager = LayerManager::new();

        manager.set_visible(LayerType::Grid, true);
        assert!(manager.is_visible(LayerType::Grid));

        manager.set_visible(LayerType::Grid, false);
        assert!(!manager.is_visible(LayerType::Grid));
    }

    #[test]
    fn lock_layer() {
        let mut manager = LayerManager::new();

        assert!(!manager.is_locked(LayerType::Shapes));
        manager.set_locked(LayerType::Shapes, true);
        assert!(manager.is_locked(LayerType::Shapes));
    }

    #[test]
    fn toggle_locked() {
        let mut manager = LayerManager::new();

        assert!(!manager.is_locked(LayerType::Canvas));
        manager.toggle_locked(LayerType::Canvas);
        assert!(manager.is_locked(LayerType::Canvas));
        manager.toggle_locked(LayerType::Canvas);
        assert!(!manager.is_locked(LayerType::Canvas));
    }

    #[test]
    fn layers_in_render_order() {
        let manager = LayerManager::new();
        let layers: Vec<_> = manager.layers_in_order().collect();

        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].layer_type(), &LayerType::Canvas);
        assert_eq!(layers[1].layer_type(), &LayerType::Detections);
        assert_eq!(layers[2].layer_type(), &LayerType::Shapes);
        assert_eq!(layers[3].layer_type(), &LayerType::Grid);
    }

    #[test]
    fn validation_succeeds_for_valid_manager() {
        let manager = LayerManager::new();
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn layer_type_ordering() {
        // Verify enum ordering matches render order
        assert!(LayerType::Canvas < LayerType::Detections);
        assert!(LayerType::Detections < LayerType::Shapes);
        assert!(LayerType::Shapes < LayerType::Grid);
    }

    #[test]
    fn layer_display() {
        assert_eq!(LayerType::Canvas.to_string(), "Canvas");
        assert_eq!(LayerType::Grid.to_string(), "Grid");
    }

    #[test]
    fn layer_creation() {
        let layer = Layer::new("Test", LayerType::Shapes);
        assert_eq!(layer.name(), "Test");
        assert_eq!(layer.layer_type(), &LayerType::Shapes);
        assert!(layer.visible());
        assert!(!layer.locked());

        let hidden_layer = Layer::new_hidden("Hidden", LayerType::Grid);
        assert!(!hidden_layer.visible());
    }

    #[test]
    fn layer_mutation() {
        let mut layer = Layer::new("Test", LayerType::Shapes);

        layer.set_name("New Name");
        assert_eq!(layer.name(), "New Name");

        layer.toggle_visibility();
        assert!(!layer.visible());

        layer.set_locked(true);
        assert!(layer.locked());

        layer.toggle_locked();
        assert!(!layer.locked());
    }

    #[test]
    fn enum_iteration() {
        let types: Vec<_> = LayerType::iter().collect();
        assert_eq!(types.len(), 4);
        assert_eq!(
            types,
            vec![
                LayerType::Canvas,
                LayerType::Detections,
                LayerType::Shapes,
                LayerType::Grid
            ]
        );
    }
}
