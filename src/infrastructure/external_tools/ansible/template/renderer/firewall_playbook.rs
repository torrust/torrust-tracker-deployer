//! # Firewall Playbook Template Renderer
//!
//! This module handles rendering of the `configure-firewall.yml.tera` template
//! with SSH port configuration. It's responsible for creating the Ansible playbook
//! that configures UFW firewall while preserving SSH access.
//!
//! ## Responsibilities
//!
//! - Load the `configure-firewall.yml.tera` template file
//! - Process template with SSH port configuration
//! - Render final `configure-firewall.yml` file for Ansible consumption
//!
//! ## Usage
//!
//! ```rust
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::renderer::firewall_playbook::FirewallPlaybookTemplateRenderer;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::wrappers::firewall_playbook::FirewallPlaybookContext;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::wrappers::inventory::context::AnsiblePort;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = FirewallPlaybookTemplateRenderer::new(template_manager);
//!
//! let ssh_port = AnsiblePort::new(22)?;
//! let firewall_context = FirewallPlaybookContext::new(ssh_port)?;
//! renderer.render(&firewall_context, temp_dir.path())?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::external_tools::ansible::template::wrappers::firewall_playbook::{
    FirewallPlaybookContext, FirewallPlaybookTemplate,
};

/// Errors that can occur during firewall playbook template rendering
#[derive(Error, Debug)]
pub enum FirewallPlaybookTemplateError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for '{file_name}': {source}")]
    TemplatePathFailed {
        file_name: String,
        #[source]
        source: TemplateManagerError,
    },

    /// Failed to read Tera template file content
    #[error("Failed to read Tera template file '{file_name}': {source}")]
    TeraTemplateReadFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create File object from template content
    #[error("Failed to create File object for '{file_name}': {source}")]
    FileCreationFailed {
        file_name: String,
        #[source]
        source: crate::domain::template::file::Error,
    },

    /// Failed to create firewall playbook template with provided context
    #[error("Failed to create FirewallPlaybookTemplate: {source}")]
    FirewallPlaybookTemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    /// Failed to render firewall playbook template to output file
    #[error("Failed to render firewall playbook template to file: {source}")]
    FirewallPlaybookTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Handles rendering of the configure-firewall.yml.tera template for Ansible deployments
///
/// This collaborator is responsible for all firewall playbook template-specific operations:
/// - Loading the configure-firewall.yml.tera template
/// - Processing it with SSH port configuration
/// - Rendering the final configure-firewall.yml file for Ansible consumption
pub struct FirewallPlaybookTemplateRenderer {
    template_manager: Arc<TemplateManager>,
}

impl FirewallPlaybookTemplateRenderer {
    /// Template filename for the firewall playbook Tera template
    const FIREWALL_TEMPLATE_FILE: &'static str = "configure-firewall.yml.tera";

    /// Output filename for the rendered firewall playbook file
    const FIREWALL_OUTPUT_FILE: &'static str = "configure-firewall.yml";

