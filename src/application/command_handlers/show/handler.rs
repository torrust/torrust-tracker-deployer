//! Show command handler implementation
//!
//! **Purpose**: Display environment information and status
//!
//! This handler retrieves and displays information about an environment
//! from storage. It is a read-only operation that does not modify any state
//! or make any network calls.
//!
//! ## Display Strategy
//!
//! The show command displays state-aware information:
//!
//! 1. **Basic Info (all states)**: Environment name, state, provider
//! 2. **Infrastructure (Provisioned+)**: IP, SSH port, SSH user, SSH key path
//! 3. **Next Step**: Guidance based on current state
//!
//! ## Design Rationale
//!
//! This command accepts an `EnvironmentName` in its `execute` method to align with other
//! command handlers (`ProvisionCommandHandler`, `ConfigureCommandHandler`). This design:
//!
//! - Loads environment from repository (consistent pattern across all handlers)
//! - Allows showing environments regardless of compile-time state (runtime extraction)
//! - Read-only operation - no state modifications

use std::sync::Arc;

use tracing::instrument;

use super::errors::ShowCommandHandlerError;
use super::info::{EnvironmentInfo, GrafanaInfo, InfrastructureInfo, PrometheusInfo, ServiceInfo};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::domain::EnvironmentName;

/// Default SSH port when not specified
const DEFAULT_SSH_PORT: u16 = 22;

/// `ShowCommandHandler` extracts and formats environment information for display
///
/// **Purpose**: Read-only information extraction from environment state
///
/// This handler loads an environment from storage and extracts information
/// relevant to the environment's current state. It never modifies state
/// or makes network calls.
///
/// ## Information Extraction
///
/// - **All states**: Name, state name, provider
/// - **Provisioned+**: Infrastructure details (IP, SSH credentials)
/// - **All states**: Next step guidance
pub struct ShowCommandHandler {
    repository: Arc<dyn EnvironmentRepository>,
}

impl ShowCommandHandler {
    /// Create a new `ShowCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    /// Execute the show command workflow
    ///
    /// Loads the environment and extracts state-aware information for display.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to show
    ///
    /// # Returns
    ///
    /// * `Ok(EnvironmentInfo)` - Information about the environment
    /// * `Err(ShowCommandHandlerError)` - If the environment cannot be loaded
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment state file is corrupted or unreadable
    #[instrument(
        name = "show_command",
        skip_all,
        fields(
            command_type = "show",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<EnvironmentInfo, ShowCommandHandlerError> {
        let any_env = self.load_environment(env_name)?;

        Ok(Self::extract_info(&any_env))
    }

    /// Load environment from repository
    fn load_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<AnyEnvironmentState, ShowCommandHandlerError> {
        if !self.repository.exists(env_name)? {
            return Err(ShowCommandHandlerError::EnvironmentNotFound {
                name: env_name.to_string(),
            });
        }

        self.repository.load(env_name)?.ok_or_else(|| {
            ShowCommandHandlerError::EnvironmentNotFound {
                name: env_name.to_string(),
            }
        })
    }

    /// Extract information from environment based on its state
    fn extract_info(any_env: &AnyEnvironmentState) -> EnvironmentInfo {
        let name = any_env.name().to_string();
        let state = any_env.state_display_name().to_string();
        let provider = any_env.provider_display_name().to_string();
        let created_at = any_env.created_at();
        let state_name = any_env.state_name().to_string();

        let mut info = EnvironmentInfo::new(name, state, provider, created_at, state_name);

        // Add infrastructure info if instance IP is available
        if let Some(instance_ip) = any_env.instance_ip() {
            let ssh_creds = any_env.ssh_credentials();
            let ssh_port = any_env.ssh_port();

            let infra = InfrastructureInfo::new(
                instance_ip,
                if ssh_port == 0 {
                    DEFAULT_SSH_PORT
                } else {
                    ssh_port
                },
                ssh_creds.ssh_username.to_string(),
                ssh_creds.ssh_priv_key_path.to_string_lossy().to_string(),
            );
            info = info.with_infrastructure(infra);

            // Add service info for Released/Running states
            if Self::should_show_services(any_env.state_name()) {
                // Always compute from tracker config to show proper service information
                // including TLS domains, localhost hints, and HTTPS status
                let tracker_config = any_env.tracker_config();
                let grafana_config = any_env.grafana_config();
                let services =
                    ServiceInfo::from_tracker_config(tracker_config, instance_ip, grafana_config);
                info = info.with_services(services);

                // Add Prometheus info if configured
                if any_env.prometheus_config().is_some() {
                    info = info.with_prometheus(PrometheusInfo::default_internal());
                }

                // Add Grafana info if configured
                if let Some(grafana) = any_env.grafana_config() {
                    info = info.with_grafana(GrafanaInfo::from_config(grafana, instance_ip));
                }
            }
        }

        info
    }

    /// Determine if services should be shown based on state
    ///
    /// Services are shown for states where the tracker configuration has been
    /// deployed and services may be running (Released, Running, or related failed states).
    fn should_show_services(state_name: &str) -> bool {
        matches!(
            state_name,
            "released" | "running" | "release_failed" | "run_failed"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod should_show_services {
        use super::*;

        #[test]
        fn it_should_show_services_for_released_state() {
            assert!(ShowCommandHandler::should_show_services("released"));
        }

        #[test]
        fn it_should_show_services_for_running_state() {
            assert!(ShowCommandHandler::should_show_services("running"));
        }

        #[test]
        fn it_should_show_services_for_release_failed_state() {
            assert!(ShowCommandHandler::should_show_services("release_failed"));
        }

        #[test]
        fn it_should_show_services_for_run_failed_state() {
            assert!(ShowCommandHandler::should_show_services("run_failed"));
        }

        #[test]
        fn it_should_not_show_services_for_created_state() {
            assert!(!ShowCommandHandler::should_show_services("created"));
        }

        #[test]
        fn it_should_not_show_services_for_provisioned_state() {
            assert!(!ShowCommandHandler::should_show_services("provisioned"));
        }

        #[test]
        fn it_should_not_show_services_for_configured_state() {
            assert!(!ShowCommandHandler::should_show_services("configured"));
        }
    }
}
