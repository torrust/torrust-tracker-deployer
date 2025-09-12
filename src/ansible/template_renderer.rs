//! # Ansible Template Renderer
//!
//! This module handles `Ansible` template rendering for deployment stages.
//! It manages the creation of build directories, copying static template files (playbooks and configs),
//! and processing dynamic Tera templates with runtime variables (like inventory.yml.tera).
//!
//! ## Key Features
//!
//! - **Static file copying**: Handles Ansible playbooks and configuration files that don't need templating
//! - **Dynamic template rendering**: Processes Tera templates with runtime variables like IP addresses and SSH keys
//! - **Structured error handling**: Provides specific error types with detailed context and source chaining
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring deployment processes
//! - **Testable design**: Modular structure that allows for comprehensive unit testing
//!
//! ## Usage
//!
//! ```rust
//! # use std::str::FromStr;
//! # use tempfile::TempDir;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use torrust_tracker_deploy::ansible::AnsibleTemplateRenderer;
//! use torrust_tracker_deploy::template::TemplateManager;
//! use torrust_tracker_deploy::template::wrappers::ansible::inventory::{
//!     InventoryContext, AnsibleHost, SshPrivateKeyFile
//! };
//!
//! let temp_dir = TempDir::new()?;
//! let renderer = AnsibleTemplateRenderer::new(temp_dir.path());
//! let template_manager = TemplateManager::new("/path/to/templates");
//!
//! let host = AnsibleHost::from_str("192.168.1.100")?;
//! let ssh_key = SshPrivateKeyFile::new("/path/to/ssh/key")?;
//! let inventory_context = InventoryContext::builder()
//!     .with_host(host)
//!     .with_ssh_priv_key_path(ssh_key)
//!     .build()?;
//!
//! // Note: This would require actual template files to work
//! // renderer.render(&template_manager, &inventory_context).await?;
//! # Ok(())
//! # }
//! ```

use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::template::file::File;
use crate::template::wrappers::ansible::inventory::{InventoryContext, InventoryTemplate};
use crate::template::{FileOperationError, TemplateManager, TemplateManagerError};

/// Errors that can occur during configuration template rendering
#[derive(Error, Debug)]
pub enum ConfigurationTemplateError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to get template path from template manager
    #[error("Failed to get template path for '{file_name}': {source}")]
    TemplatePathFailed {
        file_name: String,
        #[source]
        source: TemplateManagerError,
    },

    /// Failed to copy static template file
    #[error("Failed to copy static template file '{file_name}' to build directory: {source}")]
    StaticFileCopyFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
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
        source: crate::template::file::Error,
    },

    /// Failed to create inventory template with provided context
    #[error("Failed to create InventoryTemplate: {source}")]
    InventoryTemplateCreationFailed {
        #[source]
        source: crate::template::TemplateEngineError,
    },

    /// Failed to render inventory template to output file
    #[error("Failed to render inventory template to file: {source}")]
    InventoryTemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Renders `Ansible` configuration templates to a build directory
///
/// This collaborator is responsible for preparing `Ansible` templates for deployment stages.
/// It handles both static files (playbooks, configuration) and dynamic Tera templates that
/// require runtime variable substitution (inventory files with IP addresses).
pub struct AnsibleTemplateRenderer {
    build_dir: PathBuf,
}

impl AnsibleTemplateRenderer {
    /// Default relative path for `Ansible` configuration files
    const ANSIBLE_BUILD_PATH: &'static str = "ansible";

    /// Default template path prefix for `Ansible` templates
    const ANSIBLE_TEMPLATE_PATH: &'static str = "ansible";

    /// Name of the dynamic inventory template file
    const INVENTORY_TEMPLATE_FILE: &'static str = "inventory.yml.tera";

    /// Name of the rendered inventory output file
    const INVENTORY_OUTPUT_FILE: &'static str = "inventory.yml";

