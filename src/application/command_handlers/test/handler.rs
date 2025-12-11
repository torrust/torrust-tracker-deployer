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
//! 1. **Docker Compose Service Status** - Verifies containers are running
//! 2. **External Health Checks** - Tests service accessibility from outside the VM:
//!    - Tracker API health endpoint (required): `http://<vm-ip>:<api-port>/api/health_check`
//!    - HTTP Tracker health endpoint (optional): `http://<vm-ip>:<tracker-port>/api/health_check`
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

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::TestCommandHandlerError;
use crate::adapters::ssh::SshConfig;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::EnvironmentName;
use crate::infrastructure::remote_actions::{RemoteAction, RunningServicesValidator};

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

        // Extract tracker ports from configuration
        let tracker_config = any_env.tracker_config();

        // Get HTTP API port from bind_address (e.g., "0.0.0.0:1212" -> 1212)
        let tracker_api_port = Some(Self::extract_port_from_bind_address(
            &tracker_config.http_api.bind_address,
        ))
        .ok_or_else(|| TestCommandHandlerError::InvalidTrackerConfiguration {
            message: format!(
                "Invalid HTTP API bind_address: {}. Expected format: 'host:port'",
                tracker_config.http_api.bind_address
            ),
        })?;

        // Get all HTTP Tracker ports
        let http_tracker_ports: Vec<u16> = tracker_config
            .http_trackers
            .iter()
            .map(|tracker| Self::extract_port_from_bind_address(&tracker.bind_address))
            .collect();

        let ssh_config =
            SshConfig::with_default_port(any_env.ssh_credentials().clone(), instance_ip);

        // Validate running services with external accessibility checks
        let services_validator =
            RunningServicesValidator::new(ssh_config, tracker_api_port, http_tracker_ports.clone());

        services_validator.execute(&instance_ip).await?;

        info!(
            command = "test",
            environment = %env_name,
            instance_ip = ?instance_ip,
            tracker_api_port = tracker_api_port,
            http_tracker_ports = ?http_tracker_ports,
            "Service testing workflow completed successfully"
        );

        Ok(())
    }

    /// Extract port number from `SocketAddr` (e.g., `"0.0.0.0:1212".parse()` returns 1212)
    fn extract_port_from_bind_address(bind_address: &SocketAddr) -> u16 {
        bind_address.port()
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
