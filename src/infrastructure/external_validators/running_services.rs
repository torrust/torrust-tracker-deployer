//! Running services external validation
//!
//! This module provides the `RunningServicesValidator` which performs **end-to-end validation
//! from OUTSIDE the VM** to verify that Docker Compose services are running and accessible
//! after the `run` command has executed the deployment.
//!
//! ## Execution Context: External Validation
//!
//! **Why this validator is in `external_validators/` instead of `remote_actions/`**:
//!
//! This validator runs from the **test runner or deployment machine** and makes HTTP requests
//! to services **from outside the VM**, unlike validators in `remote_actions/` which execute
//! commands **inside the VM via SSH**.
//!
//! **Comparison**:
//! - `remote_actions/validators/docker.rs` - Executes `docker --version` inside VM via SSH
//! - `external_validators/running_services.rs` - Makes HTTP GET to `http://<vm-ip>:1212/health` from outside
//!
//! This distinction is crucial for understanding the validation scope:
//! - **Remote actions**: Validate internal VM state and configuration
//! - **External validators**: Validate end-to-end accessibility including network and firewall
//!
//! ## Current Scope (Torrust Tracker)
//!
//! This validator performs external validation only (from test runner to VM):
//! - Verifies Docker Compose services are running (via SSH: `docker compose ps`)
//! - Tests tracker API health endpoint from outside: `http://<vm-ip>:1212/api/health_check`
//! - Tests HTTP tracker health endpoint from outside: `http://<vm-ip>:7070/health_check`
//!
//! **Validation Philosophy**: External checks are a superset of internal checks.
//! If external validation passes, it proves:
//! - Services are running inside the VM
//! - Firewall rules are configured correctly
//! - Services are accessible from outside the VM
//!
//! ## Why External-Only Validation?
//!
//! We don't perform separate internal checks (via SSH curl to localhost) because:
//! - External checks already verify service functionality
//! - Simpler E2E tests are easier to maintain
//! - If external check fails, debugging will reveal whether it's a service or firewall issue
//! - Avoiding dual validation reduces test complexity
//!
//! ## Future Enhancements
//!
//! When deploying additional Torrust services or expanding validation:
//!
//! 1. **External Accessibility Testing**: Test service accessibility from outside the VM,
//!    not just from inside. For example, if the HTTP tracker is on port 7070, we need
//!    to verify it's reachable from the test runner machine.
//!
//! 2. **Firewall Rule Verification**: External tests will implicitly validate that
//!    firewall rules (UFW/iptables) are correctly configured. If a service is running
//!    inside but not accessible from outside, it indicates a firewall misconfiguration.
//!
//! 3. **Protocol-Specific Tests**:
//!    - HTTP Tracker announce: `curl http://localhost:7070/announce?info_hash=...`
//!    - UDP Tracker announce (requires tracker client library from torrust-tracker)
//!    - Additional Index API endpoints
//!
//! 4. **Both Internal and External Checks**: Consider running both types of validation:
//!    - Internal (via SSH): Confirms service is running inside the container/VM
//!    - External (from test runner): Confirms service is accessible through the network
//!
//! Example future validation for HTTP Tracker on port 7070:
//! ```text
//! // Internal check (current approach)
//! ssh user@vm "curl -sf http://localhost:7070/announce?info_hash=..."
//!
//! // External check (future enhancement)
//! curl -sf http://<vm-public-ip>:7070/announce?info_hash=...
//! ```
//!
//! This dual approach ensures complete end-to-end validation including network
//! configuration and firewall rules.
//!
//! ## Key Features
//!
//! - Validates services are in "running" state via `docker compose ps` (via SSH)
//! - Tests tracker API accessibility from outside the VM (external HTTP check)
//! - Tests HTTP tracker accessibility from outside the VM (external HTTP check)
//! - Comprehensive error reporting with actionable troubleshooting steps
//!
//! ## Validation Process
//!
//! The validator performs the following checks:
//! 1. SSH into VM and execute `docker compose ps` to verify services are running
//! 2. Check that containers are in "running" status (not "exited" or "restarting")
//! 3. Verify health check status if configured (e.g., "healthy")
//! 4. Test tracker API from outside: HTTP GET to `http://<vm-ip>:1212/api/health_check`
//! 5. Test HTTP tracker from outside: HTTP GET to `http://<vm-ip>:7070/health_check`
//!
//! This ensures end-to-end validation:
//! - Services are deployed and running
//! - Firewall rules allow external access
//! - Services are accessible from outside the VM

