//! Render command handler implementation

use std::convert::TryInto;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::RenderCommandHandlerError;
use crate::application::command_handlers::create::config::{
    CreateConfigError, EnvironmentCreationConfig,
};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::{Created, Environment, EnvironmentParams};
use crate::domain::EnvironmentName;

/// Input mode for render command
///
/// The render command supports two mutually exclusive input modes:
/// - From an existing environment (by name)
/// - From a configuration file (without creating environment)
#[derive(Debug, Clone)]
pub enum RenderInputMode {
    /// Load from existing environment in repository
    EnvironmentName(EnvironmentName),
    /// Load from configuration file
    ConfigFile(PathBuf),
}

/// Result of artifact generation
///
/// Contains paths and metadata about generated artifacts
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Name of the environment (from env or config)
    pub environment_name: String,
    /// IP address used in artifact generation
    pub target_ip: IpAddr,
    /// Path to generated artifacts
    pub output_dir: PathBuf,
    /// Source of configuration
    pub config_source: String,
}

/// `RenderCommandHandler` generates deployment artifacts without deployment
///
/// This command handler provides a way to preview or generate deployment
/// artifacts (docker-compose files, Ansible playbooks, tracker config, etc.)
/// without executing any infrastructure operations.
///
/// # State Management
///
/// - **Created State Only**: Command works for environments in "Created" state
/// - **Already Provisioned**: Returns informational result (not error) with artifacts location
/// - **No State Modification**: Does not change environment state or execute deployments
///
/// # Dual Input Modes
///
/// 1. **Environment Name Mode**: Loads existing environment from repository
/// 2. **Config File Mode**: Parses configuration file directly (no env creation)
///
/// # Workflow
///
/// 1. Determine input mode (env-name or env-file)
/// 2. Load/parse configuration
/// 3. Validate state (Created only for existing environments)
/// 4. Parse target IP address
/// 5. Render all deployment templates to build/{env}/ directory
pub struct RenderCommandHandler {
    repository: Arc<dyn EnvironmentRepository>,
}

