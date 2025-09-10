use crate::ansible::AnsibleTemplateRenderer;
use crate::command_wrappers::ansible::AnsibleClient;
use crate::command_wrappers::lxd::LxdClient;
use crate::command_wrappers::opentofu::OpenTofuClient;
use crate::command_wrappers::ssh::SshClient;
use crate::config::Config;
use crate::template::TemplateManager;
use crate::tofu::TofuTemplateRenderer;

/// Service clients and renderers for performing actions
pub struct Services {
    // Command wrappers
    pub opentofu_client: OpenTofuClient,
    pub ssh_client: SshClient,
    pub lxd_client: LxdClient,
    pub ansible_client: AnsibleClient,

    // Template related services
    pub template_manager: TemplateManager,
    pub tofu_template_renderer: TofuTemplateRenderer,
    pub ansible_template_renderer: AnsibleTemplateRenderer,
}

impl Services {
    /// Create a new services container using the provided configuration
    #[must_use]
    pub fn new(config: &Config) -> Self {
        // Create template manager
        let template_manager = TemplateManager::new(config.templates_dir.clone());

        // Create SSH client with the configured key and username
        let ssh_client = SshClient::new(&config.ssh_key_path, &config.ssh_username, config.verbose);

        // Create OpenTofu client pointing to build/opentofu_subfolder directory
        let opentofu_client = OpenTofuClient::new(
            config.build_dir.join(&config.opentofu_subfolder),
            config.verbose,
        );

        // Create LXD client for instance management
        let lxd_client = LxdClient::new(config.verbose);

        // Create Ansible client pointing to build/ansible_subfolder directory
        let ansible_client = AnsibleClient::new(
            config.build_dir.join(&config.ansible_subfolder),
            config.verbose,
        );

        // Create provision template renderer
        let provision_renderer =
            TofuTemplateRenderer::new(config.build_dir.clone(), config.verbose);

        // Create configuration template renderer
        let configuration_renderer =
            AnsibleTemplateRenderer::new(config.build_dir.clone(), config.verbose);

        Self {
            // Command wrappers
            opentofu_client,
            ssh_client,
            lxd_client,
            ansible_client,

            // Template related services
            template_manager,
            tofu_template_renderer: provision_renderer,
            ansible_template_renderer: configuration_renderer,
        }
    }
}
