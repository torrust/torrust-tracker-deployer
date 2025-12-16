//! Prometheus smoke test validator for remote instances
//!
//! This module provides the `PrometheusValidator` which performs a smoke test
//! on a running Prometheus instance to verify it's operational and accessible.
//!
//! ## Key Features
//!
//! - Validates Prometheus web UI is accessible via HTTP
//! - Checks Prometheus returns a successful HTTP response
//! - Performs validation from inside the VM (not exposed externally)
//!
//! ## Validation Approach
//!
//! Since Prometheus is not exposed outside the VM (protected by firewall),
//! validation must be performed from inside the VM via SSH:
//!
//! 1. Connect to VM via SSH
//! 2. Execute `curl` command to fetch Prometheus homepage
//! 3. Verify successful HTTP response (200 OK)
//!
//! This smoke test confirms Prometheus is:
//! - Running and bound to the expected port
//! - Responding to HTTP requests
//! - Web UI is functional
//!
//! ## Future Enhancements
//!
//! For more comprehensive validation, consider:
//!
//! 1. **Configuration Validation**:
//!    - Parse Prometheus config file to verify scrape targets
//!    - Check that tracker endpoints are configured correctly
//!    - Validate scrape interval matches environment config
//!
//! 2. **Data Collection Validation**:
//!    - Query Prometheus API for active targets
//!    - Verify tracker metrics are being collected
//!    - Check that scrape jobs are succeeding (not in "down" state)
//!    - Example: `curl http://localhost:9090/api/v1/targets | jq`
//!
//! 3. **Metric Availability**:
//!    - Query specific tracker metrics (e.g., `torrust_tracker_info`)
//!    - Verify metrics have recent timestamps
//!    - Example: `curl http://localhost:9090/api/v1/query?query=up`
//!
//! These enhancements require:
//! - JSON parsing of Prometheus API responses
//! - Async coordination (waiting for first scrape to complete)
//! - More complex error handling
//!
//! The current smoke test provides a good baseline validation that can be
//! extended as needed.

use std::net::IpAddr;
use tracing::{info, instrument};

use crate::adapters::ssh::SshClient;
use crate::adapters::ssh::SshConfig;
use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};

/// Default Prometheus port (not exposed outside VM)
const DEFAULT_PROMETHEUS_PORT: u16 = 9090;

/// Action that validates Prometheus is running and accessible
pub struct PrometheusValidator {
    ssh_client: SshClient,
    prometheus_port: u16,
}

impl PrometheusValidator {
    /// Create a new `PrometheusValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    /// * `prometheus_port` - Port where Prometheus is running (defaults to 9090 if None)
    #[must_use]
    pub fn new(ssh_config: SshConfig, prometheus_port: Option<u16>) -> Self {
        let ssh_client = SshClient::new(ssh_config);
        Self {
            ssh_client,
            prometheus_port: prometheus_port.unwrap_or(DEFAULT_PROMETHEUS_PORT),
        }
    }
}

impl RemoteAction for PrometheusValidator {
    fn name(&self) -> &'static str {
        "prometheus-smoke-test"
    }

    #[instrument(
        name = "prometheus_smoke_test",
        skip(self),
        fields(
            action_type = "validation",
            component = "prometheus",
            server_ip = %server_ip,
            prometheus_port = self.prometheus_port
        )
    )]
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "prometheus_smoke_test",
            prometheus_port = self.prometheus_port,
            "Running Prometheus smoke test"
        );

        // Perform smoke test: curl Prometheus homepage and check for success
        // Using -f flag to make curl fail on HTTP errors (4xx, 5xx)
        // Using -s flag for silent mode (no progress bar)
        // Using -o /dev/null to discard response body (we only care about status code)
        let command = format!(
            "curl -f -s -o /dev/null http://localhost:{} && echo 'success'",
            self.prometheus_port
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
                    "Prometheus smoke test failed. Prometheus may not be running or accessible on port {}. \
                     Check that 'docker compose ps' shows Prometheus container as running.",
                    self.prometheus_port
                ),
            });
        }

        info!(
            action = "prometheus_smoke_test",
            status = "success",
            "Prometheus is running and responding to HTTP requests"
        );

        Ok(())
    }
}
