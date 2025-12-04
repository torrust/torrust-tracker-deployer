//! E2E Test Runner for black-box testing.
//!
//! This module provides a high-level abstraction for running E2E tests
//! against a specific environment. It encapsulates the process runner
//! and environment name, providing methods for each test task.
//!
//! # Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::E2eTestRunner;
//!
//! let test_runner = E2eTestRunner::new("e2e-full")
//!     .with_cleanup_on_failure(true);
//!
//! test_runner.create_environment(&config_path)?;
//! test_runner.provision_infrastructure()?;
//! test_runner.configure_services()?;
//! test_runner.validate_deployment()?;
//! test_runner.destroy_infrastructure()?;
//! ```

use std::path::Path;

use anyhow::Result;
use tracing::{error, info, warn};

use crate::testing::e2e::ProcessRunner;

/// A process runner bound to a specific E2E test environment.
///
/// This provides a higher-level abstraction for running E2E test tasks
/// against a named environment, eliminating the need to pass both the
/// runner and environment name to every task function.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::E2eTestRunner;
///
/// let test_runner = E2eTestRunner::new("e2e-provision")
///     .with_cleanup_on_failure(true);
///
/// test_runner.provision_infrastructure()?;
/// test_runner.destroy_infrastructure()?;
/// ```
pub struct E2eTestRunner {
    runner: ProcessRunner,
    environment_name: String,
    cleanup_on_failure: bool,
}

impl E2eTestRunner {
    /// Creates a new E2E test runner for the specified environment.
    ///
    /// By default, cleanup on failure is disabled. Use [`with_cleanup_on_failure`]
    /// to enable automatic infrastructure destruction when tasks fail.
    ///
    /// [`with_cleanup_on_failure`]: Self::with_cleanup_on_failure
    #[must_use]
    pub fn new(environment_name: impl Into<String>) -> Self {
        Self {
            runner: ProcessRunner::new(),
            environment_name: environment_name.into(),
            cleanup_on_failure: false,
        }
    }

    /// Enables or disables automatic cleanup on failure.
    ///
    /// When enabled, if a task like `provision_infrastructure` or `configure_services`
    /// fails, the runner will attempt to destroy the infrastructure before returning
    /// the error.
    #[must_use]
    pub fn with_cleanup_on_failure(mut self, cleanup: bool) -> Self {
        self.cleanup_on_failure = cleanup;
        self
    }

    /// Returns the environment name.
    #[must_use]
    pub fn environment_name(&self) -> &str {
        &self.environment_name
    }

    /// Returns whether cleanup on failure is enabled.
    #[must_use]
    pub fn cleanup_on_failure(&self) -> bool {
        self.cleanup_on_failure
    }

    /// Creates the environment from the configuration file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the environment configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the create command fails.
    ///
    /// # Panics
    ///
    /// Panics if the config path contains invalid UTF-8.
    pub fn create_environment(&self, config_path: &Path) -> Result<()> {
        info!(
            step = "create_environment",
            environment = %self.environment_name,
            config_path = %config_path.display(),
            "Creating environment from config file"
        );

        let create_result = self
            .runner
            .run_create_command(config_path.to_str().expect("Valid UTF-8 path"))
            .map_err(|e| anyhow::anyhow!("Failed to execute create command: {e}"))?;

        if !create_result.success() {
            error!(
                step = "create_environment",
                environment = %self.environment_name,
                exit_code = ?create_result.exit_code(),
                stderr = %create_result.stderr(),
                "Create environment command failed"
            );
            return Err(anyhow::anyhow!(
                "Create environment failed with exit code {:?}",
                create_result.exit_code()
            ));
        }

        info!(
            step = "create_environment",
            environment = %self.environment_name,
            status = "success",
            "Environment created successfully"
        );

        Ok(())
    }

