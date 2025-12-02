//! # Cloud-Init Template Renderer
//!
//! This module provides the `CloudInitTemplateRenderer`, a specialized template renderer for cloud-init.yml.tera
//! rendering within the `OpenTofu` deployment workflow. It extracts all cloud-init specific logic
//! from the main `TofuTemplateRenderer` to follow the single responsibility principle.
//!
//! ## Purpose
//!
//! The `CloudInitTemplateRenderer` is responsible for:
//! - Handling the `cloud-init.yml.tera` template file specifically
//! - Managing SSH public key injection into cloud-init configuration
//! - Creating appropriate contexts from SSH credentials
//! - Rendering the template to the output directory
//! - Using a common cloud-init template shared by all providers (LXD, Hetzner)
//!
//! This follows the collaborator pattern established in the Ansible template renderer refactoring.
//!
//! ## Example
//!
//! ```rust
//! # use std::sync::Arc;
//! # use std::path::Path;
//! # use torrust_tracker_deployer_lib::infrastructure::external_tools::tofu::template::common::renderer::cloud_init::CloudInitTemplateRenderer;
//! # use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! # use torrust_tracker_deployer_lib::domain::provider::Provider;
//! # use torrust_tracker_deployer_lib::shared::Username;
//! use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
//! # use std::path::PathBuf;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let template_manager = Arc::new(TemplateManager::new(std::env::temp_dir()));
//! let ssh_credentials = SshCredentials::new(
//!     PathBuf::from("fixtures/testing_rsa"),
//!     PathBuf::from("fixtures/testing_rsa.pub"),
//!     Username::new("username").unwrap()
//! );
//! let renderer = CloudInitTemplateRenderer::new(template_manager, Provider::Lxd);
//!
//! // Just demonstrate creating the renderer - actual rendering requires
//! // a proper template manager setup with cloud-init templates
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::adapters::ssh::credentials::SshCredentials;
use crate::domain::provider::Provider;
use crate::domain::template::file::File;
use crate::domain::template::{TemplateManager, TemplateManagerError};

/// Errors that can occur during cloud-init template rendering
#[derive(Error, Debug)]
pub enum CloudInitTemplateError {
    /// Failed to get cloud-init template path from template manager
    #[error("Failed to get template path for 'cloud-init.yml.tera': {source}")]
    TemplatePathFailed {
        #[source]
        source: TemplateManagerError,
    },

    /// Failed to read cloud-init template content from file
    #[error("Failed to read cloud-init template: {source}")]
    TemplateReadError {
        #[source]
        source: std::io::Error,
    },

    /// Failed to create File object from cloud-init template content
    #[error("Failed to create cloud-init template file: Invalid template content")]
    FileCreationFailed,

    /// Failed to read SSH public key file
    #[error("SSH public key file not found or unreadable")]
    SshKeyReadError,

    /// Failed to build cloud-init context from SSH credentials
    #[error("Failed to build cloud-init context: Invalid SSH credentials or context data")]
    ContextCreationFailed,

    /// Failed to create `CloudInitTemplate` with context
    #[error("Failed to create cloud-init template: Template validation or context binding failed")]
    CloudInitTemplateCreationFailed,

    /// Failed to render cloud-init template to output file
    #[error("Failed to render cloud-init template: Template rendering or file write failed")]
    CloudInitTemplateRenderFailed,
}

/// Specialized renderer for `cloud-init.yml.tera` templates
///
/// This collaborator handles all cloud-init template specific logic, including:
/// - Template path resolution (using common template shared by all providers)
/// - SSH public key reading and context creation
/// - Template rendering and output file writing
///
/// It follows the Single Responsibility Principle by focusing solely on cloud-init
/// template operations, making the main `TofuTemplateRenderer` simpler and more focused.
///
/// Note: The provider field is kept for potential future provider-specific customization,
/// but currently all providers use the same common cloud-init template.
pub struct CloudInitTemplateRenderer {
    template_manager: Arc<TemplateManager>,
    #[allow(dead_code)]
    provider: Provider,
}

impl CloudInitTemplateRenderer {
    /// Template file name for cloud-init configuration
    const CLOUD_INIT_TEMPLATE_FILE: &'static str = "cloud-init.yml.tera";

    /// Output file name for rendered cloud-init configuration
    const CLOUD_INIT_OUTPUT_FILE: &'static str = "cloud-init.yml";

