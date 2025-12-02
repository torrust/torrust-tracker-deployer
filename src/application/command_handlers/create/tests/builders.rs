//! Test builders for Create Command
//!
//! This module provides test builders that simplify test setup by managing
//! dependencies and lifecycle for `CreateCommandHandler` tests.

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tempfile::TempDir;

use crate::application::command_handlers::create::config::{
    EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig,
};
use crate::application::command_handlers::create::CreateCommandHandler;
use crate::domain::environment::{Environment, EnvironmentName};
use crate::domain::provider::{LxdConfig, ProviderConfig};
use crate::domain::ProfileName;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::Clock;
use crate::testing::MockClock;

/// Test builder for `CreateCommandHandler` with sensible defaults and customization options
///
/// This builder simplifies test setup by:
/// - Managing `TempDir` lifecycle
/// - Providing sensible defaults for all dependencies
/// - Allowing selective customization of dependencies
/// - Supporting pre-populated test environments
///
/// # Examples
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
///
/// // Simple command with defaults
/// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new().build();
///
/// // Command with fixed time for deterministic testing
/// use chrono::{TimeZone, Utc};
/// let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
/// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
///     .with_fixed_time(fixed_time)
///     .build();
///
/// // Command with existing environment to test conflict detection
/// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
///     .with_existing_environment("production")
///     .build();
/// ```
pub struct CreateCommandHandlerTestBuilder {
    /// Optional base directory for environment storage
    base_directory: Option<std::path::PathBuf>,

    /// Optional fixed time for deterministic testing
    fixed_time: Option<DateTime<Utc>>,

    /// List of environment names that should already exist
    existing_environments: Vec<String>,
}

impl CreateCommandHandlerTestBuilder {
    /// Create a new test builder with default settings
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
    ///
    /// let builder = CreateCommandHandlerTestBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_directory: None,
            fixed_time: None,
            existing_environments: Vec::new(),
        }
    }

    /// Set a custom base directory for the test environment
    ///
    /// By default, a temporary directory is created. Use this method to
    /// specify a custom location.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new().unwrap();
    /// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
    ///     .with_base_directory(temp_dir.path())
    ///     .build();
    /// ```
    #[must_use]
    pub fn with_base_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_directory = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set a fixed time for deterministic testing
    ///
    /// This configures the mock clock to return a specific timestamp,
    /// making tests reproducible.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
    /// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
    ///     .with_fixed_time(fixed_time)
    ///     .build();
    /// ```
    #[must_use]
    pub fn with_fixed_time(mut self, time: DateTime<Utc>) -> Self {
        self.fixed_time = Some(time);
        self
    }

    /// Add an existing environment to simulate conflicts
    ///
    /// This method pre-creates an environment in the repository so that
    /// tests can verify conflict detection behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
    ///
    /// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
    ///     .with_existing_environment("production")
    ///     .with_existing_environment("staging")
    ///     .build();
    /// ```
    #[must_use]
    pub fn with_existing_environment(mut self, name: &str) -> Self {
        self.existing_environments.push(name.to_string());
        self
    }

    /// Build the `CreateCommandHandler` with configured dependencies
    ///
    /// Returns a tuple of (`CreateCommandHandler`, `TempDir`). The `TempDir` must be
    /// kept alive for the duration of the test to prevent cleanup.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::CreateCommandHandlerTestBuilder;
    ///
    /// let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new().build();
    /// // Use command for testing
    /// ```
    #[must_use]
    pub fn build(self) -> (CreateCommandHandler, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_dir = self
            .base_directory
            .clone()
            .unwrap_or_else(|| temp_dir.path().to_path_buf());

        // Create mock clock with fixed or current time
        let clock_time = self.fixed_time.unwrap_or_else(Utc::now);
        let clock: Arc<dyn Clock> = Arc::new(MockClock::new(clock_time));

        // Create repository with file-based persistence
        let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
        let repository = repository_factory.create(base_dir.clone());

        // Pre-create existing environments if specified
        for env_name in &self.existing_environments {
            self.create_existing_environment(&repository, env_name, &base_dir);
        }

        let command = CreateCommandHandler::new(repository, clock);

        (command, temp_dir)
    }

    /// Helper to create an existing environment in the repository
    #[allow(clippy::unused_self)] // Builder pattern - self is consumed in build()
    fn create_existing_environment(
        &self,
        repository: &Arc<
            dyn crate::domain::environment::repository::EnvironmentRepository + Send + Sync,
        >,
        name: &str,
        base_dir: &Path,
    ) {
        use crate::adapters::ssh::SshCredentials;
        use crate::shared::Username;

        // Create temporary SSH key files
        let private_key = base_dir.join(format!("{name}_key"));
        let public_key = base_dir.join(format!("{name}_key.pub"));

        std::fs::write(&private_key, "test_private_key").expect("Failed to write private key");
        std::fs::write(&public_key, "test_public_key").expect("Failed to write public key");

        // Create environment
        let env_name = EnvironmentName::new(name).expect("Invalid environment name in test");
        let username = Username::new("torrust".to_string()).expect("Invalid username in test");
        let ssh_credentials = SshCredentials::new(private_key, public_key, username);
        let provider_config = ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
        });

        let environment = Environment::new(env_name, provider_config, ssh_credentials, 22);

        // Save to repository
        repository
            .save(&environment.into_any())
            .expect("Failed to save existing environment in test");
    }
}

