//! Tests for DrawingInstance and FormPage functionality

use form_factor_core::{FieldBounds, FieldValue, FormInstance};
use form_factor_drawing::{DrawingInstance, FormPage};

#[test]
fn test_create_instance_from_template() {
    let instance = DrawingInstance::from_template("test_template", 3);

    assert_eq!(instance.template_id(), "test_template");
    assert_eq!(instance.page_count(), 3);
    assert_eq!(instance.instance_name(), None);
    assert_eq!(instance.field_values().len(), 0);
}

#[test]
fn test_set_instance_name() {
    let mut instance = DrawingInstance::from_template("test_template", 1);
    assert_eq!(instance.instance_name(), None);

    instance.set_instance_name("My Form Instance");
    assert_eq!(instance.instance_name(), Some("My Form Instance"));
}

#[test]
fn test_set_field_value() {
    let mut instance = DrawingInstance::from_template("test_template", 1);

    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );

    let result = instance.set_field_value("test_field", value);
    assert!(result.is_ok());

    let retrieved = instance.field_value("test_field");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().as_text(), Some("Test Value"));
}

#[test]
fn test_update_field_value() {
    let mut instance = DrawingInstance::from_template("test_template", 1);

    let value1 = FieldValue::new_text(
        "test_field",
        "Initial Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    instance.set_field_value("test_field", value1).unwrap();

    let value2 = FieldValue::new_text(
        "test_field",
        "Updated Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    instance.set_field_value("test_field", value2).unwrap();

    let retrieved = instance.field_value("test_field");
    assert_eq!(retrieved.unwrap().as_text(), Some("Updated Value"));
}

#[test]
fn test_multiple_field_values() {
    let mut instance = DrawingInstance::from_template("test_template", 1);

    let value1 = FieldValue::new_text(
        "field1",
        "Value 1",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    let value2 = FieldValue::new_text(
        "field2",
        "Value 2",
        FieldBounds::new(10.0, 60.0, 100.0, 30.0),
        0,
    );
    let value3 =
        FieldValue::new_boolean("field3", true, FieldBounds::new(10.0, 100.0, 20.0, 20.0), 0);

    instance.set_field_value("field1", value1).unwrap();
    instance.set_field_value("field2", value2).unwrap();
    instance.set_field_value("field3", value3).unwrap();

    assert_eq!(instance.field_values().len(), 3);
    assert!(instance.field_value("field1").is_some());
    assert!(instance.field_value("field2").is_some());
    assert!(instance.field_value("field3").is_some());
}

#[test]
fn test_field_values_for_page() {
    let mut instance = DrawingInstance::from_template("test_template", 3);

    let value_page0 = FieldValue::new_text(
        "field_page0",
        "Page 0",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    let value_page1_1 = FieldValue::new_text(
        "field_page1_1",
        "Page 1 Field 1",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        1,
    );
    let value_page1_2 = FieldValue::new_text(
        "field_page1_2",
        "Page 1 Field 2",
        FieldBounds::new(10.0, 60.0, 100.0, 30.0),
        1,
    );
    let value_page2 = FieldValue::new_text(
        "field_page2",
        "Page 2",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        2,
    );

    instance
        .set_field_value("field_page0", value_page0)
        .unwrap();
    instance
        .set_field_value("field_page1_1", value_page1_1)
        .unwrap();
    instance
        .set_field_value("field_page1_2", value_page1_2)
        .unwrap();
    instance
        .set_field_value("field_page2", value_page2)
        .unwrap();

    let page0_values = instance.field_values_for_page(0);
    let page1_values = instance.field_values_for_page(1);
    let page2_values = instance.field_values_for_page(2);

    assert_eq!(page0_values.len(), 1);
    assert_eq!(page1_values.len(), 2);
    assert_eq!(page2_values.len(), 1);
}

#[test]
fn test_page_access() {
    let instance = DrawingInstance::from_template("test_template", 3);

    assert!(instance.page(0).is_some());
    assert!(instance.page(1).is_some());
    assert!(instance.page(2).is_some());
    assert!(instance.page(3).is_none());

    assert_eq!(instance.page(0).unwrap().page_index, 0);
    assert_eq!(instance.page(1).unwrap().page_index, 1);
    assert_eq!(instance.page(2).unwrap().page_index, 2);
}

#[test]
fn test_page_mut() {
    let mut instance = DrawingInstance::from_template("test_template", 2);

    let page = instance.page_mut(0);
    assert!(page.is_some());

    let page = page.unwrap();
    assert_eq!(page.page_index, 0);
}

#[test]
fn test_pages_slice() {
    let instance = DrawingInstance::from_template("test_template", 3);

    let pages = instance.pages();
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].page_index, 0);
    assert_eq!(pages[1].page_index, 1);
    assert_eq!(pages[2].page_index, 2);
}

#[test]
fn test_metadata() {
    let mut instance = DrawingInstance::from_template("test_template", 1);

    assert_eq!(instance.metadata().len(), 0);

    instance.add_metadata("created", "2024-12-04");
    instance.add_metadata("author", "test_user");

    assert_eq!(instance.metadata().len(), 2);
    assert_eq!(
        instance.metadata().get("created"),
        Some(&"2024-12-04".to_string())
    );
    assert_eq!(
        instance.metadata().get("author"),
        Some(&"test_user".to_string())
    );
}

#[test]
fn test_json_serialization() {
    let mut instance = DrawingInstance::from_template("test_template", 2);
    instance.set_instance_name("Test Instance");

    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    instance.set_field_value("test_field", value).unwrap();

    let json = instance.to_json().unwrap();

    assert!(json.contains("test_template"));
    assert!(json.contains("Test Instance"));
    assert!(json.contains("test_field"));
    assert!(json.contains("Test Value"));
}

#[test]
fn test_json_deserialization() {
    let mut original = DrawingInstance::from_template("test_template", 2);
    original.set_instance_name("Test Instance");

    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    original.set_field_value("test_field", value).unwrap();
    original.add_metadata("key", "value");

    let json = original.to_json().unwrap();
    let deserialized = DrawingInstance::from_json(&json).unwrap();

    assert_eq!(deserialized.template_id(), original.template_id());
    assert_eq!(deserialized.instance_name(), original.instance_name());
    assert_eq!(deserialized.page_count(), original.page_count());
    assert_eq!(deserialized.field_values().len(), 1);
    assert_eq!(
        deserialized.field_value("test_field").unwrap().as_text(),
        Some("Test Value")
    );
}

#[test]
fn test_validation_state() {
    use form_factor_core::ValidationResult;

    let mut instance = DrawingInstance::from_template("test_template", 1);

    assert!(!instance.is_validated());
    assert!(instance.validation_results().is_none());

    let validation = ValidationResult::success("1.0.0");
    instance.set_validation_results(validation);

    assert!(instance.is_validated());
    assert!(instance.validation_results().is_some());
    assert!(instance.validation_results().unwrap().is_valid());
}

#[test]
fn test_form_page_new() {
    let page = FormPage::new(5);

    assert_eq!(page.page_index, 5);
    assert!(page.image_path().is_none());
    assert_eq!(page.shapes().len(), 0);
    assert_eq!(page.detections().len(), 0);
}

#[test]
fn test_form_page_canvas_access() {
    let page = FormPage::new(0);

    let canvas = page.canvas();
    assert_eq!(canvas.shapes().len(), 0);
}

#[test]
fn test_field_value_with_confidence() {
    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    )
    .with_confidence(0.95);

    assert_eq!(value.confidence, Some(0.95));
    assert!(!value.verified);
}

#[test]
fn test_field_value_with_verified() {
    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    )
    .with_verified(true);

    assert!(value.verified);
}