    /// Provisions the infrastructure for the environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the provision command fails.
    /// If `cleanup_on_failure` is enabled, attempts to destroy infrastructure before returning.
    pub fn provision_infrastructure(&self) -> Result<()> {
        info!(
            step = "provision",
            environment = %self.environment_name,
            "Provisioning infrastructure"
        );

        let provision_result = self
            .runner
            .run_provision_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute provision command: {e}"))?;

        if !provision_result.success() {
            error!(
                step = "provision",
                environment = %self.environment_name,
                exit_code = ?provision_result.exit_code(),
                stderr = %provision_result.stderr(),
                "Provision command failed"
            );

            self.attempt_cleanup_on_failure();

            return Err(anyhow::anyhow!(
                "Provision failed with exit code {:?}",
                provision_result.exit_code()
            ));
        }

        info!(
            step = "provision",
            environment = %self.environment_name,
            status = "success",
            "Infrastructure provisioned successfully"
        );

        Ok(())
    }

    /// Registers an existing instance with the environment.
    ///
    /// This is an alternative to `provision_infrastructure` for use with
    /// existing VMs, physical servers, or containers.
    ///
    /// # Arguments
    ///
    /// * `instance_ip` - IP address of the existing instance to register
    ///
    /// # Errors
    ///
    /// Returns an error if the register command fails.
    pub fn register_instance(&self, instance_ip: &str) -> Result<()> {
        info!(
            step = "register",
            environment = %self.environment_name,
            instance_ip = %instance_ip,
            "Registering existing instance"
        );

        let register_result = self
            .runner
            .run_register_command(&self.environment_name, instance_ip)
            .map_err(|e| anyhow::anyhow!("Failed to execute register command: {e}"))?;

        if !register_result.success() {
            error!(
                step = "register",
                environment = %self.environment_name,
                exit_code = ?register_result.exit_code(),
                stderr = %register_result.stderr(),
                "Register command failed"
            );

            return Err(anyhow::anyhow!(
                "Register failed with exit code {:?}",
                register_result.exit_code()
            ));
        }

        info!(
            step = "register",
            environment = %self.environment_name,
            instance_ip = %instance_ip,
            status = "success",
            "Instance registered successfully"
        );

        Ok(())
    }

    /// Configures services on the provisioned infrastructure.
    ///
    /// # Errors
    ///
    /// Returns an error if the configure command fails.
    /// If `cleanup_on_failure` is enabled, attempts to destroy infrastructure before returning.
    pub fn configure_services(&self) -> Result<()> {
        info!(
            step = "configure",
            environment = %self.environment_name,
            "Configuring services"
        );

        let configure_result = self
            .runner
            .run_configure_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute configure command: {e}"))?;

        if !configure_result.success() {
            error!(
                step = "configure",
                environment = %self.environment_name,
                exit_code = ?configure_result.exit_code(),
                stderr = %configure_result.stderr(),
                "Configure command failed"
            );

            self.attempt_cleanup_on_failure();

            return Err(anyhow::anyhow!(
                "Configure failed with exit code {:?}",
                configure_result.exit_code()
            ));
        }

        info!(
            step = "configure",
            environment = %self.environment_name,
            status = "success",
            "Services configured successfully"
        );

        Ok(())
    }

    /// Releases software to the provisioned and configured infrastructure.
    ///
    /// # Errors
    ///
    /// Returns an error if the release command fails.
    /// If `cleanup_on_failure` is enabled, attempts to destroy infrastructure before returning.
    pub fn release_software(&self) -> Result<()> {
        info!(
            step = "release",
            environment = %self.environment_name,
            "Releasing software"
        );

        let release_result = self
            .runner
            .run_release_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute release command: {e}"))?;

        if !release_result.success() {
            error!(
                step = "release",
                environment = %self.environment_name,
                exit_code = ?release_result.exit_code(),
                stderr = %release_result.stderr(),
                "Release command failed"
            );

            self.attempt_cleanup_on_failure();

            return Err(anyhow::anyhow!(
                "Release failed with exit code {:?}",
                release_result.exit_code()
            ));
        }

        info!(
            step = "release",
            environment = %self.environment_name,
            status = "success",
            "Software released successfully"
        );

        Ok(())
    }

