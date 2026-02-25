//! Tests for the Run Command Controller
//!
//! This module contains integration tests for the run command controller,
//! testing error handling and workflow execution.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;
use tempfile::TempDir;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;
use crate::presentation::cli::controllers::constants::DEFAULT_LOCK_TIMEOUT;
use crate::presentation::cli::controllers::run::handler::RunCommandController;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::testing::TestUserOutput;
use crate::presentation::cli::views::{UserOutput, VerbosityLevel};
use crate::shared::clock::Clock;
use crate::shared::SystemClock;

/// Create test dependencies for run command handler tests
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
    use crate::presentation::cli::controllers::run::errors::RunSubcommandError;

    #[tokio::test]
    async fn it_should_reject_names_with_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = RunCommandController::new(repository, clock, user_output)
            .execute("invalid_name", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(RunSubcommandError::InvalidEnvironmentName { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_reject_empty_names() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = RunCommandController::new(repository, clock, user_output)
            .execute("", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(RunSubcommandError::InvalidEnvironmentName { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_reject_names_starting_with_hyphen() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = RunCommandController::new(repository, clock, user_output)
            .execute("-invalid", OutputFormat::Text)
            .await;

        assert!(matches!(
            result,
            Err(RunSubcommandError::InvalidEnvironmentName { .. })
        ));
    }
}

mod real_workflow {
    use super::*;
    use crate::presentation::cli::controllers::run::errors::RunSubcommandError;

    #[tokio::test]
    async fn it_should_return_not_accessible_when_environment_does_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Valid environment name but environment doesn't exist
        let result = RunCommandController::new(repository, clock, user_output)
            .execute("production", OutputFormat::Text)
            .await;

        assert!(
            matches!(
                result,
                Err(RunSubcommandError::EnvironmentNotAccessible { .. })
            ),
            "Should fail with EnvironmentNotAccessible for non-existent environment"
        );
    }

    #[tokio::test]
    async fn it_should_return_not_accessible_for_hyphenated_names_when_env_does_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = RunCommandController::new(repository, clock, user_output)
            .execute("my-test-env", OutputFormat::Text)
            .await;

        assert!(
            matches!(
                result,
                Err(RunSubcommandError::EnvironmentNotAccessible { .. })
            ),
            "Should fail with EnvironmentNotAccessible for non-existent environment"
        );
    }
}
