//! Backup template rendering service
//!
//! This service handles rendering of backup configuration templates,
//! including database configuration conversion and schedule handling.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{info, instrument};

use crate::domain::backup::BackupConfig;
use crate::domain::tracker::DatabaseConfig;
use crate::domain::TemplateManager;
use crate::infrastructure::templating::backup::template::wrapper::backup_config::context::{
    BackupContext, BackupDatabaseConfig,
};
use crate::infrastructure::templating::backup::{
    BackupProjectGenerator, BackupProjectGeneratorError,
};
use crate::infrastructure::templating::TemplateMetadata;

/// Service for rendering backup configuration templates
///
/// This service encapsulates the logic for rendering backup configurations,
/// including:
/// - Converting domain `DatabaseConfig` to template `BackupDatabaseConfig`
/// - Building `BackupContext` with schedule information
/// - Conditional rendering (only when backup is configured)
pub struct BackupTemplateRenderingService {
    templates_dir: PathBuf,
    build_dir: PathBuf,
}

impl BackupTemplateRenderingService {
    /// Create a new service with explicit dependencies
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing template source files
    /// * `build_dir` - Directory where rendered templates will be written
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf) -> Self {
        Self {
            templates_dir,
            build_dir,
        }
    }

    /// Render backup templates if backup is configured
    ///
    /// This method converts the domain database configuration to the backup
    /// format and renders the backup configuration templates. Returns `None`
    /// if backup is not configured.
    ///
    /// # Arguments
    ///
    /// * `backup_config` - Optional backup configuration
    /// * `database_config` - Tracker database configuration
    /// * `created_at` - Environment creation timestamp
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` with path to the rendered backup build directory if
    /// backup is configured, or `None` if backup should not be deployed.
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails
    #[instrument(
        name = "backup_rendering_service",
        skip_all,
        fields(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display()
        )
    )]
    pub async fn render(
        &self,
        backup_config: Option<&BackupConfig>,
        database_config: &DatabaseConfig,
        created_at: DateTime<Utc>,
    ) -> Result<Option<PathBuf>, BackupTemplateRenderingServiceError> {
        // Check if backup configuration exists
        let Some(backup_config) = backup_config else {
            info!(
                reason = "backup_not_configured",
                "Skipping backup template rendering - backup not configured"
            );
            return Ok(None);
        };

        info!(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            "Rendering backup configuration templates"
        );

        let template_manager = Arc::new(TemplateManager::new(self.templates_dir.clone()));
        let generator = BackupProjectGenerator::new(self.build_dir.clone(), template_manager);

        let backup_database_config = convert_database_config_to_backup(database_config);
        let metadata = TemplateMetadata::new(created_at);
        let context = BackupContext::from_config(metadata, backup_config, backup_database_config);

        generator
            .render(&context, backup_config.schedule())
            .await
            .map_err(BackupTemplateRenderingServiceError::RenderingFailed)?;

        let backup_dir_path = self.build_dir.join("backup/etc");

        info!(
            backup_dir_path = %backup_dir_path.display(),
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
                "/data/storage/tracker/lib/database/{}",
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

/// Errors that can occur during backup template rendering
#[derive(Debug, thiserror::Error)]
pub enum BackupTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Backup template rendering failed: {0}")]
    RenderingFailed(#[from] BackupProjectGeneratorError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::domain::backup::BackupConfig;
    use crate::domain::tracker::{DatabaseConfig, SqliteConfig};

    #[tokio::test]
    async fn it_should_create_service_with_from_paths() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");

        let service = BackupTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        assert_eq!(service.templates_dir, templates_dir.path());
        assert_eq!(service.build_dir, build_dir.path());
    }

    #[tokio::test]
    async fn it_should_return_none_when_backup_not_configured() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");

        let service = BackupTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        let database_config = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());
        let created_at = Utc::now();

        let result = service.render(None, &database_config, created_at).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_should_render_backup_templates_when_backup_is_configured() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");

        let service = BackupTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
        );

        let backup_config = BackupConfig::default();
        let database_config = DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap());
        let created_at = Utc::now();

        let result = service
            .render(Some(&backup_config), &database_config, created_at)
            .await;

        assert!(result.is_ok());
        let backup_dir = result.unwrap();
        assert!(backup_dir.is_some());
        let backup_dir = backup_dir.unwrap();
        assert!(backup_dir.to_string_lossy().contains("backup/etc"));
    }

    #[test]
    fn it_should_convert_sqlite_config_to_backup_format() {
        let sqlite_config = SqliteConfig::new("tracker.db").unwrap();
        let database_config = DatabaseConfig::Sqlite(sqlite_config.clone());

        let backup_config = convert_database_config_to_backup(&database_config);

        match backup_config {
            BackupDatabaseConfig::Sqlite { path } => {
                assert!(path.contains(sqlite_config.database_name()));
            }
            BackupDatabaseConfig::Mysql { .. } => {
                panic!("Expected Sqlite config");
            }
        }
    }
}
