//! Integration tests for layer module
//!
//! These tests validate serialization, deserialization, and API contracts
//! that are critical for real-world usage.

use form_factor::{FormError, LayerError, LayerManager, LayerType};

// ============================================================================
// High Priority: Serialization/Deserialization Tests
// ============================================================================

#[test]
fn serialization_roundtrip_preserves_all_data() -> Result<(), FormError> {
    let mut manager = LayerManager::new();

    // Modify state
    manager.set_visible(LayerType::Grid, true);
    manager.set_locked(LayerType::Canvas, true);
    manager.set_visible(LayerType::Shapes, false);
    manager
        .get_layer_mut(LayerType::Detections)
        .set_name("Custom Detections".to_string());

    // Serialize
    let json = serde_json::to_string(&manager)?;

    // Deserialize
    let restored: LayerManager =
        serde_json::from_str(&json)?;

    // Verify all state preserved
    assert!(restored.is_visible(LayerType::Grid));
    assert!(restored.is_locked(LayerType::Canvas));
    assert!(!restored.is_visible(LayerType::Shapes));
    assert_eq!(
        restored.get_layer(LayerType::Detections).name(),
        "Custom Detections"
    );

    // Verify layers that weren't modified
    assert!(restored.is_visible(LayerType::Canvas));
    assert!(!restored.is_locked(LayerType::Grid));
    Ok(())
}

#[test]
fn serialization_format_is_stable() -> Result<(), FormError> {
    let manager = LayerManager::new();
    let json = serde_json::to_string_pretty(&manager)?;

    // Verify JSON contains expected structure
    assert!(json.contains("Canvas"));
    assert!(json.contains("Detections"));
    assert!(json.contains("Shapes"));
    assert!(json.contains("Grid"));
    assert!(json.contains("visible"));
    assert!(json.contains("locked"));
    Ok(())
}

#[test]
fn deserialization_validates_layer_integrity() -> Result<(), FormError> {
    let manager = LayerManager::new();
    let json = serde_json::to_string(&manager)?;
    let restored: LayerManager =
        serde_json::from_str(&json)?;

    // Validation should pass for properly deserialized data
    assert!(restored.validate().is_ok());
    Ok(())
}

#[test]
fn deserialization_handles_extra_fields() -> Result<(), FormError> {
    // JSON with extra unknown fields (forward compatibility)
    let json = r#"{
        "layers": {
            "Canvas": {
                "name": "Canvas",
                "layer_type": "Canvas",
                "visible": true,
                "locked": false,
                "future_field": "ignored"
            },
            "Detections": {
                "name": "Detections",
                "layer_type": "Detections",
                "visible": true,
                "locked": false
            },
            "Template": {
                "name": "Template",
                "layer_type": "Template",
                "visible": true,
                "locked": false
            },
            "Instance": {
                "name": "Instance",
                "layer_type": "Instance",
                "visible": true,
                "locked": false
            },
            "Shapes": {
                "name": "Shapes",
                "layer_type": "Shapes",
                "visible": true,
                "locked": false
            },
            "Grid": {
                "name": "Grid",
                "layer_type": "Grid",
                "visible": false,
                "locked": false
            }
        },
        "extra_manager_field": "also ignored"
    }"#;

    let manager: LayerManager = serde_json::from_str(json)?;
    assert_eq!(manager.len(), 6);
    Ok(())
}

#[test]
fn serialization_preserves_custom_layer_names() -> Result<(), FormError> {
    let mut manager = LayerManager::new();

    // Customize all layer names
    manager
        .get_layer_mut(LayerType::Canvas)
        .set_name("Background Image".to_string());
    manager
        .get_layer_mut(LayerType::Detections)
        .set_name("Auto-detected Regions".to_string());
    manager
        .get_layer_mut(LayerType::Shapes)
        .set_name("Manual Annotations".to_string());
    manager
        .get_layer_mut(LayerType::Grid)
        .set_name("Alignment Grid".to_string());

    // Round-trip
    let json = serde_json::to_string(&manager)?;
    let restored: LayerManager =
        serde_json::from_str(&json)?;

    assert_eq!(
        restored.get_layer(LayerType::Canvas).name(),
        "Background Image"
    );
    assert_eq!(
        restored.get_layer(LayerType::Detections).name(),
        "Auto-detected Regions"
    );
    assert_eq!(
        restored.get_layer(LayerType::Shapes).name(),
        "Manual Annotations"
    );
    assert_eq!(restored.get_layer(LayerType::Grid).name(), "Alignment Grid");
    Ok(())
}

