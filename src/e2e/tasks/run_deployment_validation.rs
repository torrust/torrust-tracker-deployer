//! Deployment validation task for E2E testing
//!
//! This module provides the E2E testing task for validating that deployments
//! are working correctly after configuration. It performs comprehensive checks
//! to ensure all required services and components are properly installed and running.
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
//! This is a generic task that can be used with any infrastructure type:
//! - Container-based testing environments
//! - VM-based testing environments
//! - Physical server deployments
//! - Cloud instance deployments

use std::net::SocketAddr;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::SshCredentials;
use crate::infrastructure::remote_actions::{
    DockerComposeValidator, DockerValidator, RemoteAction,
};

/// Run deployment validation tests on a configured instance
///
/// This function performs comprehensive validation of a deployed instance,
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
/// use torrust_tracker_deploy::e2e::tasks::run_deployment_validation::run_deployment_validation;
/// use torrust_tracker_deploy::config::SshCredentials;
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222);
///     let ssh_credentials = SshCredentials::new(
///         "./id_rsa".into(),
///         "./id_rsa.pub".into(),
///         "testuser".to_string()
///     );
///     
///     run_deployment_validation(socket_addr, &ssh_credentials).await?;
///     println!("All deployment validations passed");
///     Ok(())
/// }
/// ```
pub async fn run_deployment_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Running deployment validation tests"
    );

    let ip_addr = socket_addr.ip();

    // Create SSH connection with the instance's address and port
    let ssh_connection = ssh_credentials
        .clone()
        .with_host_and_port(ip_addr, socket_addr.port());

    // Validate Docker installation
    info!("Validating Docker installation");
    let docker_validator = DockerValidator::new(ssh_connection.clone());
    docker_validator
        .execute(&ip_addr)
        .await
        .context("Docker validation failed")?;

    // Validate Docker Compose installation
    info!("Validating Docker Compose installation");
    let compose_validator = DockerComposeValidator::new(ssh_connection);
    compose_validator
        .execute(&ip_addr)
        .await
        .context("Docker Compose validation failed")?;

    info!(
        socket_addr = %socket_addr,
        status = "success",
        "All deployment validation tests passed successfully"
    );

    Ok(())
}
