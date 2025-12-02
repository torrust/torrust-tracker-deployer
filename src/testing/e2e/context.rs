//! Test context management for E2E testing
//!
//! This module provides the `TestContext` which manages the complete setup
//! and configuration of test contexts for end-to-end deployment testing.
//!
//! ## Key Features
//!
//! - Temporary directory and SSH key management
//! - Service container initialization with test configuration
//! - Template preparation and cleanup for test isolation
//! - Comprehensive error handling for environment setup failures
//!
//! ## Context Lifecycle
//!
//! 1. **Setup** - Create temporary directories, SSH keys, clean templates
//! 2. **Configuration** - Initialize services with test-specific settings
//! 3. **Usage** - Provide services for test execution
//! 4. **Cleanup** - Automatic cleanup via RAII (`TempDir`)
//!
//! The context ensures each test runs in isolation with its own
//! temporary resources and configuration.

use tempfile::TempDir;
use tracing::{info, warn};

use super::container::Services;
use crate::config::Config;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::domain::Environment;
use crate::infrastructure::external_tools::tofu::OPENTOFU_SUBFOLDER;

/// Errors that can occur during test context creation and initialization
#[derive(Debug, thiserror::Error)]
pub enum TestContextError {
    /// Invalid template directory path
    #[error("Templates directory cannot be empty")]
    EmptyTemplatesDirectory,

    /// Templates directory contains only whitespace
    #[error("Templates directory cannot be empty or whitespace-only")]
    WhitespaceOnlyTemplatesDirectory,

    /// Failed to determine current directory
    #[error("Failed to determine current directory (project root): {0}")]
    CurrentDirectoryError(#[from] std::io::Error),

    /// Failed to create temporary directory
    #[error("Failed to create temporary directory for test context SSH keys: {source}")]
    TempDirectoryCreationError { source: std::io::Error },

    /// Failed to setup SSH keys
    #[error("Failed to setup SSH keys for test context: {source}")]
    SshKeySetupError { source: anyhow::Error },

    /// Failed to prepare environment (templates, etc.)
    #[error("Failed to clean and prepare templates directory: {source}")]
    ContextPreparationError { source: anyhow::Error },
}

/// Type of test context indicating what infrastructure is used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestContextType {
    /// Container-based testing using Docker containers via testcontainers crate.
    /// No manual cleanup needed as containers are automatically destroyed.
    Container,
    /// Virtual machine-based testing using LXD VMs provisioned via `OpenTofu`.
    /// Requires `OpenTofu` resource cleanup when the environment is dropped.
    VirtualMachine,
}

/// Main test context combining configuration and services
pub struct TestContext {
    pub config: Config,
    pub services: Services,
    /// The complete environment configuration containing instance name, SSH keys, and paths.
    /// Stored as `AnyEnvironmentState` to track the actual current state throughout the
    /// deployment lifecycle (Created → Provisioning → Provisioned → Configuring → Configured, etc.)
    pub environment: AnyEnvironmentState,
    /// Whether to keep the deployment environment after completion.
    ///
    /// When `false`, the environment will be automatically cleaned up (destroyed)
    /// after the test process completes. When `true`, the environment
    /// will be left running for manual inspection or debugging.
    pub keep_env: bool,
    /// Temporary directory for SSH keys. Must be kept alive for the lifetime
    /// of the test context to prevent cleanup of SSH key files.
    /// This field is not directly read but must be retained for RAII cleanup.
    temp_dir: Option<tempfile::TempDir>,
    /// The type of test context, determining what cleanup is needed.
    context_type: TestContextType,
}

