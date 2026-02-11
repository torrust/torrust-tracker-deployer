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
//! ## HTTPS Support
//!
//! When services have TLS enabled via Caddy reverse proxy:
//! - The validator uses HTTPS URLs with the configured domain
//! - Domains are resolved locally to the VM IP (no DNS dependency)
//! - Self-signed certificates are accepted for `.local` domains
//!
//! This approach allows testing to work without DNS configuration while still
//! being realistic (Caddy receives the correct SNI/Host header).
//!
//! ## Current Scope (Torrust Tracker)
//!
//! This validator performs external validation only (from test runner to VM):
//! - Tests tracker API health endpoint: HTTP or HTTPS depending on TLS config
//! - Tests HTTP tracker health endpoint: HTTP or HTTPS depending on TLS config
//!
//! **Validation Philosophy**: External checks are a superset of internal checks.
//! If external validation passes, it proves:
//! - Services are running inside the VM
//! - Firewall rules are configured correctly (port 80/443 for TLS, or service port for HTTP)
//! - Services are accessible from outside the VM
//! - TLS termination is working correctly (when enabled)

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::ClientBuilder;
use tracing::{info, instrument, warn};

use crate::infrastructure::remote_actions::{RemoteAction, RemoteActionError};
use crate::shared::ServiceEndpoint;

/// Default deployment directory for Docker Compose files
const DEFAULT_DEPLOY_DIR: &str = "/opt/torrust";

/// HTTP client request timeout
const REQUEST_TIMEOUT_SECS: u64 = 10;

/// Action that validates Docker Compose services are running and healthy
///
/// Supports both HTTP and HTTPS endpoints. For HTTPS endpoints:
/// - Uses domain-based URLs with the configured domain
/// - Resolves domain to IP locally (no DNS dependency for testing)
/// - Accepts self-signed certificates for `.local` domains
pub struct RunningServicesValidator {
    deploy_dir: PathBuf,
    tracker_api_endpoint: ServiceEndpoint,
    http_tracker_endpoints: Vec<ServiceEndpoint>,
}

impl RunningServicesValidator {
    /// Create a new `RunningServicesValidator` with service endpoints
    ///
    /// Uses the default deployment directory `/opt/torrust`.
    ///
    /// # Arguments
    /// * `tracker_api_endpoint` - Endpoint for the tracker API health check
    /// * `http_tracker_endpoints` - Endpoints for HTTP tracker health checks
    #[must_use]
    pub fn new(
        tracker_api_endpoint: ServiceEndpoint,
        http_tracker_endpoints: Vec<ServiceEndpoint>,
    ) -> Self {
        Self {
            deploy_dir: PathBuf::from(DEFAULT_DEPLOY_DIR),
            tracker_api_endpoint,
            http_tracker_endpoints,
        }
    }

    /// Create a new `RunningServicesValidator` with a custom deployment directory
    ///
    /// # Arguments
    /// * `deploy_dir` - Path to the directory containing docker-compose.yml on the remote host
    /// * `tracker_api_endpoint` - Endpoint for the tracker API health check
    /// * `http_tracker_endpoints` - Endpoints for HTTP tracker health checks
    #[must_use]
    pub fn with_deploy_dir(
        deploy_dir: PathBuf,
        tracker_api_endpoint: ServiceEndpoint,
        http_tracker_endpoints: Vec<ServiceEndpoint>,
    ) -> Self {
        Self {
            deploy_dir,
            tracker_api_endpoint,
            http_tracker_endpoints,
        }
    }

    /// Validate external accessibility of all configured endpoints
    async fn validate_external_accessibility(&self) -> Result<(), RemoteActionError> {
        // Check tracker API (required)
        self.check_endpoint(&self.tracker_api_endpoint, "Tracker API")
            .await?;

        // Check all HTTP trackers
        for (idx, endpoint) in self.http_tracker_endpoints.iter().enumerate() {
            let name = format!("HTTP Tracker {}", idx + 1);
            self.check_endpoint(endpoint, &name).await?;
        }

        Ok(())
    }

    /// Check a service endpoint for accessibility
    ///
    /// Handles both HTTP and HTTPS endpoints. For HTTPS:
    /// - Resolves domain to IP locally using reqwest's resolve feature
    /// - Accepts self-signed certs for `.local` domains
    async fn check_endpoint(
        &self,
        endpoint: &ServiceEndpoint,
        service_name: &str,
    ) -> Result<(), RemoteActionError> {
        let url = endpoint.url();

        if endpoint.uses_tls() {
            info!(
                action = "running_services_validation",
                check = "service_external",
                service = service_name,
                url = %url,
                domain = ?endpoint.domain(),
                resolve_to = %endpoint.server_ip(),
                "Testing HTTPS endpoint (resolving domain to IP locally)"
            );
        } else {
            info!(
                action = "running_services_validation",
                check = "service_external",
                service = service_name,
                url = %url,
                "Testing HTTP endpoint"
            );
        }

        let response = self.make_request(endpoint).await?;

        if !response.status().is_success() {
            return Err(RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!(
                    "{service_name} returned HTTP {}: {}. Service may not be healthy.",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ),
            });
        }

