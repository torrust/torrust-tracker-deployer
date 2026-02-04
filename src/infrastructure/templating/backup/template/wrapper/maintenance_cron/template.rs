//! Maintenance cron template wrapper
//!
//! Wraps the maintenance-backup.cron.tera template file with its context for rendering.

use std::path::Path;

use tera::Tera;
use thiserror::Error;

use super::context::MaintenanceCronContext;

/// Errors that can occur during maintenance cron template operations
#[derive(Error, Debug)]
pub enum MaintenanceCronTemplateError {
    /// Failed to create Tera instance
    #[error("Failed to create Tera template engine: {0}")]
    TeraCreationFailed(#[from] tera::Error),

    /// Failed to render template
    #[error("Failed to render maintenance cron template: {0}")]
    RenderingFailed(String),

    /// Failed to write rendered content to file
    #[error("Failed to write maintenance cron configuration to '{path}': {source}")]
    WriteFileFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Wrapper for maintenance-backup.cron template with rendering context
///
/// This type encapsulates the maintenance cron template and provides
/// methods to render it with the given context.
pub struct MaintenanceCronTemplate {
    /// The template content
    content: String,
    /// The rendering context
    context: MaintenanceCronContext,
}

impl MaintenanceCronTemplate {
    /// Creates a new maintenance cron template with the given content and context
    ///
    /// # Arguments
    ///
    /// * `content` - The raw template content (maintenance-backup.cron.tera)
    /// * `ctx` - The rendering context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tera template engine cannot be created
    /// - Template syntax is invalid
    pub fn new(
        content: String,
        ctx: MaintenanceCronContext,
    ) -> Result<Self, MaintenanceCronTemplateError> {
        // Validate template by creating Tera engine
        let _tera = Tera::new("/dev/null/*")?;

        Ok(Self {
            content,
            context: ctx,
        })
    }

    /// Renders the template to a file
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the rendered template should be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template rendering fails
    /// - Output file cannot be written
    pub fn render_to_file(&self, output_path: &Path) -> Result<(), MaintenanceCronTemplateError> {
        // Create Tera engine
        let mut tera = Tera::default();

        // Add the template
        tera.add_raw_template("maintenance-backup.cron", &self.content)?;

        // Render template with context
        let rendered = tera
            .render(
                "maintenance-backup.cron",
                &tera::Context::from_serialize(&self.context)?,
            )
            .map_err(|e| MaintenanceCronTemplateError::RenderingFailed(e.to_string()))?;

        // Write to file
        std::fs::write(output_path, rendered).map_err(|source| {
            MaintenanceCronTemplateError::WriteFileFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::backup::CronSchedule;

    #[test]
    fn it_should_create_template_with_valid_content() {
        let schedule = CronSchedule::default();
        let ctx = MaintenanceCronContext::new(&schedule);
        let template_content = "# Cron schedule: {{ schedule }}".to_string();

        let template = MaintenanceCronTemplate::new(template_content, ctx);

        assert!(template.is_ok());
    }

    #[test]
    fn it_should_render_template_to_file() {
        let schedule = CronSchedule::default();
        let ctx = MaintenanceCronContext::new(&schedule);
        let template_content =
            "# Cron schedule: {{ schedule }}\n# Run: /opt/torrust/maintenance-backup.sh"
                .to_string();

        let template =
            MaintenanceCronTemplate::new(template_content, ctx).expect("Failed to create template");

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("maintenance-backup.cron");

        let result = template.render_to_file(&output_path);

        assert!(result.is_ok());
        assert!(output_path.exists());

        let file_text = std::fs::read_to_string(&output_path).expect("Failed to read output");
        assert!(file_text.contains("0 3 * * *"));
        assert!(file_text.contains("/opt/torrust/maintenance-backup.sh"));
    }
}
