//! # `OpenTofu` Template Renderer
//!
//! This module handles `OpenTofu` template rendering for deployment workflows.
//! It manages the creation of build directories, copying template files, and processing them with
//! variable substitution.
//!
//! ## Future Improvements
//!
//! The following improvements could enhance this module's functionality and maintainability:
//!
//! 1. **Add comprehensive logging** - Add debug/trace logs for each operation step (directory
//!    creation, file copying, template processing) to improve debugging and monitoring.
//!
//! 2. **Extract constants for magic strings** - Create constants for hardcoded paths like "tofu",
//!    "lxd", and file names to improve maintainability and reduce duplication.
//!
//! 3. **Add input validation** - Validate template names, check for empty strings, validate paths
//!    before processing to provide early error detection and better user feedback.
//!
//! 4. **Improve error messages** - Make error messages more user-friendly and actionable with
//!    suggestions for resolution, including common troubleshooting steps.
//!
//! 5. **Add configuration validation** - Pre-validate that required template files exist before
//!    starting the rendering process to avoid partial failures.
//!
//! 6. **Extract template discovery logic** - Separate the logic for finding and listing available
//!    templates to make it reusable and testable independently.
//!
//! 7. **Add progress reporting** - Add callback mechanism or progress indicators for long-running
//!    operations to improve user experience during deployment.
//!
//! 8. **Improve file operations** - Add more robust file copying with better error handling and
//!    atomic operations to prevent partial state corruption.
//!
//! 9. **Add template caching** - Cache parsed templates to improve performance for repeated
//!    operations and reduce I/O overhead.
//!
//! 10. **Extract provider-specific logic** - Separate LXD-specific logic to make it more
//!     extensible for other providers (Multipass, Docker, etc.) following the strategy pattern.

pub mod cloud_init;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[allow(unused_imports)]
use crate::command_wrappers::lxd::instance::InstanceName;
use crate::command_wrappers::ssh::credentials::SshCredentials;
use crate::template::wrappers::tofu::lxd::variables::{
    VariablesContextBuilder, VariablesTemplate, VariablesTemplateError,
};
use crate::template::{TemplateManager, TemplateManagerError};
use crate::tofu::template::renderer::cloud_init::{
    CloudInitTemplateError, CloudInitTemplateRenderer,
};

/// Errors that can occur during provision template rendering
#[derive(Error, Debug)]
pub enum ProvisionTemplateError {
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

    /// Failed to copy template file
    #[error("Failed to copy template file '{file_name}' to build directory: {source}")]
    FileCopyFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to render cloud-init template using collaborator
    #[error("Failed to render cloud-init template: {source}")]
    CloudInitRenderingFailed {
        #[source]
        source: CloudInitTemplateError,
    },

    /// Failed to render variables template using collaborator
    #[error("Failed to render variables template: {source}")]
    VariablesRenderingFailed {
        #[source]
        source: VariablesTemplateError,
    },
}

/// Renders `OpenTofu` provision templates to a build directory
///
/// This collaborator is responsible for preparing `OpenTofu` templates for deployment workflows.
/// It copies static templates and renders Tera templates with runtime variables from the template manager to the specified build directory.
pub struct TofuTemplateRenderer {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
    ssh_credentials: SshCredentials,
    cloud_init_renderer: CloudInitTemplateRenderer,
    instance_name: InstanceName,
}

impl TofuTemplateRenderer {
    /// Default relative path for `OpenTofu` configuration files
    const OPENTOFU_BUILD_PATH: &'static str = "tofu/lxd";

    /// Default template path prefix for `OpenTofu` templates
    const OPENTOFU_TEMPLATE_PATH: &'static str = "tofu/lxd";

    /// Creates a new provision template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `ssh_credentials` - The SSH credentials for injecting public key into cloud-init
    /// * `instance_name` - The name of the instance to be created (for template rendering)
    pub fn new<P: AsRef<Path>>(
        template_manager: Arc<TemplateManager>,
        build_dir: P,
        ssh_credentials: SshCredentials,
        instance_name: InstanceName,
    ) -> Self {
        let cloud_init_renderer = CloudInitTemplateRenderer::new(template_manager.clone());

        Self {
            template_manager,
            build_dir: build_dir.as_ref().to_path_buf(),
            ssh_credentials,
            cloud_init_renderer,
            instance_name,
        }
    }

