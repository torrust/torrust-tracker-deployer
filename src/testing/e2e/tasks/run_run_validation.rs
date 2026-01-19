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
use crate::infrastructure::external_validators::{RunningServicesValidator, ServiceEndpoint};
use crate::infrastructure::remote_actions::validators::{GrafanaValidator, PrometheusValidator};
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Service validation configuration
///
/// Controls which optional service validations should be performed
/// during run validation. This allows for flexible validation
/// based on which services are enabled in the environment configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct ServiceValidation {
    /// Whether to validate Prometheus is running and accessible
    pub prometheus: bool,
    /// Whether to validate Grafana is running and accessible
    pub grafana: bool,
}

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

    /// Prometheus smoke test failed
    #[error(
        "Prometheus smoke test failed: {source}
Tip: Ensure Prometheus container is running and accessible on port 9090"
    )]
    PrometheusValidationFailed {
        #[source]
        source: RemoteActionError,
    },

    /// Grafana smoke test failed
    #[error(
        "Grafana smoke test failed: {source}
Tip: Ensure Grafana container is running and accessible on port 3100"
    )]
    GrafanaValidationFailed {
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
            Self::PrometheusValidationFailed { .. } => {
                "Prometheus Smoke Test Failed - Detailed Troubleshooting:

1. Check Prometheus container status:
   - SSH to instance: ssh user@instance-ip
   - Check container: cd /opt/torrust && docker compose ps
   - View Prometheus logs: docker compose logs prometheus

2. Verify Prometheus is accessible:
   - Test from inside VM: curl http://localhost:9090
   - Check if port 9090 is listening: ss -tlnp | grep 9090

3. Common issues:
   - Prometheus container failed to start (check logs)
   - Port 9090 already in use by another process
   - Prometheus configuration file has errors
   - Insufficient memory for Prometheus

4. Debug steps:
   - Check Prometheus config: docker compose exec prometheus cat /etc/prometheus/prometheus.yml
   - Restart Prometheus: docker compose restart prometheus
   - Check scrape targets: curl http://localhost:9090/api/v1/targets | jq

5. Re-deploy if needed:
   - Release command: cargo run -- release <environment>
   - Run command: cargo run -- run <environment>

For more information, see docs/e2e-testing/."
            }
            Self::GrafanaValidationFailed { .. } => {
                "Grafana Smoke Test Failed - Detailed Troubleshooting:

1. Check Grafana container status:
   - SSH to instance: ssh user@instance-ip
   - Check container: cd /opt/torrust && docker compose ps
   - View Grafana logs: docker compose logs grafana

2. Verify Grafana is accessible:
   - Test from inside VM: curl http://localhost:3100
   - Check if port 3100 is listening: ss -tlnp | grep 3100

3. Common issues:
   - Grafana container failed to start (check logs)
   - Port 3100 already in use by another process
   - Invalid admin credentials in environment variables
   - Insufficient memory for Grafana
   - Grafana depends on Prometheus but Prometheus not running

4. Debug steps:
   - Check environment variables: docker compose exec grafana env | grep GF_
   - Restart Grafana: docker compose restart grafana
   - Access Grafana UI: http://<vm-ip>:3100 (from your browser)
   - Check datasources: curl http://localhost:3100/api/datasources | jq

5. Re-deploy if needed:
   - Release command: cargo run -- release <environment>
   - Run command: cargo run -- run <environment>

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
/// * `tracker_api_endpoint` - Endpoint for the tracker API health check
/// * `http_tracker_endpoints` - Endpoints for HTTP tracker health checks
/// * `services` - Optional service validation configuration (defaults to no optional services)
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
/// - Optional service validation fails (when enabled)
pub async fn run_run_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    tracker_api_endpoint: ServiceEndpoint,
    http_tracker_endpoints: Vec<ServiceEndpoint>,
    services: Option<ServiceValidation>,
) -> Result<(), RunValidationError> {
    let services = services.unwrap_or_default();

    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        api_uses_tls = tracker_api_endpoint.uses_tls(),
        http_tracker_count = http_tracker_endpoints.len(),
        validate_prometheus = services.prometheus,
        validate_grafana = services.grafana,
        "Running 'run' command validation tests"
    );

    let ip_addr = socket_addr.ip();

    // Validate externally accessible services (tracker API, HTTP tracker)
    validate_external_services(ip_addr, tracker_api_endpoint, http_tracker_endpoints).await?;

    // Optionally validate Prometheus is running and accessible
    if services.prometheus {
        validate_prometheus(ip_addr, ssh_credentials, socket_addr.port()).await?;
    }
    // Optionally validate Grafana is running and accessible
    if services.grafana {
        validate_grafana(ip_addr, ssh_credentials, socket_addr.port()).await?;
    }

    info!(
        socket_addr = %socket_addr,
        status = "success",
        "All 'run' command validation tests passed successfully"
    );

    Ok(())
}