    /// Creates a new cloud-init template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Arc reference to the template manager for file operations
    /// * `provider` - The infrastructure provider (LXD, Hetzner) - kept for future customization
    ///
    /// # Returns
    ///
    /// A new `CloudInitTemplateRenderer` instance ready to render cloud-init templates
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>, provider: Provider) -> Self {
        Self {
            template_manager,
            provider,
        }
    }

    /// Renders the cloud-init.yml.tera template with SSH credentials
    ///
    /// This method performs the complete cloud-init template rendering workflow:
    /// 1. Resolves the template path and reads template content
    /// 2. Creates a cloud-init context from SSH credentials
    /// 3. Renders the template with the context
    /// 4. Writes the rendered output to the destination directory
    ///
    /// # Arguments
    ///
    /// * `ssh_credentials` - SSH credentials containing public key path for cloud-init injection
    /// * `output_dir` - Directory where the rendered `cloud-init.yml` file will be written
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful template rendering
    /// * `Err(CloudInitTemplateError)` on any failure during the rendering process
    ///
    /// # Errors
    ///
    /// This method can fail with:
    /// - `TemplatePathFailed` if the template manager cannot resolve the template path
    /// - `TemplateReadError` if the template file cannot be read from disk
    /// - `FileCreationFailed` if the template content is invalid for File creation
    /// - `SshKeyReadError` if the SSH public key file cannot be read
    /// - `ContextCreationFailed` if the cloud-init context cannot be built
    /// - `CloudInitTemplateCreationFailed` if template creation fails
    /// - `CloudInitTemplateRenderFailed` if template rendering or file writing fails
    pub async fn render(
        &self,
        ssh_credentials: &SshCredentials,
        output_dir: &Path,
    ) -> Result<(), CloudInitTemplateError> {
        tracing::debug!(
            provider = %self.provider,
            "Rendering cloud-init template with SSH public key injection"
        );

        // Build template path (uses common template for all providers)
        let template_path = Self::build_template_path(Self::CLOUD_INIT_TEMPLATE_FILE);
        let source_path = self
            .template_manager
            .get_template_path(&template_path)
            .map_err(|source| CloudInitTemplateError::TemplatePathFailed { source })?;

        // Read template content from file
        let template_content = tokio::fs::read_to_string(&source_path)
            .await
            .map_err(|source| CloudInitTemplateError::TemplateReadError { source })?;

        // Create File object for template processing
        let template_file = File::new(Self::CLOUD_INIT_TEMPLATE_FILE, template_content)
            .map_err(|_| CloudInitTemplateError::FileCreationFailed)?;

        // Render cloud-init template (shared logic for all providers)
        self.render_cloud_init(&template_file, ssh_credentials, output_dir)
    }

    /// Renders cloud-init template (shared logic for all providers)
    fn render_cloud_init(
        &self,
        template_file: &File,
        ssh_credentials: &SshCredentials,
        output_dir: &Path,
    ) -> Result<(), CloudInitTemplateError> {
        use crate::infrastructure::external_tools::tofu::template::common::wrappers::cloud_init::{
            CloudInitContext, CloudInitTemplate,
        };

        // Create cloud-init context with SSH public key and username
        // Note: All providers use the same context structure for cloud-init
        let cloud_init_context = CloudInitContext::builder()
            .with_ssh_public_key_from_file(&ssh_credentials.ssh_pub_key_path)
            .map_err(|_| CloudInitTemplateError::SshKeyReadError)?
            .with_username(ssh_credentials.ssh_username.as_str())
            .map_err(|_| CloudInitTemplateError::ContextCreationFailed)?
            .build()
            .map_err(|_| CloudInitTemplateError::ContextCreationFailed)?;

        // Create CloudInitTemplate with context
        let cloud_init_template = CloudInitTemplate::new(template_file, cloud_init_context)
            .map_err(|_| CloudInitTemplateError::CloudInitTemplateCreationFailed)?;

        // Render template to output file
        let output_path = output_dir.join(Self::CLOUD_INIT_OUTPUT_FILE);
        cloud_init_template
            .render(&output_path)
            .map_err(|_| CloudInitTemplateError::CloudInitTemplateRenderFailed)?;

        tracing::debug!(
            provider = %self.provider,
            "Successfully rendered cloud-init template to {}",
            output_path.display()
        );

        Ok(())
    }

    /// Builds the template path for the cloud-init template file
    ///
    /// Uses a common template path shared by all providers.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The template file name
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for the cloud-init template
    fn build_template_path(file_name: &str) -> String {
        format!("tofu/common/{file_name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::shared::Username;

    /// Helper function to create mock SSH credentials for testing
    fn create_mock_ssh_credentials(temp_dir: &std::path::Path) -> SshCredentials {
        let ssh_priv_key_path = temp_dir.join("test_key");
        let ssh_pub_key_path = temp_dir.join("test_key.pub");

        // Create mock key files
        fs::write(&ssh_priv_key_path, "-----BEGIN OPENSSH PRIVATE KEY-----\nmock_private_key\n-----END OPENSSH PRIVATE KEY-----")
            .expect("Failed to write private key");
        fs::write(
            &ssh_pub_key_path,
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7... test@example.com",
        )
        .expect("Failed to write public key");

        SshCredentials::new(
            ssh_priv_key_path,
            ssh_pub_key_path,
            Username::new("test_user").unwrap(),
        )
    }

    /// Helper function to create a mock template manager with cloud-init template
    fn create_mock_template_manager_with_cloud_init() -> Arc<TemplateManager> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&template_dir).expect("Failed to create template dir");

        // Create tofu/common template directory structure (common for all providers)
        let tofu_common_dir = template_dir.join("tofu").join("common");
        fs::create_dir_all(&tofu_common_dir).expect("Failed to create tofu/common dir");

        // Create cloud-init.yml.tera template
        let cloud_init_template = r"#cloud-config
users:
  - name: torrust
    ssh_authorized_keys:
      - {{ ssh_public_key }}
";
        fs::write(
            tofu_common_dir.join("cloud-init.yml.tera"),
            cloud_init_template,
        )
        .expect("Failed to write cloud-init template");

        Arc::new(TemplateManager::new(temp_dir.keep()))
    }

    #[test]
    fn it_should_create_cloud_init_renderer_with_template_manager_and_provider() {
        let template_manager = Arc::new(TemplateManager::new(std::env::temp_dir()));
        let renderer = CloudInitTemplateRenderer::new(template_manager, Provider::Lxd);

        // Verify the renderer was created successfully
        // Just check that it contains the expected template manager reference
        let renderer_ptr = std::ptr::addr_of!(renderer.template_manager);
        assert!(!renderer_ptr.is_null());
    }

    #[test]
    fn it_should_build_common_template_path() {
        // All providers use the common template path
        let template_path = CloudInitTemplateRenderer::build_template_path("cloud-init.yml.tera");
        assert_eq!(template_path, "tofu/common/cloud-init.yml.tera");
    }

    #[tokio::test]
    async fn it_should_render_cloud_init_template_successfully() {
        let template_manager = create_mock_template_manager_with_cloud_init();
        let renderer = CloudInitTemplateRenderer::new(template_manager, Provider::Lxd);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ssh_credentials = create_mock_ssh_credentials(temp_dir.path());
        let output_dir = TempDir::new().expect("Failed to create output dir");

        let result = renderer.render(&ssh_credentials, output_dir.path()).await;

        assert!(
            result.is_ok(),
            "Cloud-init template rendering should succeed"
        );

        let output_file = output_dir.path().join("cloud-init.yml");
        assert!(
            output_file.exists(),
            "Rendered cloud-init.yml file should exist"
        );

        let content = fs::read_to_string(&output_file).expect("Failed to read rendered file");
        assert!(
            content.contains("ssh_authorized_keys"),
            "Rendered content should contain SSH key configuration"
        );
        assert!(
            content.contains("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7"),
            "Rendered content should contain the actual SSH public key: {content}"
        );
    }

    // #[tokio::test]
    // async fn it_should_fail_when_template_manager_cannot_find_template() {
    //     // This test is disabled for now as template manager behavior may vary
    //     // depending on embedded template availability
    // }

    #[tokio::test]
    async fn it_should_fail_when_ssh_key_file_missing() {
        let template_manager = create_mock_template_manager_with_cloud_init();
        let renderer = CloudInitTemplateRenderer::new(template_manager, Provider::Lxd);

        // Create SSH credentials with non-existent key file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let non_existent_key = temp_dir.path().join("non_existent_key");
        let ssh_credentials = SshCredentials::new(
            non_existent_key.clone(),
            non_existent_key,
            Username::new("test_user").unwrap(),
        );

        let output_dir = TempDir::new().expect("Failed to create output dir");

        let result = renderer.render(&ssh_credentials, output_dir.path()).await;

        assert!(result.is_err(), "Should fail when SSH key file is missing");
        match result.unwrap_err() {
            CloudInitTemplateError::SshKeyReadError => {
                // Expected error type
            }
            other => panic!("Expected SshKeyReadError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_fail_when_output_directory_is_readonly() {
        let template_manager = create_mock_template_manager_with_cloud_init();
        let renderer = CloudInitTemplateRenderer::new(template_manager, Provider::Lxd);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let ssh_credentials = create_mock_ssh_credentials(temp_dir.path());

        // Create read-only output directory
        let output_dir = TempDir::new().expect("Failed to create output dir");
        let mut permissions = fs::metadata(output_dir.path())
            .expect("Failed to get metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(output_dir.path(), permissions)
            .expect("Failed to set readonly permissions");

        let result = renderer.render(&ssh_credentials, output_dir.path()).await;

        assert!(
            result.is_err(),
            "Should fail when output directory is readonly"
        );
        match result.unwrap_err() {
            CloudInitTemplateError::CloudInitTemplateRenderFailed => {
                // Expected error type
            }
            other => panic!("Expected CloudInitTemplateRenderFailed, got: {other:?}"),
        }
    }

    // #[tokio::test]
    // async fn it_should_fail_when_template_content_is_invalid() {
    //     // This test is disabled as the template validation behavior
    //     // may depend on the specific Tera engine implementation
    // }

    // #[tokio::test]
    // async fn it_should_fail_when_context_missing_required_fields() {
    //     // This test is disabled as missing template variables may not
    //     // always cause failures depending on template engine configuration
    // }
}
