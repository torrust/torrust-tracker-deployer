//! Configuration validation task for E2E testing
//!
//! This module provides the E2E testing task for validating that the `configure`
//! command executed correctly. It performs comprehensive checks to ensure all
//! required services and components are properly installed.
//!
//! ## Key Operations
//!
//! - Validates Docker installation and functionality
//! - Validates Docker Compose installation
//! - Can be extended to validate additional services and components
//! - Works with both container and VM-based infrastructure
//!
//! ## Integration
//!
//! This validation runs after the `configure` command to verify that Docker
//! and Docker Compose were installed correctly. For validating running services
//! after the `run` command, use `run_run_validation` instead.

use std::net::SocketAddr;
use thiserror::Error;
use tracing::info;

use crate::adapters::ssh::SshConfig;
use crate::adapters::ssh::SshCredentials;
use crate::infrastructure::remote_actions::{
    DockerComposeValidator, DockerValidator, RemoteAction, RemoteActionError,
};

/// Errors that can occur during configuration validation
#[derive(Debug, Error)]
pub enum ConfigurationValidationError {
    /// Docker validation failed
    #[error(
        "Docker validation failed: {source}
Tip: Ensure Docker is properly installed and the daemon is running"
    )]
    DockerValidationFailed {
        #[source]
        source: RemoteActionError,
    },

    /// Docker Compose validation failed
    #[error(
        "Docker Compose validation failed: {source}
Tip: Ensure Docker Compose is properly installed and functional"
    )]
    DockerComposeValidationFailed {
        #[source]
        source: RemoteActionError,
    },
}

impl ConfigurationValidationError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_configuration_validation::ConfigurationValidationError;
    /// # use torrust_tracker_deployer_lib::infrastructure::remote_actions::RemoteActionError;
    /// # use torrust_tracker_deployer_lib::shared::command::CommandError;
    /// let error = ConfigurationValidationError::DockerValidationFailed {
    ///     source: RemoteActionError::SshCommandFailed {
    ///         action_name: "docker_validation".to_string(),
    ///         source: CommandError::ExecutionFailed {
    ///             command: "docker --version".to_string(),
    ///             exit_code: "1".to_string(),
    ///             stdout: String::new(),
    ///             stderr: "command not found".to_string(),
    ///         },
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerValidationFailed { .. } => {
                "Docker Validation Failed - Detailed Troubleshooting:

1. Check Docker installation:
   - SSH to instance: ssh user@instance-ip
   - Check if Docker is installed: docker --version
   - Check if Docker daemon is running: sudo systemctl status docker

2. Verify Docker installation process:
   - Check Ansible logs for installation errors
   - Verify Docker installation playbook completed successfully
   - Ensure no package repository connectivity issues during installation

3. Common issues:
   - Docker installed but daemon not started: sudo systemctl start docker
   - User not in docker group: sudo usermod -aG docker $USER (requires logout/login)
   - Docker binary not in PATH: check /usr/bin/docker exists
   - Insufficient permissions: verify user has sudo access

4. Re-install if needed:
   - Re-run configuration command to attempt Docker installation again
   - Or manually install Docker following official documentation

For more information, see docs/e2e-testing/."
            }

            Self::DockerComposeValidationFailed { .. } => {
                "Docker Compose Validation Failed - Detailed Troubleshooting:

1. Check Docker Compose installation:
   - SSH to instance: ssh user@instance-ip
   - Check if Docker Compose is installed: docker compose version
   - Verify Docker Compose plugin is available

2. Verify installation process:
   - Check Ansible logs for installation errors
   - Verify Docker Compose installation playbook completed successfully
   - Ensure Docker is installed (Docker Compose requires Docker)

3. Common issues:
   - Using old 'docker-compose' syntax instead of 'docker compose'
   - Docker Compose plugin not installed alongside Docker
   - Wrong Docker Compose version for current Docker version
   - Installation failed but was not detected

4. Re-install if needed:
   - Re-run configuration command to attempt installation again
   - Or manually install Docker Compose following official documentation

For more information, see docs/e2e-testing/."
            }
        }
    }
}

/// Run configuration validation tests on a configured instance
///
/// This function performs comprehensive validation of a configured instance,
/// checking that all required services and components are properly installed
/// and functioning. It uses SSH to connect to the target instance and run
/// validation commands.
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
/// - Docker validation fails (not installed or not working)
/// - Docker Compose validation fails (not installed or not working)
/// - Any other validation checks fail
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::run_configuration_validation::run_configuration_validation;
/// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
/// use torrust_tracker_deployer_lib::shared::username::Username;
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222);
///     let ssh_credentials = SshCredentials::new(
///         "./id_rsa".into(),
///         "./id_rsa.pub".into(),
///         Username::new("testuser").unwrap()
///     );
///     
///     run_configuration_validation(socket_addr, &ssh_credentials).await?;
///     println!("All configuration validations passed");
///     Ok(())
/// }
/// ```
pub async fn run_configuration_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
) -> Result<(), ConfigurationValidationError> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Running configuration validation tests"
    );

    let ip_addr = socket_addr.ip();

    // Validate Docker installation
    validate_docker_installation(ip_addr, ssh_credentials, socket_addr.port()).await?;

    // Validate Docker Compose installation
    validate_docker_compose_installation(ip_addr, ssh_credentials, socket_addr.port()).await?;

    info!(
        socket_addr = %socket_addr,
        status = "success",
        "All configuration validation tests passed successfully"
    );

    Ok(())
}

/// Validate Docker installation on a configured instance
///
/// This function validates that Docker is properly installed and functioning
/// on the target instance by connecting via SSH and running validation commands.
///
/// # Arguments
///
/// * `ip_addr` - IP address of the target instance
/// * `ssh_credentials` - SSH credentials for connecting to the instance
///
/// # Returns
///
/// Returns `Ok(())` when Docker validation passes successfully.
///
/// # Errors
///
/// Returns an error if:
/// - SSH connection cannot be established
/// - Docker validation fails (not installed or not working)
async fn validate_docker_installation(
    ip_addr: std::net::IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
) -> Result<(), ConfigurationValidationError> {
    info!("Validating Docker installation");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let docker_validator = DockerValidator::new(ssh_config);
    docker_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| ConfigurationValidationError::DockerValidationFailed { source })?;

    Ok(())
}

/// Validate Docker Compose installation on a configured instance
///
/// This function validates that Docker Compose is properly installed and functioning
/// on the target instance by connecting via SSH and running validation commands.
///
/// # Arguments
///
/// * `ip_addr` - IP address of the target instance
/// * `ssh_credentials` - SSH credentials for connecting to the instance
/// * `port` - SSH port to connect to
///
/// # Returns
///
/// Returns `Ok(())` when Docker Compose validation passes successfully.
///
/// # Errors
///
/// Returns an error if:
/// - SSH connection cannot be established
/// - Docker Compose validation fails (not installed or not working)
async fn validate_docker_compose_installation(
    ip_addr: std::net::IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
) -> Result<(), ConfigurationValidationError> {
    info!("Validating Docker Compose installation");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let compose_validator = DockerComposeValidator::new(ssh_config);
    compose_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| ConfigurationValidationError::DockerComposeValidationFailed { source })?;

    Ok(())
}
