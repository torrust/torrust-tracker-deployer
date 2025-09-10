use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::template::{TemplateManager, TemplateManagerError};

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
}

/// Renders `OpenTofu` provision templates to a build directory
///
/// This collaborator is responsible for preparing `OpenTofu` templates for the provision stage
/// of the deployment pipeline. It copies static templates from the template manager to the
/// specified build directory.
pub struct ProvisionTemplateRenderer {
    build_dir: PathBuf,
    verbose: bool,
}

impl ProvisionTemplateRenderer {
    /// Default relative path for `OpenTofu` configuration files
    const OPENTOFU_BUILD_PATH: &'static str = "tofu/lxd";

    /// Default template path prefix for `OpenTofu` templates
    const OPENTOFU_TEMPLATE_PATH: &'static str = "tofu/lxd";

    /// Creates a new provision template renderer
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `verbose` - Whether to enable verbose logging
    pub fn new<P: AsRef<Path>>(build_dir: P, verbose: bool) -> Self {
        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            verbose,
        }
    }

    /// Renders provision templates (`OpenTofu`) to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for `OpenTofu`
    /// 2. Copies static templates (main.tf, cloud-init.yml) from the template manager
    /// 3. Provides verbose logging if enabled
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to source templates from
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
    pub async fn render(
        &self,
        template_manager: &TemplateManager,
    ) -> Result<(), ProvisionTemplateError> {
        tracing::info!("🏗️  Stage 1: Rendering provision templates to build directory...");

        // Create build directory structure
        let build_tofu_dir = self.create_build_directory().await?;

        // List of templates to copy for the provision stage
        let template_files = vec!["main.tf", "cloud-init.yml"];

        // Copy all template files
        self.copy_templates(template_manager, &template_files, &build_tofu_dir)
            .await?;

        if self.verbose {
            tracing::debug!(
                "   ✅ Provision templates copied to: {}",
                build_tofu_dir.display()
            );
        }

        tracing::info!("✅ Stage 1 complete: Provision templates ready");
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
    /// * `template_manager` - The template manager to source templates from
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
        template_manager: &TemplateManager,
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

            let source_path =
                template_manager
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn it_should_create_renderer_with_build_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        let renderer = ProvisionTemplateRenderer::new(&build_path, false);

        assert_eq!(renderer.build_dir, build_path);
        assert!(!renderer.verbose);
    }

    #[tokio::test]
    async fn it_should_create_renderer_with_verbose_logging() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");

        let renderer = ProvisionTemplateRenderer::new(&build_path, true);

        assert!(renderer.verbose);
    }

    #[tokio::test]
    async fn it_should_build_correct_opentofu_directory_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("tofu/lxd");

        let renderer = ProvisionTemplateRenderer::new(&build_path, false);
        let actual_path = renderer.build_opentofu_directory();

        assert_eq!(actual_path, expected_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_template_path_for_file() {
        let template_path = ProvisionTemplateRenderer::build_template_path("main.tf");

        assert_eq!(template_path, "tofu/lxd/main.tf");
    }

    #[tokio::test]
    async fn it_should_build_template_path_with_different_file_names() {
        assert_eq!(
            ProvisionTemplateRenderer::build_template_path("cloud-init.yml"),
            "tofu/lxd/cloud-init.yml"
        );
        assert_eq!(
            ProvisionTemplateRenderer::build_template_path("variables.tf"),
            "tofu/lxd/variables.tf"
        );
        assert_eq!(
            ProvisionTemplateRenderer::build_template_path("outputs.tf"),
            "tofu/lxd/outputs.tf"
        );
    }

    #[tokio::test]
    async fn it_should_create_build_directory_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("tofu/lxd");

        let renderer = ProvisionTemplateRenderer::new(&build_path, false);
        let created_path = renderer
            .create_build_directory()
            .await
            .expect("Failed to create build directory");

        assert_eq!(created_path, expected_path);
        assert!(created_path.exists(), "Build directory should exist");
        assert!(created_path.is_dir(), "Created path should be a directory");
    }
}