#[test]
fn multiple_serialization_roundtrips_are_stable() -> Result<(), FormError> {
    let mut manager = LayerManager::new();
    manager.set_visible(LayerType::Grid, true);
    manager.set_locked(LayerType::Canvas, true);

    // Multiple round-trips
    for _ in 0..5 {
        let json = serde_json::to_string(&manager)?;
        manager = serde_json::from_str(&json)?;
    }

    // State should be unchanged
    assert!(manager.is_visible(LayerType::Grid));
    assert!(manager.is_locked(LayerType::Canvas));
    Ok(())
}

// ============================================================================
// High Priority: get_layer_mut Persistence Tests
// ============================================================================

#[test]
fn get_layer_mut_modifications_persist() {
    let mut manager = LayerManager::new();

    {
        let layer = manager.get_layer_mut(LayerType::Shapes);
        layer.set_name("Modified Shapes".to_string());
        layer.set_visible(false);
        layer.set_locked(true);
    }

    // Verify changes persisted after mutable borrow dropped
    let layer = manager.get_layer(LayerType::Shapes);
    assert_eq!(layer.name(), "Modified Shapes");
    assert!(!layer.visible());
    assert!(layer.locked());
}

#[test]
fn get_layer_mut_for_all_layer_types() {
    let mut manager = LayerManager::new();

    // Modify each layer type
    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        let layer = manager.get_layer_mut(layer_type);
        layer.set_name(format!("Modified {}", layer_type));
    }

    // Verify all modifications
    assert_eq!(
        manager.get_layer(LayerType::Canvas).name(),
        "Modified Canvas"
    );
    assert_eq!(
        manager.get_layer(LayerType::Detections).name(),
        "Modified Detections"
    );
    assert_eq!(
        manager.get_layer(LayerType::Shapes).name(),
        "Modified Shapes"
    );
    assert_eq!(manager.get_layer(LayerType::Grid).name(), "Modified Grid");
}

#[test]
fn get_layer_mut_and_immutable_methods_are_consistent() {
    let mut manager = LayerManager::new();

    // Use mutable API
    manager.get_layer_mut(LayerType::Canvas).set_visible(false);

    // Verify with immutable API
    assert!(!manager.is_visible(LayerType::Canvas));
    assert!(!manager.get_layer(LayerType::Canvas).visible());
}

#[test]
fn layer_mutations_through_different_apis_are_consistent() {
    let mut manager = LayerManager::new();

    // Method 1: Direct mutation method
    manager.set_visible(LayerType::Shapes, false);

    // Method 2: get_layer_mut
    manager.get_layer_mut(LayerType::Grid).set_visible(true);

    // Method 3: toggle
    manager.toggle_layer(LayerType::Canvas);

    // Verify all mutations worked
    assert!(!manager.is_visible(LayerType::Shapes));
    assert!(manager.is_visible(LayerType::Grid));
    assert!(!manager.is_visible(LayerType::Canvas));
}

// ============================================================================
// Medium Priority: Direct API Testing
// ============================================================================

#[test]
fn get_layer_returns_correct_layer() {
    let manager = LayerManager::new();

    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        let layer = manager.get_layer(layer_type);
        assert_eq!(layer.layer_type(), &layer_type);
    }
}

#[test]
fn get_layer_returns_immutable_reference() {
    let manager = LayerManager::new();
    let layer1 = manager.get_layer(LayerType::Canvas);
    let layer2 = manager.get_layer(LayerType::Canvas);

    // Both references should point to the same data
    assert_eq!(layer1.name(), layer2.name());
    assert_eq!(layer1.visible(), layer2.visible());
}

#[test]
fn default_trait_matches_new() {
    let manager1 = LayerManager::new();
    let manager2 = LayerManager::default();

    // Compare all layer states
    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        assert_eq!(
            manager1.is_visible(layer_type),
            manager2.is_visible(layer_type)
        );
        assert_eq!(
            manager1.is_locked(layer_type),
            manager2.is_locked(layer_type)
        );
        assert_eq!(
            manager1.get_layer(layer_type).name(),
            manager2.get_layer(layer_type).name()
        );
    }
}

#[test]
fn len_returns_correct_count() {
    let manager = LayerManager::new();
    assert_eq!(manager.len(), 6);
}

#[test]
fn is_empty_always_returns_false() {
    let manager = LayerManager::new();
    assert!(!manager.is_empty());
}

#[test]
fn layers_in_order_iteration_count() {
    let manager = LayerManager::new();
    let count = manager.layers_in_order().count();
    assert_eq!(count, 6);
}

