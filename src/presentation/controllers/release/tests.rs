//! Tests for the Release Command Controller
//!
//! This module contains integration tests for the release command controller,
//! testing error handling and workflow execution.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;
use tempfile::TempDir;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
use crate::presentation::controllers::release::handler::ReleaseCommandController;
use crate::presentation::views::testing::TestUserOutput;
use crate::presentation::views::{UserOutput, VerbosityLevel};
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
    let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
    let repository = repository_factory.create(data_dir);
    let clock = Arc::new(SystemClock);

    (user_output, repository, clock)
}

mod environment_name_validation {
    use super::*;
    use crate::presentation::controllers::release::errors::ReleaseSubcommandError;

    #[tokio::test]
    async fn it_should_reject_names_with_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("invalid_name")
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
            .execute("")
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
            .execute("-invalid")
            .await;

        assert!(matches!(
            result,
            Err(ReleaseSubcommandError::InvalidEnvironmentName { .. })
        ));
    }
}

mod scaffolding_workflow {
    use super::*;

    #[tokio::test]
    async fn it_should_complete_successfully_for_valid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // In scaffolding mode, valid names should succeed without actual release
        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("production")
            .await;

        assert!(
            result.is_ok(),
            "Scaffolding should succeed for valid environment names"
        );
    }

    #[tokio::test]
    async fn it_should_accept_hyphenated_environment_names() {
        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output)
            .execute("my-test-env")
            .await;

        assert!(result.is_ok(), "Scaffolding should accept hyphenated names");
    }
}