/// Validate externally accessible services (tracker API, HTTP tracker)
///
/// This function validates that the tracker API and HTTP tracker services
/// are running and responding to health check requests.
///
/// # Arguments
///
/// * `ip_addr` - IP address of the target instance
/// * `tracker_api_endpoint` - Endpoint for the tracker API health check
/// * `http_tracker_endpoints` - Endpoints for HTTP tracker health checks
///
/// # Returns
///
/// Returns `Ok(())` when all services are validated successfully.
///
/// # Errors
///
/// Returns an error if any service is not running or unhealthy.
async fn validate_external_services(
    ip_addr: IpAddr,
    tracker_api_endpoint: ServiceEndpoint,
    http_tracker_endpoints: Vec<ServiceEndpoint>,
) -> Result<(), RunValidationError> {
    info!(
        api_uses_tls = tracker_api_endpoint.uses_tls(),
        http_tracker_count = http_tracker_endpoints.len(),
        "Validating externally accessible services (tracker API, HTTP tracker)"
    );

    let services_validator =
        RunningServicesValidator::new(tracker_api_endpoint, http_tracker_endpoints);
    services_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| RunValidationError::RunningServicesValidationFailed { source })?;

    Ok(())
}

/// Validate Prometheus is running and accessible via smoke test
///
/// This function performs a smoke test on Prometheus by connecting via SSH
/// and executing a curl command to verify the web UI is accessible.
///
/// # Note
///
/// Prometheus runs on port 9090 inside the VM but is NOT exposed externally
/// (blocked by firewall). Validation must be performed from inside the VM.
async fn validate_prometheus(
    ip_addr: IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
) -> Result<(), RunValidationError> {
    info!("Validating Prometheus is running and accessible");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let prometheus_validator = PrometheusValidator::new(ssh_config, None);
    prometheus_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| RunValidationError::PrometheusValidationFailed { source })?;

    Ok(())
}

/// Validate Grafana is running and accessible via smoke test
///
/// This function performs a smoke test on Grafana by connecting via SSH
/// and executing a curl command to verify the web UI is accessible.
///
/// # Note
///
/// Grafana runs on port 3000 inside the container but is exposed on port 3100
/// on the host via docker-compose port mapping. Docker published ports bypass
/// UFW firewall, so Grafana is accessible externally. However, for consistency
/// with other validators, we test from inside the VM via SSH.
async fn validate_grafana(
    ip_addr: IpAddr,
    ssh_credentials: &SshCredentials,
    port: u16,
) -> Result<(), RunValidationError> {
    info!("Validating Grafana is running and accessible");

    let ssh_config = SshConfig::new(ssh_credentials.clone(), SocketAddr::new(ip_addr, port));

    let grafana_validator = GrafanaValidator::new(ssh_config, None);
    grafana_validator
        .execute(&ip_addr)
        .await
        .map_err(|source| RunValidationError::GrafanaValidationFailed { source })?;

    Ok(())
}
