//! Test environment management for E2E testing
//!
//! This module provides the `TestEnvironment` which manages the complete setup
//! and configuration of test environments for end-to-end deployment testing.
//!
//! ## Key Features
//!
//! - Temporary directory and SSH key management
//! - Service container initialization with test configuration
//! - Template preparation and cleanup for test isolation
//! - Comprehensive error handling for environment setup failures
//!
//! ## Environment Lifecycle
//!
//! 1. **Setup** - Create temporary directories, SSH keys, clean templates
//! 2. **Configuration** - Initialize services with test-specific settings
//! 3. **Usage** - Provide services for test execution
//! 4. **Cleanup** - Automatic cleanup via RAII (`TempDir`)
//!
//! The environment ensures each test runs in isolation with its own
//! temporary resources and configuration.

use anyhow::Context;
use tempfile::TempDir;
use tracing::{info, warn};

use crate::config::{Config, InstanceName, SshCredentials};
use crate::container::Services;
use crate::domain::Username;

/// Errors that can occur during test environment creation and initialization
#[derive(Debug, thiserror::Error)]
pub enum TestEnvironmentError {
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
    #[error("Failed to create temporary directory for test environment SSH keys: {source}")]
    TempDirectoryCreationError { source: std::io::Error },

    /// Failed to setup SSH keys
    #[error("Failed to setup SSH keys for test environment: {source}")]
    SshKeySetupError { source: anyhow::Error },

    /// Failed to prepare environment (templates, etc.)
    #[error("Failed to clean and prepare templates directory: {source}")]
    EnvironmentPreparationError { source: anyhow::Error },
}

/// SSH private key filename for testing
const SSH_PRIVATE_KEY_FILENAME: &str = "testing_rsa";

/// SSH public key filename for testing
const SSH_PUBLIC_KEY_FILENAME: &str = "testing_rsa.pub";

/// Type of test environment indicating what infrastructure is used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEnvironmentType {
    /// Container-based testing using Docker containers via testcontainers crate.
    /// No manual cleanup needed as containers are automatically destroyed.
    Container,
    /// Virtual machine-based testing using LXD VMs provisioned via `OpenTofu`.
    /// Requires `OpenTofu` resource cleanup when the environment is dropped.
    VirtualMachine,
}

/// Main test environment combining configuration and services
pub struct TestEnvironment {
    pub config: Config,
    pub services: Services,
    /// Temporary directory for SSH keys. Must be kept alive for the lifetime
    /// of the test environment to prevent cleanup of SSH key files.
    /// This field is not directly read but must be retained for RAII cleanup.
    temp_dir: Option<tempfile::TempDir>,
    /// The type of test environment, determining what cleanup is needed.
    environment_type: TestEnvironmentType,
}

impl TestEnvironment {
    /// Creates and initializes a new test environment with custom SSH user
    ///
    /// This method performs the complete setup including validation, SSH key setup,
    /// template preparation, and environment initialization in a single call.
    ///
    /// # Arguments
    ///
    /// * `keep_env` - Whether to keep the environment after tests complete
    /// * `templates_dir` - Path to the templates directory
    /// * `ssh_user` - SSH username to use for connections
    /// * `instance_name` - Name for the instance to be deployed
    /// * `environment_type` - The type of test environment (Container or `VirtualMachine`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input validation fails (empty or invalid templates directory)
    /// - Current directory cannot be determined
    /// - Temporary directory creation fails
    /// - SSH key setup fails
    /// - Template preparation fails
    pub fn initialized(
        keep_env: bool,
        templates_dir: impl Into<std::path::PathBuf>,
        ssh_user: &Username,
        instance_name: InstanceName,
        environment_type: TestEnvironmentType,
    ) -> Result<Self, TestEnvironmentError> {
        let templates_dir = templates_dir.into();

        Self::validate_inputs(&templates_dir)?;

        let project_root = Self::get_project_root()?;
        let temp_dir = Self::create_temp_directory()?;

        let ssh_credentials = Self::setup_ssh_credentials(&project_root, &temp_dir, ssh_user)?;

        let config = Self::create_config(
            keep_env,
            ssh_credentials,
            instance_name,
            &templates_dir,
            &project_root,
        );

        let services = Services::new(&config);

        let env = Self {
            config,
            services,
            temp_dir: Some(temp_dir),
            environment_type,
        };

        env.init()?;

        Ok(env)
    }

    /// Initializes the test environment by preparing templates and logging setup
    ///
    /// This method performs the final environment setup with side effects.
    /// It is called internally by `initialized()` as part of the initialization process.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template preparation fails
    fn init(&self) -> Result<(), TestEnvironmentError> {
        Self::prepare_environment(&self.services)?;
        self.log_environment_info();
        Ok(())
    }

    /// Validates input parameters
    fn validate_inputs(templates_dir: &std::path::Path) -> Result<(), TestEnvironmentError> {
        if templates_dir.as_os_str().is_empty() {
            return Err(TestEnvironmentError::EmptyTemplatesDirectory);
        }

        // Check if the path string representation is only whitespace
        if let Some(dir_str) = templates_dir.to_str() {
            if dir_str.trim().is_empty() {
                return Err(TestEnvironmentError::WhitespaceOnlyTemplatesDirectory);
            }
        }

        Ok(())
    }