impl TestContext {
    /// Creates a new test environment with custom SSH user (private constructor)
    ///
    /// This method performs the setup including validation, SSH key setup,
    /// and configuration creation, but does NOT initialize the environment.
    /// Callers must explicitly call `.init()` to complete the setup.
    ///
    /// # Arguments
    ///
    /// * `keep_env` - Whether to keep the environment after tests complete
    /// * `environment` - The Environment entity containing all necessary configuration
    /// * `context_type` - The type of test environment (Container or `VirtualMachine`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input validation fails (empty or invalid templates directory)
    /// - Current directory cannot be determined
    /// - Temporary directory creation fails
    /// - SSH key setup fails
    fn new(
        keep_env: bool,
        environment: Environment,
        context_type: TestContextType,
    ) -> Result<Self, TestContextError> {
        let templates_dir = environment.templates_dir();

        Self::validate_inputs(&templates_dir)?;

        let project_root = Self::get_project_root()?;
        let temp_dir = Self::create_temp_directory()?;

        let config = Config::new(
            environment.templates_dir().clone(),
            project_root,
            environment.build_dir().clone(),
        );

        let services = Services::new(
            &config,
            environment.ssh_credentials().clone(),
            environment.instance_name().clone(),
            environment.profile_name().clone(),
        );

        let env = Self {
            config,
            services,
            environment: environment.into_any(), // Convert to AnyEnvironmentState for runtime state tracking
            keep_env,
            temp_dir: Some(temp_dir),
            context_type,
        };

        Ok(env)
    }

