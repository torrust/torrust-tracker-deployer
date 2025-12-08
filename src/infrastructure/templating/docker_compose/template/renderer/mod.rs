//! # Docker Compose Template Renderer
//!
//! This module handles Docker Compose template rendering for deployment workflows.
//! It manages the creation of build directories and copying static template files
//! (docker-compose.yml) to the build directory.
//!
//! ## Design Decision
//!
//! Unlike Ansible and Tofu, Docker Compose files are typically used as static files,
//! with runtime configuration handled via environment variables. Docker Compose
//! supports environment variable substitution natively:
//!
//! - `.env` file auto-loaded from the same directory
//! - `${VAR:-default}` syntax for variable substitution
//! - `--env-file` flag at runtime
//!
//! Therefore, we use a simpler renderer that copies files as-is rather than
//! processing Tera templates. This keeps the implementation simple and follows
//! Docker Compose conventions.
//!
//! ## Template System Integration
//!
//! This renderer integrates with the embedded template system:
//! - Templates are embedded in the binary at compile time
//! - On first use, templates are extracted to the environment's templates directory
//! - Templates are then copied from the extracted location to the build directory
//!
//! See `docs/technical/template-system-architecture.md` for details on the
//! double-indirection pattern used by the template system.
//!
//! ## Key Features
//!
//! - **Static file copying**: Handles Docker Compose files that don't need Tera templating
//! - **Embedded template extraction**: Extracts templates from binary on-demand
//! - **Structured error handling**: Provides specific error types with detailed context
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring
//! - **Testable design**: Modular structure that allows for comprehensive unit testing
//!
//! ## Usage
//!
//! ```rust,no_run
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::renderer::DockerComposeTemplateRenderer;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//!
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = DockerComposeTemplateRenderer::new(template_manager, temp_dir.path());
//!
//! // Render (copy) templates to build directory
//! let build_compose_dir = renderer.render().await?;
//! # Ok(())
//! # }
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, info, trace};

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::shared::{ErrorKind, Traceable};

/// Renders Docker Compose templates to a build directory
///
/// This renderer is responsible for preparing Docker Compose templates for deployment
/// workflows. Currently, it handles static files that are copied as-is to the build
/// directory. If dynamic Tera templates are needed in the future (e.g., for dynamic
/// service definitions), this renderer can be extended to handle them.
pub struct DockerComposeTemplateRenderer {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl DockerComposeTemplateRenderer {
    /// The docker-compose.yml filename
    const COMPOSE_FILE: &'static str = "docker-compose.yml";

    /// Default relative path for Docker Compose configuration files
    const DOCKER_COMPOSE_BUILD_PATH: &'static str = "docker-compose";

    /// Template path prefix for docker-compose templates (relative to templates root)
    const DOCKER_COMPOSE_TEMPLATE_PATH: &'static str = "docker-compose";

    /// Creates a new Docker Compose template renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager for extracting embedded templates
    /// * `build_dir` - The destination build directory
    #[must_use]
    pub fn new<P: AsRef<Path>>(template_manager: Arc<TemplateManager>, build_dir: P) -> Self {
        Self {
            template_manager,
            build_dir: build_dir.as_ref().to_path_buf(),
        }
    }

    /// Renders Docker Compose templates to the build directory
    ///
    /// This method:
    /// 1. Creates the docker-compose subdirectory in the build directory
    /// 2. Extracts the docker-compose.yml from embedded templates (if not already extracted)
    /// 3. Copies the docker-compose.yml from extracted templates to build directory
    ///
    /// # Returns
    ///
    /// Returns the path to the build docker-compose directory on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Template extraction fails
    /// - File copying fails
    pub async fn render(&self) -> Result<PathBuf, DockerComposeTemplateError> {
        info!(
            template_type = "docker_compose",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Docker Compose templates"
        );

        // Create build directory structure
        let build_compose_dir = self.create_build_directory().await?;

        // Copy static Docker Compose files
        self.copy_static_templates(&build_compose_dir).await?;

        info!(
            template_type = "docker_compose",
            output_dir = %build_compose_dir.display(),
            status = "complete",
            "Docker Compose templates rendered successfully"
        );

        Ok(build_compose_dir)
    }

