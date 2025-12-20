//! `Ansible` Project Generator
//!
//! This module handles `Ansible` template rendering for deployment workflows.
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

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::ansible::template::renderer::inventory::{
    InventoryRenderer, InventoryRendererError,
};
use crate::infrastructure::templating::ansible::template::renderer::variables::{
    VariablesRenderer, VariablesRendererError,
};
use crate::infrastructure::templating::ansible::template::wrappers::inventory::InventoryContext;

/// Errors that can occur during configuration template rendering
#[derive(Error, Debug)]
pub enum AnsibleProjectGeneratorError {
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

    /// Failed to render inventory template using collaborator
    #[error("Failed to render inventory template: {source}")]
    InventoryRenderingFailed {
        #[source]
        source: InventoryRendererError,
    },

    /// Failed to render variables template using collaborator
    #[error("Failed to render variables template: {source}")]
    VariablesRenderingFailed {
        #[source]
        source: VariablesRendererError,
    },

    /// Failed to create context from inventory data
    #[error("Failed to create {context_type} context: {message}")]
    ContextCreationFailed {
        context_type: String,
        message: String,
    },
}

/// Renders `Ansible` configuration templates to a build directory
///
/// This collaborator is responsible for preparing `Ansible` templates for deployment workflows.
/// It handles both static files (playbooks, configuration) and dynamic Tera templates that
/// require runtime variable substitution (inventory files with IP addresses).
pub struct AnsibleProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    inventory_renderer: InventoryRenderer,
    variables_renderer: VariablesRenderer,
}

impl AnsibleProjectGenerator {
    /// Default relative path for `Ansible` configuration files
    const ANSIBLE_BUILD_PATH: &'static str = "ansible";

    /// Default template path prefix for `Ansible` templates
    const ANSIBLE_TEMPLATE_PATH: &'static str = "ansible";