    /// Creates a new configuration template renderer
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    pub fn new<P: AsRef<Path>>(build_dir: P) -> Self {
        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
        }
    }

    /// Renders configuration templates (`Ansible`) to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for `Ansible`
    /// 2. Renders dynamic Tera templates with runtime variables (inventory.yml.tera)
    /// 3. Copies static templates (playbooks, ansible.cfg) from the template manager
    /// 4. Provides debug logging via the tracing crate
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    /// * `inventory_context` - Runtime context for inventory template rendering (IP, SSH keys)
    ///
    /// # Returns
    ///
    /// * `Result<(), ConfigurationTemplateError>` - Success or error from the template rendering operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Template copying fails
    /// - Template manager cannot provide required templates
    /// - Dynamic template rendering fails
    /// - Runtime variable substitution fails
    pub async fn render(
        &self,
        template_manager: &TemplateManager,
        inventory_context: &InventoryContext,
    ) -> Result<(), ConfigurationTemplateError> {
        tracing::info!(
            stage = "configuration_rendering",
            template_type = "ansible",
            "Rendering configuration templates with variables"
        );

        // Create build directory structure
        let build_ansible_dir = self.create_build_directory().await?;

        // Render dynamic inventory template with runtime variables
        Self::render_inventory_template(template_manager, inventory_context, &build_ansible_dir)?;

        // Copy static Ansible files (config and playbooks)
        self.copy_static_templates(template_manager, &build_ansible_dir)
            .await?;

        tracing::debug!(
            stage = "configuration_rendering",
            template_type = "ansible",
            output_dir = %build_ansible_dir.display(),
            "Configuration templates rendered"
        );
        tracing::debug!(
            stage = "configuration_rendering",
            template_type = "ansible_inventory",
            ansible_host = %inventory_context.ansible_host(),
            "Inventory rendered with IP"
        );
        tracing::debug!(
            stage = "configuration_rendering",
            template_type = "ansible_inventory",
            ssh_key = %inventory_context.ansible_ssh_private_key_file(),
            "Inventory rendered with SSH key"
        );

        tracing::info!(
            stage = "configuration_rendering",
            template_type = "ansible",
            status = "complete",
            "Configuration templates ready"
        );
        Ok(())
    }

    /// Builds the full `Ansible` build directory path
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The complete path to the `Ansible` build directory
    fn build_ansible_directory(&self) -> PathBuf {
        self.build_dir.join(Self::ANSIBLE_BUILD_PATH)
    }

    /// Builds the template path for a specific file in the `Ansible` template directory
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the template file
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for the specified file
    fn build_template_path(file_name: &str) -> String {
        format!("{}/{file_name}", Self::ANSIBLE_TEMPLATE_PATH)
    }

    /// Creates the `Ansible` build directory structure
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, ConfigurationTemplateError>` - The created build directory path or an error
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    async fn create_build_directory(&self) -> Result<PathBuf, ConfigurationTemplateError> {
        let build_ansible_dir = self.build_ansible_directory();
        tokio::fs::create_dir_all(&build_ansible_dir)
            .await
            .map_err(
                |source| ConfigurationTemplateError::DirectoryCreationFailed {
                    directory: build_ansible_dir.display().to_string(),
                    source,
                },
            )?;
        Ok(build_ansible_dir)
    }

    /// Renders the dynamic inventory template with runtime variables
    ///
    /// This method handles the complex case of Tera template rendering where variables
    /// like IP addresses and SSH key paths are substituted at runtime after infrastructure
    /// provisioning is complete.
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Source of template files
    /// * `inventory_context` - Runtime context with IP and SSH key information
    /// * `destination_dir` - Directory where the rendered inventory file will be written
    ///
    /// # Returns
    ///
    /// * `Result<(), ConfigurationTemplateError>` - Success or error from template rendering
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be read
    /// - Template content is invalid
    /// - Variable substitution fails
    /// - Output file cannot be written
    fn render_inventory_template(
        template_manager: &TemplateManager,
        inventory_context: &InventoryContext,
        destination_dir: &Path,
    ) -> Result<(), ConfigurationTemplateError> {
        tracing::debug!("Rendering dynamic inventory template with runtime variables");

        // Get the inventory template path
        let inventory_template_path = template_manager
            .get_template_path(&Self::build_template_path(Self::INVENTORY_TEMPLATE_FILE))
            .map_err(|source| ConfigurationTemplateError::TemplatePathFailed {
                file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let inventory_template_content = std::fs::read_to_string(&inventory_template_path)
            .map_err(
                |source| ConfigurationTemplateError::TeraTemplateReadFailed {
                    file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                    source,
                },
            )?;

        // Create File object for template processing
        let inventory_template_file =
            File::new(Self::INVENTORY_TEMPLATE_FILE, inventory_template_content).map_err(
                |source| ConfigurationTemplateError::FileCreationFailed {
                    file_name: Self::INVENTORY_TEMPLATE_FILE.to_string(),
                    source,
                },
            )?;

        // Create InventoryTemplate with runtime context
        let inventory_template =
            InventoryTemplate::new(&inventory_template_file, inventory_context.clone()).map_err(
                |source| ConfigurationTemplateError::InventoryTemplateCreationFailed { source },
            )?;

        // Render to output file
        let inventory_output_path = destination_dir.join(Self::INVENTORY_OUTPUT_FILE);
        inventory_template
            .render(&inventory_output_path)
            .map_err(
                |source| ConfigurationTemplateError::InventoryTemplateRenderFailed { source },
            )?;

        tracing::debug!(
            "Successfully rendered inventory template to {}",
            inventory_output_path.display()
        );

        Ok(())
    }

    /// Copies static Ansible template files that don't require variable substitution
    ///
    /// This includes configuration files and playbooks that are used as-is without
    /// runtime variable substitution.
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Source of template files
    /// * `destination_dir` - Directory where static files will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), ConfigurationTemplateError>` - Success or error from file copying operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide required template paths
    /// - File copying fails for any of the specified files
    async fn copy_static_templates(
        &self,
        template_manager: &TemplateManager,
        destination_dir: &Path,
    ) -> Result<(), ConfigurationTemplateError> {
        tracing::debug!("Copying static Ansible template files");

        // Copy configuration file
        self.copy_static_file(template_manager, "ansible.cfg", destination_dir)
            .await?;

        // Copy all playbook files
        for playbook in &[
            "update-apt-cache.yml",
            "install-docker.yml",
            "install-docker-compose.yml",
            "wait-cloud-init.yml",
        ] {
            self.copy_static_file(template_manager, playbook, destination_dir)
                .await?;
        }

        tracing::debug!(
            "Successfully copied {} static template files",
            5 // ansible.cfg + 4 playbooks
        );

        Ok(())
    }

    /// Copies a single static template file from template manager to destination
    ///
    /// # Arguments
    ///
    /// * `template_manager` - Source of template files
    /// * `file_name` - Name of the file to copy (without path prefix)
    /// * `destination_dir` - Directory where the file will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), ConfigurationTemplateError>` - Success or error from the file copying operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide the template path
    /// - File copying fails
    async fn copy_static_file(
        &self,
        template_manager: &TemplateManager,
        file_name: &str,
        destination_dir: &Path,
    ) -> Result<(), ConfigurationTemplateError> {
        let template_path = Self::build_template_path(file_name);

        let source_path = template_manager
            .get_template_path(&template_path)
            .map_err(|source| ConfigurationTemplateError::TemplatePathFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        let dest_path = destination_dir.join(file_name);

        tracing::trace!(
            "Copying static file {} to {}",
            source_path.display(),
            dest_path.display()
        );

        tokio::fs::copy(&source_path, &dest_path)
            .await
            .map_err(|source| ConfigurationTemplateError::StaticFileCopyFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        tracing::debug!("Successfully copied static file {}", file_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::wrappers::ansible::inventory::{
        AnsibleHost, InventoryContext, SshPrivateKeyFile,
    };
    use std::str::FromStr;
    use tempfile::TempDir;

    /// Helper to create a valid inventory context for testing
    #[allow(dead_code)]
    fn create_test_inventory_context() -> InventoryContext {
        let host = AnsibleHost::from_str("192.168.1.100").expect("Valid IP address");
        let ssh_key = SshPrivateKeyFile::new("/path/to/ssh/key").expect("Valid SSH key path");

        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build()
            .expect("Valid inventory context")
    }

    #[tokio::test]
    async fn it_should_create_renderer_with_build_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        let renderer = AnsibleTemplateRenderer::new(&build_path);

        assert_eq!(renderer.build_dir, build_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_ansible_directory_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("ansible");

        let renderer = AnsibleTemplateRenderer::new(&build_path);
        let actual_path = renderer.build_ansible_directory();

        assert_eq!(actual_path, expected_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_template_path_for_file() {
        let template_path = AnsibleTemplateRenderer::build_template_path("inventory.yml.tera");

        assert_eq!(template_path, "ansible/inventory.yml.tera");
    }

    #[tokio::test]
    async fn it_should_build_template_path_for_static_file() {
        let template_path = AnsibleTemplateRenderer::build_template_path("ansible.cfg");

        assert_eq!(template_path, "ansible/ansible.cfg");
    }

    #[tokio::test]
    async fn it_should_create_build_directory_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let renderer = AnsibleTemplateRenderer::new(&build_path);

        let result = renderer.create_build_directory().await;

        assert!(result.is_ok());
        let created_dir = result.unwrap();
        assert_eq!(created_dir, build_path.join("ansible"));
        assert!(created_dir.exists());
        assert!(created_dir.is_dir());
    }

    #[tokio::test]
    async fn it_should_fail_gracefully_when_build_directory_creation_fails() {
        // Try to create a directory where we don't have permissions
        // Use a path that's likely to fail on most systems
        let invalid_path = Path::new("/root/invalid/path/that/should/not/exist");
        let renderer = AnsibleTemplateRenderer::new(invalid_path);

        let result = renderer.create_build_directory().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigurationTemplateError::DirectoryCreationFailed { directory, .. } => {
                assert!(directory.contains("invalid"));
            }
            other => panic!("Expected DirectoryCreationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_have_correct_template_file_constants() {
        assert_eq!(
            AnsibleTemplateRenderer::INVENTORY_TEMPLATE_FILE,
            "inventory.yml.tera"
        );
        assert_eq!(
            AnsibleTemplateRenderer::INVENTORY_OUTPUT_FILE,
            "inventory.yml"
        );
        assert_eq!(AnsibleTemplateRenderer::ANSIBLE_BUILD_PATH, "ansible");
        assert_eq!(AnsibleTemplateRenderer::ANSIBLE_TEMPLATE_PATH, "ansible");
    }
}
