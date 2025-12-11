//! Dependency injection container for testing services
//!
//! This module provides the `Services` struct that acts as a dependency injection container,
//! holding all the service clients and template renderers needed for E2E testing operations.
//! It centralizes service construction and makes them easily accessible throughout tests.
//!
//! ## Services Included
//!
//! - **Command clients**: `OpenTofu`, LXD, Ansible clients for external tool interaction
//! - **Template services**: Template manager and specialized renderers for different tools
//! - **Configuration**: Centralized configuration management
//!
//! ## Usage in Tests
//!
//! This container is primarily used in E2E tests to create all necessary service dependencies
//! in a consistent way. In production, services are created on-demand depending on which
//! command the user is executing.

use std::sync::Arc;
use std::time::Duration;

use crate::adapters::ansible::AnsibleClient;
use crate::adapters::lxd::LxdClient;
use crate::adapters::ssh::SshCredentials;
use crate::adapters::tofu::OpenTofuClient;
use crate::config::Config;
use crate::domain::provider::ProviderConfig;
use crate::domain::template::TemplateManager;
use crate::domain::InstanceName;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::infrastructure::templating::ansible::AnsibleProjectGenerator;
use crate::infrastructure::templating::ansible::ANSIBLE_SUBFOLDER;
use crate::infrastructure::templating::tofu::TofuProjectGenerator;
use crate::shared::Clock;
use crate::testing::e2e::LXD_OPENTOFU_SUBFOLDER;

/// Default lock timeout for repository operations
///
/// This timeout controls how long repository operations will wait to acquire
/// file locks before giving up. This prevents operations from hanging indefinitely
/// if another process has locked the state file.
///
/// TODO: Make this configurable via Config in the future
const REPOSITORY_LOCK_TIMEOUT_SECS: u64 = 30;

/// Service clients and renderers for performing actions in tests
pub struct Services {
    // Command wrappers
    pub opentofu_client: Arc<OpenTofuClient>,
    pub lxd_client: Arc<LxdClient>,
    pub ansible_client: Arc<AnsibleClient>,

    // Template related services
    pub template_manager: Arc<TemplateManager>,
    pub tofu_template_renderer: Arc<TofuProjectGenerator>,
    pub ansible_project_generator: Arc<AnsibleProjectGenerator>,

    // Infrastructure services
    /// Clock service for testable time management
    pub clock: Arc<dyn Clock>,

    // Persistence layer
    /// Factory for creating environment-specific repositories
    pub repository_factory: Arc<RepositoryFactory>,
}

impl Services {
    /// Create a new services container using the provided configuration
    #[must_use]
    pub fn new(
        config: &Config,
        ssh_credentials: SshCredentials,
        instance_name: InstanceName,
        provider_config: ProviderConfig,
    ) -> Self {
        // Create template manager
        let template_manager = TemplateManager::new(config.templates_dir.clone());
        let template_manager = Arc::new(template_manager);

        // Create OpenTofu client pointing to build/opentofu_subfolder directory
        let opentofu_client = OpenTofuClient::new(config.build_dir.join(LXD_OPENTOFU_SUBFOLDER));

        // Create LXD client for instance management
        let lxd_client = LxdClient::new();

        // Create Ansible client pointing to build/ansible_subfolder directory
        let ansible_client = AnsibleClient::new(config.build_dir.join(ANSIBLE_SUBFOLDER));

        // Create provision template renderer
        let tofu_template_renderer = TofuProjectGenerator::new(
            template_manager.clone(),
            config.build_dir.clone(),
            ssh_credentials,
            22, // Default SSH port for tests
            instance_name,
            provider_config,
        );

        // Create configuration template renderer
        let ansible_project_generator =
            AnsibleProjectGenerator::new(config.build_dir.clone(), template_manager.clone());

        // Create repository factory
        let repository_factory =
            RepositoryFactory::new(Duration::from_secs(REPOSITORY_LOCK_TIMEOUT_SECS));

        // Create clock service (production implementation uses system time)
        let clock: Arc<dyn Clock> = Arc::new(crate::shared::SystemClock);

        Self {
            // Command wrappers
            opentofu_client: Arc::new(opentofu_client),
            lxd_client: Arc::new(lxd_client),
            ansible_client: Arc::new(ansible_client),

            // Template related services
            template_manager: template_manager.clone(),
            tofu_template_renderer: Arc::new(tofu_template_renderer),
            ansible_project_generator: Arc::new(ansible_project_generator),

            // Infrastructure services
            clock,

            // Persistence layer
            repository_factory: Arc::new(repository_factory),
        }
    }
}
