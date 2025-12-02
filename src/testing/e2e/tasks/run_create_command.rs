//! Environment creation task for E2E testing
//!
//! This module provides the E2E testing task for creating deployment environments
//! using the `CreateCommandHandler`. It orchestrates environment creation with proper
//! validation and repository persistence.
//!
//! ## Key Operations
//!
//! - Creates environments using the `CreateCommandHandler`
//! - Handles environment creation workflow for E2E tests
//! - Provides structured error handling and reporting
//!
//! ## Integration
//!
//! This is a generic task that works with the command handler pattern:
//! - Uses the `RepositoryFactory` to create repositories
//! - Creates `EnvironmentCreationConfig` from test parameters
//! - Integrates with the existing `CreateCommandHandler` workflow

use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::application::command_handlers::create::config::{
    EnvironmentCreationConfig, EnvironmentSection, LxdProviderSection, ProviderSection,
    SshCredentialsConfig,
};
use crate::application::command_handlers::create::{
    CreateCommandHandler, CreateCommandHandlerError,
};
use crate::domain::environment::Created;
use crate::domain::Environment;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::Clock;

/// Create a new environment using the `CreateCommandHandler`
///
/// This function executes environment creation using the `CreateCommandHandler` for E2E tests.
/// It builds the configuration from parameters and creates the environment,
/// ensuring proper validation and persistence.
///
/// # Arguments
///
/// * `repository_factory` - Repository factory for creating environment repositories
/// * `clock` - Clock service for timestamp generation
/// * `working_dir` - Working directory for environment storage
/// * `environment_name` - Name of the environment to create
/// * `ssh_private_key_path` - Path to SSH private key file
/// * `ssh_public_key_path` - Path to SSH public key file
/// * `ssh_username` - SSH username for the environment
/// * `ssh_port` - SSH port number
///
/// # Returns
///
/// Returns the created `Environment<Created>` on success.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration is invalid
/// - Environment already exists
/// - Repository operations fail
#[allow(clippy::too_many_arguments)]
pub fn run_create_command(
    repository_factory: &RepositoryFactory,
    clock: Arc<dyn Clock>,
    working_dir: &std::path::Path,
    environment_name: &str,
    ssh_private_key_path: String,
    ssh_public_key_path: String,
    ssh_username: &str,
    ssh_port: u16,
) -> Result<Environment<Created>, CreateTaskError> {
    info!(
        environment_name = environment_name,
        "Creating environment via CreateCommandHandler"
    );

    // Create repository using RepositoryFactory with data directory
    let data_dir = working_dir.join("data");
    let repository = repository_factory.create(data_dir);

    // Create the command handler
    let create_command = CreateCommandHandler::new(repository, clock);

    // Build the configuration with LXD provider
    let config = EnvironmentCreationConfig::new(
        EnvironmentSection {
            name: environment_name.to_string(),
            instance_name: None, // Auto-generate from environment name
        },
        SshCredentialsConfig::new(
            ssh_private_key_path,
            ssh_public_key_path,
            ssh_username.to_string(),
            ssh_port,
        ),
        ProviderSection::Lxd(LxdProviderSection {
            profile_name: format!("lxd-{environment_name}"),
        }),
    );

    // Execute the command
    let environment = create_command
        .execute(config, working_dir)
        .map_err(|source| CreateTaskError::CreationFailed { source })?;

    info!(
        environment_name = environment.name().as_str(),
        instance_name = environment.instance_name().as_str(),
        "Environment created successfully via CreateCommandHandler"
    );

    Ok(environment)
}

/// Errors that can occur during the create task
#[derive(Debug, Error)]
pub enum CreateTaskError {
    /// Environment creation command execution failed
    #[error(
        "Failed to create environment: {source}
Tip: Check that the environment name is valid and doesn't already exist"
    )]
    CreationFailed {
        #[source]
        source: CreateCommandHandlerError,
    },
}

impl CreateTaskError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_create_command::CreateTaskError;
    /// # use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandlerError;
    /// let error = CreateTaskError::CreationFailed {
    ///     source: CreateCommandHandlerError::EnvironmentAlreadyExists {
    ///         name: "test-env".to_string(),
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::CreationFailed { .. } => {
                "Environment Creation Failed - Detailed Troubleshooting:

1. Check environment name:
   - Must be lowercase letters and numbers only
   - Use dashes as word separators
   - Cannot start or end with separators
   - Cannot start with numbers

2. Verify environment doesn't already exist:
   - Check the data directory for existing environment
   - Use 'destroy' command to remove existing environment if needed

3. Check SSH key files:
   - Verify SSH private key file exists and is readable
   - Verify SSH public key file exists and is readable
   - Ensure key file permissions are correct (600 for private key)

4. Check repository access:
   - Ensure data directory is writable
   - Verify no file locks are preventing write access

For more information, see the E2E testing documentation."
            }
        }
    }
}
