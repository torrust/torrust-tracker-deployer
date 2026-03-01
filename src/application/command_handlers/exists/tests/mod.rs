//! Tests for the `ExistsCommandHandler`
//!
//! These tests verify the handler's core logic:
//!
//! 1. Returns `exists = false` when the environment does not exist
//! 2. Returns `exists = true` when the environment exists
//! 3. Propagates repository errors as `ExistsCommandHandlerError::RepositoryError`

use std::sync::Arc;

use tempfile::TempDir;

use crate::application::command_handlers::exists::errors::ExistsCommandHandlerError;
use crate::application::command_handlers::exists::handler::ExistsCommandHandler;
use crate::domain::environment::repository::{EnvironmentRepository, RepositoryError};
use crate::domain::environment::state::AnyEnvironmentState;
use crate::domain::environment::testing::EnvironmentTestBuilder;
use crate::domain::EnvironmentName;
use crate::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;

fn create_test_repo() -> (Arc<FileEnvironmentRepository>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo = Arc::new(FileEnvironmentRepository::new(
        temp_dir.path().to_path_buf(),
    ));
    (repo, temp_dir)
}

#[test]
fn it_should_return_false_when_environment_does_not_exist() {
    let (repo, _temp_dir) = create_test_repo();
    let handler = ExistsCommandHandler::new(repo);

    let env_name = EnvironmentName::new("nonexistent-env").unwrap();
    let result = handler.execute(&env_name).expect("Expected Ok result");

    assert_eq!(result.name, "nonexistent-env");
    assert!(
        !result.exists,
        "Expected exists=false for non-existent environment"
    );
}

#[test]
fn it_should_return_true_when_environment_exists() {
    let (repo, _temp_dir) = create_test_repo();
    let handler = ExistsCommandHandler::new(repo.clone());

    // Save a test environment to the repository
    let (env, _data_dir, _build_dir, _env_temp) = EnvironmentTestBuilder::new()
        .with_name("test-exists-env")
        .build_with_custom_paths();
    let env_name = env.name().clone();
    repo.save(&AnyEnvironmentState::Created(env))
        .expect("Failed to save test environment");

    let result = handler.execute(&env_name).expect("Expected Ok result");

    assert_eq!(result.name, "test-exists-env");
    assert!(
        result.exists,
        "Expected exists=true for existing environment"
    );
}

#[test]
fn it_should_propagate_repository_error() {
    struct FailingRepository;

    impl EnvironmentRepository for FailingRepository {
        fn save(&self, _env: &AnyEnvironmentState) -> Result<(), RepositoryError> {
            Err(RepositoryError::Internal(anyhow::anyhow!(
                "simulated disk error"
            )))
        }

        fn load(
            &self,
            _name: &EnvironmentName,
        ) -> Result<Option<AnyEnvironmentState>, RepositoryError> {
            Err(RepositoryError::Internal(anyhow::anyhow!(
                "simulated disk error"
            )))
        }

        fn exists(&self, _name: &EnvironmentName) -> Result<bool, RepositoryError> {
            Err(RepositoryError::Internal(anyhow::anyhow!(
                "simulated disk error"
            )))
        }

        fn delete(&self, _name: &EnvironmentName) -> Result<(), RepositoryError> {
            Err(RepositoryError::Internal(anyhow::anyhow!(
                "simulated disk error"
            )))
        }
    }

    let handler = ExistsCommandHandler::new(Arc::new(FailingRepository));
    let env_name = EnvironmentName::new("test-env").unwrap();

    let result = handler.execute(&env_name);

    assert!(
        result.is_err(),
        "Expected Err result from failing repository"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            ExistsCommandHandlerError::RepositoryError(_)
        ),
        "Expected RepositoryError variant"
    );
}