    /// Renders provision templates (`OpenTofu`) to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for `OpenTofu`
    /// 2. Copies static templates (main.tf) from the template manager
    /// 3. Renders Tera templates (cloud-init.yml.tera) with runtime variables
    /// 4. Provides debug logging via the tracing crate
    ///
    /// # Returns
    ///
    /// * `Result<(), ProvisionTemplateError>` - Success or error from the template rendering operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Template copying fails
    /// - Template manager cannot provide required templates
    /// - Tera template rendering fails
    pub async fn render(&self) -> Result<(), ProvisionTemplateError> {
        tracing::info!(
            template_type = "opentofu",
            "Rendering provision templates to build directory"
        );

        // Create build directory structure
        let build_tofu_dir = self.create_build_directory().await?;

        // List of static templates to copy directly
        // Note: variables.tfvars is now dynamically rendered via VariablesTemplate
        let static_template_files = vec!["main.tf"];

        // Copy static template files
        self.copy_templates(&static_template_files, &build_tofu_dir)
            .await?;

        // Render Tera templates with runtime variables
        self.render_tera_templates(&build_tofu_dir).await?;

        tracing::debug!(
            template_type = "opentofu",
            output_dir = %build_tofu_dir.display(),
            "Provision templates copied and rendered"
        );

        tracing::info!(
            template_type = "opentofu",
            status = "complete",
            "Provision templates ready"
        );
        Ok(())
    }

    /// Builds the full `OpenTofu` build directory path
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The complete path to the `OpenTofu` build directory
    fn build_opentofu_directory(&self) -> PathBuf {
        self.build_dir.join(Self::OPENTOFU_BUILD_PATH)
    }

    /// Builds the template path for a specific file in the `OpenTofu` template directory
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the template file
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for the specified file
    fn build_template_path(file_name: &str) -> String {
        format!("{}/{file_name}", Self::OPENTOFU_TEMPLATE_PATH)
    }

    /// Creates the `OpenTofu` build directory structure
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, ProvisionTemplateError>` - The created build directory path or an error
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    async fn create_build_directory(&self) -> Result<PathBuf, ProvisionTemplateError> {
        let build_tofu_dir = self.build_opentofu_directory();
        tokio::fs::create_dir_all(&build_tofu_dir)
            .await
            .map_err(|source| ProvisionTemplateError::DirectoryCreationFailed {
                directory: build_tofu_dir.display().to_string(),
                source,
            })?;
        Ok(build_tofu_dir)
    }

    /// Copies a list of template files from the template manager to the destination directory
    ///
    /// # Arguments
    ///
    /// * `file_names` - List of file names to copy (without path prefix)
    /// * `destination_dir` - The directory where files will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), ProvisionTemplateError>` - Success or error from the file copying operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide required template paths
    /// - File copying fails for any of the specified files
    async fn copy_templates(
        &self,
        file_names: &[&str],
        destination_dir: &Path,
    ) -> Result<(), ProvisionTemplateError> {
        tracing::debug!(
            "Copying {} template files to {}",
            file_names.len(),
            destination_dir.display()
        );

        for file_name in file_names {
            let template_path = Self::build_template_path(file_name);

            let source_path = self
                .template_manager
                .get_template_path(&template_path)
                .map_err(|source| ProvisionTemplateError::TemplatePathFailed {
                    file_name: (*file_name).to_string(),
                    source,
                })?;

            let dest_path = destination_dir.join(file_name);

            tracing::trace!(
                "Copying {} to {}",
                source_path.display(),
                dest_path.display()
            );

            tokio::fs::copy(&source_path, &dest_path)
                .await
                .map_err(|source| ProvisionTemplateError::FileCopyFailed {
                    file_name: (*file_name).to_string(),
                    source,
                })?;

            tracing::debug!("Successfully copied {}", file_name);
        }

        tracing::debug!("All template files copied successfully");
        Ok(())
    }