impl RenderCommandHandler {
    /// Create a new `RenderCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    /// Execute the render workflow
    ///
    /// # Arguments
    ///
    /// * `input_mode` - Source of configuration (env-name or env-file)
    /// * `target_ip` - Target instance IP address (always required)
    /// * `working_dir` - Working directory for resolving relative paths
    ///
    /// # Returns
    ///
    /// Returns `RenderResult` with paths to generated artifacts
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found (env-name mode)
    /// * Environment already provisioned (returns informational error)
    /// * Config file not found or invalid (env-file mode)
    /// * IP address parsing fails
    /// * Template rendering fails
    #[instrument(
        name = "render_command",
        skip_all,
        fields(
            command_type = "render",
            input_mode = ?input_mode,
            target_ip = %target_ip
        )
    )]
    pub fn execute(
        &self,
        input_mode: RenderInputMode,
        target_ip: &str,
        working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        // Parse and validate target IP
        let ip_addr = self.parse_ip_address(target_ip)?;

        // Load configuration based on input mode
        match input_mode {
            RenderInputMode::EnvironmentName(ref env_name) => {
                self.render_from_environment(env_name, ip_addr, working_dir)
            }
            RenderInputMode::ConfigFile(ref config_path) => {
                self.render_from_config_file(config_path, ip_addr, working_dir)
            }
        }
    }

    /// Render artifacts from existing environment
    ///
    /// This mode loads an existing environment from the repository and validates
    /// that it's in the Created state (not yet provisioned).
    ///
    /// # Arguments
    ///
    /// * `env_name` - Name of the environment to render from
    /// * `ip_addr` - Target instance IP address
    /// * `working_dir` - Working directory for path resolution
    ///
    /// # Errors
    ///
    /// Returns error if environment not found, wrong state, or rendering fails
    fn render_from_environment(
        &self,
        env_name: &EnvironmentName,
        ip_addr: IpAddr,
        working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        info!(
            environment = %env_name,
            target_ip = %ip_addr,
            "Rendering artifacts from existing environment"
        );

        // Load environment (untyped to check state)
        let environment = self.repository.load(env_name)?.ok_or_else(|| {
            RenderCommandHandlerError::EnvironmentNotFound {
                name: env_name.clone(),
            }
        })?;

        // Try to convert to Created state
        // If it fails, environment is in a different state (already provisioned)
        let current_state = environment.state_name().to_string();
        let created_env: Environment<Created> = environment.try_into_created().map_err(|_| {
            let artifacts_path = working_dir.join("build").join(env_name.as_str());
            RenderCommandHandlerError::EnvironmentAlreadyProvisioned {
                name: env_name.clone(),
                current_state,
                artifacts_path,
            }
        })?;

        // TODO: Phase 3 - Render templates
        // This will be implemented in the next phase with template orchestration
        info!(
            environment = %env_name,
            target_ip = %ip_addr,
            "Template rendering not yet implemented (Phase 3)"
        );

        let output_dir = working_dir.join("build").join(env_name.as_str());

        Ok(RenderResult {
            environment_name: created_env.name().to_string(),
            target_ip: ip_addr,
            output_dir,
            config_source: format!("Environment: {}", created_env.name()),
        })
    }

    /// Render artifacts from configuration file
    ///
    /// This mode parses a configuration file directly without creating or
    /// loading an environment from the repository.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    /// * `ip_addr` - Target instance IP address
    /// * `working_dir` - Working directory for path resolution
    ///
    /// # Errors
    ///
    /// Returns error if file not found, parsing fails, or rendering fails
    fn render_from_config_file(
        &self,
        config_path: &Path,
        ip_addr: IpAddr,
        working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        info!(
            config_file = %config_path.display(),
            target_ip = %ip_addr,
            "Rendering artifacts from configuration file"
        );

        // Read configuration file
        let content = fs::read_to_string(config_path).map_err(|_| {
            RenderCommandHandlerError::ConfigFileNotFound {
                path: config_path.to_path_buf(),
            }
        })?;

        // Parse JSON to EnvironmentCreationConfig
        let config: EnvironmentCreationConfig =
            serde_json::from_str(&content).map_err(|source| {
                RenderCommandHandlerError::ConfigParsingFailed {
                    path: config_path.to_path_buf(),
                    source,
                }
            })?;

        // Validate configuration by converting to domain types
        let _params: EnvironmentParams =
            config.clone().try_into().map_err(|e: CreateConfigError| {
                RenderCommandHandlerError::DomainValidationFailed {
                    reason: e.to_string(),
                }
            })?;

        // TODO: Phase 3 - Render templates
        // This will be implemented in the next phase with template orchestration
        info!(
            config_file = %config_path.display(),
            target_ip = %ip_addr,
            "Template rendering not yet implemented (Phase 3)"
        );

        let env_name = &config.environment.name;
        let output_dir = working_dir.join("build").join(env_name);

        Ok(RenderResult {
            environment_name: env_name.clone(),
            target_ip: ip_addr,
            output_dir,
            config_source: format!("Config file: {}", config_path.display()),
        })
    }

    /// Parse and validate IP address
    ///
    /// # Arguments
    ///
    /// * `ip_str` - IP address string to parse
    ///
    /// # Errors
    ///
    /// Returns error if IP address format is invalid
    fn parse_ip_address(&self, ip_str: &str) -> Result<IpAddr, RenderCommandHandlerError> {
        ip_str
            .parse::<IpAddr>()
            .map_err(|_| RenderCommandHandlerError::InvalidIpAddress {
                value: ip_str.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;

    fn create_test_repository() -> Arc<dyn EnvironmentRepository> {
        let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
        repository_factory.create(PathBuf::from("."))
    }

    #[test]
    fn it_should_create_handler() {
        let repository = create_test_repository();
        let _handler = RenderCommandHandler::new(repository);
    }

    #[test]
    fn it_should_parse_valid_ipv4_address() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);

        let result = handler.parse_ip_address("192.168.1.100");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    }

    #[test]
    fn it_should_parse_valid_ipv6_address() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);

        let result = handler.parse_ip_address("2001:db8::1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            IpAddr::V6("2001:db8::1".parse::<Ipv6Addr>().unwrap())
        );
    }

    #[test]
    fn it_should_reject_invalid_ip_address() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);

        let result = handler.parse_ip_address("not-an-ip");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::InvalidIpAddress { .. }
        ));
    }

    #[test]
    fn it_should_return_error_for_nonexistent_environment() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        let env_name = EnvironmentName::new("nonexistent").unwrap();
        let result = handler.execute(
            RenderInputMode::EnvironmentName(env_name.clone()),
            "10.0.0.1",
            &working_dir,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::EnvironmentNotFound { name } if name == env_name
        ));
    }

    #[test]
    fn it_should_return_error_for_nonexistent_config_file() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        let config_path = PathBuf::from("/tmp/nonexistent-config.json");
        let result = handler.execute(
            RenderInputMode::ConfigFile(config_path.clone()),
            "10.0.0.1",
            &working_dir,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::ConfigFileNotFound { path } if path == config_path
        ));
    }

    #[test]
    fn it_should_validate_ip_before_loading_environment() {
        // This test ensures fail-fast behavior: IP validation happens first
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        let env_name = EnvironmentName::new("any-env").unwrap();

        // Even if environment exists, invalid IP should fail first
        let result = handler.execute(
            RenderInputMode::EnvironmentName(env_name),
            "invalid-ip",
            &working_dir,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::InvalidIpAddress { .. }
        ));
    }

    #[test]
    fn it_should_accept_both_input_modes() {
        // Verify both input modes can be constructed
        let env_name = EnvironmentName::new("test").unwrap();
        let _mode1 = RenderInputMode::EnvironmentName(env_name);

        let config_path = PathBuf::from("test.json");
        let _mode2 = RenderInputMode::ConfigFile(config_path);
    }
}
