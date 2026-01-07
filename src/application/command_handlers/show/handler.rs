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
use super::info::{EnvironmentInfo, InfrastructureInfo, ServiceInfo};
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
        let state = Self::format_state_name(any_env.state_name());
        let provider = Self::format_provider_name(any_env.provider_name());
        let next_step = Self::get_next_step_guidance(any_env.state_name());

        let mut info = EnvironmentInfo::new(name, state, provider, next_step);

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
                let tracker_config = any_env.tracker_config();
                let services = ServiceInfo::from_tracker_config(tracker_config, instance_ip);
                info = info.with_services(services);
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

    /// Format state name for display
    fn format_state_name(state_name: &str) -> String {
        // Convert snake_case to Title Case
        state_name
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format provider name for display
    fn format_provider_name(provider_name: &str) -> String {
        match provider_name {
            "lxd" => "LXD".to_string(),
            "hetzner" => "Hetzner Cloud".to_string(),
            other => Self::format_state_name(other),
        }
    }

    /// Get next step guidance based on current state
    fn get_next_step_guidance(state_name: &str) -> String {
        match state_name {
            "created" => "Run 'provision' to create infrastructure.".to_string(),
            "provisioning" => {
                "Provisioning in progress. Wait for completion or check logs.".to_string()
            }
            "provisioned" => "Run 'configure' to set up the system.".to_string(),
            "configuring" => {
                "Configuration in progress. Wait for completion or check logs.".to_string()
            }
            "configured" => "Run 'release' to deploy the tracker software.".to_string(),
            "releasing" => "Release in progress. Wait for completion or check logs.".to_string(),
            "released" => "Run 'run' to start the tracker services.".to_string(),
            "running" => "Services are running. Use 'test' to verify health.".to_string(),
            "destroying" => "Destruction in progress. Wait for completion.".to_string(),
            "destroyed" => {
                "Environment has been destroyed. Create a new environment to redeploy.".to_string()
            }
            "provision_failed" => {
                "Provisioning failed. Check error details and retry 'provision'.".to_string()
            }
            "configure_failed" => {
                "Configuration failed. Check error details and retry 'configure'.".to_string()
            }
            "release_failed" => {
                "Release failed. Check error details and retry 'release'.".to_string()
            }
            "run_failed" => "Run failed. Check error details and retry 'run'.".to_string(),
            "destroy_failed" => {
                "Destruction failed. Check error details and retry 'destroy'.".to_string()
            }
            _ => format!("Unknown state: {state_name}. Check environment state file."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format_state_name {
        use super::*;

        #[test]
        fn it_should_format_simple_state() {
            assert_eq!(ShowCommandHandler::format_state_name("created"), "Created");
            assert_eq!(ShowCommandHandler::format_state_name("running"), "Running");
        }

        #[test]
        fn it_should_format_compound_state() {
            assert_eq!(
                ShowCommandHandler::format_state_name("provision_failed"),
                "Provision Failed"
            );
            assert_eq!(
                ShowCommandHandler::format_state_name("configure_failed"),
                "Configure Failed"
            );
        }
    }

    mod format_provider_name {
        use super::*;

        #[test]
        fn it_should_format_lxd() {
            assert_eq!(ShowCommandHandler::format_provider_name("lxd"), "LXD");
        }

        #[test]
        fn it_should_format_hetzner() {
            assert_eq!(
                ShowCommandHandler::format_provider_name("hetzner"),
                "Hetzner Cloud"
            );
        }

        #[test]
        fn it_should_format_unknown_provider() {
            assert_eq!(ShowCommandHandler::format_provider_name("aws"), "Aws");
        }
    }

    mod get_next_step_guidance {
        use super::*;

        #[test]
        fn it_should_guide_from_created_state() {
            let guidance = ShowCommandHandler::get_next_step_guidance("created");
            assert!(guidance.contains("provision"));
        }

        #[test]
        fn it_should_guide_from_provisioned_state() {
            let guidance = ShowCommandHandler::get_next_step_guidance("provisioned");
            assert!(guidance.contains("configure"));
        }

        #[test]
        fn it_should_guide_from_running_state() {
            let guidance = ShowCommandHandler::get_next_step_guidance("running");
            assert!(guidance.contains("test"));
        }

        #[test]
        fn it_should_handle_failed_states() {
            let guidance = ShowCommandHandler::get_next_step_guidance("provision_failed");
            assert!(guidance.contains("failed"));
            assert!(guidance.contains("retry"));
        }
    }

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
