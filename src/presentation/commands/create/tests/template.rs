//! Integration Tests for Template Generation

use tempfile::TempDir;

use crate::presentation::cli::CreateAction;
use crate::presentation::commands::create;

#[test]
fn it_should_generate_template_with_default_path() {
    let temp_dir = TempDir::new().unwrap();
    
    // Change to temp directory so template is created there
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let action = CreateAction::Template {
        output_path: None,
    };

    let result = create::handle_create_command(action, temp_dir.path());
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok(), "Template generation should succeed: {:?}", result.err());

    // Verify file exists at default path
    let template_path = temp_dir.path().join("environment-template.json");
    assert!(
        template_path.exists(),
        "Template file should be created at: {}",
        template_path.display()
    );

    // Verify file content is valid JSON
    let content = std::fs::read_to_string(&template_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("Template should be valid JSON");

    // Verify structure
    assert!(parsed["environment"]["name"].is_string());
    assert!(parsed["ssh_credentials"]["private_key_path"].is_string());
    assert_eq!(parsed["ssh_credentials"]["username"], "torrust");
    assert_eq!(parsed["ssh_credentials"]["port"], 22);
}

#[test]
fn it_should_generate_template_with_custom_path() {
    let temp_dir = TempDir::new().unwrap();
    let custom_path = temp_dir.path().join("config").join("my-env.json");

    let action = CreateAction::Template {
        output_path: Some(custom_path.clone()),
    };

    let result = create::handle_create_command(action, temp_dir.path());

    assert!(result.is_ok(), "Template generation should succeed");

    // Verify file exists at custom path
    assert!(
        custom_path.exists(),
        "Template file should be created at custom path: {}",
        custom_path.display()
    );

    // Verify parent directory was created
    assert!(custom_path.parent().unwrap().exists());
}

#[test]
fn it_should_generate_valid_json_template() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("test.json");

    let action = CreateAction::Template {
        output_path: Some(template_path.clone()),
    };

    create::handle_create_command(action, temp_dir.path()).unwrap();

    // Read and parse the generated template
    let content = std::fs::read_to_string(&template_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify structure matches expectations
    assert!(parsed.is_object());
    assert!(parsed["environment"].is_object());
    assert!(parsed["ssh_credentials"].is_object());
    
    // Verify placeholder values
    assert_eq!(parsed["environment"]["name"], "REPLACE_WITH_ENVIRONMENT_NAME");
    assert_eq!(parsed["ssh_credentials"]["private_key_path"], "REPLACE_WITH_SSH_PRIVATE_KEY_PATH");
    assert_eq!(parsed["ssh_credentials"]["public_key_path"], "REPLACE_WITH_SSH_PUBLIC_KEY_PATH");
    
    // Verify default values
    assert_eq!(parsed["ssh_credentials"]["username"], "torrust");
    assert_eq!(parsed["ssh_credentials"]["port"], 22);
}

#[test]
fn it_should_create_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let deep_path = temp_dir.path().join("a").join("b").join("c").join("template.json");

    let action = CreateAction::Template {
        output_path: Some(deep_path.clone()),
    };

    let result = create::handle_create_command(action, temp_dir.path());

    assert!(result.is_ok(), "Should create parent directories");
    assert!(deep_path.exists(), "Template should be created at deep path");
    assert!(deep_path.parent().unwrap().exists(), "Parent directories should exist");
}
