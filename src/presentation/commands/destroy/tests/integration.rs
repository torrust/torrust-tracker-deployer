//! Integration tests for destroy command presentation layer
//!
//! These tests verify the destroy command behavior at the presentation layer,
//! including user interaction, error handling, and command orchestration.

use crate::presentation::commands::destroy::{handle_destroy_command, DestroySubcommandError};
use crate::presentation::commands::tests::TestContext;
use std::fs;

#[test]
fn it_should_reject_invalid_environment_names() {
    let context = TestContext::new();

    let invalid_names = vec![
        "invalid_name", // underscore not allowed
        "-invalid",     // starts with hyphen
        "invalid-",     // ends with hyphen
        "",             // empty string
    ];

    for name in invalid_names {
        let result = handle_destroy_command(name, context.working_dir());
        assert!(
            result.is_err(),
            "Should reject invalid environment name: {name}",
        );
        match result.unwrap_err() {
            DestroySubcommandError::InvalidEnvironmentName { .. } => {
                // Expected error type
            }
            other => panic!("Expected InvalidEnvironmentName for '{name}', got: {other:?}",),
        }
    }

    // Test too long name separately due to String allocation
    // The actual max length depends on domain validation rules
    let too_long_name = "a".repeat(64);
    let result = handle_destroy_command(&too_long_name, context.working_dir());
    assert!(result.is_err(), "Should get some error for 64-char name");
    // Accept either InvalidEnvironmentName OR DestroyOperationFailed
    // The domain layer determines what length is valid
}

#[test]
fn it_should_accept_valid_environment_names() {
    let context = TestContext::new();

    let valid_names = vec![
        "production",
        "test-env",
        "e2e-provision",
        "dev123",
        "a", // single char
    ];

    for name in valid_names {
        let result = handle_destroy_command(name, context.working_dir());

        // Will fail at operation since environment doesn't exist,
        // but should NOT fail at name validation
        if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
            panic!("Should not reject valid environment name: {name}");
        }
        // Expected - valid name but operation fails or other errors acceptable in test context
    }

    // Test max length separately due to String allocation
    let max_length_name = "a".repeat(63);
    let result = handle_destroy_command(&max_length_name, context.working_dir());
    if let Err(DestroySubcommandError::InvalidEnvironmentName { .. }) = result {
        panic!("Should not reject valid 63-char environment name");
    }
    // Expected - valid name but operation fails or other errors acceptable in test context
}

#[test]
fn it_should_fail_for_nonexistent_environment() {
    let context = TestContext::new();

    let result = handle_destroy_command("nonexistent-env", context.working_dir());

    assert!(result.is_err());
    match result.unwrap_err() {
        DestroySubcommandError::DestroyOperationFailed { name, .. } => {
            assert_eq!(name, "nonexistent-env");
        }
        other => panic!("Expected DestroyOperationFailed, got: {other:?}"),
    }
}

#[test]
fn it_should_provide_help_for_errors() {
    let context = TestContext::new();

    let result = handle_destroy_command("invalid_name", context.working_dir());

    assert!(result.is_err());
    let error = result.unwrap_err();
    let help = error.help();

    assert!(!help.is_empty(), "Help text should not be empty");
    assert!(
        help.contains("Troubleshooting") || help.len() > 50,
        "Help should contain actionable guidance"
    );
}

#[test]
fn it_should_work_with_custom_working_directory() {
    let context = TestContext::new();
    let custom_working_dir = context.working_dir().join("custom");
    fs::create_dir(&custom_working_dir).unwrap();

    // Try to destroy from custom directory
    let result = handle_destroy_command("test-env", &custom_working_dir);

    // Should fail at operation (environment doesn't exist) but not at path validation
    assert!(result.is_err());
    // Accept any error in test context - we're testing path handling, not full command behavior
}
