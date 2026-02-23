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

use std::net::IpAddr;
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::TestCommandHandlerError;
use super::result::{DnsIssue, DnsWarning, TestResult};
use crate::application::command_handlers::common::endpoint_builder;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::AnyEnvironmentState;
use crate::domain::EnvironmentName;
use crate::infrastructure::dns::{DnsResolutionError, DnsResolver};
use crate::infrastructure::external_validators::RunningServicesValidator;
use crate::infrastructure::remote_actions::RemoteAction;
use crate::shared::domain_name::DomainName;

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
    /// performing external health checks on the deployed services. Also performs
    /// advisory DNS resolution checks for configured domains.
    ///
    /// Returns a structured `TestResult` containing any DNS warnings found.
    /// The presentation layer is responsible for rendering these warnings.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to test
    ///
    /// # Returns
    ///
    /// * `Ok(TestResult)` - Test passed, may contain advisory DNS warnings
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
    pub async fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<TestResult, TestCommandHandlerError> {
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
        let (tracker_api_endpoint, http_tracker_endpoints) =
            endpoint_builder::build_all_tracker_endpoints(instance_ip, tracker_config);

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

        // Perform advisory DNS checks
        let dns_warnings = Self::check_dns_resolution(&any_env, instance_ip);

        info!(
            command = "test",
            environment = %env_name,
            instance_ip = ?instance_ip,
            dns_warnings = dns_warnings.len(),
            "Service testing workflow completed successfully"
        );

        Ok(TestResult::with_dns_warnings(instance_ip, dns_warnings))
    }

    /// Perform advisory DNS checks for configured domains
    ///
    /// Checks DNS resolution for all configured service domains (API, HTTP
    /// trackers, health check API, Grafana) and returns structured warnings
    /// for any domains that don't resolve or resolve to unexpected IPs.
    ///
    /// **Advisory Only**: DNS check failures are returned as warnings and
    /// do NOT affect the test result. This is because:
    /// - DNS propagation can take time
    /// - Local `.local` domains use `/etc/hosts` which may not be configured
    /// - Users may intentionally test without DNS
    fn check_dns_resolution(any_env: &AnyEnvironmentState, instance_ip: IpAddr) -> Vec<DnsWarning> {
        let domains_to_check = any_env.collect_tls_domains();

        if domains_to_check.is_empty() {
            return Vec::new();
        }

        domains_to_check
            .iter()
            .filter_map(|domain| Self::check_single_domain(domain, instance_ip))
            .collect()
    }

    /// Check a single domain and return a warning if resolution fails or mismatches
    fn check_single_domain(domain: &DomainName, expected_ip: IpAddr) -> Option<DnsWarning> {
        let resolver = DnsResolver::new();

        match resolver.resolve_and_verify(domain, expected_ip) {
            Ok(()) => None,
            Err(DnsResolutionError::ResolutionFailed { source, .. }) => Some(DnsWarning {
                domain: domain.clone(),
                expected_ip,
                issue: DnsIssue::ResolutionFailed(source.to_string()),
            }),
            Err(DnsResolutionError::IpMismatch { resolved_ip, .. }) => Some(DnsWarning {
                domain: domain.clone(),
                expected_ip,
                issue: DnsIssue::IpMismatch {
                    resolved_ips: vec![resolved_ip],
                },
            }),
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
    ) -> Result<AnyEnvironmentState, TestCommandHandlerError> {
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
