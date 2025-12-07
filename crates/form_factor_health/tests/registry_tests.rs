//! Tests for TemplateRegistry functionality

use form_factor_core::{FieldBounds, FieldDefinition, FieldType};
use form_factor_drawing::{DrawingTemplate, TemplatePage, TemplateRegistry};
use std::path::PathBuf;

/// Helper function to create a test template
fn create_test_template(id: &str, name: &str) -> DrawingTemplate {
    let field = FieldDefinition::builder()
        .id(format!("field_{}", id))
        .label("Test Field")
        .field_type(FieldType::TextRegion)
        .page_index(0)
        .bounds(FieldBounds::new(10.0, 20.0, 100.0, 30.0))
        .build()
        .unwrap();

    let mut page = TemplatePage::new(0);
    page.add_field(field);

    DrawingTemplate::builder()
        .id(id)
        .name(name)
        .version("1.0.0")
        .add_page(page)
        .build()
        .unwrap()
}

#[test]
fn test_create_registry() {
    let registry = TemplateRegistry::new();
    assert!(registry.is_ok());

    let registry = registry.unwrap();
    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_register_template() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_template", "Test Template");
    registry.register(template);

    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    assert!(registry.contains("test_template"));
}

#[test]
fn test_register_multiple_templates() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template1 = create_test_template("template1", "Template 1");
    let template2 = create_test_template("template2", "Template 2");
    let template3 = create_test_template("template3", "Template 3");

    registry.register(template1);
    registry.register(template2);
    registry.register(template3);

    assert_eq!(registry.len(), 3);
    assert!(registry.contains("template1"));
    assert!(registry.contains("template2"));
    assert!(registry.contains("template3"));
}

#[test]
fn test_register_duplicate_id_replaces() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template1 = create_test_template("test_template", "First Template");
    registry.register(template1);

    let template2 = create_test_template("test_template", "Second Template");
    registry.register(template2);

    assert_eq!(registry.len(), 1);
    let retrieved = registry.get("test_template").unwrap();
    assert_eq!(retrieved.name(), "Second Template");
}

#[test]
fn test_get_template() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_template", "Test Template");
    registry.register(template);

    let retrieved = registry.get("test_template");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), "test_template");
    assert_eq!(retrieved.unwrap().name(), "Test Template");

    let not_found = registry.get("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_get_mut_template() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_template", "Test Template");
    registry.register(template);

    let retrieved = registry.get_mut("test_template");
    assert!(retrieved.is_some());
}

#[test]
fn test_contains() {
    let mut registry = TemplateRegistry::new().unwrap();

    assert!(!registry.contains("test_template"));

    let template = create_test_template("test_template", "Test Template");
    registry.register(template);

    assert!(registry.contains("test_template"));
    assert!(!registry.contains("nonexistent"));
}

#[test]
fn test_list_ids() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template1 = create_test_template("template1", "Template 1");
    let template2 = create_test_template("template2", "Template 2");

    registry.register(template1);
    registry.register(template2);

    let ids = registry.list_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"template1"));
    assert!(ids.contains(&"template2"));
}

#[test]
fn test_list_templates() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template1 = create_test_template("template1", "Template 1");
    let template2 = create_test_template("template2", "Template 2");

    registry.register(template1);
    registry.register(template2);

    let templates = registry.list();
    assert_eq!(templates.len(), 2);
}

#[test]
fn test_clear() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template1 = create_test_template("template1", "Template 1");
    let template2 = create_test_template("template2", "Template 2");

    registry.register(template1);
    registry.register(template2);

    assert_eq!(registry.len(), 2);

    registry.clear();

    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_save_and_load_global() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_save_global", "Test Save Global");

    let save_result = registry.save(&template);
    assert!(save_result.is_ok());

    registry.clear();
    assert_eq!(registry.len(), 0);

    let load_result = registry.load_all();
    assert!(load_result.is_ok());

    assert!(registry.contains("test_save_global"));
    let loaded = registry.get("test_save_global").unwrap();
    assert_eq!(loaded.name(), "Test Save Global");

    let delete_result = registry.delete_from_global("test_save_global");
    assert!(delete_result.is_ok());
}

#[test]
fn test_save_and_load_project() {
    let temp_dir = std::env::temp_dir().join(format!(
        "form_factor_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut registry = TemplateRegistry::new()
        .unwrap()
        .with_project_dir(temp_dir.clone());

    let template = create_test_template("test_save_project", "Test Save Project");

    let save_result = registry.save_to_project(&template);
    assert!(save_result.is_ok());

    registry.clear();
    assert_eq!(registry.len(), 0);

    let load_result = registry.load_all();
    assert!(load_result.is_ok());

    assert!(registry.contains("test_save_project"));
    let loaded = registry.get("test_save_project").unwrap();
    assert_eq!(loaded.name(), "Test Save Project");

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_project_overrides_global() {
    let temp_dir = std::env::temp_dir().join(format!(
        "form_factor_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut registry = TemplateRegistry::new()
        .unwrap()
        .with_project_dir(temp_dir.clone());

    let global_template = create_test_template("override_test", "Global Template");
    registry.save_to_global(&global_template).unwrap();

    let project_template = create_test_template("override_test", "Project Template");
    registry.save_to_project(&project_template).unwrap();

    registry.clear();
    registry.load_all().unwrap();

    assert!(registry.contains("override_test"));
    let loaded = registry.get("override_test").unwrap();
    assert_eq!(loaded.name(), "Project Template");

    registry.delete_from_global("override_test").unwrap();
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_delete_from_global() {
    let mut registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_delete", "Test Delete");
    registry.save(&template).unwrap();

    assert!(
        registry.contains("test_delete") || {
            registry.load_all().unwrap();
            registry.contains("test_delete")
        }
    );

    let delete_result = registry.delete_from_global("test_delete");
    assert!(delete_result.is_ok());

    assert!(!registry.contains("test_delete"));
}

#[test]
fn test_save_without_project_dir_fails() {
    let registry = TemplateRegistry::new().unwrap();

    let template = create_test_template("test_template", "Test Template");

    let result = registry.save_to_project(&template);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No project directory")
    );
}

#[test]
fn test_with_project_dir() {
    let temp_dir = PathBuf::from("/tmp/test_project");

    let registry = TemplateRegistry::new().unwrap().with_project_dir(temp_dir);

    assert_eq!(registry.len(), 0);
}

#[test]
fn test_load_all_with_empty_directories() {
    let temp_dir = std::env::temp_dir().join(format!(
        "form_factor_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut registry = TemplateRegistry::new()
        .unwrap()
        .with_project_dir(temp_dir.clone());

    let initial_count = registry.len();

    let result = registry.load_all();
    assert!(result.is_ok());

    // Verify that loading from empty project directory succeeds
    // (global directory may have templates from other tests, so we don't check exact count)
    assert!(registry.len() >= initial_count);

    std::fs::remove_dir_all(&temp_dir).ok();
}
