//! Remote actions module (Level 3 of Three-Level Architecture)
//!
//! This module provides the lowest-level operations in the three-level architecture,
//! containing leaf-level actions that directly interact with remote systems via SSH.
//! These actions are the building blocks used by steps (Level 2) and commands (Level 1).
//!
//! ## Available Remote Actions
//!
//! - `validators::cloud_init` - Cloud-init status checking and validation
//! - `validators::docker` - Docker installation and service management
//! - `validators::docker_compose` - Docker Compose installation and validation
//!
//! ## Architecture Pattern
//!
//! Remote actions follow a consistent pattern:
//! - Take SSH connection and required parameters
//! - Execute specific remote operations via SSH
//! - Provide structured error handling with action context
//! - Return typed results for use by higher-level components
//!
//! These actions are designed to be atomic, testable, and reusable across
//! different deployment scenarios.

use std::net::IpAddr;
use thiserror::Error;

use crate::shared::command::CommandError;

pub mod validators;

pub use validators::cloud_init::CloudInitValidator;
pub use validators::docker::DockerValidator;
pub use validators::docker_compose::DockerComposeValidator;
pub use validators::running_services::RunningServicesValidator;

/// Errors that can occur during remote action execution
#[derive(Error, Debug)]
pub enum RemoteActionError {
    /// SSH command execution failed
    #[error("SSH command execution failed during '{action_name}': {source}")]
    SshCommandFailed {
        action_name: String,
        #[source]
        source: CommandError,
    },

    /// Action validation failed
    #[error("Action '{action_name}' validation failed: {message}")]
    ValidationFailed {
        action_name: String,
        message: String,
    },

    /// Action execution failed with custom error
    #[error("Action '{action_name}' execution failed: {message}")]
    ExecutionFailed {
        action_name: String,
        message: String,
    },
}

/// Trait for remote actions that can be executed on a server via SSH
///
/// Remote actions are lightweight scripts that connect to a provisioned
/// server via SSH to perform various operations such as:
///
/// - Validating server state and configuration
/// - Retrieving server information (hostname, installed packages, etc.)
/// - Executing maintenance tasks (updates, cleanup, etc.)
/// - Installing or configuring software components
#[allow(async_fn_in_trait)]
pub trait RemoteAction {
    /// Get the name of this action for logging purposes
    fn name(&self) -> &'static str;

    /// Execute the action against the specified server
    ///
    /// # Arguments
    /// * `server_ip` - The IP address of the server to execute the action on
    ///
    /// # Returns
    /// * `Ok(())` if the action executes successfully
    /// * `Err(RemoteActionError)` if the action fails or encounters an error
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError>;
}