    /// Creates a new configuration template renderer
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let inventory_renderer = InventoryRenderer::new(template_manager.clone());
        let variables_renderer = VariablesRenderer::new(template_manager.clone());

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            template_manager,
            inventory_renderer,
            variables_renderer,
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
    /// * `inventory_context` - Runtime context for inventory template rendering (IP, SSH keys)
    /// * `tracker_config` - Optional tracker configuration for firewall port extraction
    /// * `grafana_config` - Optional Grafana configuration for conditional firewall setup
    ///
    /// # Returns
    ///
    /// * `Result<(), AnsibleProjectGeneratorError>` - Success or error from the template rendering operation
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
        inventory_context: &InventoryContext,
        tracker_config: Option<&crate::domain::tracker::TrackerConfig>,
        grafana_config: Option<&crate::domain::grafana::GrafanaConfig>,
    ) -> Result<(), AnsibleProjectGeneratorError> {
        tracing::info!(
            template_type = "ansible",
            "Rendering configuration templates with variables"
        );

        // Create build directory structure
        let build_ansible_dir = self.create_build_directory().await?;

        // Render dynamic inventory template with runtime variables using collaborator
        self.inventory_renderer
            .render(inventory_context, &build_ansible_dir)
            .map_err(|source| AnsibleProjectGeneratorError::InventoryRenderingFailed { source })?;

        // Render dynamic variables template with system configuration using collaborator
        let variables_context =
            Self::create_variables_context(inventory_context, tracker_config, grafana_config)?;
        self.variables_renderer
            .render(&variables_context, &build_ansible_dir)
            .map_err(|source| AnsibleProjectGeneratorError::VariablesRenderingFailed { source })?;

        // Copy static Ansible files (config and playbooks)
        self.copy_static_templates(&self.template_manager, &build_ansible_dir)
            .await?;

        tracing::debug!(
            template_type = "ansible",
            output_dir = %build_ansible_dir.display(),
            "Configuration templates rendered"
        );

        tracing::debug!(
            template_type = "ansible_inventory",
            ansible_host = %inventory_context.ansible_host(),
            "Inventory rendered with IP"
        );

        tracing::debug!(
            template_type = "ansible_inventory",
            ssh_key = %inventory_context.ansible_ssh_private_key_file(),
            "Inventory rendered with SSH key"
        );

        tracing::info!(
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
    /// * `Result<PathBuf, AnsibleProjectGeneratorError>` - The created build directory path or an error
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    async fn create_build_directory(&self) -> Result<PathBuf, AnsibleProjectGeneratorError> {
        let build_ansible_dir = self.build_ansible_directory();
        tokio::fs::create_dir_all(&build_ansible_dir)
            .await
            .map_err(
                |source| AnsibleProjectGeneratorError::DirectoryCreationFailed {
                    directory: build_ansible_dir.display().to_string(),
                    source,
                },
            )?;
        Ok(build_ansible_dir)
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
    /// * `Result<(), AnsibleProjectGeneratorError>` - Success or error from file copying operations
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
    ) -> Result<(), AnsibleProjectGeneratorError> {
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
            "configure-security-updates.yml",
            "configure-firewall.yml",
            "configure-tracker-firewall.yml",
            "create-tracker-storage.yml",
            "init-tracker-database.yml",
            "deploy-tracker-config.yml",
            "create-prometheus-storage.yml",
            "deploy-prometheus-config.yml",
            "deploy-compose-files.yml",
            "run-compose-services.yml",
        ] {
            self.copy_static_file(template_manager, playbook, destination_dir)
                .await?;
        }

        tracing::debug!(
            "Successfully copied {} static template files",
            17 // ansible.cfg + 16 playbooks
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
    /// * `Result<(), AnsibleProjectGeneratorError>` - Success or error from the file copying operation
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
    ) -> Result<(), AnsibleProjectGeneratorError> {
        let template_path = Self::build_template_path(file_name);

        let source_path = template_manager
            .get_template_path(&template_path)
            .map_err(|source| AnsibleProjectGeneratorError::TemplatePathFailed {
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
            .map_err(
                |source| AnsibleProjectGeneratorError::StaticFileCopyFailed {
                    file_name: file_name.to_string(),
                    source,
                },
            )?;

        tracing::debug!("Successfully copied static file {}", file_name);
        Ok(())
    }

    /// Creates an `AnsibleVariablesContext` from an `InventoryContext`
    ///
    /// Extracts the SSH port from the inventory context to create
    /// a variables context for template rendering.
    ///
    /// # Arguments
    ///
    /// * `inventory_context` - The inventory context containing SSH port information
    ///
    /// # Returns
    ///
    /// * `Result<AnsibleVariablesContext, AnsibleProjectGeneratorError>` - The variables context or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the SSH port cannot be extracted or validated
    fn create_variables_context(
        inventory_context: &InventoryContext,
        tracker_config: Option<&crate::domain::tracker::TrackerConfig>,
        grafana_config: Option<&crate::domain::grafana::GrafanaConfig>,
    ) -> Result<
        crate::infrastructure::templating::ansible::template::wrappers::variables::AnsibleVariablesContext,
        AnsibleProjectGeneratorError,
    >{
        use crate::infrastructure::templating::ansible::template::wrappers::variables::AnsibleVariablesContext;

        // Extract SSH port from inventory context and create variables context with tracker config
        AnsibleVariablesContext::new(
            inventory_context.ansible_port(),
            tracker_config,
            grafana_config,
        )
        .map_err(|e| AnsibleProjectGeneratorError::ContextCreationFailed {
            context_type: "AnsibleVariables".to_string(),
            message: format!("Failed to create variables context: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::templating::ansible::template::wrappers::inventory::{
        AnsibleHost, AnsiblePort, InventoryContext, SshPrivateKeyFile,
    };
    use std::str::FromStr;
    use tempfile::TempDir;

    /// Helper to create a valid inventory context for testing
    #[allow(dead_code)]
    fn create_test_inventory_context() -> InventoryContext {
        let host = AnsibleHost::from_str("192.168.1.100").expect("Valid IP address");
        let ssh_key = SshPrivateKeyFile::new("/path/to/ssh/key").expect("Valid SSH key path");
        let ssh_port = AnsiblePort::new(22).expect("Valid SSH port");

        InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(ssh_port)
            .with_ansible_user("torrust".to_string())
            .build()
            .expect("Valid inventory context")
    }

    /// Helper to create a test template manager for testing
    #[allow(dead_code)]
    fn create_test_template_manager() -> Arc<TemplateManager> {
        // Create a temporary directory for templates in tests
        Arc::new(TemplateManager::new("/tmp/test/templates"))
    }

    #[tokio::test]
    async fn it_should_create_renderer_with_build_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let template_manager = create_test_template_manager();

        let renderer = AnsibleProjectGenerator::new(&build_path, template_manager);

        assert_eq!(renderer.build_dir, build_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_ansible_directory_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("ansible");
        let template_manager = create_test_template_manager();

        let renderer = AnsibleProjectGenerator::new(&build_path, template_manager);
        let actual_path = renderer.build_ansible_directory();

        assert_eq!(actual_path, expected_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_template_path_for_file() {
        let template_path = AnsibleProjectGenerator::build_template_path("inventory.yml.tera");

        assert_eq!(template_path, "ansible/inventory.yml.tera");
    }

    #[tokio::test]
    async fn it_should_build_template_path_for_static_file() {
        let template_path = AnsibleProjectGenerator::build_template_path("ansible.cfg");

        assert_eq!(template_path, "ansible/ansible.cfg");
    }

    #[tokio::test]
    async fn it_should_create_build_directory_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let template_manager = create_test_template_manager();
        let renderer = AnsibleProjectGenerator::new(&build_path, template_manager);

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
        let template_manager = create_test_template_manager();
        let renderer = AnsibleProjectGenerator::new(invalid_path, template_manager);

        let result = renderer.create_build_directory().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AnsibleProjectGeneratorError::DirectoryCreationFailed { directory, .. } => {
                assert!(directory.contains("invalid"));
            }
            other => panic!("Expected DirectoryCreationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_have_correct_template_file_constants() {
        assert_eq!(AnsibleProjectGenerator::ANSIBLE_BUILD_PATH, "ansible");
        assert_eq!(AnsibleProjectGenerator::ANSIBLE_TEMPLATE_PATH, "ansible");
    }
}