    /// Builds the full Docker Compose build directory path
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The complete path to the Docker Compose build directory
    fn build_docker_compose_directory(&self) -> PathBuf {
        self.build_dir.join(Self::DOCKER_COMPOSE_BUILD_PATH)
    }

    /// Builds the template path for a specific file in the Docker Compose template directory
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the template file
    ///
    /// # Returns
    ///
    /// * `String` - The complete template path for the specified file
    fn build_template_path(file_name: &str) -> String {
        format!("{}/{file_name}", Self::DOCKER_COMPOSE_TEMPLATE_PATH)
    }

    /// Creates the Docker Compose build directory structure
    ///
    /// # Returns
    ///
    /// * `Result<PathBuf, DockerComposeTemplateError>` - The created build directory path or an error
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    async fn create_build_directory(&self) -> Result<PathBuf, DockerComposeTemplateError> {
        let build_compose_dir = self.build_docker_compose_directory();

        debug!(
            directory = %build_compose_dir.display(),
            "Creating Docker Compose build directory"
        );

        tokio::fs::create_dir_all(&build_compose_dir)
            .await
            .map_err(
                |source| DockerComposeTemplateError::DirectoryCreationFailed {
                    directory: build_compose_dir.display().to_string(),
                    source,
                },
            )?;

        trace!(
            directory = %build_compose_dir.display(),
            "Docker Compose build directory created"
        );

        Ok(build_compose_dir)
    }

    /// Copies static Docker Compose template files that don't require variable substitution
    ///
    /// # Arguments
    ///
    /// * `destination_dir` - Directory where static files will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), DockerComposeTemplateError>` - Success or error from file copying operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide required template paths
    /// - File copying fails for any of the specified files
    async fn copy_static_templates(
        &self,
        destination_dir: &Path,
    ) -> Result<(), DockerComposeTemplateError> {
        debug!("Copying static Docker Compose template files");

        // Copy docker-compose.yml
        self.copy_static_file(Self::COMPOSE_FILE, destination_dir)
            .await?;

        debug!(
            "Successfully copied {} static template files",
            1 // docker-compose.yml
        );

        Ok(())
    }

    /// Copies a single static template file from template manager to destination
    ///
    /// This method uses the `TemplateManager` to get the template path, which will
    /// extract the template from embedded resources if it doesn't already exist.
    ///
    /// # Arguments
    ///
    /// * `file_name` - Name of the file to copy (without path prefix)
    /// * `destination_dir` - Directory where the file will be copied
    ///
    /// # Returns
    ///
    /// * `Result<(), DockerComposeTemplateError>` - Success or error from the file copying operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template manager cannot provide the template path
    /// - File copying fails
    async fn copy_static_file(
        &self,
        file_name: &str,
        destination_dir: &Path,
    ) -> Result<(), DockerComposeTemplateError> {
        let template_path = Self::build_template_path(file_name);
        let dest_path = destination_dir.join(file_name);

        debug!(
            template_path = %template_path,
            destination = %dest_path.display(),
            "Copying static file from extracted templates"
        );

        // Get the template path (extracts from embedded resources if needed)
        let source_path = self
            .template_manager
            .get_template_path(&template_path)
            .map_err(|source| DockerComposeTemplateError::TemplatePathFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        trace!(
            source = %source_path.display(),
            destination = %dest_path.display(),
            "Template extracted, copying to build directory"
        );

        // Copy the file
        tokio::fs::copy(&source_path, &dest_path)
            .await
            .map_err(|source| DockerComposeTemplateError::StaticFileCopyFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        debug!("Successfully copied static file {}", file_name);
        Ok(())
    }
}

/// Errors that can occur during Docker Compose template rendering
#[derive(Debug, Error)]
pub enum DockerComposeTemplateError {
    /// Failed to create the build directory
    #[error("Failed to create Docker Compose build directory '{directory}': {source}")]
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
}

impl DockerComposeTemplateError {
    /// Returns troubleshooting help for this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DirectoryCreationFailed { .. } => {
                "Failed to create the Docker Compose build directory. Please check:\n\
                 1. Disk space availability\n\
                 2. Write permissions on the build directory\n\
                 3. Parent directories exist and are accessible"
            }
            Self::TemplatePathFailed { .. } => {
                "Failed to extract Docker Compose template from embedded resources. This indicates:\n\
                 1. The docker-compose template may be missing from the binary\n\
                 2. The templates directory may not be writable\n\
                 3. There may be a filesystem permission issue\n\
                 Please report this as a bug if the problem persists."
            }
            Self::StaticFileCopyFailed { .. } => {
                "Failed to copy Docker Compose file. Please check:\n\
                 1. Source file is readable\n\
                 2. Destination directory has write permissions\n\
                 3. Disk space availability"
            }
        }
    }
}