    /// Runs services on the released infrastructure.
    ///
    /// # Skip Condition
    ///
    /// When `TORRUST_TD_SKIP_RUN_IN_CONTAINER` environment variable is set to `"true"`,
    /// this step is skipped because Docker daemon is not available in the test container
    /// (no Docker-in-Docker configuration).
    ///
    /// # Errors
    ///
    /// Returns an error if the run command fails.
    /// If `cleanup_on_failure` is enabled, attempts to destroy infrastructure before returning.
    pub fn run_services(&self) -> Result<()> {
        // Check if run should be skipped (Docker not available in test container)
        let skip_run = std::env::var("TORRUST_TD_SKIP_RUN_IN_CONTAINER")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        if skip_run {
            info!(
                step = "run",
                environment = %self.environment_name,
                "Skipping run command due to TORRUST_TD_SKIP_RUN_IN_CONTAINER (Docker not available in test container)"
            );
            return Ok(());
        }

        info!(
            step = "run",
            environment = %self.environment_name,
            "Running services"
        );

        let run_result = self
            .runner
            .run_run_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute run command: {e}"))?;

        if !run_result.success() {
            error!(
                step = "run",
                environment = %self.environment_name,
                exit_code = ?run_result.exit_code(),
                stderr = %run_result.stderr(),
                "Run command failed"
            );

            self.attempt_cleanup_on_failure();

            return Err(anyhow::anyhow!(
                "Run failed with exit code {:?}",
                run_result.exit_code()
            ));
        }

        info!(
            step = "run",
            environment = %self.environment_name,
            status = "success",
            "Services started successfully"
        );

        Ok(())
    }

    /// Validates the deployment by running the test command.
    ///
    /// # Errors
    ///
    /// Returns an error if the test command fails.
    pub fn validate_deployment(&self) -> Result<()> {
        info!(
            step = "test",
            environment = %self.environment_name,
            "Validating deployment"
        );

        let test_result = self
            .runner
            .run_test_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute test command: {e}"))?;

        if !test_result.success() {
            error!(
                step = "test",
                environment = %self.environment_name,
                exit_code = ?test_result.exit_code(),
                stderr = %test_result.stderr(),
                "Test command failed"
            );
            return Err(anyhow::anyhow!(
                "Test failed with exit code {:?}",
                test_result.exit_code()
            ));
        }

        info!(
            step = "test",
            environment = %self.environment_name,
            status = "success",
            "Deployment validated successfully"
        );

        Ok(())
    }

    /// Destroys the infrastructure for the environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the destroy command fails.
    pub fn destroy_infrastructure(&self) -> Result<()> {
        info!(
            step = "destroy",
            environment = %self.environment_name,
            "Destroying infrastructure"
        );

        let destroy_result = self
            .runner
            .run_destroy_command(&self.environment_name)
            .map_err(|e| anyhow::anyhow!("Failed to execute destroy command: {e}"))?;

        if !destroy_result.success() {
            error!(
                step = "destroy",
                environment = %self.environment_name,
                exit_code = ?destroy_result.exit_code(),
                stderr = %destroy_result.stderr(),
                "Destroy command failed"
            );
            return Err(anyhow::anyhow!(
                "Destroy failed with exit code {:?}",
                destroy_result.exit_code()
            ));
        }

        info!(
            step = "destroy",
            environment = %self.environment_name,
            status = "success",
            "Infrastructure destroyed successfully"
        );

        Ok(())
    }

    /// Attempts to clean up infrastructure if `cleanup_on_failure` is enabled.
    ///
    /// This is called internally when tasks fail. It logs a warning and
    /// attempts to destroy the infrastructure, ignoring any errors since
    /// we're already in an error state.
    fn attempt_cleanup_on_failure(&self) {
        if self.cleanup_on_failure {
            warn!(
                step = "cleanup_after_failure",
                environment = %self.environment_name,
                "Attempting to destroy infrastructure after failure"
            );
            // Ignore destroy result - we're already in an error state
            drop(self.runner.run_destroy_command(&self.environment_name));
        }
    }
}