impl Default for CreateCommandHandlerTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a valid test configuration
///
/// This function creates a complete `EnvironmentCreationConfig` with temporary
/// SSH key files for testing.
///
/// # Arguments
///
/// * `temp_dir` - Temporary directory where SSH keys will be created
/// * `env_name` - Name for the environment
///
/// # Examples
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::application::command_handlers::create::tests::create_valid_test_config;
/// use tempfile::TempDir;
///
/// let temp_dir = TempDir::new().unwrap();
/// let config = create_valid_test_config(&temp_dir, "test-environment");
/// ```
#[must_use]
pub fn create_valid_test_config(temp_dir: &TempDir, env_name: &str) -> EnvironmentCreationConfig {
    use std::fs;

    // Create temporary SSH key files
    let private_key = temp_dir.path().join("id_rsa");
    let public_key = temp_dir.path().join("id_rsa.pub");
    fs::write(&private_key, "test_private_key").expect("Failed to write private key");
    fs::write(&public_key, "test_public_key").expect("Failed to write public key");

    EnvironmentCreationConfig::new(
        EnvironmentSection {
            name: env_name.to_string(),
        },
        SshCredentialsConfig::new(
            private_key.to_string_lossy().to_string(),
            public_key.to_string_lossy().to_string(),
            "torrust".to_string(),
            22,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_build_command_with_defaults() {
        let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new().build();

        // Verify command is created (basic smoke test)
        assert_eq!(Arc::strong_count(&command.environment_repository), 1);
    }

    #[test]
    fn it_should_build_command_with_custom_time() {
        use chrono::TimeZone;

        let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
            .with_fixed_time(fixed_time)
            .build();

        // The clock should be set to the fixed time
        assert_eq!(command.clock.now(), fixed_time);
    }

    #[test]
    fn it_should_build_command_with_existing_environments() {
        let (command, _temp_dir) = CreateCommandHandlerTestBuilder::new()
            .with_existing_environment("production")
            .with_existing_environment("staging")
            .build();

        // Verify environments exist in repository
        let prod_name = EnvironmentName::new("production").unwrap();
        let staging_name = EnvironmentName::new("staging").unwrap();

        assert!(command.environment_repository.exists(&prod_name).unwrap());
        assert!(command
            .environment_repository
            .exists(&staging_name)
            .unwrap());
    }

    #[test]
    fn it_should_create_valid_test_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_valid_test_config(&temp_dir, "test-env");

        assert_eq!(config.environment.name, "test-env");
        assert_eq!(config.ssh_credentials.username, "torrust");
        assert_eq!(config.ssh_credentials.port, 22);

        // Verify SSH key files were created
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");
        assert!(private_key.exists());
        assert!(public_key.exists());
    }
}