#[test]
fn layers_in_order_preserves_render_sequence() {
    let manager = LayerManager::new();
    let order: Vec<_> = manager.layers_in_order().map(|l| *l.layer_type()).collect();

    assert_eq!(
        order,
        vec![
            LayerType::Canvas,
            LayerType::Detections,
            LayerType::Template,
            LayerType::Instance,
            LayerType::Shapes,
            LayerType::Grid,
        ]
    );
}

// ============================================================================
// Medium Priority: Complete Display Trait Coverage
// ============================================================================

#[test]
fn layer_type_display_all_variants() {
    assert_eq!(LayerType::Canvas.to_string(), "Canvas");
    assert_eq!(LayerType::Detections.to_string(), "Detections");
    assert_eq!(LayerType::Shapes.to_string(), "Shapes");
    assert_eq!(LayerType::Grid.to_string(), "Grid");
}

#[test]
fn layer_type_display_format() {
    let formatted = format!("Layer: {}", LayerType::Canvas);
    assert_eq!(formatted, "Layer: Canvas");
}

// ============================================================================
// Medium Priority: Validation Tests
// ============================================================================

#[test]
fn validation_succeeds_for_new_manager() {
    let manager = LayerManager::new();
    assert!(manager.validate().is_ok());
}

#[test]
fn validation_succeeds_after_modifications() {
    let mut manager = LayerManager::new();

    manager.set_visible(LayerType::Grid, true);
    manager.set_locked(LayerType::Canvas, true);
    manager
        .get_layer_mut(LayerType::Shapes)
        .set_name("Modified".to_string());

    assert!(manager.validate().is_ok());
}

#[test]
fn validation_succeeds_after_deserialization() -> Result<(), FormError> {
    let json = r#"{
        "layers": {
            "Canvas": {"name": "Canvas", "layer_type": "Canvas", "visible": true, "locked": false},
            "Detections": {"name": "Detections", "layer_type": "Detections", "visible": true, "locked": false},
            "Template": {"name": "Template", "layer_type": "Template", "visible": true, "locked": false},
            "Instance": {"name": "Instance", "layer_type": "Instance", "visible": true, "locked": false},
            "Shapes": {"name": "Shapes", "layer_type": "Shapes", "visible": true, "locked": false},
            "Grid": {"name": "Grid", "layer_type": "Grid", "visible": false, "locked": false}
        }
    }"#;

    let manager: LayerManager = serde_json::from_str(json)?;
    assert!(manager.validate().is_ok());
    Ok(())
}

// ============================================================================
// Medium Priority: Error Handling Tests
// ============================================================================

#[test]
fn layer_error_display_format() {
    let error = LayerError::LayerNotFound(LayerType::Canvas);
    let display = format!("{}", error);
    assert_eq!(display, "Layer not found: Canvas");
}

#[test]
fn layer_error_implements_error_trait() {
    let error = LayerError::LayerNotFound(LayerType::Grid);
    let _: &dyn std::error::Error = &error;
}

// ============================================================================
// Medium Priority: Complex State Transition Tests
// ============================================================================

#[test]
fn complex_visibility_state_transitions() {
    let mut manager = LayerManager::new();

    // Complex sequence: hide, lock, unhide, unlock
    manager.set_visible(LayerType::Shapes, false);
    assert!(!manager.is_visible(LayerType::Shapes));

    manager.set_locked(LayerType::Shapes, true);
    assert!(manager.is_locked(LayerType::Shapes));
    assert!(!manager.is_visible(LayerType::Shapes)); // Still hidden

    manager.set_visible(LayerType::Shapes, true);
    assert!(manager.is_visible(LayerType::Shapes));
    assert!(manager.is_locked(LayerType::Shapes)); // Still locked

    manager.set_locked(LayerType::Shapes, false);
    assert!(!manager.is_locked(LayerType::Shapes));
    assert!(manager.is_visible(LayerType::Shapes)); // Still visible
}

#[test]
fn independent_layer_state_management() {
    let mut manager = LayerManager::new();

    // Each layer's state should be independent
    manager.set_visible(LayerType::Canvas, false);
    manager.set_locked(LayerType::Detections, true);
    manager.set_visible(LayerType::Shapes, false);
    manager.set_locked(LayerType::Grid, true);

    // Verify independence
    assert!(!manager.is_visible(LayerType::Canvas));
    assert!(!manager.is_locked(LayerType::Canvas));

    assert!(manager.is_visible(LayerType::Detections));
    assert!(manager.is_locked(LayerType::Detections));

    assert!(!manager.is_visible(LayerType::Shapes));
    assert!(!manager.is_locked(LayerType::Shapes));

    assert!(!manager.is_visible(LayerType::Grid));
    assert!(manager.is_locked(LayerType::Grid));
}

