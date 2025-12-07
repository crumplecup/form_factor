//! Integration test for template-instance lifecycle
//!
//! This test demonstrates the complete lifecycle:
//! - Create and save a template to registry
//! - Load template from registry
//! - Create an instance from the template
//! - Fill and validate the instance
//! - Save instance to JSON
//! - Load instance from JSON
//! - Clean up

use form_factor_core::{
    FieldBounds, FieldDefinition, FieldType, FieldValue, FormInstance, FormTemplate,
};
use form_factor_drawing::{DrawingInstance, DrawingTemplate, TemplatePage, TemplateRegistry};

#[test]
fn test_complete_template_instance_lifecycle() {
    // Step 1: Create a template
    let name_field = FieldDefinition::builder()
        .id("full_name")
        .label("Full Name")
        .field_type(FieldType::FullName)
        .page_index(0)
        .bounds(FieldBounds::new(100.0, 50.0, 300.0, 30.0))
        .required(true)
        .help_text("Enter your full legal name")
        .build()
        .unwrap();

    let email_field = FieldDefinition::builder()
        .id("email")
        .label("Email Address")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(100.0, 100.0, 300.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let ssn_field = FieldDefinition::builder()
        .id("ssn")
        .label("Social Security Number")
        .field_type(FieldType::SSN)
        .page_index(0)
        .bounds(FieldBounds::new(100.0, 150.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(name_field);
    page.add_field(email_field);
    page.add_field(ssn_field);

    let template = DrawingTemplate::builder()
        .id("lifecycle_test_template")
        .name("Lifecycle Test Template")
        .version("1.0.0")
        .description("Template for lifecycle testing")
        .add_page(page)
        .metadata("test", "true")
        .metadata("category", "integration")
        .build()
        .unwrap();

    // Verify template structure
    assert_eq!(template.id(), "lifecycle_test_template");
    assert_eq!(template.name(), "Lifecycle Test Template");
    assert_eq!(template.version(), "1.0.0");
    assert_eq!(template.fields().len(), 3);

    // Step 2: Save template to registry (global location)
    let mut registry = TemplateRegistry::new().unwrap();
    let save_result = registry.save(&template);
    assert!(save_result.is_ok());

    // Step 3: Load template from registry
    registry.clear(); // Clear memory to ensure we're loading from disk
    let load_result = registry.load_all();
    assert!(load_result.is_ok());

    let loaded_template = registry.get("lifecycle_test_template");
    assert!(loaded_template.is_some());

    let loaded_template = loaded_template.unwrap();
    assert_eq!(loaded_template.id(), "lifecycle_test_template");
    assert_eq!(loaded_template.name(), "Lifecycle Test Template");
    assert_eq!(loaded_template.fields().len(), 3);

    // Step 4: Create an instance from the loaded template
    let mut instance = DrawingInstance::from_template(loaded_template.id(), 1);
    instance.set_instance_name("John Doe - Test Instance");
    instance.add_metadata("created_date", "2024-12-04");
    instance.add_metadata("test_run", "integration");

    assert_eq!(instance.template_id(), "lifecycle_test_template");
    assert_eq!(instance.instance_name().as_deref(), Some("John Doe - Test Instance"));

    // Step 5: Fill the instance with valid data
    let name_value = FieldValue::new_text(
        "full_name",
        "John Doe",
        FieldBounds::new(100.0, 50.0, 300.0, 30.0),
        0,
    )
    .with_confidence(0.95)
    .with_verified(true);

    let email_value = FieldValue::new_text(
        "email",
        "john.doe@example.com",
        FieldBounds::new(100.0, 100.0, 300.0, 30.0),
        0,
    )
    .with_confidence(0.92);

    let ssn_value = FieldValue::new_text(
        "ssn",
        "123-45-6789",
        FieldBounds::new(100.0, 150.0, 200.0, 30.0),
        0,
    )
    .with_verified(true);

    instance.set_field_value("full_name", name_value).unwrap();
    instance.set_field_value("email", email_value).unwrap();
    instance.set_field_value("ssn", ssn_value).unwrap();

    // Verify field values
    assert_eq!(instance.field_values().len(), 3);
    assert!(instance.field_value("full_name").is_some());
    assert!(instance.field_value("email").is_some());
    assert!(instance.field_value("ssn").is_some());

    // Step 6: Validate instance against template
    let validation_result = loaded_template.validate_instance(&instance);
    assert!(validation_result.is_valid());
    assert!(validation_result.missing_required().is_empty());
    assert!(validation_result.field_errors().is_empty());
    assert_eq!(validation_result.template_version(), "1.0.0");

    // Store validation results in instance
    instance.set_validation_results(validation_result.clone());
    assert!(instance.is_validated());
    assert!(instance.validation_results().is_some());

    // Step 7: Save instance to JSON
    let instance_json = instance.to_json().unwrap();
    assert!(instance_json.contains("lifecycle_test_template"));
    assert!(instance_json.contains("John Doe"));
    assert!(instance_json.contains("john.doe@example.com"));
    assert!(instance_json.contains("123-45-6789"));

    // Write to file
    let temp_dir = std::env::temp_dir();
    let instance_file = temp_dir.join("lifecycle_test_instance.json");
    std::fs::write(&instance_file, &instance_json).unwrap();

    // Step 8: Load instance from JSON file
    let loaded_json = std::fs::read_to_string(&instance_file).unwrap();
    let loaded_instance = DrawingInstance::from_json(&loaded_json).unwrap();

    // Verify loaded instance matches original
    assert_eq!(loaded_instance.template_id(), instance.template_id());
    assert_eq!(loaded_instance.instance_name(), instance.instance_name());
    assert_eq!(loaded_instance.page_count(), instance.page_count());
    assert_eq!(
        loaded_instance.field_values().len(),
        instance.field_values().len()
    );

    // Verify specific field values
    let loaded_name = loaded_instance.field_value("full_name").unwrap();
    assert_eq!(loaded_name.as_text(), Some("John Doe"));
    assert_eq!(loaded_name.confidence(), &Some(0.95));
    assert!(loaded_name.verified());

    let loaded_email = loaded_instance.field_value("email").unwrap();
    assert_eq!(loaded_email.as_text(), Some("john.doe@example.com"));

    let loaded_ssn = loaded_instance.field_value("ssn").unwrap();
    assert_eq!(loaded_ssn.as_text(), Some("123-45-6789"));

    // Verify metadata
    assert_eq!(
        loaded_instance.metadata().get("created_date"),
        Some(&"2024-12-04".to_string())
    );
    assert_eq!(
        loaded_instance.metadata().get("test_run"),
        Some(&"integration".to_string())
    );

    // Step 9: Re-validate loaded instance
    let revalidation_result = loaded_template.validate_instance(&loaded_instance);
    assert!(revalidation_result.is_valid());

    // Step 10: Test invalid instance scenario
    let mut invalid_instance = DrawingInstance::from_template("lifecycle_test_template", 1);

    // Only fill one required field
    let partial_value = FieldValue::new_text(
        "full_name",
        "Jane Smith",
        FieldBounds::new(100.0, 50.0, 300.0, 30.0),
        0,
    );
    invalid_instance
        .set_field_value("full_name", partial_value)
        .unwrap();

    let invalid_result = loaded_template.validate_instance(&invalid_instance);
    assert!(!invalid_result.is_valid());
    assert_eq!(invalid_result.missing_required().len(), 2); // Missing email and ssn

    // Step 11: Clean up
    std::fs::remove_file(&instance_file).ok();
    registry
        .delete_from_global("lifecycle_test_template")
        .unwrap();
}

#[test]
fn test_template_versioning() {
    // Create version 1.0.0
    let field_v1 = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page_v1 = TemplatePage::new(0);
    page_v1.add_field(field_v1);

    let template_v1 = DrawingTemplate::builder()
        .id("versioned_template")
        .name("Versioned Template")
        .version("1.0.0")
        .add_page(page_v1)
        .build()
        .unwrap();

    let mut registry = TemplateRegistry::new().unwrap();
    registry.save(&template_v1).unwrap();

    // Create instance for v1.0.0
    let mut instance_v1 = DrawingInstance::from_template("versioned_template", 1);
    let value = FieldValue::new_text(
        "test_field",
        "Test Value",
        FieldBounds::new(10.0, 20.0, 100.0, 30.0),
        0,
    );
    instance_v1.set_field_value("test_field", value).unwrap();

    let validation_v1 = template_v1.validate_instance(&instance_v1);
    assert!(validation_v1.is_valid());
    assert_eq!(validation_v1.template_version(), "1.0.0");

    // Create version 2.0.0 with additional required field
    let field1_v2 = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let field2_v2 = FieldDefinition::builder()
        .id("new_required_field")
        .label("New Required Field")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 60.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page_v2 = TemplatePage::new(0);
    page_v2.add_field(field1_v2);
    page_v2.add_field(field2_v2);

    let template_v2 = DrawingTemplate::builder()
        .id("versioned_template")
        .name("Versioned Template")
        .version("2.0.0")
        .add_page(page_v2)
        .build()
        .unwrap();

    // Validate v1 instance against v2 template (should fail)
    let validation_v2 = template_v2.validate_instance(&instance_v1);
    assert!(!validation_v2.is_valid());
    assert!(
        validation_v2
            .missing_required()
            .contains(&"new_required_field".to_string())
    );

    // Clean up
    registry.delete_from_global("versioned_template").unwrap();
}

#[test]
fn test_json_persistence_roundtrip() {
    let field = FieldDefinition::builder()
        .id("test_field")
        .label("Test Field")
        .field_type(FieldType::FullName)
        .page_index(0)
        .bounds(FieldBounds::new(50.0, 100.0, 250.0, 40.0))
        .required(true)
        .help_text("Enter your name")
        .metadata("custom_key", "custom_value")
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("roundtrip_template")
        .name("Roundtrip Template")
        .version("1.0.0")
        .description("Testing JSON roundtrip")
        .add_page(page)
        .metadata("author", "Test Suite")
        .build()
        .unwrap();

    // Template JSON roundtrip
    let template_json = template.to_json().unwrap();
    let loaded_template = DrawingTemplate::from_json(&template_json).unwrap();

    assert_eq!(loaded_template.id(), template.id());
    assert_eq!(loaded_template.name(), template.name());
    assert_eq!(loaded_template.version(), template.version());
    assert_eq!(loaded_template.description(), template.description());
    assert_eq!(loaded_template.fields().len(), template.fields().len());
    assert_eq!(loaded_template.metadata(), template.metadata());

    // Instance JSON roundtrip
    let mut instance = DrawingInstance::from_template("roundtrip_template", 1);
    instance.set_instance_name("Test Instance");
    instance.add_metadata("run_id", "test_123");

    let value = FieldValue::new_text(
        "test_field",
        "Test Name",
        FieldBounds::new(50.0, 100.0, 250.0, 40.0),
        0,
    )
    .with_confidence(0.88)
    .with_verified(true);

    instance.set_field_value("test_field", value).unwrap();

    let instance_json = instance.to_json().unwrap();
    let loaded_instance = DrawingInstance::from_json(&instance_json).unwrap();

    assert_eq!(loaded_instance.template_id(), instance.template_id());
    assert_eq!(loaded_instance.instance_name(), instance.instance_name());
    assert_eq!(loaded_instance.field_values().len(), 1);
    assert_eq!(loaded_instance.metadata(), instance.metadata());

    let loaded_value = loaded_instance.field_value("test_field").unwrap();
    assert_eq!(loaded_value.as_text(), Some("Test Name"));
    assert_eq!(loaded_value.confidence(), &Some(0.88));
    assert!(loaded_value.verified());
}
