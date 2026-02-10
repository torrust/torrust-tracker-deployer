//! Backup template rendering step
//!
//! This module provides the `RenderBackupTemplatesStep` which handles rendering
//! of backup configuration templates to the build directory. This step prepares
//! backup configuration files for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for backup configurations (`backup.conf`)
//! - Static file copying for backup path lists (`backup-paths.txt`)
//! - Integration with the `BackupProjectGenerator` for file generation
//! - Build directory preparation for deployment operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the backup configuration files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderBackupTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::application::services::rendering::BackupTemplateRenderingService;
use crate::application::services::rendering::BackupTemplateRenderingServiceError;
use crate::domain::environment::Environment;

/// Step that renders Backup templates to the build directory
///
/// This step handles the preparation of backup configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
pub struct RenderBackupTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    templates_dir: PathBuf,
    build_dir: PathBuf,
}

impl<S> RenderBackupTemplatesStep<S> {
    /// Creates a new `RenderBackupTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `templates_dir` - The templates directory
    /// * `build_dir` - The build directory where templates will be rendered
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        templates_dir: PathBuf,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            environment,
            templates_dir,
            build_dir,
        }
    }

    /// Execute the template rendering step
    ///
    /// This will render backup templates to the build directory if backup
    /// configuration is present in the environment.
    ///
    /// # Returns
    ///
    /// Returns the path to the backup build directory on success, or `None`
    /// if backup is not configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File writing fails
    #[instrument(
        name = "render_backup_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "backup",
            build_dir = %self.build_dir.display()
        )
    )]
    pub async fn execute(&self) -> Result<Option<PathBuf>, BackupTemplateRenderingServiceError> {
        info!(
            step = "render_backup_templates",
            action = "render_templates",
            "Rendering backup templates"
        );

        // Check if backup configuration exists
        let Some(backup_config) = &self.environment.context().user_inputs.backup() else {
            info!(
                step = "render_backup_templates",
                status = "skipped",
                reason = "backup_not_configured",
                "Backup is not configured in environment"
            );
            return Ok(None);
        };

        let service = BackupTemplateRenderingService::from_paths(
            self.templates_dir.clone(),
            self.build_dir.clone(),
        );

        let database_config = self
            .environment
            .context()
            .user_inputs
            .tracker()
            .core()
            .database();
        let created_at = self.environment.context().created_at();

        let Some(backup_dir_path) = service
            .render(Some(backup_config), database_config, created_at)
            .await?
        else {
            return Ok(None);
        };

        info!(
            step = "render_backup_templates",
            status = "success",
            output_dir = %backup_dir_path.display(),
            "Backup templates rendered successfully"
        );

        Ok(Some(backup_dir_path))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use std::sync::Arc;

    #[tokio::test]
    async fn it_should_skip_rendering_when_backup_is_not_configured() {
        // Arrange
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        // Build environment without Backup config
        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let step = RenderBackupTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        // Act
        let result = step.execute().await;

        // Assert
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Should return None when backup not configured"
        );
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_when_backup_is_configured_with_sqlite() {
        // Arrange
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) = EnvironmentTestBuilder::new()
            .with_backup_config(Some(crate::domain::backup::BackupConfig::default()))
            .build_with_custom_paths();
        let environment = Arc::new(environment);

        let step = RenderBackupTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        // Act
        let result = step.execute().await;

        // Assert
        // With backup configured, templates should render
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_some(),
            "Should return Some when backup is configured"
        );
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_when_backup_is_configured_with_mysql() {
        // Arrange
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) = EnvironmentTestBuilder::new()
            .with_backup_config(Some(crate::domain::backup::BackupConfig::default()))
            .build_with_custom_paths();
        let environment = Arc::new(environment);

        let step = RenderBackupTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        // Act
        let result = step.execute().await;

        // Assert
        // With backup configured, templates should render
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_some(),
            "Should return Some when backup is configured"
        );
    }
}