    /// Renders Tera templates with runtime variables using collaborators
    ///
    /// This method delegates template rendering to specialized collaborators:
    /// - cloud-init.yml.tera template rendering to the `CloudInitTemplateRenderer` collaborator
    /// - variables.tfvars.tera template rendering using the `VariablesTemplate`
    ///
    /// # Arguments
    ///
    /// * `destination_dir` - The directory where rendered templates should be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `CloudInitTemplateRenderer` fails to render the template
    /// - `VariablesTemplate` fails to render the variables template
    async fn render_tera_templates(
        &self,
        destination_dir: &Path,
    ) -> Result<(), ProvisionTemplateError> {
        tracing::debug!("Rendering Tera templates with runtime variables using collaborators");

        // Use collaborator to render cloud-init.yml.tera template
        self.cloud_init_renderer
            .render(&self.ssh_credentials, destination_dir)
            .await
            .map_err(|source| ProvisionTemplateError::CloudInitRenderingFailed { source })?;

        // Render variables.tfvars.tera template with instance name
        self.render_variables_template(destination_dir).await?;

        tracing::debug!("All Tera templates rendered successfully");
        Ok(())
    }

    /// Renders the variables.tfvars.tera template with the instance name context
    ///
    /// # Arguments
    ///
    /// * `destination_dir` - The directory where the rendered variables.tfvars file will be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide the variables.tfvars.tera template
    /// - Variables template rendering fails
    /// - File writing fails
    async fn render_variables_template(
        &self,
        destination_dir: &Path,
    ) -> Result<(), ProvisionTemplateError> {
        tracing::debug!("Rendering variables.tfvars.tera template with instance name context");

        // Get the variables.tfvars.tera template from the template manager
        let template_path = Self::build_template_path("variables.tfvars.tera");
        let template_file_path = self
            .template_manager
            .get_template_path(&template_path)
            .map_err(|source| ProvisionTemplateError::TemplatePathFailed {
                file_name: "variables.tfvars.tera".to_string(),
                source,
            })?;

        // Read the template file content
        let template_content = tokio::fs::read_to_string(&template_file_path)
            .await
            .map_err(|source| ProvisionTemplateError::FileCopyFailed {
                file_name: "variables.tfvars.tera".to_string(),
                source,
            })?;

        // Create template file wrapper
        let template_file =
            crate::template::file::File::new("variables.tfvars.tera", template_content).map_err(
                |err| ProvisionTemplateError::FileCopyFailed {
                    file_name: "variables.tfvars.tera".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
                },
            )?;

        // Build context for template rendering
        let context = VariablesContextBuilder::new()
            .with_instance_name(self.instance_name.clone())
            .build()
            .map_err(|err| ProvisionTemplateError::VariablesRenderingFailed {
                source: VariablesTemplateError::TemplateEngineError {
                    source: crate::template::TemplateEngineError::ContextSerialization {
                        source: tera::Error::msg(err.to_string()),
                    },
                },
            })?;

        // Create and render the variables template
        let variables_template = VariablesTemplate::new(&template_file, context)
            .map_err(|source| ProvisionTemplateError::VariablesRenderingFailed { source })?;

        // Write the rendered template to the destination directory
        let output_path = destination_dir.join("variables.tfvars");
        variables_template
            .render(&output_path)
            .map_err(|source| ProvisionTemplateError::VariablesRenderingFailed { source })?;

        tracing::debug!("Variables template rendered successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test instance name for unit tests
    fn test_instance_name() -> InstanceName {
        InstanceName::new("test-instance".to_string()).expect("Valid test instance name")
    }

    /// Helper function to create dummy SSH credentials for testing
    fn create_dummy_ssh_credentials(temp_dir: &Path) -> SshCredentials {
        let ssh_priv_key_path = temp_dir.join("test_key");
        let ssh_pub_key_path = temp_dir.join("test_key.pub");

        // Create dummy key files
        fs::write(&ssh_priv_key_path, "dummy_private_key").expect("Failed to write private key");
        fs::write(
            &ssh_pub_key_path,
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com",
        )
        .expect("Failed to write public key");

        SshCredentials::new(ssh_priv_key_path, ssh_pub_key_path, "testuser".to_string())
    }

    #[tokio::test]
    async fn it_should_create_renderer_with_build_directory() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());

        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );

