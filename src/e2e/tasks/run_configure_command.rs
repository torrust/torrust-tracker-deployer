//! Ansible configuration task for E2E testing
//!
//! This module provides the E2E testing task for running Ansible configuration
//! on target instances. It executes Ansible playbooks to configure services
//! and applications on the deployed infrastructure.
//!
//! ## Key Operations
//!
//! - Executes Ansible playbooks using the `ConfigureCommand`
//! - Handles configuration workflow for both containers and VMs
//! - Provides structured error handling and reporting
//!
//! ## Integration
//!
//! This is a generic task that works with infrastructure-agnostic configuration:
//! - Uses rendered Ansible inventories from provision simulation
//! - Works with both container and VM-based infrastructure
//! - Integrates with the existing `ConfigureCommand` workflow
//!
//! ## Notes
//!
//! Currently, this task has limitations when used with containers due to
//! inventory addressing differences. Container-specific inventory templates
//! may be needed for full container support.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::application::commands::ConfigureCommand;
use crate::container::Services;
use crate::e2e::environment::TestEnvironment;

/// Run Ansible configuration on a target instance
///
/// This function executes Ansible playbooks to configure services and applications
/// on the target instance. It uses the existing `ConfigureCommand` workflow and
/// handles both successful configurations and expected failures.
///
/// # Arguments
///
/// * `socket_addr` - Socket address where the target instance can be reached
/// * `test_env` - Test environment containing configuration and services
/// * `expect_success` - Whether to expect configuration to succeed (for testing purposes)
///
/// # Returns
///
/// Returns `Ok(())` when:
/// - Configuration succeeds (if `expect_success` is true)
/// - Expected failure occurs (if `expect_success` is false)
///
/// # Errors
///
/// Returns an error if:
/// - Configuration fails unexpectedly (when `expect_success` is true)
/// - Services cannot be initialized
/// - `ConfigureCommand` execution encounters unexpected errors
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::run_configure_command::run_ansible_configuration;
/// use torrust_tracker_deploy::e2e::environment::TestEnvironment;
/// use torrust_tracker_deploy::config::InstanceName;
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// fn main() -> anyhow::Result<()> {
///     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222);
///     let instance_name = InstanceName::new("test-instance".to_string())?;
///     let test_env = TestEnvironment::new(false, "./templates".to_string(), instance_name)?;
///     
///     run_ansible_configuration(socket_addr, &test_env, true)?;
///     println!("Ansible configuration completed successfully");
///     Ok(())
/// }
/// ```
pub fn run_ansible_configuration(
    socket_addr: SocketAddr,
    test_env: &TestEnvironment,
    expect_success: bool,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        expect_success = expect_success,
        "Running Ansible configuration on instance"
    );

    // Initialize services from test environment configuration
    let services = Services::new(&test_env.config);
    let configure_command = ConfigureCommand::new(Arc::clone(&services.ansible_client));

    // Execute configuration command
    match configure_command.execute().map_err(anyhow::Error::from) {
        Ok(()) => {
            if expect_success {
                info!(
                    socket_addr = %socket_addr,
                    status = "success",
                    "Configuration completed successfully"
                );
            } else {
                warn!(
                    socket_addr = %socket_addr,
                    status = "unexpected_success",
                    "Configuration succeeded when failure was expected"
                );
            }
        }
        Err(e) => {
            if expect_success {
                return Err(e.context("Ansible configuration failed unexpectedly"));
            }
            info!(
                socket_addr = %socket_addr,
                status = "expected_failure",
                error = %e,
                "Configuration failed as expected"
            );
        }
    }

    info!(
        socket_addr = %socket_addr,
        status = "complete",
        "Ansible configuration workflow completed"
    );

    Ok(())
}

/// Configure infrastructure using Ansible playbooks (compatibility wrapper)
///
/// This is a simplified wrapper around the `ConfigureCommand` for use in full E2E tests.
/// For more advanced configuration with success/failure handling, use `run_ansible_configuration`.
///
/// # Errors
///
/// Returns an error if:
/// - `ConfigureCommand` execution fails
/// - Infrastructure configuration fails
pub fn configure_infrastructure(env: &TestEnvironment) -> Result<()> {
    info!("Configuring test infrastructure");

    // Use the new ConfigureCommand to handle all infrastructure configuration steps
    let configure_command = ConfigureCommand::new(Arc::clone(&env.services.ansible_client));

    configure_command
        .execute()
        .map_err(anyhow::Error::from)
        .context("Failed to configure infrastructure")?;

    info!(
        status = "complete",
        "Infrastructure configuration completed successfully"
    );

    Ok(())
}