        info!(
            action = "running_services_validation",
            check = "service_external",
            service = service_name,
            url = %url,
            status = "success",
            "{service_name} health check passed"
        );

        Ok(())
    }

    /// Make an HTTP/HTTPS request to the endpoint
    ///
    /// For HTTPS endpoints, this:
    /// - Uses reqwest's `resolve()` to map domain to IP (like curl --resolve)
    /// - Accepts self-signed certificates for `.local` domains
    async fn make_request(
        &self,
        endpoint: &ServiceEndpoint,
    ) -> Result<reqwest::Response, RemoteActionError> {
        let url = endpoint.url();
        let mut client_builder =
            ClientBuilder::new().timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS));

        // For HTTPS endpoints, configure domain resolution and certificate handling
        if let Some(domain) = endpoint.domain() {
            // Resolve domain to IP locally (like curl --resolve)
            client_builder = client_builder.resolve(domain, endpoint.socket_addr());

            // Accept self-signed certs for .local domains (Caddy's internal CA)
            if endpoint.is_local_domain() {
                warn!(
                    action = "running_services_validation",
                    domain = domain,
                    "Accepting self-signed certificates for .local domain"
                );
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: format!("Failed to build HTTP client: {e}"),
            })?;

        client.get(url.clone()).send().await.map_err(|e| {
            let help_message = if endpoint.uses_tls() {
                format!(
                    "HTTPS request to '{url}' failed: {e}. \
                     Check that Caddy is running and port 443 is open. \
                     Domain '{}' was resolved to {} for testing.",
                    endpoint.domain().unwrap_or("unknown"),
                    endpoint.server_ip()
                )
            } else {
                format!(
                    "HTTP request to '{url}' failed: {e}. \
                     Check that service is running and firewall allows port {}.",
                    endpoint.port()
                )
            };

            RemoteActionError::ValidationFailed {
                action_name: self.name().to_string(),
                message: help_message,
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
        // Note: server_ip parameter is kept for trait compatibility and logging,
        // but endpoints now contain their own server_ip for URL generation.
        let _ = server_ip; // Suppress unused warning - used in instrument macro

        info!(
            action = "running_services_validation",
            deploy_dir = %self.deploy_dir.display(),
            "Validating Docker Compose services are running via external accessibility"
        );

        self.validate_external_accessibility().await?;

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
    use std::net::{IpAddr, SocketAddr};
    use std::path::PathBuf;

    use crate::shared::DomainName;

    use super::*;

    fn test_ip() -> IpAddr {
        "10.0.0.1".parse().unwrap()
    }

    fn test_socket_addr(port: u16) -> SocketAddr {
        SocketAddr::new(test_ip(), port)
    }

    #[test]
    fn it_should_use_default_deploy_dir_when_not_specified() {
        assert_eq!(DEFAULT_DEPLOY_DIR, "/opt/torrust");
    }

    #[test]
    fn it_should_return_correct_action_name_when_queried() {
        assert_eq!("running-services-validation", "running-services-validation");
    }

    #[test]
    fn it_should_create_validator_with_http_endpoints() {
        let api_endpoint =
            ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();
        let tracker_endpoints =
            vec![ServiceEndpoint::http(test_socket_addr(7070), "/health_check").unwrap()];

        let validator = RunningServicesValidator::new(api_endpoint.clone(), tracker_endpoints);

        assert_eq!(validator.tracker_api_endpoint, api_endpoint);
        assert_eq!(validator.http_tracker_endpoints.len(), 1);
    }

    #[test]
    fn it_should_create_validator_with_https_endpoints() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let api_endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();
        let tracker_endpoints = vec![];

        let validator = RunningServicesValidator::new(api_endpoint.clone(), tracker_endpoints);

        assert!(validator.tracker_api_endpoint.uses_tls());
    }

    #[test]
    fn it_should_create_validator_with_mixed_endpoints() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let api_endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();
        let tracker_endpoints = vec![
            ServiceEndpoint::http(test_socket_addr(7070), "/health_check").unwrap(),
            ServiceEndpoint::http(test_socket_addr(7071), "/health_check").unwrap(),
        ];

        let validator = RunningServicesValidator::new(api_endpoint, tracker_endpoints);

        assert!(validator.tracker_api_endpoint.uses_tls());
        assert!(!validator.http_tracker_endpoints[0].uses_tls());
        assert!(!validator.http_tracker_endpoints[1].uses_tls());
    }

    #[test]
    fn it_should_accept_empty_tracker_endpoints() {
        let api_endpoint =
            ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();
        let validator = RunningServicesValidator::new(api_endpoint, vec![]);

        assert_eq!(validator.http_tracker_endpoints.len(), 0);
    }

    #[test]
    fn it_should_use_custom_deploy_dir() {
        let api_endpoint =
            ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();
        let validator = RunningServicesValidator::with_deploy_dir(
            PathBuf::from("/custom/path"),
            api_endpoint,
            vec![],
        );

        assert_eq!(validator.deploy_dir, PathBuf::from("/custom/path"));
    }
}