    /// Creates a new test environment from an Environment entity
    ///
    /// This method provides a simplified interface that accepts an Environment entity
    /// containing all the necessary configuration, rather than individual parameters.
    ///
    /// **Important**: This method does NOT initialize the environment. You must call
    /// `.init()` on the returned `TestContext` to complete the setup.
    ///
    /// # Arguments
    ///
    /// * `keep_env` - Whether to keep the environment after tests complete
    /// * `environment` - The Environment entity containing instance name, SSH keys, and paths
    /// * `context_type` - The type of test environment (Container or `VirtualMachine`)
    ///
    /// # Returns
    ///
    /// A `TestContext` that requires `.init()` to be called before use.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input validation fails (empty or invalid templates directory)
    /// - Current directory cannot be determined
    /// - Temporary directory creation fails
    /// - SSH key setup fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deployer_lib::domain::provider::{LxdConfig, ProviderConfig};
    /// use torrust_tracker_deployer_lib::domain::ProfileName;
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
    /// use torrust_tracker_deployer_lib::testing::e2e::context::{TestContext, TestContextType};
    /// use std::path::PathBuf;
    /// use tempfile::TempDir;
    ///
    /// // Use temporary directory to avoid creating real directories
    /// let temp_dir = TempDir::new()?;
    /// let temp_path = temp_dir.path();
    ///
    /// let env_name = EnvironmentName::new("test-example".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     temp_path.join("testing_rsa"),
    ///     temp_path.join("testing_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let provider_config = ProviderConfig::Lxd(LxdConfig {
    ///     profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
    /// });
    /// let environment = Environment::new(env_name, provider_config, ssh_credentials, 22);
    ///
    /// let test_context = TestContext::from_environment(
    ///     false,
    ///     environment,
    ///     TestContextType::Container,
    /// )?.init()?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_environment(
        keep_env: bool,
        environment: Environment,
        context_type: TestContextType,
    ) -> Result<Self, TestContextError> {
        Self::new(keep_env, environment, context_type)
    }

    /// Initializes the test environment by preparing templates and logging setup
    ///
    /// This method performs the final environment setup with side effects.
    /// It must be called explicitly after creating a `TestContext` to complete the setup.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template preparation fails
    /// - Environment persistence fails
    pub fn init(self) -> Result<Self, TestContextError> {
        Self::prepare_environment(&self.services)?;
        self.persist_initial_environment_state()?;
        self.log_environment_info();
        Ok(self)
    }

    /// Validates input parameters
    fn validate_inputs(templates_dir: &std::path::Path) -> Result<(), TestContextError> {
        if templates_dir.as_os_str().is_empty() {
            return Err(TestContextError::EmptyTemplatesDirectory);
        }

        // Check if the path string representation is only whitespace
        if let Some(dir_str) = templates_dir.to_str() {
            if dir_str.trim().is_empty() {
                return Err(TestContextError::WhitespaceOnlyTemplatesDirectory);
            }
        }

        Ok(())
    }

    /// Gets the current project root directory
    fn get_project_root() -> Result<std::path::PathBuf, TestContextError> {
        std::env::current_dir().map_err(TestContextError::CurrentDirectoryError)
    }

    /// Creates a temporary directory for SSH keys
    fn create_temp_directory() -> Result<TempDir, TestContextError> {
        TempDir::new().map_err(|e| TestContextError::TempDirectoryCreationError { source: e })
    }

    /// Prepares the test environment (templates, etc.)
    fn prepare_environment(services: &Services) -> Result<(), TestContextError> {
        info!(
            operation = "clean_templates",
            "Cleaning templates directory to ensure fresh embedded templates"
        );

        services
            .template_manager
            .reset_templates_dir()
            .map_err(|e| TestContextError::ContextPreparationError {
                source: anyhow::anyhow!(e),
            })?;
        Ok(())
    }

    /// Persists the environment immediately after creation
    ///
    /// This method ensures the environment is saved in its initial Created state
    /// before any commands are executed that might change the state.
    ///
    /// # Errors
    ///
    /// Returns an error if environment persistence fails
    fn persist_initial_environment_state(&self) -> Result<(), TestContextError> {
        let repository = self.create_repository();
        info!(
            environment = %self.environment.name(),
            state = %self.environment.state_name(),
            "Persisting initial environment state"
        );
        repository
            .save(&self.environment)
            .map_err(|e| TestContextError::ContextPreparationError {
                source: anyhow::anyhow!("Failed to persist initial environment state: {e}"),
            })
    }

    /// Logs environment information
    fn log_environment_info(&self) {
        // Warn if keep_env is enabled with Container environment type
        if self.keep_env && self.context_type == TestContextType::Container {
            warn!(
                environment_type = "container",
                keep_env = true,
                "keep_env flag is enabled but Container environments are automatically destroyed by testcontainers - the flag will be ignored"
            );
            // TODO: Investigate if testcontainers crate supports keeping containers alive after test completion
            // This would require exploring testcontainers configuration options or lifecycle management
        }

        if let Some(temp_path) = self.temp_dir_path() {
            info!(
                environment = "temporary_directory",
                path = %temp_path.display(),
                "Temporary directory created"
            );
        }

        info!(
            environment = "templates_directory",
            path = %self.services.template_manager.templates_dir().display(),
            "Templates directory configured"
        );

        // Log the temp directory path to demonstrate the field is used
        if let Some(temp_path) = self.temp_dir_path() {
            info!(
                temp_dir_path = %temp_path.display(),
                "Test context initialized with temporary directory"
            );
        }
    }

    /// Gets the temporary directory path for logging or debugging purposes
    #[must_use]
    pub fn temp_dir_path(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(tempfile::TempDir::path)
    }

    /// Updates the test context environment from a provisioned environment
    ///
    /// This method updates the internal environment state after provisioning
    /// completes, ensuring the `TestContext` maintains the latest and accurate environment state.
    ///
    /// # Arguments
    ///
    /// * `provisioned_env` - The provisioned environment returned by `ProvisionCommandHandler`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::testing::e2e::context::TestContext;
    /// # use torrust_tracker_deployer_lib::domain::Environment;
    /// # fn example(test_context: &mut TestContext, provisioned_env: Environment<torrust_tracker_deployer_lib::domain::environment::Provisioned>) {
    /// // After provisioning succeeds, update the test context
    /// test_context.update_from_provisioned(provisioned_env);
    /// # }
    /// ```
    pub fn update_from_provisioned(
        &mut self,
        provisioned_env: crate::domain::Environment<crate::domain::environment::Provisioned>,
    ) {
        // Replace the environment with the provisioned state using type erasure
        // This properly represents the actual state (Provisioned) rather than keeping it in Created state
        self.environment = provisioned_env.into_any();
    }

    /// Updates the test context environment from a configured environment
    ///
    /// This method updates the internal environment state after configuration
    /// completes, ensuring the `TestContext` maintains the latest and accurate environment state.
    ///
    /// # Arguments
    ///
    /// * `configured_env` - The configured environment returned by `ConfigureCommandHandler`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::testing::e2e::context::TestContext;
    /// # use torrust_tracker_deployer_lib::domain::Environment;
    /// # fn example(test_context: &mut TestContext, configured_env: Environment<torrust_tracker_deployer_lib::domain::environment::Configured>) {
    /// // After configuration succeeds, update the test context
    /// test_context.update_from_configured(configured_env);
    /// # }
    /// ```
    pub fn update_from_configured(
        &mut self,
        configured_env: crate::domain::Environment<crate::domain::environment::Configured>,
    ) {
        // Replace the environment with the configured state using type erasure
        // This properly represents the actual state (Configured) rather than keeping it in Created state
        self.environment = configured_env.into_any();
    }

    /// Updates the test context environment from a destroyed environment
    ///
    /// This method updates the internal environment state after destruction
    /// completes, ensuring the `TestContext` maintains the latest and accurate environment state.
    ///
    /// # Arguments
    ///
    /// * `destroyed_env` - The destroyed environment returned by `DestroyCommandHandler`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::testing::e2e::context::TestContext;
    /// # use torrust_tracker_deployer_lib::domain::Environment;
    /// # fn example(test_context: &mut TestContext, destroyed_env: Environment<torrust_tracker_deployer_lib::domain::environment::Destroyed>) {
    /// // After destruction succeeds, update the test context
    /// test_context.update_from_destroyed(destroyed_env);
    /// # }
    /// ```
    pub fn update_from_destroyed(
        &mut self,
        destroyed_env: crate::domain::Environment<crate::domain::environment::Destroyed>,
    ) {
        // Replace the environment with the destroyed state using type erasure
        // This properly represents the actual state (Destroyed) rather than keeping it in previous state
        self.environment = destroyed_env.into_any();
    }

    /// Creates a repository for the current environment
    ///
    /// This is a convenience method that creates an `EnvironmentRepository`
    /// configured for this test context's environment. The repository is
    /// created using the repository factory with the environment's data directory.
    ///
    /// # Returns
    ///
    /// An `Arc<dyn EnvironmentRepository>` that can be used to persist and load
    /// environment state for this test context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::testing::e2e::context::TestContext;
    /// # fn example(test_context: &TestContext) {
    /// let repository = test_context.create_repository();
    /// // Use repository for state persistence...
    /// # }
    /// ```
    #[must_use]
    pub fn create_repository(
        &self,
    ) -> std::sync::Arc<dyn crate::domain::environment::repository::EnvironmentRepository> {
        // Pass the parent "data" directory, not the environment-specific directory
        // The repository will add the environment name subdirectory automatically
        // e.g., "{project_root}/data" + "e2e-provision" = "{project_root}/data/e2e-provision/environment.json"
        let base_data_dir = self.config.project_root.join("data");
        self.services.repository_factory.create(base_data_dir)
    }
}

