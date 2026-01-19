//! Test command handler implementation
//!
//! **Purpose**: Smoke test for running Torrust Tracker services
//!
//! This handler validates that a deployed Tracker application is running and accessible
//! from external clients. The command performs comprehensive end-to-end verification
//! including service status, health checks, and external accessibility validation.
//!
//! ## Validation Strategy
//!
//! The test command validates deployed services through:
//!
//! 1. **External Health Checks** - Tests service accessibility from outside the VM:
//!    - Tracker API health endpoint (required)
//!    - HTTP Tracker health endpoint (required)
//!
//! ## HTTPS Support
//!
//! When services have TLS enabled via Caddy reverse proxy:
//! - Uses HTTPS URLs with the configured domain
//! - Resolves domains locally to the VM IP (no DNS dependency for testing)
//! - Accepts self-signed certificates for `.local` domains
//!
//! This approach allows testing to work without DNS configuration while still
//! being realistic (Caddy receives the correct SNI/Host header).
//!
//! ## Why External-Only Validation?
//!
//! We perform external accessibility checks (from test runner to VM) rather than
//! internal checks (via SSH to localhost) because:
//! - External checks are a superset of internal checks
//! - If services are accessible externally, they must be running internally
//! - External checks validate firewall configuration automatically
//! - Simpler test implementation reduces maintenance burden
//!
//! ## Port Configuration
//!
//! The test command extracts tracker ports from the environment's tracker configuration:
//! - HTTP API port from `environment.context.user_inputs.tracker.http_api.bind_address`
//! - HTTP Tracker port from `environment.context.user_inputs.tracker.http_trackers[0].bind_address`
//!
//! For rationale and alternatives, see:
//! - `docs/decisions/test-command-as-smoke-test.md` - Architectural decision record

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::TestCommandHandlerError;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::tracker::config::{HttpApiConfig, HttpTrackerConfig};
use crate::domain::EnvironmentName;
use crate::infrastructure::external_validators::{RunningServicesValidator, ServiceEndpoint};
use crate::infrastructure::remote_actions::RemoteAction;

/// `TestCommandHandler` orchestrates smoke testing for running Torrust Tracker services
///
/// **Purpose**: Post-deployment smoke test to verify the application is running and accessible
///
/// This handler validates that deployed services are operational and accessible from
/// external clients by performing comprehensive health checks on the Tracker API and
/// HTTP Tracker endpoints.
///
/// ## Validation Steps
///
/// 1. **Service Status** - Verifies Docker Compose services are running via SSH
/// 2. **Tracker API Health** (required) - Tests external accessibility of HTTP API
/// 3. **HTTP Tracker Health** (optional) - Tests external accessibility of HTTP tracker
///
/// ## Port Discovery
///
/// The handler extracts tracker ports from the environment's tracker configuration:
/// - HTTP API port from `tracker.http_api.bind_address`
/// - HTTP Tracker port from `tracker.http_trackers[0].bind_address`
///
/// ## Design Rationale
///
/// This command accepts an `EnvironmentName` in its `execute` method to align with other
/// command handlers (`ProvisionCommandHandler`, `ConfigureCommandHandler`). This design:
///
/// - Loads environment from repository (consistent pattern across all handlers)
/// - Allows testing environments regardless of compile-time state (runtime validation)
/// - Requires the environment to have an instance IP set (checked at runtime)
/// - Enables repository integration for future enhancements (e.g., tracking test history)
pub struct TestCommandHandler {
    repository: TypedEnvironmentRepository,
}

impl TestCommandHandler {
    /// Create a new `TestCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self {
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the complete testing and validation workflow
    ///
    /// Validates that the Torrust Tracker services are running and accessible by
    /// performing external health checks on the deployed services.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to test
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment does not have an instance IP set
    /// * Tracker configuration is invalid or missing required ports
    /// * Running services validation fails:
    ///   - Services are not running
    ///   - Health check endpoints are not accessible
    ///   - Firewall rules block external access
    #[instrument(
        name = "test_command",
        skip_all,
        fields(
            command_type = "test",
            environment = %env_name
        )
    )]
    pub async fn execute(&self, env_name: &EnvironmentName) -> Result<(), TestCommandHandlerError> {
        let any_env = self.load_environment(env_name)?;

        let instance_ip =
            any_env
                .instance_ip()
                .ok_or_else(|| TestCommandHandlerError::MissingInstanceIp {
                    environment_name: env_name.to_string(),
                })?;

        // Extract tracker config
        let tracker_config = any_env.tracker_config();

        // Build service endpoints from configuration (with server IP)
        let tracker_api_endpoint = Self::build_api_endpoint(instance_ip, &tracker_config.http_api);
        let http_tracker_endpoints: Vec<ServiceEndpoint> = tracker_config
            .http_trackers
            .iter()
            .map(|config| Self::build_http_tracker_endpoint(instance_ip, config))
            .collect();

        // Log endpoint information
        info!(
            command = "test",
            environment = %env_name,
            instance_ip = ?instance_ip,
            api_endpoint_tls = tracker_api_endpoint.uses_tls(),
            api_endpoint_domain = ?tracker_api_endpoint.domain(),
            http_tracker_count = http_tracker_endpoints.len(),
            "Starting service health checks"
        );

        // Validate running services with external accessibility checks
        let services_validator =
            RunningServicesValidator::new(tracker_api_endpoint, http_tracker_endpoints);

        services_validator.execute(&instance_ip).await?;

        info!(
            command = "test",
            environment = %env_name,
            instance_ip = ?instance_ip,
            "Service testing workflow completed successfully"
        );

        Ok(())
    }

    /// Build a `ServiceEndpoint` from the HTTP API configuration
    fn build_api_endpoint(server_ip: std::net::IpAddr, config: &HttpApiConfig) -> ServiceEndpoint {
        let port = config.bind_address.port();
        let path = "/api/health_check";
        let socket_addr = std::net::SocketAddr::new(server_ip, port);

        if let Some(domain) = config.tls_domain() {
            ServiceEndpoint::https(domain, path, server_ip)
                .expect("Valid TLS domain should produce valid HTTPS URL")
        } else {
            ServiceEndpoint::http(socket_addr, path)
                .expect("Valid socket address should produce valid HTTP URL")
        }
    }

    /// Build a `ServiceEndpoint` from the HTTP Tracker configuration
    fn build_http_tracker_endpoint(
        server_ip: std::net::IpAddr,
        config: &HttpTrackerConfig,
    ) -> ServiceEndpoint {
        let port = config.bind_address.port();
        let path = "/health_check";
        let socket_addr = std::net::SocketAddr::new(server_ip, port);

        if let Some(domain) = config.tls_domain() {
            ServiceEndpoint::https(domain, path, server_ip)
                .expect("Valid TLS domain should produce valid HTTPS URL")
        } else {
            ServiceEndpoint::http(socket_addr, path)
                .expect("Valid socket address should produce valid HTTP URL")
        }
    }

    /// Load environment from storage
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    fn load_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<crate::domain::environment::state::AnyEnvironmentState, TestCommandHandlerError>
    {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(TestCommandHandlerError::StatePersistence)?;

        any_env.ok_or_else(|| TestCommandHandlerError::EnvironmentNotFound {
            name: env_name.to_string(),
        })
    }
}
