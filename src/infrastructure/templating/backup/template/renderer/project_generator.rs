//! Backup Project Generator
//!
//! Orchestrates backup template rendering for deployment workflows.
//! Handles both dynamic templates (.tera) and static files (backup-paths.txt).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::backup::template::renderer::backup_config::{
    BackupConfigRenderer, BackupConfigRendererError,
};
use crate::infrastructure::templating::backup::template::wrapper::BackupContext;

/// Errors that can occur during backup project generation
#[derive(Error, Debug)]
pub enum BackupProjectGeneratorError {
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

    /// Failed to render backup.conf template
    #[error("Failed to render backup configuration: {source}")]
    BackupConfigRenderingFailed {
        #[source]
        source: BackupConfigRendererError,
    },
}

/// Renders backup templates to a build directory
///
/// This orchestrator is responsible for preparing backup templates for deployment.
/// It handles:
/// - Dynamic template rendering (backup.conf.tera with variables)
/// - Static file copying (backup-paths.txt)
pub struct BackupProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    backup_config_renderer: BackupConfigRenderer,
}

impl BackupProjectGenerator {
    /// Default relative path for backup configuration files
    const BACKUP_BUILD_PATH: &'static str = "backup/etc";

    /// Default template path prefix for backup templates
    const BACKUP_TEMPLATE_PATH: &'static str = "backup";

    /// Creates a new backup project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let backup_config_renderer = BackupConfigRenderer::new(template_manager.clone());

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            template_manager,
            backup_config_renderer,
        }
    }

    /// Renders backup templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for backup
    /// 2. Renders dynamic Tera templates with runtime variables (backup.conf.tera)
    /// 3. Copies static templates (backup-paths.txt)
    /// 4. Provides debug logging via the tracing crate
    ///
    /// # Arguments
    ///
    /// * `context` - Runtime context for backup template rendering (retention, database config)
    ///
    /// # Returns
    ///
    /// * `Result<(), BackupProjectGeneratorError>` - Success or error from the template rendering operation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Template copying fails
    /// - Template manager cannot provide required templates
    /// - Dynamic template rendering fails
    /// - Runtime variable substitution fails
    pub async fn render(&self, context: &BackupContext) -> Result<(), BackupProjectGeneratorError> {
        tracing::info!(template_type = "backup", "Rendering backup templates");

        // Create build directory structure
        let build_backup_dir = self.create_build_directory().await?;

        // Render dynamic backup.conf template with runtime variables using renderer
        self.backup_config_renderer
            .render(context, &build_backup_dir)
            .map_err(
                |source| BackupProjectGeneratorError::BackupConfigRenderingFailed { source },
            )?;

        // Copy static backup-paths.txt file
        self.copy_static_templates(&self.template_manager, &build_backup_dir)
            .await?;

        tracing::debug!(
            template_type = "backup",
            output_dir = %build_backup_dir.display(),
            "Backup templates rendered"
        );

        tracing::info!(
            template_type = "backup",
            status = "complete",
            "Backup templates ready"
        );
        Ok(())
    }

    /// Builds the full backup build directory path
    fn build_backup_directory(&self) -> PathBuf {
        self.build_dir.join(Self::BACKUP_BUILD_PATH)
    }

    /// Builds the template path for a specific file in the backup template directory
    fn build_template_path(file_name: &str) -> String {
        format!("{}/{file_name}", Self::BACKUP_TEMPLATE_PATH)
    }

    /// Creates the backup build directory structure
    async fn create_build_directory(&self) -> Result<PathBuf, BackupProjectGeneratorError> {
        let build_backup_dir = self.build_backup_directory();
        tokio::fs::create_dir_all(&build_backup_dir)
            .await
            .map_err(
                |source| BackupProjectGeneratorError::DirectoryCreationFailed {
                    directory: build_backup_dir.display().to_string(),
                    source,
                },
            )?;
        Ok(build_backup_dir)
    }

    /// Copies static backup template files that don't require variable substitution
    ///
    /// Currently only copies backup-paths.txt which contains the static list of files to backup.
    async fn copy_static_templates(
        &self,
        template_manager: &TemplateManager,
        destination_dir: &Path,
    ) -> Result<(), BackupProjectGeneratorError> {
        tracing::debug!("Copying static backup template files");

        // Copy backup paths list
        self.copy_static_file(template_manager, "backup-paths.txt", destination_dir)
            .await?;

        tracing::debug!("Successfully copied 1 static template file");

        Ok(())
    }

    /// Copies a single static template file from template manager to destination
    async fn copy_static_file(
        &self,
        template_manager: &TemplateManager,
        file_name: &str,
        destination_dir: &Path,
    ) -> Result<(), BackupProjectGeneratorError> {
        let template_path = template_manager
            .get_template_path(&Self::build_template_path(file_name))
            .map_err(|source| BackupProjectGeneratorError::TemplatePathFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        let destination_path = destination_dir.join(file_name);
        tokio::fs::copy(&template_path, &destination_path)
            .await
            .map_err(|source| BackupProjectGeneratorError::StaticFileCopyFailed {
                file_name: file_name.to_string(),
                source,
            })?;

        tracing::debug!(
            source = %template_path.display(),
            destination = %destination_path.display(),
            "Copied static template file"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::backup::BackupDatabaseConfig;
    use crate::infrastructure::templating::TemplateMetadata;
    use chrono::TimeZone;
    use chrono::Utc;

    fn create_template_manager_with_embedded() -> (Arc<TemplateManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = Arc::new(TemplateManager::new(temp_dir.path()));
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_with_sqlite() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");
        let generator = BackupProjectGenerator::new(build_dir.path(), template_manager);

        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/storage/tracker/lib/tracker.db".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);

        let result = generator.render(&context).await;

        assert!(result.is_ok());

        // Verify backup.conf was rendered
        let backup_conf = build_dir.path().join("backup/etc/backup.conf");
        assert!(backup_conf.exists());
        let file_content =
            std::fs::read_to_string(backup_conf).expect("Failed to read backup.conf");
        assert!(file_content.contains("DB_TYPE=sqlite"));

        // Verify backup-paths.txt was copied
        let backup_paths = build_dir.path().join("backup/etc/backup-paths.txt");
        assert!(backup_paths.exists());
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_with_mysql() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let build_dir = TempDir::new().expect("Failed to create build directory");
        let generator = BackupProjectGenerator::new(build_dir.path(), template_manager);

        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Mysql {
            host: "mysql".to_string(),
            port: 3306,
            database: "torrust_tracker".to_string(),
            user: "tracker_user".to_string(),
            password: "tracker_password".to_string(),
        };
        let context = BackupContext::new(metadata, 14, db_config);

        let result = generator.render(&context).await;

        assert!(result.is_ok());

        let backup_conf = build_dir.path().join("backup/etc/backup.conf");
        assert!(backup_conf.exists());
        let file_content =
            std::fs::read_to_string(backup_conf).expect("Failed to read backup.conf");
        assert!(file_content.contains("DB_TYPE=mysql"));
        assert!(file_content.contains("DB_HOST=mysql"));
    }
}
