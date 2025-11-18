//! # Inventory Template Renderer
//!
//! This module handles rendering of the `inventory.yml.tera` template with runtime variables.
//! It's responsible for creating Ansible inventory files from dynamic host information.
//!
//! ## Responsibilities
//!
//! - Load the `inventory.yml.tera` template file
//! - Process template with runtime context (hosts, SSH keys, etc.)
//! - Render final `inventory.yml` file for Ansible consumption
//!
//! ## Usage
//!
//! ```rust
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::renderer::inventory::InventoryTemplateRenderer;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::wrappers::inventory::InventoryContext;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = InventoryTemplateRenderer::new(template_manager);
//!
//! let inventory_context = InventoryContext::builder().build()?;
//! renderer.render(&inventory_context, temp_dir.path())?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::{
    InventoryContext, InventoryTemplate,
};

/// Errors that can occur during inventory template rendering
#[derive(Error, Debug)]
pub enum InventoryTemplateError {
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

    /// Failed to create inventory template with provided context
    #[error("Failed to create InventoryTemplate: {source}")]
    InventoryTemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    /// Failed to render inventory template to output file
    #[error("Failed to render inventory template to file: {source}")]
    InventoryTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Handles rendering of the inventory.yml.tera template for Ansible deployments
///
/// This collaborator is responsible for all inventory template-specific operations:
/// - Loading the inventory.yml.tera template
/// - Processing it with runtime context (hosts, SSH keys, etc.)
/// - Rendering the final inventory.yml file for Ansible consumption
pub struct InventoryTemplateRenderer {
    template_manager: Arc<TemplateManager>,
}

impl InventoryTemplateRenderer {
    /// Template filename for the inventory Tera template
    const INVENTORY_TEMPLATE_FILE: &'static str = "inventory.yml.tera";

    /// Output filename for the rendered inventory file
    const INVENTORY_OUTPUT_FILE: &'static str = "inventory.yml";

