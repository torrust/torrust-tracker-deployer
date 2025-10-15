//! File-based implementation of the `EnvironmentRepository` trait
//!
//! This module provides a file-based implementation that persists environment
//! state to JSON files using the generic `JsonFileRepository` as a collaborator.
//!
//! # Architecture
//!
//! This implementation follows the delegation pattern:
//! - `FileEnvironmentRepository` implements domain-specific logic
//! - `JsonFileRepository` handles generic file I/O, locking, and atomic writes
//!
//! # File Structure
//!
//! ```text
//! ./data/{env_name}/environment.json       # Environment state
//! ./data/{env_name}/environment.json.lock  # Lock file (contains process ID)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;
//! use torrust_tracker_deployer_lib::domain::environment::repository::EnvironmentRepository;
//!
//! let repo = FileEnvironmentRepository::new(PathBuf::from("./data"));
//!
//! // Operations automatically handle locking and atomic writes
//! // repo.save(&env)?;
//! // let loaded = repo.load(&env_name)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::{EnvironmentRepository, RepositoryError};
use crate::domain::environment::state::AnyEnvironmentState;
use crate::infrastructure::persistence::filesystem::json_file_repository::{
    JsonFileError, JsonFileRepository,
};

/// File-based implementation of `EnvironmentRepository`
///
/// Persists environment state to JSON files with atomic writes and file locking.
/// Uses `JsonFileRepository` as a collaborator for generic file operations.
///
/// # Directory Structure
///
/// Each environment gets its own directory under the base directory:
/// - `{base_dir}/{env_name}/environment.json` - Environment file
/// - `{base_dir}/{env_name}/environment.json.lock` - Lock file
pub struct FileEnvironmentRepository {
    /// Base directory for environment state files (typically "./data")
    base_dir: PathBuf,
    /// Generic JSON file repository for file operations
    json_repo: JsonFileRepository,
}

impl FileEnvironmentRepository {
    /// Create a new file-based environment repository
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory where state files will be stored.
    ///   Directory will be created if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;
    ///
    /// let repo = FileEnvironmentRepository::new(PathBuf::from("./data"));
    /// ```
    #[must_use]
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            json_repo: JsonFileRepository::new(Duration::from_secs(10)),
        }
    }

    /// Create repository with custom lock timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for lock acquisition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;
    ///
    /// let repo = FileEnvironmentRepository::new(PathBuf::from("./data"))
    ///     .with_lock_timeout(Duration::from_secs(30));
    /// ```
    #[must_use]
    pub fn with_lock_timeout(mut self, timeout: Duration) -> Self {
        self.json_repo = JsonFileRepository::new(timeout);
        self
    }

    /// Get the environment file path for an environment
    fn environment_file_path(&self, name: &EnvironmentName) -> PathBuf {
        self.base_dir.join(name.as_str()).join("environment.json")
    }

    /// Get the directory path for an environment
    fn env_dir_path(&self, name: &EnvironmentName) -> PathBuf {
        self.base_dir.join(name.as_str())
    }

    /// Convert `JsonFileError` to `RepositoryError`
    fn convert_json_error(error: JsonFileError) -> RepositoryError {
        match error {
            JsonFileError::NotFound { .. } => RepositoryError::NotFound,
            JsonFileError::Conflict { .. } => RepositoryError::Conflict,
            JsonFileError::Internal(e) => RepositoryError::Internal(e),
        }
    }
}

impl EnvironmentRepository for FileEnvironmentRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), RepositoryError> {
        let file_path = self.environment_file_path(env.name());

