//! Tests for template validation functionality

use form_factor_core::{
    FieldBounds, FieldDefinition, FieldType, FieldValue, FormInstance, FormTemplate,
};
use form_factor_drawing::{DrawingInstance, DrawingTemplate, TemplatePage};

/// Helper to create a simple template with required fields
fn create_template_with_required_fields() -> DrawingTemplate {
    let field1 = FieldDefinition::builder()
        .id("required_email")
        .label("Email Address")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let field2 = FieldDefinition::builder()
        .id("optional_phone")
        .label("Phone Number")
        .field_type(FieldType::PhoneNumber)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 60.0, 200.0, 30.0))
        .required(false)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field1);
    page.add_field(field2);

    DrawingTemplate::builder()
        .id("test_template")
        .name("Test Template")
        .version("1.0.0")
        .add_page(page)
        .build()
        .unwrap()
}

#[test]
fn test_validation_success_all_required_fields() {
    let template = create_template_with_required_fields();
    let mut instance = DrawingInstance::from_template("test_template", 1);

    let email_value = FieldValue::new_text(
        "required_email",
        "test@example.com",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("required_email", email_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
    assert!(result.missing_required().is_empty());
    assert!(result.field_errors().is_empty());
}

#[test]
fn test_validation_missing_required_field() {
    let template = create_template_with_required_fields();
    let instance = DrawingInstance::from_template("test_template", 1);

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
    assert_eq!(result.missing_required().len(), 1);
    assert!(
        result
            .missing_required()
            .contains(&"required_email".to_string())
    );
}

#[test]
fn test_validation_optional_field_can_be_empty() {
    let template = create_template_with_required_fields();
    let mut instance = DrawingInstance::from_template("test_template", 1);

    let email_value = FieldValue::new_text(
        "required_email",
        "test@example.com",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("required_email", email_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_email_pattern() {
    let field = FieldDefinition::builder()
        .id("email_field")
        .label("Email")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("email_template")
        .name("Email Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("email_template", 1);

    let invalid_email = FieldValue::new_text(
        "email_field",
        "not-an-email",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("email_field", invalid_email)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
    assert!(!result.field_errors().is_empty());
}

#[test]
fn test_validation_valid_email() {
    let field = FieldDefinition::builder()
        .id("email_field")
        .label("Email")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("email_template")
        .name("Email Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("email_template", 1);

    let valid_email = FieldValue::new_text(
        "email_field",
        "user@example.com",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("email_field", valid_email)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_ssn_pattern() {
    let field = FieldDefinition::builder()
        .id("ssn_field")
        .label("SSN")
        .field_type(FieldType::SSN)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("ssn_template")
        .name("SSN Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("ssn_template", 1);

    let invalid_ssn = FieldValue::new_text(
        "ssn_field",
        "123456789",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance.set_field_value("ssn_field", invalid_ssn).unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());

    let mut instance = DrawingInstance::from_template("ssn_template", 1);
    let valid_ssn = FieldValue::new_text(
        "ssn_field",
        "123-45-6789",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance.set_field_value("ssn_field", valid_ssn).unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_phone_number_pattern() {
    let field = FieldDefinition::builder()
        .id("phone_field")
        .label("Phone")
        .field_type(FieldType::PhoneNumber)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("phone_template")
        .name("Phone Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("phone_template", 1);
    let valid_phone = FieldValue::new_text(
        "phone_field",
        "(555) 123-4567",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("phone_field", valid_phone)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_zip_code_pattern() {
    let field = FieldDefinition::builder()
        .id("zip_field")
        .label("ZIP Code")
        .field_type(FieldType::ZipCode)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("zip_template")
        .name("ZIP Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("zip_template", 1);
    let valid_zip = FieldValue::new_text(
        "zip_field",
        "12345",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance.set_field_value("zip_field", valid_zip).unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());

    let mut instance = DrawingInstance::from_template("zip_template", 1);
    let valid_zip_plus4 = FieldValue::new_text(
        "zip_field",
        "12345-6789",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("zip_field", valid_zip_plus4)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_custom_pattern() {
    let field = FieldDefinition::builder()
        .id("custom_field")
        .label("Custom Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .validation_pattern(r"^[A-Z]{3}-\d{4}$")
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("custom_template")
        .name("Custom Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("custom_template", 1);
    let invalid_value = FieldValue::new_text(
        "custom_field",
        "ABC123",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("custom_field", invalid_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());

    let mut instance = DrawingInstance::from_template("custom_template", 1);
    let valid_value = FieldValue::new_text(
        "custom_field",
        "ABC-1234",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("custom_field", valid_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());
}

#[test]
fn test_validation_type_mismatch() {
    let field = FieldDefinition::builder()
        .id("checkbox_field")
        .label("Checkbox")
        .field_type(FieldType::Checkbox)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 20.0, 20.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("checkbox_template")
        .name("Checkbox Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("checkbox_template", 1);

    let wrong_type_value = FieldValue::new_text(
        "checkbox_field",
        "text instead of boolean",
        FieldBounds::new(10.0, 20.0, 20.0, 20.0),
        0,
    );
    instance
        .set_field_value("checkbox_field", wrong_type_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
}

#[test]
fn test_validation_template_id_mismatch() {
    let template = create_template_with_required_fields();
    let mut instance = DrawingInstance::from_template("wrong_template_id", 1);

    let email_value = FieldValue::new_text(
        "required_email",
        "test@example.com",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("required_email", email_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
}

#[test]
fn test_validation_empty_required_field() {
    let field = FieldDefinition::builder()
        .id("required_text")
        .label("Required Text")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
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

    let mut instance = DrawingInstance::from_template("test_template", 1);

    let empty_value = FieldValue::new_empty(
        "required_text",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("required_text", empty_value)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
    assert!(
        result
            .missing_required()
            .contains(&"required_text".to_string())
    );
}

#[test]
fn test_validation_date_pattern() {
    let field = FieldDefinition::builder()
        .id("date_field")
        .label("Date")
        .field_type(FieldType::Date)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("date_template")
        .name("Date Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("date_template", 1);
    let valid_date = FieldValue::new_text(
        "date_field",
        "12/31/2024",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance.set_field_value("date_field", valid_date).unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());

    let mut instance = DrawingInstance::from_template("date_template", 1);
    let invalid_date = FieldValue::new_text(
        "date_field",
        "2024-12-31",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("date_field", invalid_date)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
}

#[test]
fn test_validation_state_code_pattern() {
    let field = FieldDefinition::builder()
        .id("state_field")
        .label("State")
        .field_type(FieldType::State)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    let template = DrawingTemplate::builder()
        .id("state_template")
        .name("State Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("state_template", 1);
    let valid_state = FieldValue::new_text(
        "state_field",
        "CA",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("state_field", valid_state)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(result.is_valid());

    let mut instance = DrawingInstance::from_template("state_template", 1);
    let invalid_state = FieldValue::new_text(
        "state_field",
        "California",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("state_field", invalid_state)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
}

#[test]
fn test_validation_multiple_errors() {
    let field1 = FieldDefinition::builder()
        .id("required_email")
        .label("Email")
        .field_type(FieldType::Email)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let field2 = FieldDefinition::builder()
        .id("required_ssn")
        .label("SSN")
        .field_type(FieldType::SSN)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 60.0, 200.0, 30.0))
        .required(true)
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field1);
    page.add_field(field2);

    let template = DrawingTemplate::builder()
        .id("multi_field_template")
        .name("Multi Field Template")
        .add_page(page)
        .build()
        .unwrap();

    let mut instance = DrawingInstance::from_template("multi_field_template", 1);

    let invalid_email = FieldValue::new_text(
        "required_email",
        "not-an-email",
        FieldBounds::new(10.0, 20.0, 200.0, 30.0),
        0,
    );
    instance
        .set_field_value("required_email", invalid_email)
        .unwrap();

    let result = template.validate_instance(&instance);
    assert!(!result.is_valid());
    assert!(!result.missing_required().is_empty());
    assert!(!result.field_errors().is_empty());
}