#[test]
fn toggle_operations_are_reversible() {
    let mut manager = LayerManager::new();

    let initial_visible = manager.is_visible(LayerType::Canvas);
    let initial_locked = manager.is_locked(LayerType::Canvas);

    // Double toggle should return to initial state
    manager.toggle_layer(LayerType::Canvas);
    manager.toggle_layer(LayerType::Canvas);
    assert_eq!(manager.is_visible(LayerType::Canvas), initial_visible);

    manager.toggle_locked(LayerType::Canvas);
    manager.toggle_locked(LayerType::Canvas);
    assert_eq!(manager.is_locked(LayerType::Canvas), initial_locked);
}

// ============================================================================
// Medium Priority: Layer Type Property Tests
// ============================================================================

#[test]
fn layer_type_equality() {
    assert_eq!(LayerType::Canvas, LayerType::Canvas);
    assert_ne!(LayerType::Canvas, LayerType::Grid);
}

#[test]
fn layer_type_ordering() {
    assert!(LayerType::Canvas < LayerType::Detections);
    assert!(LayerType::Detections < LayerType::Shapes);
    assert!(LayerType::Shapes < LayerType::Grid);

    // Verify render order is ascending
    let mut types = vec![
        LayerType::Grid,
        LayerType::Canvas,
        LayerType::Shapes,
        LayerType::Detections,
    ];
    types.sort();
    assert_eq!(
        types,
        vec![
            LayerType::Canvas,
            LayerType::Detections,
            LayerType::Shapes,
            LayerType::Grid,
        ]
    );
}

#[test]
fn layer_type_hash_consistency() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(LayerType::Canvas, "canvas");
    map.insert(LayerType::Grid, "grid");

    assert_eq!(map.get(&LayerType::Canvas), Some(&"canvas"));
    assert_eq!(map.get(&LayerType::Grid), Some(&"grid"));
    assert_eq!(map.get(&LayerType::Shapes), None);
}

#[test]
#[allow(clippy::clone_on_copy)] // Intentionally testing both clone and copy
fn layer_type_clone_and_copy() {
    let original = LayerType::Shapes;
    let cloned = original.clone();
    let copied = original;

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
}

// ============================================================================
// Edge Cases and Regression Tests
// ============================================================================

#[test]
fn layer_name_can_be_empty_string() {
    let mut manager = LayerManager::new();
    manager.get_layer_mut(LayerType::Canvas).set_name("".to_string());
    assert_eq!(manager.get_layer(LayerType::Canvas).name(), "");
}

#[test]
fn layer_name_can_contain_special_characters() {
    let mut manager = LayerManager::new();
    manager
        .get_layer_mut(LayerType::Shapes)
        .set_name("Layer 🎨 with émojis & spëcial chars!".to_string());
    assert_eq!(
        manager.get_layer(LayerType::Shapes).name(),
        "Layer 🎨 with émojis & spëcial chars!"
    );
}

#[test]
fn layer_name_can_be_very_long() {
    let mut manager = LayerManager::new();
    let long_name = "a".repeat(10000);
    manager.get_layer_mut(LayerType::Grid).set_name(long_name.clone());
    assert_eq!(manager.get_layer(LayerType::Grid).name(), &long_name);
}

#[test]
fn all_layers_visible_simultaneously() {
    let mut manager = LayerManager::new();

    // Make all layers visible
    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        manager.set_visible(layer_type, true);
    }

    // Verify all visible
    assert!(manager.is_visible(LayerType::Canvas));
    assert!(manager.is_visible(LayerType::Detections));
    assert!(manager.is_visible(LayerType::Shapes));
    assert!(manager.is_visible(LayerType::Grid));
}

#[test]
fn all_layers_hidden_simultaneously() {
    let mut manager = LayerManager::new();

    // Hide all layers
    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        manager.set_visible(layer_type, false);
    }

    // Verify all hidden
    assert!(!manager.is_visible(LayerType::Canvas));
    assert!(!manager.is_visible(LayerType::Detections));
    assert!(!manager.is_visible(LayerType::Shapes));
    assert!(!manager.is_visible(LayerType::Grid));
}

#[test]
fn all_layers_locked_simultaneously() {
    let mut manager = LayerManager::new();

    // Lock all layers
    for layer_type in [
        LayerType::Canvas,
        LayerType::Detections,
        LayerType::Shapes,
        LayerType::Grid,
    ] {
        manager.set_locked(layer_type, true);
    }

    // Verify all locked
    assert!(manager.is_locked(LayerType::Canvas));
    assert!(manager.is_locked(LayerType::Detections));
    assert!(manager.is_locked(LayerType::Shapes));
    assert!(manager.is_locked(LayerType::Grid));
}