        self.json_repo
            .save(&file_path, env)
            .map_err(Self::convert_json_error)
    }

    fn load(&self, name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, RepositoryError> {
        let file_path = self.environment_file_path(name);

        self.json_repo
            .load(&file_path)
            .map_err(Self::convert_json_error)
    }

    fn exists(&self, name: &EnvironmentName) -> Result<bool, RepositoryError> {
        let file_path = self.environment_file_path(name);
        Ok(self.json_repo.exists(&file_path))
    }

    fn delete(&self, name: &EnvironmentName) -> Result<(), RepositoryError> {
        let file_path = self.environment_file_path(name);

        self.json_repo
            .delete(&file_path)
            .map_err(Self::convert_json_error)?;

        // Optionally, remove the environment directory if it's empty
        let env_dir = self.env_dir_path(name);
        if let Ok(mut entries) = fs::read_dir(&env_dir) {
            if entries.next().is_none() {
                // Directory is empty, remove it (best effort - ignore errors)
                drop(fs::remove_dir(&env_dir));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ssh::credentials::SshCredentials;
    use crate::domain::environment::Environment;
    use crate::shared::Username;
    use rstest::rstest;
    use std::error::Error as StdError;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_ssh_credentials() -> SshCredentials {
        let username = Username::new("test-user".to_string()).unwrap();
        SshCredentials::new(
            PathBuf::from("/tmp/test_key"),
            PathBuf::from("/tmp/test_key.pub"),
            username,
        )
    }

    fn create_test_environment(name: &str) -> Environment {
        let env_name = EnvironmentName::new(name.to_string()).unwrap();
        let ssh_credentials = create_test_ssh_credentials();
        Environment::new(env_name, ssh_credentials, 22)
    }

    #[test]
    fn it_should_create_repository_with_default_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        assert_eq!(repo.base_dir, temp_dir.path());
        assert_eq!(repo.json_repo.lock_timeout, Duration::from_secs(10));
    }

    #[test]
    fn it_should_create_repository_with_custom_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let custom_timeout = Duration::from_secs(30);

        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf())
            .with_lock_timeout(custom_timeout);

        assert_eq!(repo.json_repo.lock_timeout, custom_timeout);
    }

    #[rstest]
    #[case("e2e-config", "e2e-config/environment.json")]
    #[case("e2e-full", "e2e-full/environment.json")]
    #[case("e2e-provision", "e2e-provision/environment.json")]
    #[case("my-env", "my-env/environment.json")]
    fn it_should_create_environment_file_in_environment_specific_subdirectory(
        #[case] env_name: &str,
        #[case] expected_relative_path: &str,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment(env_name);
        let env_name_obj = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save the state
        repo.save(&state).unwrap();

        // Verify the file was created in the correct subdirectory
        let expected_path = temp_dir.path().join(expected_relative_path);
        assert!(
            expected_path.exists(),
            "Expected state file to exist at: {}",
            expected_path.display()
        );

        // Verify the file is a valid JSON file with environment data
        let loaded = repo.load(&env_name_obj).unwrap();
        assert!(loaded.is_some(), "Should be able to load the saved state");
        assert_eq!(
            loaded.unwrap().name(),
            &env_name_obj,
            "Loaded state should have the correct environment name"
        );

        // Verify the subdirectory structure
        let env_dir = temp_dir.path().join(env_name);
        assert!(
            env_dir.exists() && env_dir.is_dir(),
            "Environment directory should exist: {}",
            env_dir.display()
        );
    }

    #[test]
    fn it_should_save_and_load_environment_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save
        repo.save(&state).unwrap();

        // Load
        let loaded = repo.load(&env_name).unwrap();
        assert!(loaded.is_some());

        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.name(), &env_name);
    }

    #[test]
    fn it_should_return_none_when_loading_nonexistent_environment() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env_name = EnvironmentName::new("nonexistent".to_string()).unwrap();
        let result = repo.load(&env_name).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn it_should_check_if_environment_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Before save
        assert!(!repo.exists(&env_name).unwrap());

        // Save
        repo.save(&state).unwrap();

        // After save
        assert!(repo.exists(&env_name).unwrap());
    }

    #[test]
    fn it_should_delete_environment_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save then delete
        repo.save(&state).unwrap();
        assert!(repo.exists(&env_name).unwrap());

        repo.delete(&env_name).unwrap();
        assert!(!repo.exists(&env_name).unwrap());
    }

    #[test]
    fn it_should_delete_nonexistent_environment_without_error() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env_name = EnvironmentName::new("nonexistent".to_string()).unwrap();

        // Delete should succeed even if environment doesn't exist (idempotent)
        repo.delete(&env_name).unwrap();
    }

    #[test]
    fn it_should_create_directory_structure_automatically() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let state = AnyEnvironmentState::Created(env);

        // Save should create directory structure
        repo.save(&state).unwrap();

        let env_dir = temp_dir.path().join("test-env");
        assert!(env_dir.exists());
        assert!(env_dir.is_dir());

        let environment_file = env_dir.join("environment.json");
        assert!(environment_file.exists());
        assert!(environment_file.is_file());
    }

    #[test]
    fn it_should_overwrite_existing_environment() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env1 = create_test_environment("test-env");
        let env_name = env1.name().clone();
        let state1 = AnyEnvironmentState::Created(env1);

        // Save first version
        repo.save(&state1).unwrap();

        // Transition to provisioning state
        let env2 = create_test_environment("test-env");
        let state2 = AnyEnvironmentState::Provisioning(env2.start_provisioning());

        // Save second version (should overwrite)
        repo.save(&state2).unwrap();

        // Load should return latest version
        let loaded = repo.load(&env_name).unwrap().unwrap();
        assert_eq!(loaded.state_name(), "provisioning");
    }

    #[test]
    fn it_should_use_atomic_writes() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let state = AnyEnvironmentState::Created(env);

        // Save
        repo.save(&state).unwrap();

        // Verify no temporary file exists after save
        let temp_file = temp_dir
            .path()
            .join("test-env")
            .join("environment.json.tmp");
        assert!(!temp_file.exists());

        // Verify environment file exists
        let environment_file = temp_dir.path().join("test-env").join("environment.json");
        assert!(environment_file.exists());
    }

    #[test]
    fn it_should_preserve_json_structure() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let state = AnyEnvironmentState::Created(env);

        // Save
        repo.save(&state).unwrap();

        // Read raw JSON
        let environment_file = temp_dir.path().join("test-env").join("environment.json");
        let json_content = fs::read_to_string(environment_file).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn it_should_handle_concurrent_access_with_locking() {
        use crate::infrastructure::persistence::filesystem::file_lock::FileLock;

        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf())
            .with_lock_timeout(Duration::from_millis(100));

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save to create the file
        repo.save(&state).unwrap();

        // Acquire lock manually
        let environment_path = repo.environment_file_path(&env_name);
        let _lock = FileLock::acquire(&environment_path, Duration::from_secs(5)).unwrap();

        // Try to load while lock is held - should timeout and return Conflict
        let result = repo.load(&env_name);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, RepositoryError::Conflict));
    }

    #[test]
    fn it_should_return_conflict_error_on_lock_timeout() {
        use crate::infrastructure::persistence::filesystem::file_lock::FileLock;

        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf())
            .with_lock_timeout(Duration::from_millis(50));

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save to create the file
        repo.save(&state).unwrap();

        // Hold lock in one scope
        let environment_path = repo.environment_file_path(&env_name);
        let _lock = FileLock::acquire(&environment_path, Duration::from_secs(5)).unwrap();

        // Try to save while lock is held
        let result = repo.save(&state);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::Conflict));
    }

    #[test]
    fn it_should_display_error_messages_correctly() {
        assert_eq!(
            RepositoryError::NotFound.to_string(),
            "Environment not found"
        );

        assert_eq!(
            RepositoryError::Conflict.to_string(),
            "Conflict: another process is accessing this environment"
        );

        let internal_err = RepositoryError::Internal(anyhow::anyhow!("test error"));
        assert!(internal_err.to_string().contains("Internal error"));
        assert!(internal_err.to_string().contains("test error"));
    }

    #[test]
    fn it_should_preserve_error_source_chain() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let anyhow_error = anyhow::Error::from(io_error).context("operation failed");
        let repo_error = RepositoryError::Internal(anyhow_error);

        // Verify error chain is preserved
        let mut source = repo_error.source();
        let mut chain_length = 0;

        while let Some(err) = source {
            chain_length += 1;
            source = err.source();
        }

        assert!(
            chain_length >= 2,
            "Error chain should have at least 2 levels"
        );
    }

    #[test]
    fn it_should_clean_up_empty_directories_on_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();
        let state = AnyEnvironmentState::Created(env);

        // Save
        repo.save(&state).unwrap();

        let env_dir = temp_dir.path().join("test-env");
        assert!(env_dir.exists());

        // Delete
        repo.delete(&env_name).unwrap();

        // Directory should be removed if empty
        if env_dir.exists() {
            // If directory still exists, it should only contain lock file
            let entries: Vec<_> = fs::read_dir(&env_dir).unwrap().collect();
            assert!(
                entries.len() <= 1,
                "Directory should be empty or contain only lock file"
            );
        }
    }

    #[test]
    fn it_should_handle_state_transitions() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileEnvironmentRepository::new(temp_dir.path().to_path_buf());

        let env = create_test_environment("test-env");
        let env_name = env.name().clone();

        // Save Created state
        let created_state = AnyEnvironmentState::Created(env.clone());
        repo.save(&created_state).unwrap();

        // Transition to Provisioning
        let provisioning_state = AnyEnvironmentState::Provisioning(env.start_provisioning());
        repo.save(&provisioning_state).unwrap();

        // Load and verify
        let loaded = repo.load(&env_name).unwrap().unwrap();
        assert_eq!(loaded.state_name(), "provisioning");
    }
}
