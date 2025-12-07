//! Integration test for multi-page workflow
//!
//! This test demonstrates a complete multi-page form workflow:
//! - Create a template with multiple pages
//! - Create an instance and fill each page
//! - Navigate between pages
//! - Validate the complete instance

use form_factor_core::{
    FieldBounds, FieldDefinition, FieldType, FieldValue, FormInstance, FormTemplate,
    instance::{FieldValueBuilder, FieldValueBuilderError},
};
use form_factor_drawing::{DrawingInstance, DrawingTemplate, TemplatePage};

#[test]
fn test_multi_page_workflow() -> Result<(), FieldValueBuilderError> {
    // Step 1: Create a multi-page template (3 pages)
    let page1_field1 = FieldDefinition::builder()
        .id("employee_name")
        .label("Employee Name")
        .field_type(FieldType::FullName)
        .page_index(0_usize)
        .bounds(FieldBounds::new(100.0, 50.0, 300.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let page1_field2 = FieldDefinition::builder()
        .id("employee_email")
        .label("Email Address")
        .field_type(FieldType::Email)
        .page_index(0_usize)
        .bounds(FieldBounds::new(100.0, 100.0, 300.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let page2_field1 = FieldDefinition::builder()
        .id("street_address")
        .label("Street Address")
        .field_type(FieldType::StreetAddress)
        .page_index(1_usize)
        .bounds(FieldBounds::new(100.0, 50.0, 300.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let page2_field2 = FieldDefinition::builder()
        .id("city")
        .label("City")
        .field_type(FieldType::City)
        .page_index(1_usize)
        .bounds(FieldBounds::new(100.0, 100.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let page2_field3 = FieldDefinition::builder()
        .id("state")
        .label("State")
        .field_type(FieldType::State)
        .page_index(1_usize)
        .bounds(FieldBounds::new(320.0, 100.0, 80.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let page3_field1 = FieldDefinition::builder()
        .id("signature")
        .label("Employee Signature")
        .field_type(FieldType::Signature)
        .page_index(2_usize)
        .bounds(FieldBounds::new(100.0, 400.0, 300.0, 60.0))
        .required(true)
        .build()
        .unwrap();

    let page3_field2 = FieldDefinition::builder()
        .id("date_signed")
        .label("Date")
        .field_type(FieldType::DateSigned)
        .page_index(2_usize)
        .bounds(FieldBounds::new(100.0, 480.0, 150.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page1 = TemplatePage::new(0);
    page1.add_field(page1_field1);
    page1.add_field(page1_field2);

    let mut page2 = TemplatePage::new(1);
    page2.add_field(page2_field1);
    page2.add_field(page2_field2);
    page2.add_field(page2_field3);

    let mut page3 = TemplatePage::new(2);
    page3.add_field(page3_field1);
    page3.add_field(page3_field2);

    let template = DrawingTemplate::builder()
        .id("employee_onboarding")
        .name("Employee Onboarding Form")
        .version("1.0.0")
        .description("Three-page employee onboarding form")
        .add_page(page1)
        .add_page(page2)
        .add_page(page3)
        .metadata("department", "HR")
        .metadata("year", "2024")
        .build()
        .unwrap();

    // Verify template structure
    assert_eq!(template.page_count(), 3);
    assert_eq!(template.fields().len(), 7);
    assert_eq!(template.fields_for_page(0).len(), 2);
    assert_eq!(template.fields_for_page(1).len(), 3);
    assert_eq!(template.fields_for_page(2).len(), 2);

    // Step 2: Create an instance from the template
    let mut instance = DrawingInstance::from_template("employee_onboarding", 3);
    instance.set_instance_name("John Doe - New Hire");

    assert_eq!(instance.template_id(), "employee_onboarding");
    assert_eq!(instance.page_count(), 3);
    assert_eq!(
        instance.instance_name().as_deref(),
        Some("John Doe - New Hire")
    );

    // Step 3: Fill out page 1 (Personal Information)
    let name_value = FieldValue::new_text(
        "employee_name",
        "John Doe",
        FieldBounds::new(100.0, 50.0, 300.0, 30.0),
        0,
    )
    .with_confidence(0.95);

    let email_value = FieldValue::new_text(
        "employee_email",
        "john.doe@company.com",
        FieldBounds::new(100.0, 100.0, 300.0, 30.0),
        0,
    )
    .with_confidence(0.92);

    instance
        .set_field_value("employee_name", name_value)
        .unwrap();
    instance
        .set_field_value("employee_email", email_value)
        .unwrap();

    // Verify page 1 fields
    let page1_values = instance.field_values_for_page(0);
    assert_eq!(page1_values.len(), 2);

    // Step 4: Fill out page 2 (Address Information)
    let address_value = FieldValue::new_text(
        "street_address",
        "123 Main Street",
        FieldBounds::new(100.0, 50.0, 300.0, 30.0),
        1,
    );

    let city_value = FieldValue::new_text(
        "city",
        "San Francisco",
        FieldBounds::new(100.0, 100.0, 200.0, 30.0),
        1,
    );

    let state_value =
        FieldValue::new_text("state", "CA", FieldBounds::new(320.0, 100.0, 80.0, 30.0), 1);

    instance
        .set_field_value("street_address", address_value)
        .unwrap();
    instance.set_field_value("city", city_value).unwrap();
    instance.set_field_value("state", state_value).unwrap();

    // Verify page 2 fields
    let page2_values = instance.field_values_for_page(1);
    assert_eq!(page2_values.len(), 3);

    // Step 5: Fill out page 3 (Signature)
    use form_factor_core::FieldContent;

    let signature_value = FieldValueBuilder::default()
        .field_id("signature")
        .content(FieldContent::Signature {
            present: true,
            shape_index: None,
        })
        .bounds(FieldBounds::new(100.0, 400.0, 300.0, 60.0))
        .page_index(2_usize)
        .confidence(None)
        .verified(true)
        .build()?;

    let date_value = FieldValue::new_text(
        "date_signed",
        "12/4/2024",
        FieldBounds::new(100.0, 480.0, 150.0, 30.0),
        2,
    );

    instance
        .set_field_value("signature", signature_value)
        .unwrap();
    instance.set_field_value("date_signed", date_value).unwrap();

    // Verify page 3 fields
    let page3_values = instance.field_values_for_page(2);
    assert_eq!(page3_values.len(), 2);

    // Step 6: Verify all fields are filled
    let all_values = instance.field_values();
    assert_eq!(all_values.len(), 7);

    // Step 7: Navigate between pages
    assert!(instance.page(0).is_some());
    assert!(instance.page(1).is_some());
    assert!(instance.page(2).is_some());
    assert!(instance.page(3).is_none()); // Out of bounds

    // Step 8: Validate the complete instance
    let validation_result = template.validate_instance(&instance);
    assert!(validation_result.is_valid());
    assert!(validation_result.missing_required().is_empty());
    assert!(validation_result.field_errors().is_empty());

    // Step 9: Test incomplete form validation
    let mut incomplete_instance = DrawingInstance::from_template("employee_onboarding", 3);

    // Only fill page 1
    let name_value = FieldValue::new_text(
        "employee_name",
        "Jane Smith",
        FieldBounds::new(100.0, 50.0, 300.0, 30.0),
        0,
    );
    incomplete_instance
        .set_field_value("employee_name", name_value)
        .unwrap();

    let incomplete_validation = template.validate_instance(&incomplete_instance);
    assert!(!incomplete_validation.is_valid());
    assert!(!incomplete_validation.missing_required().is_empty());

    // Should be missing: employee_email, street_address, city, state, signature, date_signed
    assert_eq!(incomplete_validation.missing_required().len(), 6);

    // Step 10: Test serialization/deserialization of multi-page instance
    let json = instance.to_json().unwrap();
    let deserialized = DrawingInstance::from_json(&json).unwrap();

    assert_eq!(deserialized.template_id(), instance.template_id());
    assert_eq!(deserialized.page_count(), instance.page_count());
    assert_eq!(
        deserialized.field_values().len(),
        instance.field_values().len()
    );

    // Verify all fields survived round-trip
    for field_id in [
        "employee_name",
        "employee_email",
        "street_address",
        "city",
        "state",
        "signature",
        "date_signed",
    ] {
        assert!(deserialized.field_value(field_id).is_some());
    }

    Ok(())
}

#[test]
fn test_page_navigation() {
    let mut page1 = TemplatePage::new(0);
    let page2 = TemplatePage::new(1);
    let page3 = TemplatePage::new(2);

    let field1 = FieldDefinition::builder()
        .id("field1")
        .label("Field 1")
        .field_type(FieldType::TextRegion)
        .page_index(0_usize)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    page1.add_field(field1);

    let _template = DrawingTemplate::builder()
        .id("nav_test")
        .name("Navigation Test")
        .add_page(page1)
        .add_page(page2)
        .add_page(page3)
        .build()
        .unwrap();

    let instance = DrawingInstance::from_template("nav_test", 3);

    // Test page access
    for i in 0..3 {
        let page = instance.page(i);
        assert!(page.is_some());
        assert_eq!(page.unwrap().page_index, i);
    }

    // Test pages slice
    let pages = instance.pages();
    assert_eq!(pages.len(), 3);
    for (i, page) in pages.iter().enumerate() {
        assert_eq!(page.page_index, i);
    }
}

#[test]
fn test_field_distribution_across_pages() -> Result<(), FieldValueBuilderError> {
    let mut page1 = TemplatePage::new(0);
    let mut page2 = TemplatePage::new(1);

    for i in 0..5 {
        let field = FieldDefinition::builder()
            .id(format!("page1_field{}", i))
            .label(format!("Field {}", i))
            .field_type(FieldType::TextRegion)
            .page_index(0_usize)
            .bounds(FieldBounds::new(
                10.0,
                20.0 + (i as f32 * 40.0),
                100.0,
                30.0,
            ))
            .build()
            .unwrap();
        page1.add_field(field);
    }

    for i in 0..3 {
        let field = FieldDefinition::builder()
            .id(format!("page2_field{}", i))
            .label(format!("Field {}", i))
            .field_type(FieldType::TextRegion)
            .page_index(1_usize)
            .bounds(FieldBounds::new(
                10.0,
                20.0 + (i as f32 * 40.0),
                100.0,
                30.0,
            ))
            .build()
            .unwrap();
        page2.add_field(field);
    }

    let template = DrawingTemplate::builder()
        .id("distribution_test")
        .name("Distribution Test")
        .add_page(page1)
        .add_page(page2)
        .build()
        .unwrap();

    assert_eq!(template.fields().len(), 8);
    assert_eq!(template.fields_for_page(0).len(), 5);
    assert_eq!(template.fields_for_page(1).len(), 3);
    assert_eq!(template.fields_for_page(2).len(), 0);

    let mut instance = DrawingInstance::from_template("distribution_test", 2);

    for i in 0..5 {
        let value = FieldValue::new_text(
            format!("page1_field{}", i),
            format!("Value {}", i),
            FieldBounds::new(10.0, 20.0, 100.0, 30.0),
            0,
        );
        instance
            .set_field_value(&format!("page1_field{}", i), value)
            .unwrap();
    }

    for i in 0..3 {
        let value = FieldValue::new_text(
            format!("page2_field{}", i),
            format!("Value {}", i),
            FieldBounds::new(10.0, 20.0, 100.0, 30.0),
            1,
        );
        instance
            .set_field_value(&format!("page2_field{}", i), value)
            .unwrap();
    }

    assert_eq!(instance.field_values_for_page(0).len(), 5);
    assert_eq!(instance.field_values_for_page(1).len(), 3);
    assert_eq!(instance.field_values_for_page(2).len(), 0);
    Ok(())
}