impl std::fmt::Debug for TestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestContext")
            .field("keep_env", &self.keep_env)
            .field("templates_dir", &self.config.templates_dir)
            .field("project_root", &self.config.project_root)
            .field("build_dir", &self.config.build_dir)
            .field("has_temp_dir", &self.temp_dir.is_some())
            .finish_non_exhaustive() // Services field is complex and not needed for debugging
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if !self.keep_env {
            // Only cleanup OpenTofu resources for VirtualMachine environments
            // Container environments use Docker/testcontainers which handle their own cleanup
            match self.context_type {
                TestContextType::VirtualMachine => {
                    // Skip cleanup if infrastructure already destroyed
                    // This prevents unnecessary cleanup attempts when DestroyCommand already cleaned up
                    if matches!(self.environment, AnyEnvironmentState::Destroyed(_)) {
                        return;
                    }

                    // Try basic cleanup in case async cleanup failed
                    // Using emergency_destroy for consistent OpenTofu handling
                    let tofu_dir = self.config.build_dir.join(OPENTOFU_SUBFOLDER);

                    if let Err(e) = crate::adapters::tofu::emergency_destroy(&tofu_dir) {
                        eprintln!("Warning: Failed to cleanup OpenTofu resources during TestContext drop: {e}");
                    }
                }
                TestContextType::Container => {
                    // Container environments are managed by testcontainers
                    // No OpenTofu cleanup needed
                }
            }
        }
    }
}
