//! Grafana smoke test validator for remote instances
//!
//! This module provides the `GrafanaValidator` which performs a smoke test
//! on a running Grafana instance to verify it's operational and accessible.
//!
//! ## Key Features
//!
//! - Validates Grafana web UI is accessible via HTTP
//! - Checks Grafana returns a successful HTTP response
//! - Optionally validates admin credentials work (login test)
//! - Performs validation from inside the VM (not externally exposed by firewall)
//!
//! ## Validation Approach
//!
//! Grafana is exposed on port 3100 via Docker, but validation is performed
//! from inside the VM via SSH for consistency with other service validators:
//!
//! 1. Connect to VM via SSH
//! 2. Execute `curl` command to fetch Grafana homepage
//! 3. Verify successful HTTP response (200 OK)
//!
//! This smoke test confirms Grafana is:
//! - Running and bound to the expected port (3000 internally, 3100 externally)
//! - Responding to HTTP requests
//! - Web UI is functional
//!
//! ## Port Mapping
//!
//! - Internal (container): 3000 (Grafana default)
//! - External (host): 3100 (docker-compose port mapping)
//! - Validation uses: 3100 (tests the published port from inside VM)
//!
//! ## Future Enhancements
//!
//! For more comprehensive validation, consider:
//!
//! 1. **Authentication Validation**:
//!    - Test admin login with configured credentials
//!    - Verify authentication works correctly
//!    - Example: `curl -u admin:password http://localhost:3100/api/health`
//!
//! 2. **Datasource Validation**:
//!    - Query Grafana API for configured datasources
//!    - Verify Prometheus datasource is configured
//!    - Check datasource connectivity to Prometheus
//!    - Example: `curl http://localhost:3100/api/datasources | jq`
//!
//! 3. **Dashboard Availability**:
//!    - Query for available dashboards
//!    - Verify default dashboards are loaded
//!    - Check dashboard functionality
//!
//! These enhancements require:
//! - JSON parsing of Grafana API responses
//! - Credential management for authentication tests
//! - More complex error handling
//!
//! The current smoke test provides a good baseline validation that can be
//! extended as needed.

use std::net::IpAddr;
use tracing::{info, instrument};

use crate::adapters::ssh::SshClient;
use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Default Grafana external port (exposed by docker-compose)
const DEFAULT_GRAFANA_PORT: u16 = 3100;

/// Action that validates Grafana is running and accessible
pub struct GrafanaValidator {
    ssh_client: SshClient,
    grafana_port: u16,
}

impl GrafanaValidator {
    /// Create a new `GrafanaValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    /// * `grafana_port` - Port where Grafana is accessible (defaults to 3100 if None)
    #[must_use]
    pub fn new(ssh_config: SshConfig, grafana_port: Option<u16>) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self {
            ssh_client,
            grafana_port: grafana_port.unwrap_or(DEFAULT_GRAFANA_PORT),
        }
    }
}

impl RemoteAction for GrafanaValidator {
    fn name(&self) -> &'static str {
        "grafana-smoke-test"
    }

    #[instrument(
        name = "grafana_smoke_test",
        skip(self),
        fields(
            action_type = "validation",
            component = "grafana",
            server_ip = %server_ip,
            grafana_port = self.grafana_port
        )
    )]
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "grafana_smoke_test",
            grafana_port = self.grafana_port,
            "Running Grafana smoke test"
        );

        // Perform smoke test: curl Grafana homepage and check for success
        // Using -f flag to make curl fail on HTTP errors (4xx, 5xx)
        // Using -s flag for silent mode (no progress bar)
        // Using -o /dev/null to discard response body (we only care about status code)
        let command = format!(
            "curl -f -s -o /dev/null http://localhost:{} && echo 'success'",
            self.grafana_port
        );

        let output = self.ssh_client.execute(&command).map_err(|source| {
            RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            }
        })?;

        if !output.trim().contains("success") {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "Grafana smoke test failed. Grafana may not be running or accessible on port {}. \
                     Check that 'docker compose ps' shows Grafana container as running.",
                    self.grafana_port
                ),
            });
        }

        info!(
            action = "grafana_smoke_test",
            status = "success",
            "Grafana is running and responding to HTTP requests"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod grafana_validator {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn it_should_have_correct_name() {
            use crate::adapters::ssh::SshCredentials;
            use crate::shared::Username;
            use std::net::SocketAddr;

            let credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                Username::new("test").unwrap(),
            );
            let ssh_config = SshConfig::new(credentials, SocketAddr::from(([127, 0, 0, 1], 22)));
            let validator = GrafanaValidator::new(ssh_config, None);

            assert_eq!(validator.name(), "grafana-smoke-test");
        }

        #[test]
        fn it_should_use_default_port_when_none_provided() {
            use crate::adapters::ssh::SshCredentials;
            use crate::shared::Username;
            use std::net::SocketAddr;

            let credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                Username::new("test").unwrap(),
            );
            let ssh_config = SshConfig::new(credentials, SocketAddr::from(([127, 0, 0, 1], 22)));
            let validator = GrafanaValidator::new(ssh_config, None);

            assert_eq!(validator.grafana_port, DEFAULT_GRAFANA_PORT);
        }

        #[test]
        fn it_should_use_custom_port_when_provided() {
            use crate::adapters::ssh::SshCredentials;
            use crate::shared::Username;
            use std::net::SocketAddr;

            let credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                Username::new("test").unwrap(),
            );
            let ssh_config = SshConfig::new(credentials, SocketAddr::from(([127, 0, 0, 1], 22)));
            let custom_port = 4000;
            let validator = GrafanaValidator::new(ssh_config, Some(custom_port));

            assert_eq!(validator.grafana_port, custom_port);
        }
    }
}
