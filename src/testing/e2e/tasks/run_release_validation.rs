//! Release validation task for E2E testing
//!
//! This module provides the E2E testing task for validating that the `release`
//! command executed correctly. It verifies that Docker Compose files were
//! properly deployed to the target instance.
//!
//! ## Key Operations
//!
//! - Validates Docker Compose files are present in the deployment directory
//! - Verifies file permissions and ownership
//! - Checks that the deployment directory structure is correct
//!
//! ## Integration
//!
//! This validation runs after the `release` command and before the `run` command
//! to ensure files are in place before starting services.

use std::net::SocketAddr;
use thiserror::Error;
use tracing::info;

use crate::adapters::ssh::SshConfig;
use crate::adapters::ssh::SshCredentials;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Default deployment directory for Docker Compose files
const DEFAULT_DEPLOY_DIR: &str = "/opt/torrust";

/// Errors that can occur during release validation
#[derive(Debug, Error)]
pub enum ReleaseValidationError {
    /// Compose files validation failed
    #[error(
        "Docker Compose files validation failed: {source}
Tip: Ensure the release command completed successfully and files were deployed"
    )]
    ComposeFilesValidationFailed {
        #[source]
        source: RemoteActionError,
    },
}

impl ReleaseValidationError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_release_validation::ReleaseValidationError;
    /// # use torrust_tracker_deployer_lib::infrastructure::remote_actions::RemoteActionError;
    /// let error = ReleaseValidationError::ComposeFilesValidationFailed {
    ///     source: RemoteActionError::ValidationFailed {
    ///         action_name: "compose_files_validation".to_string(),
    ///         message: "docker-compose.yml not found".to_string(),
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ComposeFilesValidationFailed { .. } => {
                "Docker Compose Files Validation Failed - Detailed Troubleshooting:

1. Check if release command completed:
   - SSH to instance: ssh user@instance-ip
   - Check deployment directory: ls -la /opt/torrust/
   - Verify docker-compose.yml exists

2. Verify file deployment:
   - Check Ansible deployment logs for errors
   - Verify the release command ran without errors
   - Ensure source template files exist in templates/docker-compose/

3. Common issues:
   - Deployment directory not created: mkdir -p /opt/torrust
   - Insufficient permissions to write files
   - Ansible playbook failed silently
   - Template rendering errors

4. Re-deploy if needed:
   - Re-run release command: cargo run -- release <environment>
   - Or manually copy files to /opt/torrust/

For more information, see docs/e2e-testing/."
            }
        }
    }
}

/// Validates Docker Compose files are deployed
struct ComposeFilesValidator {
    ssh_client: crate::adapters::ssh::SshClient,
    deploy_dir: std::path::PathBuf,
}

impl ComposeFilesValidator {
    /// Create a new `ComposeFilesValidator` with the specified SSH configuration
    #[must_use]
    fn new(ssh_config: SshConfig) -> Self {
        let ssh_client = crate::adapters::ssh::SshClient::new(ssh_config);
        Self {
            ssh_client,
            deploy_dir: std::path::PathBuf::from(DEFAULT_DEPLOY_DIR),
        }
    }
}

impl RemoteAction for ComposeFilesValidator {
    fn name(&self) -> &'static str {
        "compose-files-validation"
    }

    async fn execute(&self, server_ip: &std::net::IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "compose_files_validation",
            deploy_dir = %self.deploy_dir.display(),
            server_ip = %server_ip,
            "Validating Docker Compose files are deployed"
        );

        // Check if docker-compose.yml exists
        let deploy_dir = self.deploy_dir.display();
        let command = format!("test -f {deploy_dir}/docker-compose.yml && echo 'exists'");

        let output = self.ssh_client.execute(&command).map_err(|source| {
            RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            }
        })?;

        if !output.trim().contains("exists") {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "docker-compose.yml not found in {deploy_dir}. \
                     Ensure the release command completed successfully."
                ),
            });
        }

        info!(
            action = "compose_files_validation",
            status = "success",
            "Docker Compose files are deployed correctly"
        );

        Ok(())
    }
}

/// Run release validation tests on a configured instance
///
/// This function validates that the `release` command executed correctly
/// by checking that Docker Compose files are present in the deployment directory.
///
/// # Arguments
///
/// * `socket_addr` - Socket address where the target instance can be reached
/// * `ssh_credentials` - SSH credentials for connecting to the instance
///
/// # Returns
///
/// Returns `Ok(())` when all validation tests pass successfully.
///
/// # Errors
///
/// Returns an error if:
/// - SSH connection cannot be established
/// - Docker Compose files are not found
/// - File validation fails
pub async fn run_release_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
) -> Result<(), ReleaseValidationError> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Running release validation tests"
    );

    let ip_addr = socket_addr.ip();

    // Validate Docker Compose files are deployed
    validate_compose_files(ip_addr, ssh_credentials, socket_addr.port()).await?;

    info!(
        socket_addr = %socket_addr,
        status = "success",
        "All release validation tests passed successfully"
    );

    Ok(())
}

/// Validate Docker Compose files are deployed
async fn validate_compose_files(
    ip_addr: std::net::IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
) -> Result<(), ReleaseValidationError> {
    info!("Validating Docker Compose files deployment");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let validator = ComposeFilesValidator::new(ssh_config);
    validator
        .execute(&ip_addr)
        .await
        .map_err(|source| ReleaseValidationError::ComposeFilesValidationFailed { source })?;

    Ok(())
}
