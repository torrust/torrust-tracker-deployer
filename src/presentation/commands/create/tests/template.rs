//! Integration Tests for Template Generation

use std::sync::{Arc, Mutex};

use crate::presentation::cli::CreateAction;
use crate::presentation::commands::create;
use crate::presentation::commands::tests::TestContext;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};

/// Helper to create test `UserOutput`
fn create_test_user_output() -> Arc<Mutex<UserOutput>> {
    Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)))
}

#[test]
fn it_should_generate_template_with_default_path() {
    let test_context = TestContext::new();

    // Change to temp directory so template is created there
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(test_context.working_dir()).unwrap();

    let action = CreateAction::Template { output_path: None };
    let user_output = create_test_user_output();

    let result = create::handle_create_command(action, test_context.working_dir(), &user_output);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(
        result.is_ok(),
        "Template generation should succeed: {:?}",
        result.err()
    );

    // Verify file exists at default path
    let template_path = test_context.working_dir().join("environment-template.json");
    assert!(
        template_path.exists(),
        "Template file should be created at: {}",
        template_path.display()
    );

    // Verify file content is valid JSON
    let file_content = std::fs::read_to_string(&template_path).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&file_content).expect("Template should be valid JSON");

    // Verify structure
    assert!(parsed["environment"]["name"].is_string());
    assert!(parsed["ssh_credentials"]["private_key_path"].is_string());
    assert_eq!(parsed["ssh_credentials"]["username"], "torrust");
    assert_eq!(parsed["ssh_credentials"]["port"], 22);
}

#[test]
fn it_should_generate_template_with_custom_path() {
    let test_context = TestContext::new();
    let custom_path = test_context
        .working_dir()
        .join("config")
        .join("my-env.json");

    let action = CreateAction::Template {
        output_path: Some(custom_path.clone()),
    };
    let user_output = create_test_user_output();

    let result = create::handle_create_command(action, test_context.working_dir(), &user_output);

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
    let test_context = TestContext::new();
    let template_path = test_context.working_dir().join("test.json");

    let action = CreateAction::Template {
        output_path: Some(template_path.clone()),
    };
    let user_output = create_test_user_output();

    create::handle_create_command(action, test_context.working_dir(), &user_output).unwrap();

    // Read and parse the generated template
    let file_content = std::fs::read_to_string(&template_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&file_content).unwrap();

    // Verify structure matches expectations
    assert!(parsed.is_object());
    assert!(parsed["environment"].is_object());
    assert!(parsed["ssh_credentials"].is_object());

    // Verify placeholder values
    assert_eq!(
        parsed["environment"]["name"],
        "REPLACE_WITH_ENVIRONMENT_NAME"
    );
    assert_eq!(
        parsed["ssh_credentials"]["private_key_path"],
        "REPLACE_WITH_SSH_PRIVATE_KEY_PATH"
    );
    assert_eq!(
        parsed["ssh_credentials"]["public_key_path"],
        "REPLACE_WITH_SSH_PUBLIC_KEY_PATH"
    );

    // Verify default values
    assert_eq!(parsed["ssh_credentials"]["username"], "torrust");
    assert_eq!(parsed["ssh_credentials"]["port"], 22);
}

#[test]
fn it_should_create_parent_directories() {
    let test_context = TestContext::new();
    let deep_path = test_context
        .working_dir()
        .join("a")
        .join("b")
        .join("c")
        .join("template.json");

    let action = CreateAction::Template {
        output_path: Some(deep_path.clone()),
    };
    let user_output = create_test_user_output();

    let result = create::handle_create_command(action, test_context.working_dir(), &user_output);

    assert!(result.is_ok(), "Should create parent directories");
    assert!(
        deep_path.exists(),
        "Template should be created at deep path"
    );
    assert!(
        deep_path.parent().unwrap().exists(),
        "Parent directories should exist"
    );
}
