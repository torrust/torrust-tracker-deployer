use std::sync::Arc;

use crate::ansible::AnsibleTemplateRenderer;
use crate::command_wrappers::ansible::AnsibleClient;
use crate::command_wrappers::lxd::LxdClient;
use crate::command_wrappers::opentofu::OpenTofuClient;
use crate::config::Config;
use crate::template::TemplateManager;
use crate::tofu::TofuTemplateRenderer;

/// Service clients and renderers for performing actions
pub struct Services {
    // Command wrappers
    pub opentofu_client: Arc<OpenTofuClient>,
    pub lxd_client: Arc<LxdClient>,
    pub ansible_client: Arc<AnsibleClient>,

    // Template related services
    pub template_manager: Arc<TemplateManager>,
    pub tofu_template_renderer: Arc<TofuTemplateRenderer>,
    pub ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
}

impl Services {
    /// Create a new services container using the provided configuration
    #[must_use]
    pub fn new(config: &Config) -> Self {
        // Create template manager
        let template_manager = TemplateManager::new(config.templates_dir.clone());
        let template_manager = Arc::new(template_manager);

        // Create OpenTofu client pointing to build/opentofu_subfolder directory
        let opentofu_client =
            OpenTofuClient::new(config.build_dir.join(&config.opentofu_subfolder));

        // Create LXD client for instance management
        let lxd_client = LxdClient::new();

        // Create Ansible client pointing to build/ansible_subfolder directory
        let ansible_client = AnsibleClient::new(config.build_dir.join(&config.ansible_subfolder));

        // Create provision template renderer
        let provision_renderer =
            TofuTemplateRenderer::new(template_manager.clone(), config.build_dir.clone());

        // Create configuration template renderer
        let configuration_renderer = AnsibleTemplateRenderer::new(config.build_dir.clone());

        Self {
            // Command wrappers
            opentofu_client: Arc::new(opentofu_client),
            lxd_client: Arc::new(lxd_client),
            ansible_client: Arc::new(ansible_client),

            // Template related services
            template_manager: template_manager.clone(),
            tofu_template_renderer: Arc::new(provision_renderer),
            ansible_template_renderer: Arc::new(configuration_renderer),
        }
    }
}