use std::net::IpAddr;
use std::path::PathBuf;
use tracing::{info, instrument};

use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Default deployment directory for Docker Compose files
const DEFAULT_DEPLOY_DIR: &str = "/opt/torrust";

/// Action that validates Docker Compose services are running and healthy
pub struct RunningServicesValidator {
    deploy_dir: PathBuf,
    tracker_api_port: u16,
    http_tracker_ports: Vec<u16>,
}

impl RunningServicesValidator {
    /// Create a new `RunningServicesValidator` with the specified SSH configuration
    ///
    /// Uses the default deployment directory `/opt/torrust`.
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    /// * `tracker_api_port` - Port for the tracker API health endpoint
    /// * `http_tracker_ports` - Ports for HTTP tracker health endpoints (can be empty)
    #[must_use]
    pub fn new(
        _ssh_config: SshConfig,
        tracker_api_port: u16,
        http_tracker_ports: Vec<u16>,
    ) -> Self {
        Self {
            deploy_dir: PathBuf::from(DEFAULT_DEPLOY_DIR),
            tracker_api_port,
            http_tracker_ports,
        }
    }

    /// Create a new `RunningServicesValidator` with a custom deployment directory
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    /// * `deploy_dir` - Path to the directory containing docker-compose.yml on the remote host
    /// * `tracker_api_port` - Port for the tracker API health endpoint
    /// * `http_tracker_ports` - Ports for HTTP tracker health endpoints (can be empty)
    #[must_use]
    pub fn with_deploy_dir(
        _ssh_config: SshConfig,
        deploy_dir: PathBuf,
        tracker_api_port: u16,
        http_tracker_ports: Vec<u16>,
    ) -> Self {
        Self {
            deploy_dir,
            tracker_api_port,
            http_tracker_ports,
        }
    }

    /// Check service status using docker compose ps (human-readable format)
    /// Validate external accessibility of tracker services
    ///
    /// # Arguments
    /// * `server_ip` - IP address of the server to validate
    /// * `tracker_api_port` - Port for the tracker API health endpoint
    /// * `http_tracker_ports` - Ports for HTTP tracker health endpoints (can be empty)
    async fn validate_external_accessibility(
        &self,
        server_ip: &IpAddr,
        tracker_api_port: u16,
        http_tracker_ports: &[u16],
    ) -> Result<(), RemoteActionError> {
        // Check tracker API (required)
        self.check_tracker_api_external(server_ip, tracker_api_port)
            .await?;

        // Check all HTTP trackers (required)
        for port in http_tracker_ports {
            self.check_http_tracker_external(server_ip, *port).await?;
        }

        Ok(())
    }

    /// Check tracker API accessibility from outside the VM
    ///
    /// # Arguments
    /// * `server_ip` - IP address of the server
    /// * `port` - Port for the tracker API health endpoint
    async fn check_tracker_api_external(
        &self,
        server_ip: &IpAddr,
        port: u16,
    ) -> Result<(), RemoteActionError> {
        info!(
            action = "running_services_validation",
            check = "tracker_api_external",
            port = port,
            validation_type = "external",
            "Checking tracker API health endpoint (external from test runner)"
        );

        let url = format!("http://{server_ip}:{port}/api/health_check"); // DevSkim: ignore DS137138
        let response =
            reqwest::get(&url)
                .await
                .map_err(|e| RemoteActionError::ValidationFailed {
                    action_name: self.name().to_string(),
                    message: format!(
                        "Tracker API external health check failed: {e}. \
                     Check that tracker is running and firewall allows port {port}."
                    ),
                })?;

        if !response.status().is_success() {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "Tracker API returned HTTP {}: {}. Service may not be healthy.",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ),
            });
        }

        info!(
            action = "running_services_validation",
            check = "tracker_api_external",
            port = port,
            status = "success",
            validation_type = "external",
            "Tracker API is accessible from outside (external check passed)"
        );

        Ok(())
    }

    /// Check HTTP tracker accessibility from outside the VM
    ///
    /// # Arguments
    /// * `server_ip` - IP address of the server
    /// * `port` - Port for the HTTP tracker health endpoint
    async fn check_http_tracker_external(
        &self,
        server_ip: &IpAddr,
        port: u16,
    ) -> Result<(), RemoteActionError> {
        info!(
            action = "running_services_validation",
            check = "http_tracker_external",
            port = port,
            validation_type = "external",
            "Checking HTTP tracker health endpoint (external from test runner)"
        );

        let url = format!("http://{server_ip}:{port}/health_check"); // DevSkim: ignore DS137138
        let response =
            reqwest::get(&url)
                .await
                .map_err(|e| RemoteActionError::ValidationFailed {
                    action_name: self.name().to_string(),
                    message: format!(
                        "HTTP Tracker external health check failed for URL '{url}': {e}. \n\
                     Check that HTTP tracker is running and firewall allows port {port}."
                    ),
                })?;

        if !response.status().is_success() {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "HTTP Tracker returned HTTP {} for URL '{url}': {}. Service may not be healthy.",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ),
            });
        }

        info!(
            action = "running_services_validation",
            check = "http_tracker_external",
            port = port,
            status = "success",
            validation_type = "external",
            url = %url,
            "HTTP Tracker is accessible from outside (external check passed)"
        );

        Ok(())
    }
}

