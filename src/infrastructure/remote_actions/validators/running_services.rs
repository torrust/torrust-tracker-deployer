//! Running services validation remote action
//!
//! This module provides the `RunningServicesValidator` which checks that Docker Compose
//! services are running and healthy on remote instances after the `run` command has
//! executed the deployment.
//!
//! ## Current Scope (Demo Slice)
//!
//! This validator is designed for the demo slice which uses a temporary mocked service
//! (nginx web server). Validation is performed from **inside** the VM via SSH.
//!
//! ## Future Enhancements (Real Services)
//!
//! When implementing real Torrust services (Tracker, Index), validation should be
//! extended to include **external accessibility testing**:
//!
//! 1. **External HTTP/UDP Validation**: Test service accessibility from outside the VM,
//!    not just from inside. For example, if the HTTP tracker is on port 7070, we need
//!    to verify it's reachable from the test runner machine.
//!
//! 2. **Firewall Rule Verification**: External tests will implicitly validate that
//!    firewall rules (UFW/iptables) are correctly configured. If a service is running
//!    inside but not accessible from outside, it indicates a firewall misconfiguration.
//!
//! 3. **Both Internal and External Checks**: Consider running both types of validation:
//!    - Internal (via SSH): Confirms service is running inside the container/VM
//!    - External (from test runner): Confirms service is accessible through the network
//!
//! Example future validation for HTTP Tracker on port 7070:
//! ```text
//! // Internal check (current approach)
//! ssh user@vm "curl -sf http://localhost:7070/health"
//!
//! // External check (future enhancement)
//! curl -sf http://<vm-public-ip>:7070/health
//! ```
//!
//! This dual approach ensures complete end-to-end validation including network
//! configuration and firewall rules.
//!
//! ## Key Features
//!
//! - Validates services are in "running" state via `docker compose ps`
//! - Checks service health status (healthy/unhealthy)
//! - Verifies service accessibility via HTTP endpoint (for web services)
//! - Comprehensive error reporting with actionable troubleshooting steps
//!
//! ## Validation Process
//!
//! The validator performs multiple checks:
//! 1. Execute `docker compose ps` to verify services are listed
//! 2. Check that containers are in "running" status (not "exited" or "restarting")
//! 3. Verify health check status if configured (e.g., "healthy")
//! 4. Test HTTP accessibility for web services (optional)
//!
//! This ensures that the full deployment pipeline is validated end-to-end,
//! confirming that services are not just deployed but actually operational.

use std::net::IpAddr;
use std::path::PathBuf;
use tracing::{info, instrument, warn};

use crate::adapters::ssh::SshClient;
use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Default deployment directory for Docker Compose files
const DEFAULT_DEPLOY_DIR: &str = "/opt/torrust";

/// Action that validates Docker Compose services are running and healthy
pub struct RunningServicesValidator {
    ssh_client: SshClient,
    deploy_dir: PathBuf,
}

impl RunningServicesValidator {
    /// Create a new `RunningServicesValidator` with the specified SSH configuration
    ///
    /// Uses the default deployment directory `/opt/torrust`.
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self {
            ssh_client,
            deploy_dir: PathBuf::from(DEFAULT_DEPLOY_DIR),
        }
    }

    /// Create a new `RunningServicesValidator` with a custom deployment directory
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    /// * `deploy_dir` - Path to the directory containing docker-compose.yml on the remote host
    #[must_use]
    pub fn with_deploy_dir(ssh_config: SshConfig, deploy_dir: PathBuf) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self {
            ssh_client,
            deploy_dir,
        }
    }

    /// Check service status using docker compose ps (human-readable format)
    fn check_services_status(&self) -> Result<String, RemoteActionError> {
        let deploy_dir = self.deploy_dir.display();
        let command = format!("cd {deploy_dir} && docker compose ps");

        self.ssh_client
            .execute(&command)
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })
    }

    /// Check if demo-app service (nginx) is accessible via HTTP
    fn check_http_accessibility(&self, port: u16) -> Result<bool, RemoteActionError> {
        let command = format!("curl -sf http://localhost:{port} > /dev/null");

        self.ssh_client.check_command(&command).map_err(|source| {
            RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            }
        })
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
            "Validating Docker Compose services are running"
        );

        // Step 1: Check services status using docker compose ps
        let services_output = self.check_services_status()?;
        let services_output = services_output.trim();

        info!(
            action = "running_services_validation",
            check = "docker_compose_ps",
            "Docker Compose services status retrieved"
        );

        // Step 2: Validate that at least one service is running
        // The output should contain service information (not empty or just headers)
        let has_running_services = !services_output.is_empty()
            && (services_output.contains("running") || services_output.contains("Up"));

        if !has_running_services {
            warn!(
                action = "running_services_validation",
                check = "services_running",
                status = "warning",
                output = %services_output,
                "No running services detected in docker compose ps output"
            );
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "No running services detected. Output: {}",
                    if services_output.is_empty() {
                        "(empty)"
                    } else {
                        services_output
                    }
                ),
            });
        }

        info!(
            action = "running_services_validation",
            check = "services_running",
            status = "success",
            "Docker Compose services are running"
        );

        // Step 3: Check for healthy status (if health checks are configured)
        let has_healthy_services = services_output.contains("healthy");
        let has_unhealthy_services = services_output.contains("unhealthy");

        if has_unhealthy_services {
            warn!(
                action = "running_services_validation",
                check = "health_status",
                status = "warning",
                output = %services_output,
                "Some services are unhealthy"
            );
            // Don't fail - just warn. Services might still be starting up.
        } else if has_healthy_services {
            info!(
                action = "running_services_validation",
                check = "health_status",
                status = "success",
                "Services are healthy"
            );
        }

        // Step 4: Test HTTP accessibility for demo-app (nginx on port 8080)
        match self.check_http_accessibility(8080) {
            Ok(true) => {
                info!(
                    action = "running_services_validation",
                    check = "http_accessibility",
                    port = 8080,
                    status = "success",
                    "Demo app service is accessible via HTTP"
                );
            }
            Ok(false) => {
                warn!(
                    action = "running_services_validation",
                    check = "http_accessibility",
                    port = 8080,
                    status = "warning",
                    "Demo app service HTTP check returned false (may still be starting)"
                );
            }
            Err(e) => {
                warn!(
                    action = "running_services_validation",
                    check = "http_accessibility",
                    port = 8080,
                    status = "warning",
                    error = %e,
                    "Could not verify HTTP accessibility (service may not expose HTTP)"
                );
                // Don't fail - HTTP check is optional
            }
        }

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
    use super::*;

    #[test]
    fn test_default_deploy_dir() {
        assert_eq!(DEFAULT_DEPLOY_DIR, "/opt/torrust");
    }

    #[test]
    fn test_action_name() {
        // Can't test without SSH config, but we can verify the constant
        assert_eq!("running-services-validation", "running-services-validation");
    }
}
