//! Tests for DrawingTemplate builder functionality

use form_factor_core::{FieldBounds, FieldDefinition, FieldType, FormTemplate};
use form_factor_drawing::{DrawingTemplate, TemplatePage};

#[test]
fn test_builder_fluent_api() {
    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .version("1.0.0")
        .description("A test template")
        .add_page(TemplatePage::new(0))
        .metadata("author", "test")
        .metadata("created", "2024-01-01")
        .build();

    assert!(template.is_ok());
    let template = template.unwrap();
    assert_eq!(template.id(), "test_template");
    assert_eq!(template.name(), "Test Template");
    assert_eq!(template.version(), "1.0.0");
    assert_eq!(template.description(), Some("A test template"));
    assert_eq!(template.page_count(), 1);
    assert_eq!(template.metadata().get("author"), Some(&"test".to_string()));
    assert_eq!(
        template.metadata().get("created"),
        Some(&"2024-01-01".to_string())
    );
}

#[test]
fn test_builder_missing_id() {
    let result = DrawingTemplate::builder()
        .name("Test Template")
        .add_page(TemplatePage::new(0))
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("id is required"));
}

#[test]
fn test_builder_missing_name() {
    let result = DrawingTemplate::builder()
        .id("test_template")
        .add_page(TemplatePage::new(0))
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("name is required"));
}

#[test]
fn test_builder_default_version() {
    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(TemplatePage::new(0))
        .build()
        .unwrap();

    assert_eq!(template.version(), "1.0.0");
}

#[test]
fn test_builder_no_pages() {
    let result = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("at least one page"));
}

#[test]
fn test_add_multiple_pages() {
    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(TemplatePage::new(0))
        .add_page(TemplatePage::new(1))
        .add_page(TemplatePage::new(2))
        .build()
        .unwrap();

    assert_eq!(template.page_count(), 3);
}

#[test]
fn test_page_builder() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let page = TemplatePage::builder(0)
        .dimensions(800, 1100)
        .add_field(field)
        .build();

    assert_eq!(page.page_index, 0);
    assert_eq!(page.dimensions, Some((800, 1100)));
    assert_eq!(page.fields.len(), 1);
    assert_eq!(page.fields[0].id, "test_field");
}

#[test]
fn test_page_add_field_method() {
    let mut page = TemplatePage::new(0);
    assert_eq!(page.fields.len(), 0);

    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    page.add_field(field);
    assert_eq!(page.fields.len(), 1);
}

#[test]
fn test_duplicate_field_ids() {
    let field1 = FieldDefinition::builder()
        .id("duplicate_id")
        .label("Field 1")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let field2 = FieldDefinition::builder()
        .id("duplicate_id")
        .label("Field 2")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 100.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field1);
    page.add_field(field2);

    let result = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(page)
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("duplicate_id"));
}

#[test]
fn test_invalid_page_index_in_field() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(5) // Template only has 1 page
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let result = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(page)
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid page index"));
}

#[test]
fn test_metadata_multiple_entries() {
    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(TemplatePage::new(0))
        .metadata("key1", "value1")
        .metadata("key2", "value2")
        .metadata("key3", "value3")
        .build()
        .unwrap();

    assert_eq!(template.metadata().len(), 3);
    assert_eq!(template.metadata().get("key1"), Some(&"value1".to_string()));
    assert_eq!(template.metadata().get("key2"), Some(&"value2".to_string()));
    assert_eq!(template.metadata().get("key3"), Some(&"value3".to_string()));
}

#[test]
fn test_template_with_fields_across_pages() {
    let field1 = FieldDefinition::builder()
        .id("field_page_0")
        .label("Field on Page 0")
        .field_type(FieldType::FullName)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .build()
        .unwrap();

    let field2 = FieldDefinition::builder()
        .id("field_page_1")
        .label("Field on Page 1")
        .field_type(FieldType::Email)
        .page_index(1)
        .bounds(FieldBounds::new(10.0, 50.0, 200.0, 30.0))
        .build()
        .unwrap();

    let mut page0 = TemplatePage::new(0);
    page0.add_field(field1);

    let mut page1 = TemplatePage::new(1);
    page1.add_field(field2);

    let template = DrawingTemplate::builder()
        .id("multi_page_template")
        .name("Multi-Page Template")
        .add_page(page0)
        .add_page(page1)
        .build()
        .unwrap();

    assert_eq!(template.page_count(), 2);
    assert_eq!(template.fields().len(), 2);
    assert_eq!(template.fields_for_page(0).len(), 1);
    assert_eq!(template.fields_for_page(1).len(), 1);
    assert_eq!(template.fields_for_page(0)[0].id, "field_page_0");
    assert_eq!(template.fields_for_page(1)[0].id, "field_page_1");
}

#[test]
fn test_field_by_id() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(page)
        .build()
        .unwrap();

    assert!(template.field_by_id("test_field").is_some());
    assert_eq!(template.field_by_id("test_field").unwrap().id, "test_field");
    assert!(template.field_by_id("nonexistent").is_none());
}

#[test]
fn test_json_serialization() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .version("1.0.0")
        .add_page(page)
        .build()
        .unwrap();

    let json = template.to_json().unwrap();
    assert!(json.contains("test_template"));
    assert!(json.contains("Test Template"));
    assert!(json.contains("test_field"));
}

#[test]
fn test_json_deserialization() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let original = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .version("1.0.0")
        .add_page(page)
        .build()
        .unwrap();

    let json = original.to_json().unwrap();
    let deserialized = DrawingTemplate::from_json(&json).unwrap();

    assert_eq!(deserialized.id(), original.id());
    assert_eq!(deserialized.name(), original.name());
    assert_eq!(deserialized.version(), original.version());
    assert_eq!(deserialized.page_count(), original.page_count());
    assert_eq!(deserialized.fields().len(), original.fields().len());
}

#[test]
fn test_page_dimensions() {
    let page = TemplatePage::builder(0).dimensions(1200, 1600).build();

    let template = DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .add_page(page)
        .build()
        .unwrap();

    assert_eq!(template.page_dimensions(0), Some((1200, 1600)));
    assert_eq!(template.page_dimensions(1), None);
}