impl Traceable for DockerComposeTemplateError {
    fn trace_format(&self) -> String {
        match self {
            Self::DirectoryCreationFailed { directory, .. } => {
                format!("DockerComposeTemplateRenderer::DirectoryCreationFailed - {directory}")
            }
            Self::TemplatePathFailed { file_name, .. } => {
                format!("DockerComposeTemplateRenderer::TemplatePathFailed - {file_name}")
            }
            Self::StaticFileCopyFailed { file_name, .. } => {
                format!("DockerComposeTemplateRenderer::StaticFileCopyFailed - {file_name}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        None
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::DirectoryCreationFailed { .. } | Self::StaticFileCopyFailed { .. } => {
                ErrorKind::FileSystem
            }
            Self::TemplatePathFailed { .. } => ErrorKind::Configuration,
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::docker_compose::DOCKER_COMPOSE_SUBFOLDER;

    /// Creates a `TemplateManager` that uses the embedded templates
    ///
    /// This tests the real integration with embedded templates by creating
    /// a `TemplateManager` pointing to a temp directory where templates
    /// will be extracted on-demand.
    fn create_template_manager_with_embedded() -> (Arc<TemplateManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = Arc::new(TemplateManager::new(temp_dir.path()));
        (manager, temp_dir)
    }

    /// Helper to create a test template manager for testing
    fn create_test_template_manager() -> Arc<TemplateManager> {
        Arc::new(TemplateManager::new("/tmp/test/templates"))
    }

    #[tokio::test]
    async fn it_should_create_renderer_with_build_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let template_manager = create_test_template_manager();

        let renderer = DockerComposeTemplateRenderer::new(template_manager, &build_path);

        assert_eq!(renderer.build_dir, build_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_docker_compose_directory_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let expected_path = build_path.join("docker-compose");
        let template_manager = create_test_template_manager();

        let renderer = DockerComposeTemplateRenderer::new(template_manager, &build_path);
        let actual_path = renderer.build_docker_compose_directory();

        assert_eq!(actual_path, expected_path);
    }

    #[tokio::test]
    async fn it_should_build_correct_template_path_for_file() {
        let template_path =
            DockerComposeTemplateRenderer::build_template_path("docker-compose.yml");

        assert_eq!(template_path, "docker-compose/docker-compose.yml");
    }

    #[tokio::test]
    async fn it_should_render_docker_compose_files_from_embedded_templates() {
        let (template_manager, _templates_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let renderer = DockerComposeTemplateRenderer::new(template_manager, build_dir.path());

        let result = renderer.render().await;

        assert!(result.is_ok());
        let compose_build_dir = result.unwrap();
        assert!(compose_build_dir.join("docker-compose.yml").exists());
    }

    #[tokio::test]
    async fn it_should_create_build_directory() {
        let (template_manager, _templates_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let renderer = DockerComposeTemplateRenderer::new(template_manager, build_dir.path());

        let result = renderer.render().await;

        assert!(result.is_ok());
        let compose_build_dir = build_dir.path().join(DOCKER_COMPOSE_SUBFOLDER);
        assert!(compose_build_dir.exists());
        assert!(compose_build_dir.is_dir());
    }

    #[tokio::test]
    async fn it_should_copy_compose_file_content_from_embedded() {
        let (template_manager, templates_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let renderer =
            DockerComposeTemplateRenderer::new(template_manager.clone(), build_dir.path());

        let result = renderer.render().await;
        assert!(result.is_ok());

        // The template should have been extracted to templates_dir
        let source_content = tokio::fs::read_to_string(
            templates_dir
                .path()
                .join(DOCKER_COMPOSE_SUBFOLDER)
                .join("docker-compose.yml"),
        )
        .await
        .expect("Failed to read source");

        let dest_content = tokio::fs::read_to_string(
            build_dir
                .path()
                .join(DOCKER_COMPOSE_SUBFOLDER)
                .join("docker-compose.yml"),
        )
        .await
        .expect("Failed to read destination");

        assert_eq!(source_content, dest_content);

        // Verify it contains expected content from embedded template
        assert!(dest_content.contains("nginx:alpine"));
        assert!(dest_content.contains("demo-app"));
    }

    #[tokio::test]
    async fn it_should_create_build_directory_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let build_path = temp_dir.path().join("build");
        let (template_manager, _templates_dir) = create_template_manager_with_embedded();
        let renderer = DockerComposeTemplateRenderer::new(template_manager, &build_path);

        let result = renderer.create_build_directory().await;

        assert!(result.is_ok());
        let created_dir = result.unwrap();
        assert_eq!(created_dir, build_path.join("docker-compose"));
        assert!(created_dir.exists());
        assert!(created_dir.is_dir());
    }

    #[tokio::test]
    async fn it_should_fail_gracefully_when_build_directory_creation_fails() {
        let invalid_path = Path::new("/root/invalid/path/that/should/not/exist");
        let template_manager = create_test_template_manager();
        let renderer = DockerComposeTemplateRenderer::new(template_manager, invalid_path);

        let result = renderer.create_build_directory().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DockerComposeTemplateError::DirectoryCreationFailed { directory, .. } => {
                assert!(directory.contains("invalid"));
            }
            other => panic!("Expected DirectoryCreationFailed, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_have_correct_template_file_constants() {
        assert_eq!(
            DockerComposeTemplateRenderer::DOCKER_COMPOSE_BUILD_PATH,
            "docker-compose"
        );
        assert_eq!(
            DockerComposeTemplateRenderer::DOCKER_COMPOSE_TEMPLATE_PATH,
            "docker-compose"
        );
        assert_eq!(
            DockerComposeTemplateRenderer::COMPOSE_FILE,
            "docker-compose.yml"
        );
    }

    #[test]
    fn error_should_provide_help_for_template_path_failed() {
        let error = DockerComposeTemplateError::TemplatePathFailed {
            file_name: "docker-compose.yml".to_string(),
            source: TemplateManagerError::TemplateNotFound {
                relative_path: "docker-compose/docker-compose.yml".to_string(),
            },
        };
        let help = error.help();
        assert!(help.contains("extract Docker Compose template"));
    }

    #[test]
    fn error_should_implement_traceable() {
        let error = DockerComposeTemplateError::TemplatePathFailed {
            file_name: "docker-compose.yml".to_string(),
            source: TemplateManagerError::TemplateNotFound {
                relative_path: "docker-compose/docker-compose.yml".to_string(),
            },
        };
        assert!(error.trace_format().contains("TemplatePathFailed"));
        assert!(error.trace_source().is_none());
        assert!(matches!(error.error_kind(), ErrorKind::Configuration));
    }

    #[test]
    fn directory_creation_error_should_provide_help() {
        let error = DockerComposeTemplateError::DirectoryCreationFailed {
            directory: "/path/to/dir".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };
        let help = error.help();
        assert!(help.contains("create the Docker Compose build directory"));
    }

    #[test]
    fn static_file_copy_error_should_provide_help() {
        let error = DockerComposeTemplateError::StaticFileCopyFailed {
            file_name: "docker-compose.yml".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };
        let help = error.help();
        assert!(help.contains("copy Docker Compose file"));
    }
}
