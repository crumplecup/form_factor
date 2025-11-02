use serde::{Deserialize, Serialize};

/// Types of layers in the canvas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Canvas,
    Shapes,
    Grid,
}

/// A layer with visibility control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub locked: bool,
}

impl Layer {
    /// Create a new layer
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            name: name.into(),
            layer_type,
            visible: true,
            locked: false,
        }
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggle locked state
    pub fn toggle_locked(&mut self) {
        self.locked = !self.locked;
    }
}

/// Manages the collection of layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerManager {
    layers: Vec<Layer>,
}

impl LayerManager {
    /// Create a new layer manager with default layers
    pub fn new() -> Self {
        let mut grid_layer = Layer::new("Grid", LayerType::Grid);
        grid_layer.visible = false; // Grid is hidden by default

        Self {
            layers: vec![
                grid_layer,
                Layer::new("Shapes", LayerType::Shapes),
                Layer::new("Canvas", LayerType::Canvas),
            ],
        }
    }

    /// Get all layers
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Get a mutable reference to all layers
    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }

    /// Check if a layer type is visible
    pub fn is_visible(&self, layer_type: LayerType) -> bool {
        self.layers
            .iter()
            .find(|l| l.layer_type == layer_type)
            .is_none_or(|l| l.visible)
    }

    /// Toggle layer visibility by type
    pub fn toggle_layer(&mut self, layer_type: LayerType) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.layer_type == layer_type) {
            layer.toggle_visibility();
        }
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}