        assert_eq!(renderer.build_dir, build_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_opentofu_directory_path() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("tofu/lxd");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());

        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );
        let actual_path = renderer.build_opentofu_directory();

        assert_eq!(actual_path, expected_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_template_path_for_file() {
        let template_path = TofuTemplateRenderer::build_template_path("main.tf");

        assert_eq!(template_path, "tofu/lxd/main.tf");
    }

    #[tokio::test]
    async fn it_should_build_template_path_with_different_file_names() {
        assert_eq!(
            TofuTemplateRenderer::build_template_path("cloud-init.yml"),
            "tofu/lxd/cloud-init.yml"
        );
        assert_eq!(
            TofuTemplateRenderer::build_template_path("variables.tf"),
            "tofu/lxd/variables.tf"
        );
        assert_eq!(
            TofuTemplateRenderer::build_template_path("outputs.tf"),
            "tofu/lxd/outputs.tf"
        );
    }

    #[tokio::test]
    async fn it_should_create_build_directory_successfully() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("tofu/lxd");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );
        let created_path = renderer
            .create_build_directory()
            .await
            .expect("Failed to create build directory");

        assert_eq!(created_path, expected_path);
        assert!(created_path.exists(), "Build directory should exist");
        assert!(created_path.is_dir(), "Created path should be a directory");
    }

    // Error Handling Tests
    #[tokio::test]
    async fn it_should_fail_when_directory_creation_denied() {
        // Create a read-only directory to simulate permission denied
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let readonly_path = temp_dir.path().join("readonly");
        tokio::fs::create_dir(&readonly_path)
            .await
            .expect("Failed to create readonly dir");

        // Make the directory read-only
        let mut perms = tokio::fs::metadata(&readonly_path)
            .await
            .unwrap()
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o444); // Read-only permissions
        tokio::fs::set_permissions(&readonly_path, perms)
            .await
            .unwrap();

        let build_path = readonly_path.join("build");
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );

        let result = renderer.create_build_directory().await;

        assert!(
            result.is_err(),
            "Should fail when directory creation is denied"
        );
        match result.unwrap_err() {
            ProvisionTemplateError::DirectoryCreationFailed {
                directory,
                source: _,
            } => {
                assert!(
                    directory.contains("tofu/lxd"),
                    "Error should contain the full path"
                );
            }
            other => panic!("Expected DirectoryCreationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_fail_when_template_manager_cannot_find_template() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        // Create a template manager with empty templates directory
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );

        // Try to copy a non-existent template
        let result = renderer
            .copy_templates(&["nonexistent.tf"], &build_path)
            .await;

        assert!(result.is_err(), "Should fail when template is not found");
        match result.unwrap_err() {
            ProvisionTemplateError::TemplatePathFailed {
                file_name,
                source: _,
            } => {
                assert_eq!(file_name, "nonexistent.tf");
            }
            other => panic!("Expected TemplatePathFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_fail_when_file_copy_permission_denied() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        // Create the destination directory first, then make it read-only
        tokio::fs::create_dir_all(&build_path)
            .await
            .expect("Failed to create build directory");

        let mut perms = tokio::fs::metadata(&build_path)
            .await
            .unwrap()
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o444); // Read-only permissions
        tokio::fs::set_permissions(&build_path, perms)
            .await
            .unwrap();

        // Create template manager and ensure it has the template we need
        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates dir");

        // Create a test template file manually since we can't rely on embedded resources
        let template_dir = temp_dir.path().join("tofu/lxd");
        tokio::fs::create_dir_all(&template_dir)
            .await
            .expect("Failed to create template dir");
        tokio::fs::write(template_dir.join("test.tf"), "# Test template")
            .await
            .expect("Failed to write test template");

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            temp_dir.path(),
            ssh_credentials,
            test_instance_name(),
        );

        let result = renderer.copy_templates(&["test.tf"], &build_path).await;

        assert!(result.is_err(), "Should fail when file copy is denied");
        match result.unwrap_err() {
            ProvisionTemplateError::FileCopyFailed {
                file_name,
                source: _,
            } => {
                assert_eq!(file_name, "test.tf");
            }
            other => panic!("Expected FileCopyFailed, got: {other:?}"),
        }
    }

    // Input Validation Edge Case Tests
    #[test]
    fn it_should_handle_empty_file_name() {
        let template_path = TofuTemplateRenderer::build_template_path("");
        assert_eq!(template_path, "tofu/lxd/");
    }

    #[test]
    fn it_should_handle_file_names_with_path_separators() {
        // File names with forward slashes should be handled literally
        let template_path = TofuTemplateRenderer::build_template_path("sub/dir/file.tf");
        assert_eq!(template_path, "tofu/lxd/sub/dir/file.tf");

        // File names with backslashes (Windows-style)
        let template_path = TofuTemplateRenderer::build_template_path("sub\\dir\\file.tf");
        assert_eq!(template_path, "tofu/lxd/sub\\dir\\file.tf");

        // Relative path components
        let template_path = TofuTemplateRenderer::build_template_path("../main.tf");
        assert_eq!(template_path, "tofu/lxd/../main.tf");
    }

    #[test]
    fn it_should_handle_special_characters_in_file_names() {
        // File names with spaces
        let template_path = TofuTemplateRenderer::build_template_path("main file.tf");
        assert_eq!(template_path, "tofu/lxd/main file.tf");

        // File names with unicode characters
        let template_path = TofuTemplateRenderer::build_template_path("файл.tf");
        assert_eq!(template_path, "tofu/lxd/файл.tf");

        // File names with special characters
        let template_path = TofuTemplateRenderer::build_template_path("main@#$%.tf");
        assert_eq!(template_path, "tofu/lxd/main@#$%.tf");
    }

    #[test]
    fn it_should_handle_very_long_file_names() {
        // Create a very long file name (300 characters)
        let long_name = "a".repeat(300) + ".tf";
        let template_path = TofuTemplateRenderer::build_template_path(&long_name);
        assert_eq!(template_path, format!("tofu/lxd/{long_name}"));
    }

    // File System Edge Case Tests
    #[tokio::test]
    async fn it_should_handle_existing_build_directory() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let tofu_path = build_path.join("tofu/lxd");

        // Pre-create the directory structure
        tokio::fs::create_dir_all(&tofu_path)
            .await
            .expect("Failed to create existing directory");
        assert!(tofu_path.exists(), "Directory should already exist");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );
        let created_path = renderer
            .create_build_directory()
            .await
            .expect("Should handle existing directory gracefully");

        assert_eq!(created_path, tofu_path);
        assert!(created_path.exists(), "Directory should still exist");
    }

    #[tokio::test]
    async fn it_should_handle_empty_template_files_array() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            &build_path,
            ssh_credentials,
            test_instance_name(),
        );

        // Should succeed with empty array
        let result = renderer.copy_templates(&[], &build_path).await;

        assert!(result.is_ok(), "Should handle empty template files array");
    }

    #[tokio::test]
    async fn it_should_handle_duplicate_files_in_array() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        tokio::fs::create_dir_all(&build_path)
            .await
            .expect("Failed to create build directory");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates dir");

        // Create a test template file manually
        let template_dir = temp_dir.path().join("tofu/lxd");
        tokio::fs::create_dir_all(&template_dir)
            .await
            .expect("Failed to create template dir");
        tokio::fs::write(template_dir.join("main.tf"), "# Main template")
            .await
            .expect("Failed to write test template");

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            temp_dir.path(),
            ssh_credentials,
            test_instance_name(),
        );

        // Copy the same file twice - should succeed (overwrite)
        let result = renderer
            .copy_templates(&["main.tf", "main.tf"], &build_path)
            .await;

        assert!(
            result.is_ok(),
            "Should handle duplicate files by overwriting"
        );
        assert!(
            build_path.join("main.tf").exists(),
            "File should exist after duplicate copy"
        );
    }

    // Async Operation Edge Case Tests
    #[tokio::test]
    async fn it_should_handle_concurrent_renderer_operations() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path1 = temp_dir.path().join("build1");
        let build_path2 = temp_dir.path().join("build2");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates dir");

        // Create test template files
        let template_dir = temp_dir.path().join("tofu/lxd");
        tokio::fs::create_dir_all(&template_dir)
            .await
            .expect("Failed to create template dir");
        tokio::fs::write(template_dir.join("test1.tf"), "# Test template 1")
            .await
            .expect("Failed to write test template 1");
        tokio::fs::write(template_dir.join("test2.tf"), "# Test template 2")
            .await
            .expect("Failed to write test template 2");

        let ssh_credentials1 = create_dummy_ssh_credentials(temp_dir.path());
        let renderer1 = TofuTemplateRenderer::new(
            template_manager.clone(),
            &build_path1,
            ssh_credentials1,
            test_instance_name(),
        );
        let ssh_credentials2 = create_dummy_ssh_credentials(temp_dir.path());
        let renderer2 = TofuTemplateRenderer::new(
            template_manager,
            &build_path2,
            ssh_credentials2,
            test_instance_name(),
        );

        tokio::fs::create_dir_all(&build_path1)
            .await
            .expect("Failed to create build path 1");
        tokio::fs::create_dir_all(&build_path2)
            .await
            .expect("Failed to create build path 2");

        // Run both operations concurrently
        let (result1, result2) = tokio::join!(
            renderer1.copy_templates(&["test1.tf"], &build_path1),
            renderer2.copy_templates(&["test2.tf"], &build_path2)
        );

        assert!(result1.is_ok(), "First concurrent operation should succeed");
        assert!(
            result2.is_ok(),
            "Second concurrent operation should succeed"
        );
        assert!(
            build_path1.join("test1.tf").exists(),
            "First template should exist"
        );
        assert!(
            build_path2.join("test2.tf").exists(),
            "Second template should exist"
        );
    }

    #[tokio::test]
    async fn it_should_handle_partial_failure_scenarios() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        tokio::fs::create_dir_all(&build_path)
            .await
            .expect("Failed to create build directory");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates dir");

        // Create only one of the two template files we'll try to copy
        let template_dir = temp_dir.path().join("tofu/lxd");
        tokio::fs::create_dir_all(&template_dir)
            .await
            .expect("Failed to create template dir");
        tokio::fs::write(template_dir.join("exists.tf"), "# Existing template")
            .await
            .expect("Failed to write existing template");

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            temp_dir.path(),
            ssh_credentials,
            test_instance_name(),
        );

        // Try to copy both existing and non-existing files
        let result = renderer
            .copy_templates(&["exists.tf", "missing.tf"], &build_path)
            .await;

        // Should fail on the missing template
        assert!(result.is_err(), "Should fail when one template is missing");

        // The first file might have been copied before the failure
        // This tests the partial failure behavior
        match result.unwrap_err() {
            ProvisionTemplateError::TemplatePathFailed {
                file_name,
                source: _,
            } => {
                assert_eq!(file_name, "missing.tf");
            }
            other => panic!("Expected TemplatePathFailed for missing file, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_handle_large_number_of_files() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        tokio::fs::create_dir_all(&build_path)
            .await
            .expect("Failed to create build directory");

        let template_manager = Arc::new(TemplateManager::new(temp_dir.path()));
        template_manager
            .ensure_templates_dir()
            .expect("Failed to ensure templates dir");

        // Create many template files
        let template_dir = temp_dir.path().join("tofu/lxd");
        tokio::fs::create_dir_all(&template_dir)
            .await
            .expect("Failed to create template dir");

        let mut file_names = Vec::new();
        for i in 0..50 {
            // Create 50 files
            let file_name = format!("template_{i}.tf");
            tokio::fs::write(template_dir.join(&file_name), format!("# Template {i}"))
                .await
                .expect("Failed to write template file");
            file_names.push(file_name);
        }

        let ssh_credentials = create_dummy_ssh_credentials(temp_dir.path());
        let renderer = TofuTemplateRenderer::new(
            template_manager,
            temp_dir.path(),
            ssh_credentials,
            test_instance_name(),
        );

        let file_refs: Vec<&str> = file_names.iter().map(std::string::String::as_str).collect();
        let result = renderer.copy_templates(&file_refs, &build_path).await;

        assert!(result.is_ok(), "Should handle large number of files");

        // Verify all files were copied
        for file_name in &file_names {
            assert!(
                build_path.join(file_name).exists(),
                "File {file_name} should exist"
            );
        }
    }
}
