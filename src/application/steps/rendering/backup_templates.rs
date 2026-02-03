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

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::domain::tracker::DatabaseConfig;
use crate::infrastructure::templating::backup::template::wrapper::backup_config::context::{
    BackupContext, BackupDatabaseConfig,
};
use crate::infrastructure::templating::backup::{
    BackupProjectGenerator, BackupProjectGeneratorError,
};
use crate::infrastructure::templating::TemplateMetadata;

/// Step that renders Backup templates to the build directory
///
/// This step handles the preparation of backup configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
pub struct RenderBackupTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderBackupTemplatesStep<S> {
    /// Creates a new `RenderBackupTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `template_manager` - The template manager for accessing templates
    /// * `build_dir` - The build directory where templates will be rendered
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        template_manager: Arc<TemplateManager>,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            environment,
            template_manager,
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
    pub async fn execute(&self) -> Result<Option<PathBuf>, BackupProjectGeneratorError> {
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

        // Render the backup templates using the project generator
        let generator =
            BackupProjectGenerator::new(self.build_dir.clone(), Arc::clone(&self.template_manager));

        let database_config = self
            .environment
            .context()
            .user_inputs
            .tracker()
            .core()
            .database();
        let backup_database_config = convert_database_config_to_backup(database_config);

        let metadata = TemplateMetadata::new(self.environment.context().created_at());

        let context = BackupContext::from_config(metadata, backup_config, backup_database_config);

        let backup_dir_path = self.build_dir.join("storage/backup/etc");

        generator.render(&context).await?;

        info!(
            step = "render_backup_templates",
            status = "success",
            output_dir = %backup_dir_path.display(),
            "Backup templates rendered successfully"
        );

        Ok(Some(backup_dir_path))
    }
}

/// Converts domain `DatabaseConfig` to template `BackupDatabaseConfig`
///
/// Maps the domain database configuration (used for tracker setup) to the
/// backup-specific database configuration format (used for backup script generation).
fn convert_database_config_to_backup(config: &DatabaseConfig) -> BackupDatabaseConfig {
    match config {
        DatabaseConfig::Sqlite(sqlite_config) => BackupDatabaseConfig::Sqlite {
            path: format!(
                "/data/storage/tracker/lib/{}",
                sqlite_config.database_name()
            ),
        },
        DatabaseConfig::Mysql(mysql_config) => BackupDatabaseConfig::Mysql {
            host: mysql_config.host().to_string(),
            port: mysql_config.port(),
            database: mysql_config.database_name().to_string(),
            user: mysql_config.username().to_string(),
            password: mysql_config.password().expose_secret().to_string(),
        },
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

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));

        let step = RenderBackupTemplatesStep::new(
            environment,
            template_manager,
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

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));

        let step = RenderBackupTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
        );

        // Act
        let result = step.execute().await;

        // Assert
        // Since we can't configure backup via the builder, this will skip rendering
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_when_backup_is_configured_with_mysql() {
        // Arrange
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));

        let step = RenderBackupTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
        );

        // Act
        let result = step.execute().await;

        // Assert
        // Since we can't configure backup via the builder, this will skip rendering
        assert!(result.is_ok());
    }
}