#[test]
fn test_field_value_confidence_and_verified() {
    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    )
    .with_confidence(0.85)
    .with_verified(true);

    assert_eq!(value.confidence, Some(0.85));
    assert!(value.verified);
}

#[test]
fn test_field_value_boolean() {
    let value = FieldValue::new_boolean(
        "checkbox_field",
        true,
        FieldBounds::new(10.0, 20.0, 20.0, 20.0),
        0,
    );

    assert_eq!(value.as_boolean(), Some(true));
    assert_eq!(value.as_text(), None);
}

#[test]
fn test_field_value_empty() {
    let value = FieldValue::new_empty("empty_field", FieldBounds::new(10.0, 20.0, 100.0, 30.0), 0);

    assert!(value.is_empty());
    assert_eq!(value.as_text(), None);
    assert_eq!(value.as_boolean(), None);
}

#[test]
fn test_shapes_for_page() {
    let instance = DrawingInstance::from_template("test_template", 2);

    let shapes_page0 = instance.shapes_for_page(0);
    let shapes_page1 = instance.shapes_for_page(1);
    let shapes_page2 = instance.shapes_for_page(2);

    assert_eq!(shapes_page0.len(), 0);
    assert_eq!(shapes_page1.len(), 0);
    assert_eq!(shapes_page2.len(), 0);
}

#[test]
fn test_detections_for_page() {
    let instance = DrawingInstance::from_template("test_template", 2);

    let detections_page0 = instance.detections_for_page(0);
    let detections_page1 = instance.detections_for_page(1);
    let detections_page2 = instance.detections_for_page(2);

    assert_eq!(detections_page0.len(), 0);
    assert_eq!(detections_page1.len(), 0);
    assert_eq!(detections_page2.len(), 0);
}
