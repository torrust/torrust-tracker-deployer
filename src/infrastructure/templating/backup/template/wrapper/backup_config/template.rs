//! Backup template wrapper
//!
//! Wraps the backup.conf.tera template file with its context for rendering.

use std::path::Path;

use tera::Tera;
use thiserror::Error;

use super::context::BackupContext;

/// Errors that can occur during backup template operations
#[derive(Error, Debug)]
pub enum BackupTemplateError {
    /// Failed to create Tera instance
    #[error("Failed to create Tera template engine: {0}")]
    TeraCreationFailed(#[from] tera::Error),

    /// Failed to render template
    #[error("Failed to render backup template: {0}")]
    RenderingFailed(String),

    /// Failed to write rendered content to file
    #[error("Failed to write backup configuration to '{path}': {source}")]
    WriteFileFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Wrapper for backup.conf template with rendering context
///
/// This type encapsulates the backup configuration template and provides
/// methods to render it with the given context.
pub struct BackupTemplate {
    /// The template content
    content: String,
    /// The rendering context
    context: BackupContext,
}

impl BackupTemplate {
    /// Creates a new backup template with the given content and context
    ///
    /// # Arguments
    ///
    /// * `content` - The raw template content (backup.conf.tera)
    /// * `context` - The rendering context
    ///
    /// # Errors
    ///
    /// Returns an error if the template content is invalid Tera syntax
    pub fn new(
        template_content: String,
        context: BackupContext,
    ) -> Result<Self, BackupTemplateError> {
        // Validate template syntax by attempting to create a Tera instance
        let mut tera = Tera::default();
        tera.add_raw_template("backup.conf", &template_content)?;

        Ok(Self {
            content: template_content,
            context,
        })
    }

    /// Renders the template with the context
    ///
    /// # Returns
    ///
    /// The rendered template content as a String
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    pub fn render(&self) -> Result<String, BackupTemplateError> {
        let mut tera = Tera::default();
        tera.add_raw_template("backup.conf", &self.content)
            .map_err(|e| BackupTemplateError::RenderingFailed(e.to_string()))?;

        let context = tera::Context::from_serialize(&self.context)
            .map_err(|e| BackupTemplateError::RenderingFailed(e.to_string()))?;

        tera.render("backup.conf", &context)
            .map_err(|e| BackupTemplateError::RenderingFailed(e.to_string()))
    }

    /// Renders the template and writes it to a file
    ///
    /// # Arguments
    ///
    /// * `output_path` - The path where the rendered configuration will be written
    ///
    /// # Errors
    ///
    /// Returns an error if rendering or file writing fails
    pub fn render_to_file(&self, output_path: &Path) -> Result<(), BackupTemplateError> {
        let content = self.render()?;
        std::fs::write(output_path, content).map_err(|source| {
            BackupTemplateError::WriteFileFailed {
                path: output_path.display().to_string(),
                source,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::templating::backup::BackupDatabaseConfig;
    use crate::infrastructure::templating::TemplateMetadata;
    use chrono::TimeZone;
    use chrono::Utc;

    fn create_test_template() -> String {
        r#"# Backup Configuration
BACKUP_RETENTION_DAYS={{ retention_days }}
{%- if type == "mysql" %}
DB_TYPE=mysql
DB_HOST={{ host }}
{%- else %}
DB_TYPE=sqlite
DB_PATH={{ path }}
{%- endif %}
"#
        .to_string()
    }

    #[test]
    fn it_should_create_backup_template_with_valid_content() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/tracker.db".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);
        let template_content = create_test_template();

        let result = BackupTemplate::new(template_content, context);

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_reject_invalid_tera_syntax() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/tracker.db".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);
        let invalid_template = "{{ unclosed".to_string();

        let result = BackupTemplate::new(invalid_template, context);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_render_template_with_mysql_config() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Mysql {
            host: "mysql".to_string(),
            port: 3306,
            database: "tracker".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);
        let template_content = create_test_template();
        let template = BackupTemplate::new(template_content, context).expect("valid template");

        let rendered = template.render().expect("should render");

        assert!(rendered.contains("BACKUP_RETENTION_DAYS=7"));
        assert!(rendered.contains("DB_TYPE=mysql"));
        assert!(rendered.contains("DB_HOST=mysql"));
    }

    #[test]
    fn it_should_render_template_with_sqlite_config() {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        let metadata = TemplateMetadata::new(timestamp);
        let db_config = BackupDatabaseConfig::Sqlite {
            path: "/data/tracker.db".to_string(),
        };
        let context = BackupContext::new(metadata, 7, db_config);
        let template_content = create_test_template();
        let template = BackupTemplate::new(template_content, context).expect("valid template");

        let rendered = template.render().expect("should render");

        assert!(rendered.contains("BACKUP_RETENTION_DAYS=7"));
        assert!(rendered.contains("DB_TYPE=sqlite"));
        assert!(rendered.contains("DB_PATH=/data/tracker.db"));
    }
}
