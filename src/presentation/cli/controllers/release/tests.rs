//! Tests for the Release Command Controller
//!
//! This module contains integration tests for the release command controller,
//! testing error handling and workflow execution.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;
use tempfile::TempDir;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;
use crate::presentation::cli::controllers::constants::DEFAULT_LOCK_TIMEOUT;
use crate::presentation::cli::controllers::release::handler::ReleaseCommandController;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::testing::TestUserOutput;
use crate::presentation::cli::views::{UserOutput, VerbosityLevel};
use crate::shared::clock::Clock;
use crate::shared::SystemClock;

/// Create test dependencies for release command handler tests
#[allow(clippy::type_complexity)]
fn create_test_dependencies(
    temp_dir: &TempDir,
) -> (
    Arc<ReentrantMutex<RefCell<UserOutput>>>,
    Arc<dyn EnvironmentRepository + Send + Sync>,
    Arc<dyn Clock>,
) {
    let (user_output, _, _) = TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
    let data_dir = temp_dir.path().join("data");
    let file_repository_factory = FileRepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
    let repository = file_repository_factory.create(data_dir);
    let clock = Arc::new(SystemClock);

    (user_output, repository, clock)
}

mod environment_name_validation {
    use super::*;
    use crate::presentation::cli::controllers::release::errors::ReleaseSubcommandError;

    #[tokio::test]
    async fn it_should_reject_names_with_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("invalid_name", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(ReleaseSubcommandError::InvalidEnvironmentName { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_reject_empty_names() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(ReleaseSubcommandError::InvalidEnvironmentName { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_reject_names_starting_with_hyphen() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("-invalid", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(ReleaseSubcommandError::InvalidEnvironmentName { .. })
        ));
    }
}

mod workflow_errors {
    use super::*;
    use crate::presentation::cli::controllers::release::errors::ReleaseSubcommandError;

    #[tokio::test]
    async fn it_should_return_application_layer_error_for_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Valid name but environment doesn't exist
        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("production", OutputFormat::Text)
            .await;

        // Should fail with ApplicationLayerError because environment doesn't exist
        assert!(
            matches!(
                result,
                Err(ReleaseSubcommandError::ApplicationLayerError { .. })
            ),
            "Should fail with ApplicationLayerError when environment doesn't exist"
        );
    }

    #[tokio::test]
    async fn it_should_return_application_layer_error_for_hyphenated_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("my-test-env", OutputFormat::Text)
            .await;

        // Should fail with ApplicationLayerError because environment doesn't exist
        assert!(
            matches!(
                result,
                Err(ReleaseSubcommandError::ApplicationLayerError { .. })
            ),
            "Should fail with ApplicationLayerError when hyphenated environment doesn't exist"
        );
    }
}
