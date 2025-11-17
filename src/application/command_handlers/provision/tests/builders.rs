//! Test builders for Provision Command
//!
//! This module provides test builders that simplify test setup by managing
//! dependencies and lifecycle for `ProvisionCommandHandler` tests.

use std::sync::Arc;

use tempfile::TempDir;

use crate::adapters::ssh::SshCredentials;
use crate::application::command_handlers::provision::ProvisionCommandHandler;
use crate::domain::{InstanceName, ProfileName};
use crate::infrastructure::external_tools::ansible::AnsibleTemplateRenderer;
use crate::infrastructure::external_tools::tofu::TofuTemplateRenderer;
use crate::shared::Username;

/// Test builder for `ProvisionCommandHandler` that manages dependencies and lifecycle
///
/// This builder simplifies test setup by:
/// - Managing `TempDir` lifecycle
/// - Providing sensible defaults for all dependencies
/// - Allowing selective customization of dependencies
/// - Returning only the command handler and necessary test artifacts
pub struct ProvisionCommandHandlerTestBuilder {
    #[allow(dead_code)]
    temp_dir: TempDir,
    ssh_credentials: Option<SshCredentials>,
}

impl ProvisionCommandHandlerTestBuilder {
    /// Create a new test builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        Self {
            temp_dir,
            ssh_credentials: None,
        }
    }

    /// Customize SSH credentials (optional - uses defaults if not called)
    #[allow(dead_code)]
    pub fn with_ssh_credentials(mut self, credentials: SshCredentials) -> Self {
        self.ssh_credentials = Some(credentials);
        self
    }

    /// Build the `ProvisionCommandHandler` with all dependencies
    ///
    /// Returns: (`command_handler`, `temp_dir`, `ssh_credentials`)
    /// The `temp_dir` must be kept alive for the duration of the test.
    #[allow(dead_code)]
    pub fn build(self) -> (ProvisionCommandHandler, TempDir, SshCredentials) {
        let template_manager = Arc::new(crate::domain::template::TemplateManager::new(
            self.temp_dir.path(),
        ));

        // Use provided SSH credentials or create defaults
        let ssh_credentials = self.ssh_credentials.unwrap_or_else(|| {
            let ssh_key_path = self.temp_dir.path().join("test_key");
            let ssh_pub_key_path = self.temp_dir.path().join("test_key.pub");
            SshCredentials::new(
                ssh_key_path,
                ssh_pub_key_path,
                Username::new("test_user").unwrap(),
            )
        });

        let tofu_renderer = Arc::new(TofuTemplateRenderer::new(
            template_manager.clone(),
            self.temp_dir.path(),
            ssh_credentials.clone(),
            InstanceName::new("torrust-tracker-vm".to_string())
                .expect("Valid hardcoded instance name"),
            ProfileName::new("default-profile".to_string()).expect("Valid hardcoded profile name"),
        ));

        let ansible_renderer = Arc::new(AnsibleTemplateRenderer::new(
            self.temp_dir.path(),
            template_manager,
        ));

        let opentofu_client = Arc::new(crate::adapters::tofu::client::OpenTofuClient::new(
            self.temp_dir.path(),
        ));

        let clock: Arc<dyn crate::shared::Clock> = Arc::new(crate::shared::SystemClock);

        let repository_factory =
            crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                std::time::Duration::from_secs(30),
            );
        let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

        let command_handler = ProvisionCommandHandler::new(
            tofu_renderer,
            ansible_renderer,
            opentofu_client,
            clock,
            repository,
        );

        (command_handler, self.temp_dir, ssh_credentials)
    }
}

impl Default for ProvisionCommandHandlerTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
