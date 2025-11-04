//! Integration tests for tool module
//!
//! Tests validate ToolMode enum behavior, serialization, and trait implementations.

use form_factor::ToolMode;
use strum::IntoEnumIterator;

// ============================================================================
// Basic Enum Behavior Tests
// ============================================================================

#[test]
fn default_tool_is_select() {
    let default_tool = ToolMode::default();
    assert_eq!(default_tool, ToolMode::Select);
}

#[test]
fn all_tool_modes_are_unique() {
    let tools: Vec<_> = ToolMode::iter().collect();
    assert_eq!(tools.len(), 6);

    // Verify all are unique
    for (i, tool1) in tools.iter().enumerate() {
        for (j, tool2) in tools.iter().enumerate() {
            if i != j {
                assert_ne!(tool1, tool2);
            }
        }
    }
}

#[test]
fn enum_iteration_has_all_tools() {
    let tools: Vec<_> = ToolMode::iter().collect();
    assert_eq!(
        tools,
        vec![
            ToolMode::Select,
            ToolMode::Rectangle,
            ToolMode::Circle,
            ToolMode::Freehand,
            ToolMode::Edit,
            ToolMode::Rotate,
        ]
    );
}

#[test]
fn enum_iteration_count() {
    assert_eq!(ToolMode::iter().count(), 6);
}

// ============================================================================
// Display Trait Tests
// ============================================================================

#[test]
fn display_all_tool_modes() {
    assert_eq!(ToolMode::Select.to_string(), "Select");
    assert_eq!(ToolMode::Rectangle.to_string(), "Rectangle");
    assert_eq!(ToolMode::Circle.to_string(), "Circle");
    assert_eq!(ToolMode::Freehand.to_string(), "Freehand");
    assert_eq!(ToolMode::Edit.to_string(), "Edit");
    assert_eq!(ToolMode::Rotate.to_string(), "Rotate");
}

#[test]
fn display_format_string() {
    let tool = ToolMode::Rectangle;
    let formatted = format!("Current tool: {}", tool);
    assert_eq!(formatted, "Current tool: Rectangle");
}

// ============================================================================
// Equality and Comparison Tests
// ============================================================================

#[test]
fn equality_works() {
    assert_eq!(ToolMode::Select, ToolMode::Select);
    assert_eq!(ToolMode::Circle, ToolMode::Circle);
    assert_ne!(ToolMode::Select, ToolMode::Rectangle);
    assert_ne!(ToolMode::Edit, ToolMode::Rotate);
}

#[test]
fn ordering_matches_declaration_order() {
    // Tools should be ordered as declared in enum
    assert!(ToolMode::Select < ToolMode::Rectangle);
    assert!(ToolMode::Rectangle < ToolMode::Circle);
    assert!(ToolMode::Circle < ToolMode::Freehand);
    assert!(ToolMode::Freehand < ToolMode::Edit);
    assert!(ToolMode::Edit < ToolMode::Rotate);
}

#[test]
fn ordering_is_consistent() {
    let mut tools = vec![
        ToolMode::Rotate,
        ToolMode::Select,
        ToolMode::Edit,
        ToolMode::Circle,
        ToolMode::Freehand,
        ToolMode::Rectangle,
    ];
    tools.sort();

    assert_eq!(
        tools,
        vec![
            ToolMode::Select,
            ToolMode::Rectangle,
            ToolMode::Circle,
            ToolMode::Freehand,
            ToolMode::Edit,
            ToolMode::Rotate,
        ]
    );
}

// ============================================================================
// Hash Trait Tests
// ============================================================================

#[test]
fn hash_works_in_hashmap() {
    use std::collections::HashMap;

    let mut tool_shortcuts = HashMap::new();
    tool_shortcuts.insert(ToolMode::Select, 'v');
    tool_shortcuts.insert(ToolMode::Rectangle, 'r');
    tool_shortcuts.insert(ToolMode::Circle, 'c');
    tool_shortcuts.insert(ToolMode::Freehand, 'p');
    tool_shortcuts.insert(ToolMode::Edit, 'e');
    tool_shortcuts.insert(ToolMode::Rotate, 't');

    assert_eq!(tool_shortcuts.get(&ToolMode::Select), Some(&'v'));
    assert_eq!(tool_shortcuts.get(&ToolMode::Rectangle), Some(&'r'));
    assert_eq!(tool_shortcuts.get(&ToolMode::Circle), Some(&'c'));
}

#[test]
fn hash_works_in_hashset() {
    use std::collections::HashSet;

    let mut active_tools = HashSet::new();
    active_tools.insert(ToolMode::Select);
    active_tools.insert(ToolMode::Rectangle);

    assert!(active_tools.contains(&ToolMode::Select));
    assert!(active_tools.contains(&ToolMode::Rectangle));
    assert!(!active_tools.contains(&ToolMode::Circle));
}

// ============================================================================
// Copy and Clone Tests
// ============================================================================

#[test]
#[allow(clippy::clone_on_copy)] // Intentionally testing both
fn clone_and_copy_work() {
    let original = ToolMode::Circle;
    let cloned = original.clone();
    let copied = original;

    assert_eq!(original, cloned);
    assert_eq!(original, copied);
}

