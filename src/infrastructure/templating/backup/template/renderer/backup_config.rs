//! Backup configuration renderer
//!
//! Renders backup.conf.tera template using `BackupContext` and `BackupTemplate` wrappers.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::backup::template::wrapper::backup_config::{
    template::BackupTemplateError, BackupContext, BackupTemplate,
};

/// Errors that can occur during backup configuration rendering
#[derive(Error, Debug)]
pub enum BackupConfigRendererError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for 'backup.conf.tera': {0}")]
    TemplatePathFailed(#[from] TemplateManagerError),

    /// Failed to read template file
    #[error("Failed to read template file at '{path}': {source}")]
    TemplateReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create or render template
    #[error("Failed to process backup template: {0}")]
    TemplateProcessingFailed(#[from] BackupTemplateError),
}

/// Renders backup.conf.tera template to backup.conf configuration file
///
/// This renderer follows the Project Generator pattern:
/// 1. Loads backup.conf.tera from the template manager
/// 2. Creates a `BackupTemplate` with `BackupContext`
/// 3. Renders the template to an output file
pub struct BackupConfigRenderer {
    template_manager: Arc<TemplateManager>,
}

impl BackupConfigRenderer {
    /// Template filename for the Backup Tera template
    const BACKUP_TEMPLATE_FILE: &'static str = "backup.conf.tera";

    /// Output filename for the rendered Backup config file
    const BACKUP_OUTPUT_FILE: &'static str = "backup.conf";

    /// Directory path for Backup templates
    const BACKUP_TEMPLATE_DIR: &'static str = "backup";

    /// Creates a new backup config renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to load templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the backup configuration to a file
    ///
    /// # Arguments
    ///
    /// * `context` - The rendering context with database and retention settings
    /// * `output_dir` - Directory where backup.conf will be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be loaded
    /// - Template file cannot be read
    /// - Template rendering fails
    /// - Output file cannot be written
    #[instrument(skip(self, context), fields(output_dir = %output_dir.display()))]
    pub fn render(
        &self,
        context: &BackupContext,
        output_dir: &Path,
    ) -> Result<(), BackupConfigRendererError> {
        // 1. Load template from template manager
        let template_path = self.template_manager.get_template_path(&format!(
            "{}/{}",
            Self::BACKUP_TEMPLATE_DIR,
            Self::BACKUP_TEMPLATE_FILE
        ))?;

        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            BackupConfigRendererError::TemplateReadFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 2. Create template with context
        let template = BackupTemplate::new(template_content, context.clone())?;

        // 3. Render to output file
        let output_path = output_dir.join(Self::BACKUP_OUTPUT_FILE);
        template.render_to_file(&output_path)?;

        tracing::debug!(
            output_file = %output_path.display(),
            "Backup configuration rendered successfully"
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

    /// Creates a `TemplateManager` that uses the embedded templates
    fn create_template_manager_with_embedded() -> (Arc<TemplateManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = Arc::new(TemplateManager::new(temp_dir.path()));
        (manager, temp_dir)
    }

    #[test]
    fn it_should_render_backup_config_with_sqlite() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let renderer = BackupConfigRenderer::new(template_manager);

        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/storage/tracker/lib/tracker.db".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);

        let output_dir = TempDir::new().expect("Failed to create temp output dir");

        let result = renderer.render(&context, output_dir.path());

        assert!(result.is_ok());

        let output_file = output_dir.path().join("backup.conf");
        assert!(output_file.exists());

        let file_content = std::fs::read_to_string(output_file).expect("Failed to read output");
        assert!(file_content.contains("BACKUP_RETENTION_DAYS=7"));
        assert!(file_content.contains("DB_TYPE=sqlite"));
        assert!(file_content.contains("DB_PATH=/data/storage/tracker/lib/tracker.db"));
    }

    #[test]
    fn it_should_render_backup_config_with_mysql() {
        let (template_manager, _temp_dir) = create_template_manager_with_embedded();
        let renderer = BackupConfigRenderer::new(template_manager);

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

        let output_dir = TempDir::new().expect("Failed to create temp output dir");

        let result = renderer.render(&context, output_dir.path());

        assert!(result.is_ok());

        let output_file = output_dir.path().join("backup.conf");
        assert!(output_file.exists());

        let file_content = std::fs::read_to_string(output_file).expect("Failed to read output");
        assert!(file_content.contains("BACKUP_RETENTION_DAYS=14"));
        assert!(file_content.contains("DB_TYPE=mysql"));
        assert!(file_content.contains("DB_HOST=mysql"));
        assert!(file_content.contains("DB_USER=tracker_user"));
        assert!(file_content.contains("DB_PASSWORD=tracker_password"));
    }
}
