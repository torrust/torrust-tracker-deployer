//! Repository factory for creating environment-specific repositories
//!
//! This module provides a factory pattern for creating `FileEnvironmentRepository`
//! instances with consistent configuration (like lock timeouts) but environment-specific
//! data directories.
//!
//! # Design Rationale
//!
//! The repository needs environment-specific data directories that are only known at runtime
//! (e.g., `data/production/`, `data/staging/`). However, configuration like lock timeout
//! is known at application startup. The factory pattern allows us to:
//!
//! 1. Configure the factory once at startup with compile-time known settings
//! 2. Create environment-specific repositories at runtime with the correct data directory
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use std::path::PathBuf;
//! use torrust_tracker_deploy::infrastructure::persistence::repository_factory::RepositoryFactory;
//!
//! // Create factory at application startup
//! let factory = RepositoryFactory::new(Duration::from_secs(30));
//!
//! // Create environment-specific repository at runtime
//! let data_dir = PathBuf::from("data/production");
//! let repo = factory.create(data_dir);
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::filesystem::file_environment_repository::FileEnvironmentRepository;

/// Factory for creating `FileEnvironmentRepository` instances
///
/// The factory is configured once at application startup with settings like lock timeout,
/// then used to create environment-specific repositories at runtime.
#[derive(Clone)]
pub struct RepositoryFactory {
    /// Lock acquisition timeout for all repositories created by this factory
    lock_timeout: Duration,
}

impl RepositoryFactory {
    /// Create a new repository factory with the specified lock timeout
    ///
    /// # Arguments
    ///
    /// * `lock_timeout` - Maximum time to wait for lock acquisition in created repositories
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use torrust_tracker_deploy::infrastructure::persistence::repository_factory::RepositoryFactory;
    ///
    /// let factory = RepositoryFactory::new(Duration::from_secs(30));
    /// ```
    #[must_use]
    pub fn new(lock_timeout: Duration) -> Self {
        Self { lock_timeout }
    }

    /// Create a new `FileEnvironmentRepository` for a specific data directory
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Base directory for environment state files (e.g., `data/production`)
    ///
    /// # Returns
    ///
    /// An `Arc`-wrapped trait object that can be used as `Arc<dyn EnvironmentRepository>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use std::path::PathBuf;
    /// use torrust_tracker_deploy::infrastructure::persistence::repository_factory::RepositoryFactory;
    ///
    /// let factory = RepositoryFactory::new(Duration::from_secs(30));
    /// let repo = factory.create(PathBuf::from("data/production"));
    /// ```
    #[must_use]
    pub fn create(&self, data_dir: PathBuf) -> Arc<dyn EnvironmentRepository> {
        let repository =
            FileEnvironmentRepository::new(data_dir).with_lock_timeout(self.lock_timeout);
        Arc::new(repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_repository_factory_with_timeout() {
        let timeout = Duration::from_secs(30);
        let factory = RepositoryFactory::new(timeout);

        // Verify factory was created (we can't directly inspect timeout, but creation succeeds)
        assert_eq!(factory.lock_timeout, timeout);
    }

    #[test]
    fn it_should_create_repository_with_specific_data_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let factory = RepositoryFactory::new(Duration::from_secs(10));

        let data_dir = temp_dir.path().join("production");
        let _repo = factory.create(data_dir);

        // Repository creation should succeed (we just verify it doesn't panic)
    }

    #[test]
    fn it_should_create_multiple_repositories_from_same_factory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let factory = RepositoryFactory::new(Duration::from_secs(10));

        let prod_dir = temp_dir.path().join("production");
        let staging_dir = temp_dir.path().join("staging");

        let _repo1 = factory.create(prod_dir);
        let _repo2 = factory.create(staging_dir);

        // Both repositories should be created successfully
    }

    #[test]
    fn it_should_be_clonable() {
        let factory = RepositoryFactory::new(Duration::from_secs(30));
        let factory_clone = factory.clone();

        assert_eq!(factory.lock_timeout, factory_clone.lock_timeout);
    }

    #[test]
    fn it_should_create_repository_that_can_save_and_load_environment() {
        #[allow(unused_imports)] // Needed for trait methods on Arc<dyn EnvironmentRepository>
        use crate::domain::environment::repository::EnvironmentRepository;
        use crate::domain::environment::{Environment, EnvironmentName};
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let factory = RepositoryFactory::new(Duration::from_secs(10));

        // Create an environment
        let env_name =
            EnvironmentName::new("test-env".to_string()).expect("Valid environment name");
        let ssh_credentials = SshCredentials::new(
            temp_dir.path().join("test_key"),
            temp_dir.path().join("test_key.pub"),
            Username::new("test_user").expect("Valid username"),
        );
        let environment = Environment::new(env_name.clone(), ssh_credentials);

        // Create repository for this specific environment's data directory
        let repo = factory.create(environment.data_dir().clone());

        // Convert to AnyEnvironmentState for repository operations
        let env_state = environment.into_any();

        // Save the environment
        repo.save(&env_state).expect("Should save successfully");

        // Load it back
        let loaded = repo
            .load(&env_name)
            .expect("Should load successfully")
            .expect("Environment should exist");

        assert_eq!(loaded.name(), &env_name);
    }
}