    /// Creates a new inventory template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the inventory.yml.tera template with the provided context
    ///
    /// This method:
    /// 1. Loads the inventory.yml.tera template from the template manager
    /// 2. Reads the template content
    /// 3. Creates a File object for template processing
    /// 4. Creates an `InventoryTemplate` with the runtime context
    /// 5. Renders the template to inventory.yml in the output directory
    ///
    /// # Arguments
    ///
    /// * `inventory_context` - The context containing hosts, SSH keys, and other variables
    /// * `output_dir` - The directory where inventory.yml should be written
    ///
    /// # Returns
    ///
    /// * `Result<(), InventoryTemplateError>` - Success or error from the template rendering operation
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
        inventory_context: &InventoryContext,
        output_dir: &Path,
    ) -> Result<(), InventoryTemplateError> {
        tracing::debug!("Rendering inventory template with runtime variables");

        // Get the inventory template path
        let inventory_template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| InventoryTemplateError::TemplatePathFailed {
                file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let inventory_template_content = std::fs::read_to_string(&inventory_template_path)
            .map_err(|source| InventoryTemplateError::TeraTemplateReadFailed {
                file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Create File object for template processing
        let inventory_template_file =
            File::new(Self::INVENTORY_TEMPLATE_FILE, inventory_template_content).map_err(
                |source| InventoryTemplateError::FileCreationFailed {
                    file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                    source,
                },
            )?;

        // Create InventoryTemplate with runtime context
        let inventory_template =
            InventoryTemplate::new(&inventory_template_file, inventory_context.clone()).map_err(
                |source| InventoryTemplateError::InventoryTemplateCreationFailed { source },
            )?;

        // Render to output file
        let inventory_output_path = output_dir.join(Self::INVENTORY_OUTPUT_FILE);
        inventory_template
            .render(&inventory_output_path)
            .map_err(|source| InventoryTemplateError::InventoryTemplateRenderFailed { source })?;

        tracing::debug!(
            "Successfully rendered inventory template to {}",
            inventory_output_path.display()
        );

        Ok(())
    }

    /// Builds the full template path for the inventory template
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for inventory.yml.tera
    fn build_template_path() -> String {
        format!("ansible/{}", Self::INVENTORY_TEMPLATE_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::str::FromStr;
    use tempfile::TempDir;

    use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::{
        AnsibleHost, AnsiblePort, SshPrivateKeyFile,
    };

    /// Helper function to create a test inventory context
    fn create_test_inventory_context(temp_dir: &Path) -> InventoryContext {
        let ssh_key_path = temp_dir.join("test_ssh_key");
        fs::write(&ssh_key_path, "dummy_ssh_key_content").expect("Failed to write test SSH key");

        let host = AnsibleHost::from_str("192.168.1.100").expect("Failed to create test host");
        let ssh_key = SshPrivateKeyFile::new(ssh_key_path).expect("Failed to create SSH key file");
        let ssh_port = AnsiblePort::new(22).expect("Failed to create SSH port");

        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(ssh_port)
            .with_ansible_user("torrust".to_string())
            .build()
            .expect("Failed to build inventory context")
    }

    /// Helper function to create a test template directory with inventory.yml.tera
    fn create_test_templates(temp_dir: &Path) -> std::io::Result<()> {
        let ansible_dir = temp_dir.join("ansible");
        fs::create_dir_all(&ansible_dir)?;

        let template_content = r#"all:
  hosts:
    torrust-tracker-vm:
      ansible_host: {{ ansible_host }}
      ansible_port: {{ ansible_port }}
      ansible_user: {{ ansible_user }}
      ansible_connection: ssh
      ansible_ssh_private_key_file: {{ ansible_ssh_private_key_file }}
      ansible_ssh_common_args: "-o StrictHostKeyChecking=no"
  vars:
    ansible_python_interpreter: /usr/bin/python3
"#;

        fs::write(ansible_dir.join("inventory.yml.tera"), template_content)?;

        Ok(())
    }

    #[test]
    fn it_should_create_inventory_renderer_with_template_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let renderer = InventoryTemplateRenderer::new(template_manager.clone());

        assert!(Arc::ptr_eq(&renderer.template_manager, &template_manager));
    }

    #[test]
    fn it_should_build_correct_template_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let _renderer = InventoryTemplateRenderer::new(template_manager);

        let template_path = InventoryTemplateRenderer::build_template_path();

        assert_eq!(template_path, "ansible/inventory.yml.tera");
    }

    #[test]
    fn it_should_render_inventory_template_successfully() {
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

        let renderer = InventoryTemplateRenderer::new(template_manager);
        let inventory_context = create_test_inventory_context(temp_dir.path());

        // Render template
        let result = renderer.render(&inventory_context, &output_dir);

        assert!(result.is_ok(), "Template rendering should succeed");

        // Verify output file exists
        let output_file = output_dir.join("inventory.yml");
        assert!(output_file.exists(), "inventory.yml should be created");

        // Verify output content contains expected values
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        println!("Generated output:\n{output_content}");
        assert!(
            output_content.contains("192.168.1.100"),
            "Output should contain the host IP"
        );
        assert!(
            output_content.contains("ansible_user: torrust"),
            "Output should contain ansible_user with correct value: {output_content}"
        );
    }

    #[test]
    fn it_should_fail_when_context_missing_required_fields() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        // Create template directory with a template that requires missing context fields
        let ansible_dir = template_dir.join("ansible");
        fs::create_dir_all(&ansible_dir).expect("Failed to create ansible directory");

        // Template requires {{ non_existent_field }} which won't be in context
        let template_content = r"all:
  hosts:
    torrust-tracker-vm:
      ansible_host: {{ ansible_host }}
      ansible_user: {{ ansible_user }}
      missing_field: {{ non_existent_field }}
";

        fs::write(ansible_dir.join("inventory.yml.tera"), template_content)
            .expect("Failed to write template with missing field");

        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Setup template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = InventoryTemplateRenderer::new(template_manager);
        let inventory_context = create_test_inventory_context(temp_dir.path());

        let result = renderer.render(&inventory_context, &output_dir);

        assert!(
            result.is_err(),
            "Should fail when template references non-existent context field"
        );
        match result.unwrap_err() {
            InventoryTemplateError::InventoryTemplateCreationFailed { .. } => {
                // Expected error type when template engine fails to process template
            }
            other => panic!("Expected InventoryTemplateCreationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_fail_when_template_content_is_invalid() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        // Create template directory with invalid template content
        let ansible_dir = template_dir.join("ansible");
        fs::create_dir_all(&ansible_dir).expect("Failed to create ansible directory");
        fs::write(
            ansible_dir.join("inventory.yml.tera"),
            "{{ invalid_template_syntax",
        )
        .expect("Failed to write invalid template");

        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Setup template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = InventoryTemplateRenderer::new(template_manager);
        let inventory_context = create_test_inventory_context(temp_dir.path());

        let result = renderer.render(&inventory_context, &output_dir);

        assert!(result.is_err(), "Should fail with invalid template syntax");
        // The exact error type will depend on the template engine's error handling
        assert!(matches!(
            result.unwrap_err(),
            InventoryTemplateError::InventoryTemplateCreationFailed { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn it_should_fail_when_output_directory_is_readonly() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");

        // Create template directory and files
        create_test_templates(&template_dir).expect("Failed to create test templates");
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // Make output directory read-only
        let mut perms = fs::metadata(&output_dir).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o444);
        fs::set_permissions(&output_dir, perms).unwrap();

        // Setup template manager and renderer
        let template_manager = Arc::new(TemplateManager::new(&template_dir));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates directory");

        let renderer = InventoryTemplateRenderer::new(template_manager);
        let inventory_context = create_test_inventory_context(temp_dir.path());

        let result = renderer.render(&inventory_context, &output_dir);

        assert!(
            result.is_err(),
            "Should fail when output directory is read-only"
        );
        match result.unwrap_err() {
            InventoryTemplateError::InventoryTemplateRenderFailed { .. } => {
                // Expected error type
            }
            other => panic!("Expected InventoryTemplateRenderFailed, got: {other:?}"),
        }
    }
}