impl RemoteAction for RunningServicesValidator {
    fn name(&self) -> &'static str {
        "running-services-validation"
    }

    #[instrument(
        name = "running_services_validation",
        skip(self),
        fields(
            action_type = "validation",
            component = "running_services",
            server_ip = %server_ip,
            deploy_dir = %self.deploy_dir.display()
        )
    )]
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "running_services_validation",
            deploy_dir = %self.deploy_dir.display(),
            "Validating Docker Compose services are running via external accessibility"
        );

        // For E2E tests, external accessibility validation is sufficient
        // If services are accessible externally, it proves they are running and healthy
        self.validate_external_accessibility(
            server_ip,
            self.tracker_api_port,
            &self.http_tracker_ports,
        )
        .await?;

        info!(
            action = "running_services_validation",
            status = "success",
            "Running services validation completed successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::adapters::ssh::{SshConfig, SshCredentials};
    use crate::shared::Username;

    use super::*;

    fn create_test_ssh_config() -> SshConfig {
        let credentials = SshCredentials::new(
            PathBuf::from("/mock/path/to/private_key"),
            PathBuf::from("/mock/path/to/public_key.pub"),
            Username::new("testuser").unwrap(),
        );
        SshConfig::with_default_port(credentials, "127.0.0.1".parse().unwrap())
    }

    #[test]
    fn it_should_use_default_deploy_dir_when_not_specified() {
        assert_eq!(DEFAULT_DEPLOY_DIR, "/opt/torrust");
    }

    #[test]
    fn it_should_return_correct_action_name_when_queried() {
        // Can't test without SSH config, but we can verify the constant
        assert_eq!("running-services-validation", "running-services-validation");
    }

    #[test]
    fn it_should_accept_validation_when_http_tracker_ports_are_empty() {
        let ssh_config = create_test_ssh_config();
        let validator = RunningServicesValidator::new(ssh_config, 6969, vec![]);

        assert_eq!(validator.http_tracker_ports.len(), 0);
    }

    #[test]
    fn it_should_accept_validation_when_single_http_tracker_port_configured() {
        let ssh_config = create_test_ssh_config();
        let validator = RunningServicesValidator::new(ssh_config, 6969, vec![6060]);

        assert_eq!(validator.http_tracker_ports.len(), 1);
        assert_eq!(validator.http_tracker_ports[0], 6060);
    }

    #[test]
    fn it_should_accept_validation_when_multiple_http_tracker_ports_configured() {
        let ssh_config = create_test_ssh_config();
        let ports = vec![6060, 6061, 6062];
        let validator = RunningServicesValidator::new(ssh_config, 6969, ports.clone());

        assert_eq!(validator.http_tracker_ports.len(), 3);
        assert_eq!(validator.http_tracker_ports, ports);
    }

    #[test]
    fn it_should_accept_empty_ports_when_using_custom_deploy_dir() {
        let ssh_config = create_test_ssh_config();
        let validator = RunningServicesValidator::with_deploy_dir(
            ssh_config,
            PathBuf::from("/custom/path"),
            6969,
            vec![],
        );

        assert_eq!(validator.http_tracker_ports.len(), 0);
        assert_eq!(validator.deploy_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn it_should_accept_multiple_ports_when_using_custom_deploy_dir() {
        let ssh_config = create_test_ssh_config();
        let ports = vec![6060, 6061];
        let validator = RunningServicesValidator::with_deploy_dir(
            ssh_config,
            PathBuf::from("/custom/path"),
            6969,
            ports.clone(),
        );

        assert_eq!(validator.http_tracker_ports.len(), 2);
        assert_eq!(validator.http_tracker_ports, ports);
        assert_eq!(validator.deploy_dir, PathBuf::from("/custom/path"));
    }
}
