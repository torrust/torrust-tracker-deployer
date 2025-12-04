//! Test module for Run Command
//!
//! This module contains test infrastructure and test cases for the `RunCommandHandler`.

use std::sync::Arc;

use chrono::Utc;
use tempfile::TempDir;

use super::handler::RunCommandHandler;
use crate::domain::EnvironmentName;
use crate::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;
use crate::testing::mock_clock::MockClock;

/// Helper to create a test handler with mock dependencies in a temp directory
fn create_test_handler() -> (RunCommandHandler, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let clock = Arc::new(MockClock::new(Utc::now()));
    let repository = Arc::new(FileEnvironmentRepository::new(
        temp_dir.path().to_path_buf(),
    ));
    let handler = RunCommandHandler::new(repository, clock);
    (handler, temp_dir)
}

#[test]
fn it_should_create_handler_with_dependencies() {
    let (handler, _temp_dir) = create_test_handler();
    // Handler was created successfully - verify basic construction
    assert!(Arc::strong_count(handler.repository.inner()) >= 1);
}

#[test]
fn it_should_return_environment_not_found_error_when_environment_does_not_exist() {
    let (handler, _temp_dir) = create_test_handler();
    let env_name = EnvironmentName::new("nonexistent-env").unwrap();

    let result = handler.execute(&env_name);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("not found"),
        "Expected 'not found' error, got: {error}"
    );
}
