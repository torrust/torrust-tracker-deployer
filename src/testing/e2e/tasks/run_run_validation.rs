//! Run command validation task for E2E testing
//!
//! This module provides the E2E testing task for validating that the `run`
//! command executed correctly. It verifies that Docker Compose services are
//! running and healthy after deployment, and specifically checks that the
//! Torrust Tracker API is accessible and responding to health checks.
//!
//! ## Current Scope (Torrust Tracker)
//!
//! This validation checks that the deployed Torrust Tracker is operational:
//! - Docker Compose services are running
//! - Tracker API responds to health check endpoint (`/api/health_check`)
//!
//! All checks are performed from **inside** the VM via SSH commands.
//!
//! ## Future Enhancements
//!
//! When deploying additional Torrust services or expanding tracker validation,
//! the validation strategy should be extended:
//!
//! 1. **External Accessibility Testing**:
//!    - Test HTTP Tracker endpoint from outside the VM (e.g., port 7070)
//!    - Test UDP Tracker announce from outside the VM (e.g., port 6969)
//!    - Test Index API endpoints from outside the VM (if deployed)
//!
//! 2. **Firewall Validation**:
//!    - External tests implicitly validate firewall rules are correct
//!    - If service runs inside but isn't accessible outside → firewall issue
//!    - This catches UFW/iptables misconfigurations
//!
//! 3. **Protocol-Specific Tests**:
//!    - HTTP Tracker announce: Test actual announce requests
//!    - UDP Tracker announce: Requires tracker client library from torrust-tracker
//!    - Additional API endpoints beyond health check
//!
//! 4. **Dual Validation Strategy**:
//!    - Internal (via SSH): Service is running inside the VM
//!    - External (from test runner): Service is accessible through network + firewall
//!
//! See `RunningServicesValidator` in `infrastructure/remote_actions/running_services.rs`
//! for more details on the implementation approach.
//!
//! ## Key Operations
//!
//! - Validates services are running via `docker compose ps`
//! - Checks service health status if configured
//! - Tests HTTP accessibility for web services (optional)
//! - Provides comprehensive error reporting with troubleshooting steps
//!
//! ## Integration
//!
//! This validation runs after the `run` command to ensure services are
//! operational before considering the deployment successful.

use std::net::{IpAddr, SocketAddr};
use thiserror::Error;
use tracing::info;

use crate::adapters::ssh::SshConfig;
use crate::adapters::ssh::SshCredentials;
use crate::infrastructure::external_validators::RunningServicesValidator;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Errors that can occur during run validation
#[derive(Debug, Error)]
pub enum RunValidationError {
    /// Running services validation failed
    #[error(
        "Running services validation failed: {source}
Tip: Ensure Docker Compose services are started and healthy"
    )]
    RunningServicesValidationFailed {
        #[source]
        source: RemoteActionError,
    },
}

impl RunValidationError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use torrust_tracker_deployer_lib::testing::e2e::tasks::run_run_validation::RunValidationError;
    /// # use torrust_tracker_deployer_lib::infrastructure::remote_actions::RemoteActionError;
    /// let error = RunValidationError::RunningServicesValidationFailed {
    ///     source: RemoteActionError::ValidationFailed {
    ///         action_name: "running_services_validation".to_string(),
    ///         message: "No running services detected".to_string(),
    ///     },
    /// };
    /// println!("{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::RunningServicesValidationFailed { .. } => {
                "Running Services Validation Failed - Detailed Troubleshooting:

1. Check running services:
   - SSH to instance: ssh user@instance-ip
   - Check services status: cd /opt/torrust && docker compose ps
   - View service logs: docker compose logs

2. Common issues:
   - Services exited immediately after starting
   - Health checks failing (check health check configuration)
   - Port conflicts with other services
   - Container image pull failures (network issues)
   - Insufficient memory or disk space

3. Debug steps:
   - Check container logs: docker compose logs demo-app
   - Restart services: docker compose restart
   - View detailed status: docker compose ps -a

4. Re-run if needed:
   - Re-run the 'run' command: cargo run -- run <environment>
   - Or manually: cd /opt/torrust && docker compose up -d

For more information, see docs/e2e-testing/."
            }
        }
    }
}

/// Run validation tests for the `run` command on a configured instance
///
/// This function validates that the `run` command executed correctly by
/// checking that Docker Compose services are running and healthy.
///
/// # Arguments
///
/// * `socket_addr` - Socket address where the target instance can be reached
/// * `ssh_credentials` - SSH credentials for connecting to the instance
/// * `tracker_api_port` - Port for the tracker API health endpoint
/// * `http_tracker_ports` - Ports for HTTP tracker health endpoints (can be empty)
///
/// # Returns
///
/// Returns `Ok(())` when all validation tests pass successfully.
///
/// # Errors
///
/// Returns an error if:
/// - SSH connection cannot be established
/// - Services are not running
/// - Services are unhealthy
pub async fn run_run_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    tracker_api_port: u16,
    http_tracker_ports: Vec<u16>,
) -> Result<(), RunValidationError> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        tracker_api_port = tracker_api_port,
        http_tracker_ports = ?http_tracker_ports,
        "Running 'run' command validation tests"
    );

    let ip_addr = socket_addr.ip();

    // Validate running services
    validate_running_services(
        ip_addr,
        ssh_credentials,
        socket_addr.port(),
        tracker_api_port,
        http_tracker_ports,
    )
    .await?;

    info!(
        socket_addr = %socket_addr,
        status = "success",
        "All 'run' command validation tests passed successfully"
    );

    Ok(())
}

/// Validate running services on a configured instance
///
/// This function validates that Docker Compose services are running and healthy
/// on the target instance. It checks the status of services started by the `run`
/// command and verifies they are operational.
async fn validate_running_services(
    ip_addr: IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
    tracker_api_port: u16,
    http_tracker_ports: Vec<u16>,
) -> Result<(), RunValidationError> {
    info!("Validating running services");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let services_validator =
        RunningServicesValidator::new(ssh_config, tracker_api_port, http_tracker_ports);
    services_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| RunValidationError::RunningServicesValidationFailed { source })?;

    Ok(())
}
