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
use crate::application::services::rendering::{
    AnsibleTemplateRenderingService, BackupTemplateRenderingService, CaddyTemplateRenderingService,
    DockerComposeTemplateRenderingService, GrafanaTemplateRenderingService,
    OpenTofuTemplateRenderingService, PrometheusTemplateRenderingService,
    TrackerTemplateRenderingService,
};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::{Created, Environment, EnvironmentParams};
use crate::domain::EnvironmentName;
use crate::shared::{Clock, SystemClock};

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
    /// * `output_dir` - Output directory for generated artifacts
    /// * `force` - Whether to overwrite existing output directory
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
    /// * Config file not found or invalid (env-file mode)
    /// * IP address parsing fails
    /// * Output directory exists and force is false
    /// * Template rendering fails
    #[instrument(
        name = "render_command",
        skip_all,
        fields(
            command_type = "render",
            input_mode = ?input_mode,
            target_ip = %target_ip,
            output_dir = %output_dir.display()
        )
    )]
    pub async fn execute(
        &self,
        input_mode: RenderInputMode,
        target_ip: &str,
        output_dir: &Path,
        force: bool,
        working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        // Parse and validate target IP
        let ip_addr = Self::parse_ip_address(target_ip)?;

        // Load configuration based on input mode
        match input_mode {
            RenderInputMode::EnvironmentName(ref env_name) => {
                // Validate output directory after environment check (fail-fast: file check before directory creation)
                Self::validate_output_directory(output_dir, force)?;

                self.render_from_environment(env_name, ip_addr, output_dir, working_dir)
                    .await
            }
            RenderInputMode::ConfigFile(ref config_path) => {
                // Validate output directory after config file check (fail-fast: file check before directory creation)
                Self::validate_output_directory(output_dir, force)?;

                self.render_from_config_file(config_path, ip_addr, output_dir, working_dir)
                    .await
            }
        }
    }

    /// Render artifacts from existing environment
    ///
    /// This mode loads an existing environment from the repository.
    ///
    /// # Arguments
    ///
    /// * `env_name` - Name of the environment to render from
    /// * `ip_addr` - Target instance IP address
    /// * `output_dir` - Output directory for generated artifacts
    /// * `working_dir` - Working directory for path resolution
    ///
    /// # Errors
    ///
    /// Returns error if environment not found or rendering fails
    async fn render_from_environment(
        &self,
        env_name: &EnvironmentName,
        ip_addr: IpAddr,
        output_dir: &Path,
        _working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        info!(
            environment = %env_name,
            target_ip = %ip_addr,
            output_dir = %output_dir.display(),
            "Rendering artifacts from existing environment"
        );

        // Load environment (untyped to check state)
        let environment = self.repository.load(env_name)?.ok_or_else(|| {
            RenderCommandHandlerError::EnvironmentNotFound {
                name: env_name.clone(),
            }
        })?;

        // Try to convert to Created state
        // Render command works for Created state (before provision)
        let current_state = environment.state_name().to_string();
        let created_env: Environment<Created> = environment.try_into_created().map_err(|_| {
            RenderCommandHandlerError::EnvironmentAlreadyProvisioned {
                name: env_name.clone(),
                current_state,
            }
        })?;

        // Render all templates
        self.render_all_templates(&created_env, ip_addr, output_dir)
            .await?;

        Ok(RenderResult {
            environment_name: created_env.name().to_string(),
            target_ip: ip_addr,
            output_dir: output_dir.to_path_buf(),
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
    /// * `output_dir` - Output directory for generated artifacts
    /// * `working_dir` - Working directory for path resolution
    ///
    /// # Errors
    ///
    /// Returns error if file not found, parsing fails, or rendering fails
    async fn render_from_config_file(
        &self,
        config_path: &Path,
        ip_addr: IpAddr,
        output_dir: &Path,
        working_dir: &Path,
    ) -> Result<RenderResult, RenderCommandHandlerError> {
        info!(
            config_file = %config_path.display(),
            target_ip = %ip_addr,
            output_dir = %output_dir.display(),
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

        // Validate configuration by converting to domain types (this moves config)
        let params: EnvironmentParams = config.try_into().map_err(|e: CreateConfigError| {
            RenderCommandHandlerError::DomainValidationFailed {
                reason: e.to_string(),
            }
        })?;

        // Create a temporary environment for template rendering (not persisted)
        let env_name = params.environment_name.clone();
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let created_env = Environment::<Created>::create(params, working_dir, clock.now())
            .map_err(|e| RenderCommandHandlerError::DomainValidationFailed {
                reason: e.to_string(),
            })?;

        // Render all templates
        self.render_all_templates(&created_env, ip_addr, output_dir)
            .await?;

        Ok(RenderResult {
            environment_name: env_name.to_string(),
            target_ip: ip_addr,
            output_dir: output_dir.to_path_buf(),
            config_source: format!("Config file: {}", config_path.display()),
        })
    }

    /// Render all deployment templates to the specified output directory
    ///
    /// This method orchestrates the rendering of all templates required for
    /// deployment: `OpenTofu`, Ansible, Docker Compose, Tracker, Prometheus,
    /// Grafana, Caddy, and Backup (conditional on configuration).
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Created state
    /// * `target_ip` - Target instance IP address
    /// * `output_dir` - Output directory for generated artifacts
    ///
    /// # Errors
    ///
    /// Returns error if any template rendering fails
    async fn render_all_templates(
        &self,
        environment: &Environment<Created>,
        target_ip: IpAddr,
        output_dir: &Path,
    ) -> Result<(), RenderCommandHandlerError> {
        info!(
            environment = %environment.name(),
            target_ip = %target_ip,
            output_dir = %output_dir.display(),
            "Rendering all deployment templates"
        );

        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let templates_dir = environment.templates_dir();
        let build_dir = output_dir.to_path_buf();
        let user_inputs = &environment.context().user_inputs;

        // 1. Render OpenTofu templates (infrastructure provisioning)
        OpenTofuTemplateRenderingService::from_params(
            templates_dir.clone(),
            build_dir.clone(),
            environment.ssh_credentials().clone(),
            environment.ssh_port(),
            environment.instance_name().clone(),
            environment.provider_config().clone(),
            clock.clone(),
        )
        .render()
        .await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 2. Render Ansible templates (configuration management)
        AnsibleTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render_templates(user_inputs, target_ip, None)
        .await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 3. Render Docker Compose templates (container orchestration)
        DockerComposeTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render(user_inputs, environment.admin_token())
        .await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 4. Render Tracker configuration templates
        TrackerTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render(user_inputs.tracker())
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 5. Render Prometheus configuration templates (if configured)
        PrometheusTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render(user_inputs.prometheus(), user_inputs.tracker())
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 6. Render Grafana provisioning templates (if configured)
        GrafanaTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render(user_inputs.grafana().is_some(), user_inputs.prometheus())
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 7. Render Caddy TLS proxy templates (if HTTPS configured)
        CaddyTemplateRenderingService::from_paths(
            templates_dir.clone(),
            build_dir.clone(),
            clock.clone(),
        )
        .render(user_inputs)
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        // 8. Render Backup configuration templates (if configured)
        BackupTemplateRenderingService::from_paths(templates_dir.clone(), build_dir.clone())
            .render(
                user_inputs.backup(),
                user_inputs.tracker().core().database(),
                environment.context().created_at(),
            )
            .await
            .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed {
                reason: e.to_string(),
            })?;

        info!(
            environment = %environment.name(),
            "All deployment templates rendered successfully"
        );

        Ok(())
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
    fn parse_ip_address(ip_str: &str) -> Result<IpAddr, RenderCommandHandlerError> {
        ip_str
            .parse::<IpAddr>()
            .map_err(|_| RenderCommandHandlerError::InvalidIpAddress {
                value: ip_str.to_string(),
            })
    }

    /// Validate output directory
    ///
    /// Checks if output directory exists and handles --force flag behavior.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Path to output directory
    /// * `force` - Whether to allow overwriting existing directory
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Directory exists and force is false
    /// - Directory creation fails
    fn validate_output_directory(
        output_dir: &Path,
        force: bool,
    ) -> Result<(), RenderCommandHandlerError> {
        if output_dir.exists() {
            if !force {
                return Err(RenderCommandHandlerError::OutputDirectoryExists {
                    path: output_dir.to_path_buf(),
                });
            }
            // With force flag, we allow overwriting
            info!(
                output_dir = %output_dir.display(),
                "Output directory exists, overwriting with --force"
            );
        } else {
            // Create output directory if it doesn't exist
            fs::create_dir_all(output_dir).map_err(|e| {
                RenderCommandHandlerError::OutputDirectoryCreationFailed {
                    path: output_dir.to_path_buf(),
                    reason: e.to_string(),
                }
            })?;
            info!(
                output_dir = %output_dir.display(),
                "Created output directory"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory;

    fn create_test_repository() -> Arc<dyn EnvironmentRepository> {
        let file_repository_factory =
            FileRepositoryFactory::new(std::time::Duration::from_secs(30));
        file_repository_factory.create(PathBuf::from("."))
    }

    #[test]
    fn it_should_create_handler() {
        let repository = create_test_repository();
        let _handler = RenderCommandHandler::new(repository);
    }

    #[test]
    fn it_should_parse_valid_ipv4_address() {
        let result = RenderCommandHandler::parse_ip_address("192.168.1.100");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    }

    #[test]
    fn it_should_parse_valid_ipv6_address() {
        let result = RenderCommandHandler::parse_ip_address("2001:db8::1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            IpAddr::V6("2001:db8::1".parse::<Ipv6Addr>().unwrap())
        );
    }

    #[test]
    fn it_should_reject_invalid_ip_address() {
        let result = RenderCommandHandler::parse_ip_address("not-an-ip");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::InvalidIpAddress { .. }
        ));
    }

    #[tokio::test]
    async fn it_should_return_error_for_nonexistent_environment() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        // Use a non-existent path for output directory (don't create it)
        let temp_dir = tempfile::tempdir().unwrap();
        let output_dir = temp_dir.path().join("nonexistent-output");

        let env_name = EnvironmentName::new("nonexistent").unwrap();
        let result = handler
            .execute(
                RenderInputMode::EnvironmentName(env_name.clone()),
                "10.0.0.1",
                output_dir.as_path(),
                false,
                &working_dir,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::EnvironmentNotFound { name } if name == env_name
        ));
    }

    #[tokio::test]
    async fn it_should_return_error_for_nonexistent_config_file() {
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        // Use a non-existent path for output directory (don't create it)
        let temp_dir = tempfile::tempdir().unwrap();
        let output_dir = temp_dir.path().join("test-output");

        let config_path = PathBuf::from("/tmp/nonexistent-config.json");
        let result = handler
            .execute(
                RenderInputMode::ConfigFile(config_path.clone()),
                "10.0.0.1",
                output_dir.as_path(),
                false,
                &working_dir,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::ConfigFileNotFound { path } if path == config_path
        ));
    }

    #[tokio::test]
    async fn it_should_validate_ip_before_loading_environment() {
        // This test ensures fail-fast behavior: IP validation happens first
        let repository = create_test_repository();
        let handler = RenderCommandHandler::new(repository);
        let working_dir = PathBuf::from(".");

        // Use a non-existent path for output directory (don't create it)
        let temp_dir = tempfile::tempdir().unwrap();
        let output_dir = temp_dir.path().join("test-output");

        let env_name = EnvironmentName::new("any-env").unwrap();

        // Even if environment exists, invalid IP should fail first
        let result = handler
            .execute(
                RenderInputMode::EnvironmentName(env_name),
                "invalid-ip",
                output_dir.as_path(),
                false,
                &working_dir,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RenderCommandHandlerError::InvalidIpAddress { .. }
        ));
    }
}
