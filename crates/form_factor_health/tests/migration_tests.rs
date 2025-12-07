//! Tests for legacy project migration

use form_factor_core::FormInstance;
use form_factor_drawing::{
    DrawingCanvas, DrawingInstance, LEGACY_TEMPLATE_ID, ProjectFormat, migrate_canvas_to_instance,
};

#[test]
fn test_migrate_canvas_preserves_project_name() {
    let mut canvas = DrawingCanvas::new();
    canvas.with_project_name("Test Project".to_string());

    let instance = migrate_canvas_to_instance(canvas);

    assert_eq!(instance.instance_name().as_deref(), Some("Test Project"));
}

#[test]
fn test_migrate_canvas_creates_single_page() {
    let canvas = DrawingCanvas::new();

    let instance = migrate_canvas_to_instance(canvas);

    assert_eq!(instance.page_count(), 1);
}

#[test]
fn test_migrate_canvas_uses_legacy_template_id() {
    let canvas = DrawingCanvas::new();

    let instance = migrate_canvas_to_instance(canvas);

    assert_eq!(instance.template_id(), LEGACY_TEMPLATE_ID);
}

#[test]
fn test_migrate_canvas_adds_migration_metadata() {
    let canvas = DrawingCanvas::new();

    let instance = migrate_canvas_to_instance(canvas);

    let metadata = instance.metadata();
    assert_eq!(
        metadata.get("migrated_from"),
        Some(&"legacy_canvas".to_string())
    );
    assert!(metadata.contains_key("migration_date"));
    assert!(metadata.contains_key("original_project_name"));
}

#[test]
fn test_detect_legacy_format_no_version() {
    // Simulate legacy format (no version field)
    let json = r#"{
        "project_name": "Legacy Project",
        "shapes": [],
        "detections": [],
        "current_tool": "Select",
        "layer_manager": {
            "layers": [],
            "active_layer": null
        },
        "form_image_path": null,
        "template_id": null,
        "zoom_level": 5.0,
        "pan_offset": [0.0, 0.0],
        "grid_rotation_angle": 0.0,
        "form_image_rotation": 0.0,
        "stroke": {"width": 2.0, "color": [0, 120, 215, 255]},
        "fill_color": [0, 120, 215, 30]
    }"#;

    let version = ProjectFormat::detect_version(json).unwrap();
    assert_eq!(version, 0);
}

#[test]
fn test_detect_legacy_format_version_1() {
    let json = r#"{
        "version": 1,
        "project_name": "Legacy Project",
        "shapes": []
    }"#;

    let version = ProjectFormat::detect_version(json).unwrap();
    assert_eq!(version, 1);
}

#[test]
fn test_detect_instance_format() {
    let json = r#"{
        "version": 2,
        "template_id": "test_template",
        "instance_name": "Test Instance",
        "pages": [],
        "field_values": {},
        "metadata": {}
    }"#;

    let version = ProjectFormat::detect_version(json).unwrap();
    assert_eq!(version, 2);
}

#[test]
fn test_load_legacy_format_and_convert() {
    // Create a legacy canvas
    let mut canvas = DrawingCanvas::new();
    canvas.with_project_name("Legacy Project".to_string());

    // Serialize as legacy format (version will default to 1)
    let json = serde_json::to_string(&canvas).unwrap();

    // Load and detect format
    let format = ProjectFormat::from_json(&json).unwrap();

    // Convert to instance
    let instance = format.into_instance();

    assert_eq!(instance.template_id(), LEGACY_TEMPLATE_ID);
    assert_eq!(instance.instance_name().as_deref(), Some("Legacy Project"));
    assert_eq!(instance.page_count(), 1);
}

#[test]
fn test_load_instance_format_unchanged() {
    // Create a new instance
    let mut instance = DrawingInstance::from_template("custom_template", 2);
    instance.set_instance_name("Test Instance");

    // Serialize as instance format (version 2)
    let json = instance.to_json().unwrap();

    // Load and detect format
    let format = ProjectFormat::from_json(&json).unwrap();

    // Convert (should be unchanged)
    let loaded_instance = format.into_instance();

    assert_eq!(loaded_instance.template_id(), "custom_template");
    assert_eq!(
        loaded_instance.instance_name().as_deref(),
        Some("Test Instance")
    );
    assert_eq!(loaded_instance.page_count(), 2);
}

#[test]
fn test_roundtrip_migration() {
    // Create legacy canvas with some data
    let mut canvas = DrawingCanvas::new();
    canvas.with_project_name("Roundtrip Test".to_string());

    // Serialize legacy format
    let legacy_json = serde_json::to_string(&canvas).unwrap();

    // Load as legacy and migrate
    let format = ProjectFormat::from_json(&legacy_json).unwrap();
    let instance = format.into_instance();

    // Verify migration
    assert_eq!(instance.template_id(), LEGACY_TEMPLATE_ID);
    assert_eq!(instance.instance_name().as_deref(), Some("Roundtrip Test"));
    assert_eq!(instance.page_count(), 1);

    // Serialize as new format
    let instance_json = instance.to_json().unwrap();

    // Load again (should detect as instance format)
    let version = ProjectFormat::detect_version(&instance_json).unwrap();
    assert_eq!(version, 2);

    let format2 = ProjectFormat::from_json(&instance_json).unwrap();
    let instance2 = format2.into_instance();

    // Verify data preserved
    assert_eq!(instance2.template_id(), LEGACY_TEMPLATE_ID);
    assert_eq!(instance2.instance_name().as_deref(), Some("Roundtrip Test"));
    assert_eq!(instance2.page_count(), 1);
}

#[test]
fn test_invalid_version_rejected() {
    let json = r#"{
        "version": 999,
        "data": "something"
    }"#;

    let result = ProjectFormat::from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_json_rejected() {
    let json = "not valid json";

    let result = ProjectFormat::detect_version(json);
    assert!(result.is_err());
}