#[test]
fn copy_semantic_allows_reuse() {
    let tool = ToolMode::Edit;
    let tool_copy = tool;

    // Original should still be usable after copy
    assert_eq!(tool, ToolMode::Edit);
    assert_eq!(tool_copy, ToolMode::Edit);
}

// ============================================================================
// Serialization/Deserialization Tests
// ============================================================================

#[test]
fn serialization_roundtrip() {
    for tool in ToolMode::iter() {
        let json = serde_json::to_string(&tool).expect("Serialization should succeed");
        let restored: ToolMode = serde_json::from_str(&json).expect("Deserialization should succeed");
        assert_eq!(tool, restored);
    }
}

#[test]
fn serialization_format() {
    let json = serde_json::to_string(&ToolMode::Rectangle).expect("Serialization should succeed");
    assert_eq!(json, r#""Rectangle""#);
}

#[test]
fn deserialization_from_string() {
    let tool: ToolMode = serde_json::from_str(r#""Select""#).expect("Deserialization should succeed");
    assert_eq!(tool, ToolMode::Select);

    let tool: ToolMode = serde_json::from_str(r#""Freehand""#).expect("Deserialization should succeed");
    assert_eq!(tool, ToolMode::Freehand);
}

#[test]
fn deserialization_fails_for_invalid_tool() {
    let result: Result<ToolMode, _> = serde_json::from_str(r#""InvalidTool""#);
    assert!(result.is_err());
}

#[test]
fn multiple_serialization_roundtrips_are_stable() {
    let mut tool = ToolMode::Rotate;

    for _ in 0..5 {
        let json = serde_json::to_string(&tool).expect("Serialization should succeed");
        tool = serde_json::from_str(&json).expect("Deserialization should succeed");
    }

    assert_eq!(tool, ToolMode::Rotate);
}

// ============================================================================
// Pattern Matching Tests
// ============================================================================

#[test]
fn pattern_matching_works() {
    let tool = ToolMode::Circle;

    let is_drawing_tool = match tool {
        ToolMode::Rectangle | ToolMode::Circle | ToolMode::Freehand => true,
        ToolMode::Select | ToolMode::Edit | ToolMode::Rotate => false,
    };

    assert!(is_drawing_tool);
}

#[test]
fn pattern_matching_with_select() {
    let tool = ToolMode::Select;

    let is_manipulation_tool = match tool {
        ToolMode::Select | ToolMode::Edit | ToolMode::Rotate => true,
        ToolMode::Rectangle | ToolMode::Circle | ToolMode::Freehand => false,
    };

    assert!(is_manipulation_tool);
}

// ============================================================================
// Debug Trait Tests
// ============================================================================

#[test]
fn debug_format() {
    let tool = ToolMode::Rectangle;
    let debug_str = format!("{:?}", tool);
    assert_eq!(debug_str, "Rectangle");
}

#[test]
fn debug_all_variants() {
    for tool in ToolMode::iter() {
        let debug_str = format!("{:?}", tool);
        assert!(!debug_str.is_empty());
    }
}

// ============================================================================
// Edge Cases and Regression Tests
// ============================================================================

#[test]
fn default_can_be_compared_with_all_tools() {
    let default_tool = ToolMode::default();

    for tool in ToolMode::iter() {
        // Should not panic
        let _ = default_tool == tool;
        let _ = default_tool != tool;
        let _ = default_tool < tool;
        let _ = default_tool <= tool;
    }
}

#[test]
fn all_tools_can_be_stored_in_vec() {
    let tools: Vec<ToolMode> = ToolMode::iter().collect();
    assert_eq!(tools.len(), 6);

    // Verify no data loss
    for (i, tool) in ToolMode::iter().enumerate() {
        assert_eq!(tools[i], tool);
    }
}

#[test]
fn tool_mode_size_is_small() {
    use std::mem::size_of;

    // ToolMode should be very small (just an enum discriminant)
    assert!(size_of::<ToolMode>() <= 8);
}

#[test]
fn option_tool_mode_size() {
    use std::mem::size_of;

    // Option<ToolMode> should benefit from null pointer optimization
    // and be same size as ToolMode or only slightly larger
    assert!(size_of::<Option<ToolMode>>() <= 16);
}

// ============================================================================
// State Transition Tests
// ============================================================================

#[test]
fn tool_can_transition_between_modes() {
    let mut current_tool = ToolMode::default();
    assert_eq!(current_tool, ToolMode::Select);

    current_tool = ToolMode::Rectangle;
    assert_eq!(current_tool, ToolMode::Rectangle);

    current_tool = ToolMode::Circle;
    assert_eq!(current_tool, ToolMode::Circle);

    current_tool = ToolMode::Select;
    assert_eq!(current_tool, ToolMode::Select);
}

#[test]
fn tool_can_cycle_through_all_modes() {
    let tools: Vec<_> = ToolMode::iter().collect();

    for tool in &tools {
        // Each tool should be assignable
        let current = *tool;
        assert!(tools.contains(&current));
    }
}