    /// Gets the current project root directory
    fn get_project_root() -> Result<std::path::PathBuf, TestEnvironmentError> {
        std::env::current_dir().map_err(TestEnvironmentError::CurrentDirectoryError)
    }

    /// Creates a temporary directory for SSH keys
    fn create_temp_directory() -> Result<TempDir, TestEnvironmentError> {
        TempDir::new().map_err(|e| TestEnvironmentError::TempDirectoryCreationError { source: e })
    }

    /// Sets up SSH credentials with temporary keys
    fn setup_ssh_credentials(
        project_root: &std::path::Path,
        temp_dir: &TempDir,
        ssh_user: &Username,
    ) -> Result<SshCredentials, TestEnvironmentError> {
        let temp_ssh_key = temp_dir.path().join(SSH_PRIVATE_KEY_FILENAME);
        let temp_ssh_pub_key = temp_dir.path().join(SSH_PUBLIC_KEY_FILENAME);

        // Copy SSH private key from fixtures to temp directory
        let fixtures_ssh_key = project_root.join("fixtures/testing_rsa");

        std::fs::copy(&fixtures_ssh_key, &temp_ssh_key)
            .context("Failed to copy SSH private key to temporary directory")
            .map_err(|e| TestEnvironmentError::SshKeySetupError { source: e })?;

        // Copy SSH public key from fixtures to temp directory
        let fixtures_ssh_pub_key = project_root.join("fixtures/testing_rsa.pub");

        std::fs::copy(&fixtures_ssh_pub_key, &temp_ssh_pub_key)
            .context("Failed to copy SSH public key to temporary directory")
            .map_err(|e| TestEnvironmentError::SshKeySetupError { source: e })?;

        // Set proper permissions on the SSH key (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_ssh_key)
                .context("Failed to get SSH key metadata")
                .map_err(|e| TestEnvironmentError::SshKeySetupError { source: e })?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&temp_ssh_key, perms)
                .context("Failed to set SSH key permissions")
                .map_err(|e| TestEnvironmentError::SshKeySetupError { source: e })?;
        }

        info!(
            operation = "ssh_key_setup",
            private_location = %temp_ssh_key.display(),
            public_location = %temp_ssh_pub_key.display(),
            "SSH keys copied to temporary location"
        );

        Ok(SshCredentials::new(
            temp_ssh_key,
            temp_ssh_pub_key,
            ssh_user.to_string(),
        ))
    }

    /// Creates the main configuration object
    fn create_config(
        keep_env: bool,
        ssh_credentials: SshCredentials,
        instance_name: InstanceName,
        templates_dir: &std::path::Path,
        project_root: &std::path::Path,
    ) -> Config {
        Config::new(
            keep_env,
            ssh_credentials,
            instance_name,
            templates_dir.to_string_lossy().to_string(),
            project_root.to_path_buf(),
            project_root.join("build"),
        )
    }

    /// Prepares the test environment (templates, etc.)
    fn prepare_environment(services: &Services) -> Result<(), TestEnvironmentError> {
        info!(
            operation = "clean_templates",
            "Cleaning templates directory to ensure fresh embedded templates"
        );

        services
            .template_manager
            .reset_templates_dir()
            .map_err(|e| TestEnvironmentError::EnvironmentPreparationError {
                source: anyhow::anyhow!(e),
            })?;
        Ok(())
    }

    /// Logs environment information
    fn log_environment_info(&self) {
        // Warn if keep_env is enabled with Container environment type
        if self.config.keep_env && self.environment_type == TestEnvironmentType::Container {
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
                "Test environment initialized with temporary directory"
            );
        }
    }

    /// Gets the temporary directory path for logging or debugging purposes
    #[must_use]
    pub fn temp_dir_path(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(tempfile::TempDir::path)
    }
}

impl std::fmt::Debug for TestEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestEnvironment")
            .field("keep_env", &self.config.keep_env)
            .field("templates_dir", &self.config.templates_dir)
            .field("project_root", &self.config.project_root)
            .field("build_dir", &self.config.build_dir)
            .field("has_temp_dir", &self.temp_dir.is_some())
            .finish_non_exhaustive() // Services field is complex and not needed for debugging
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.config.keep_env {
            // Only cleanup OpenTofu resources for VirtualMachine environments
            // Container environments use Docker/testcontainers which handle their own cleanup
            match self.environment_type {
                TestEnvironmentType::VirtualMachine => {
                    // Try basic cleanup in case async cleanup failed
                    // Using emergency_destroy for consistent OpenTofu handling
                    let tofu_dir = self.config.build_dir.join(&self.config.opentofu_subfolder);

                    if let Err(e) =
                        crate::infrastructure::adapters::opentofu::emergency_destroy(&tofu_dir)
                    {
                        eprintln!("Warning: Failed to cleanup OpenTofu resources during TestEnvironment drop: {e}");
                    }
                }
                TestEnvironmentType::Container => {
                    // Container environments are managed by testcontainers
                    // No OpenTofu cleanup needed
                }
            }
        }
    }
}