    /// Creates a new firewall playbook template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the configure-firewall.yml.tera template with the provided context
    ///
    /// This method:
    /// 1. Loads the configure-firewall.yml.tera template from the template manager
    /// 2. Reads the template content
    /// 3. Creates a File object for template processing
    /// 4. Creates a `FirewallPlaybookTemplate` with the SSH port context
    /// 5. Renders the template to configure-firewall.yml in the output directory
    ///
    /// # Arguments
    ///
    /// * `firewall_context` - The context containing SSH port configuration
    /// * `output_dir` - The directory where configure-firewall.yml should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), FirewallPlaybookTemplateError>` - Success or error from the template rendering operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be found or read
    /// - Template content is invalid
    /// - Variable substitution fails
    /// - Output file cannot be written
    pub fn render(
        &self,
        firewall_context: &FirewallPlaybookContext,
        output_dir: &Path,
    ) -> Result<(), FirewallPlaybookTemplateError> {
        tracing::debug!("Rendering firewall playbook template with SSH port configuration");

        // Get the firewall playbook template path
        let firewall_template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| FirewallPlaybookTemplateError::TemplatePathFailed {
                file_name: Self::FIREWALL_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let firewall_template_content =
            std::fs::read_to_string(&firewall_template_path).map_err(|source| {
                FirewallPlaybookTemplateError::TeraTemplateReadFailed {
                    file_name: Self::FIREWALL_TEMPLATE_FILE.to_string(),
                    source,
                }
            })?;

        // Create File object for template processing
        let firewall_template_file =
            File::new(Self::FIREWALL_TEMPLATE_FILE, firewall_template_content).map_err(
                |source| FirewallPlaybookTemplateError::FileCreationFailed {
                    file_name: Self::FIREWALL_TEMPLATE_FILE.to_string(),
                    source,
                },
            )?;

        // Create FirewallPlaybookTemplate with SSH port context
        let firewall_template =
            FirewallPlaybookTemplate::new(&firewall_template_file, firewall_context.clone())
                .map_err(|source| {
                    FirewallPlaybookTemplateError::FirewallPlaybookTemplateCreationFailed { source }
                })?;

        // Render to output file
        let firewall_output_path = output_dir.join(Self::FIREWALL_OUTPUT_FILE);
        firewall_template
            .render(&firewall_output_path)
            .map_err(|source| {
                FirewallPlaybookTemplateError::FirewallPlaybookTemplateRenderFailed { source }
            })?;

        tracing::debug!(
            "Successfully rendered firewall playbook template to {}",
            firewall_output_path.display()
        );

        Ok(())
    }

    /// Builds the full template path for the firewall playbook template
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for configure-firewall.yml.tera
    fn build_template_path() -> String {
        format!("ansible/{}", Self::FIREWALL_TEMPLATE_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::context::AnsiblePort;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test firewall context
    fn create_test_firewall_context() -> FirewallPlaybookContext {
        let ssh_port = AnsiblePort::new(22).expect("Failed to create SSH port");
        FirewallPlaybookContext::builder()
            .with_ssh_port(ssh_port)
            .build()
            .expect("Failed to build firewall context")
    }

    /// Helper function to create a test template directory with configure-firewall.yml.tera
    fn create_test_templates(temp_dir: &Path) -> std::io::Result<()> {
        let ansible_dir = temp_dir.join("ansible");
        fs::create_dir_all(&ansible_dir)?;

        let template_content = r#"---
- name: Configure UFW firewall
  hosts: all
  become: yes
  tasks:
    - name: Allow SSH on port {{ssh_port}}
      community.general.ufw:
        rule: allow
        port: "{{ssh_port}}"
        proto: tcp
"#;

        fs::write(
            ansible_dir.join("configure-firewall.yml.tera"),
            template_content,
        )?;

        Ok(())
    }

    #[test]
    fn it_should_create_firewall_renderer_with_template_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let renderer = FirewallPlaybookTemplateRenderer::new(template_manager.clone());

        assert!(Arc::ptr_eq(&renderer.template_manager, &template_manager));
    }

    #[test]
    fn it_should_build_correct_template_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let _renderer = FirewallPlaybookTemplateRenderer::new(template_manager);

        let template_path = FirewallPlaybookTemplateRenderer::build_template_path();

        assert_eq!(template_path, "ansible/configure-firewall.yml.tera");
    }

    #[test]
    fn it_should_render_firewall_template_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        // Create template directory and files
        create_test_templates(&template_dir).expect("Failed to create test templates");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Setup template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = FirewallPlaybookTemplateRenderer::new(template_manager);
        let firewall_context = create_test_firewall_context();

        // Render template
        let result = renderer.render(&firewall_context, &output_dir);

        assert!(result.is_ok(), "Template rendering should succeed");

        // Verify output file exists
        let output_file = output_dir.join("configure-firewall.yml");
        assert!(
            output_file.exists(),
            "configure-firewall.yml should be created"
        );

        // Verify output content contains expected values
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(
            output_content.contains("22"),
            "Output should contain the SSH port"
        );
        assert!(
            output_content.contains("hosts: all"),
            "Output should contain hosts: all"
        );
        assert!(
            !output_content.contains("{{ssh_port}}"),
            "Output should not contain template variables"
        );
    }

    #[test]
    fn it_should_render_with_custom_ssh_port() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        create_test_templates(&template_dir).expect("Failed to create test templates");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = FirewallPlaybookTemplateRenderer::new(template_manager);

        // Use custom SSH port
        let ssh_port = AnsiblePort::new(2222).expect("Failed to create SSH port");
        let firewall_context =
            FirewallPlaybookContext::new(ssh_port).expect("Failed to create context");

        let result = renderer.render(&firewall_context, &output_dir);

        assert!(result.is_ok());

        let output_file = output_dir.join("configure-firewall.yml");
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(
            output_content.contains("2222"),
            "Output should contain custom SSH port 2222"
        );
    }
}
