//! Tracker configuration renderer
//!
//! Renders tracker.toml.tera template using `TrackerContext` and `TrackerTemplate` wrappers.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::tracker::template::wrapper::tracker_config::{
    template::TrackerTemplateError, TrackerContext, TrackerTemplate,
};

/// Errors that can occur during tracker configuration rendering
#[derive(Error, Debug)]
pub enum TrackerConfigRendererError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for 'tracker.toml.tera': {0}")]
    TemplatePathFailed(#[from] TemplateManagerError),

    /// Failed to read template file
    #[error("Failed to read template file at '{path}': {source}")]
    TemplateReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create or render template
    #[error("Failed to process tracker template: {0}")]
    TemplateProcessingFailed(#[from] TrackerTemplateError),
}

/// Renders tracker.toml.tera template to tracker.toml configuration file
///
/// This renderer follows the Project Generator pattern:
/// 1. Loads tracker.toml.tera from the template manager
/// 2. Creates a `TrackerTemplate` with `TrackerContext`
/// 3. Renders the template to an output file
///
/// ## Phase 4 Implementation
///
/// In Phase 4, the `TrackerContext` is empty and all values are hardcoded in
/// the template. The rendering process works but performs no variable substitution.
///
/// ## Phase 6 Future
///
/// In Phase 6, `TrackerContext` will contain dynamic configuration values that
/// will be substituted during rendering.
pub struct TrackerConfigRenderer {
    template_manager: Arc<TemplateManager>,
}

impl TrackerConfigRenderer {
    /// Template filename for the Tracker Tera template
    const TRACKER_TEMPLATE_FILE: &'static str = "tracker.toml.tera";

    /// Output filename for the rendered Tracker config file
    const TRACKER_OUTPUT_FILE: &'static str = "tracker.toml";

    /// Directory path for Tracker templates
    const TRACKER_TEMPLATE_DIR: &'static str = "tracker";

    /// Creates a new tracker config renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to load templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the tracker configuration to a file
    ///
    /// # Arguments
    ///
    /// * `context` - The rendering context (empty in Phase 4)
    /// * `output_dir` - Directory where tracker.toml will be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be loaded
    /// - Template file cannot be read
    /// - Template rendering fails
    /// - Output file cannot be written
    ///
    /// # Phase 4 Behavior
    ///
    /// The context is empty, so the template is rendered without variable substitution.
    #[instrument(skip(self, context), fields(output_dir = %output_dir.display()))]
    pub fn render(
        &self,
        context: &TrackerContext,
        output_dir: &Path,
    ) -> Result<(), TrackerConfigRendererError> {
        // 1. Load template from template manager
        let template_path = self
            .template_manager
            .get_template_path(&format!("{}/{}", Self::TRACKER_TEMPLATE_DIR, Self::TRACKER_TEMPLATE_FILE))?;

        // 2. Read template content
        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            TrackerConfigRendererError::TemplateReadFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 3. Create TrackerTemplate with context
        let template = TrackerTemplate::new(template_content, context.clone())?;

        // 4. Render to output file
        let output_path = output_dir.join(Self::TRACKER_OUTPUT_FILE);
        template.render_to_file(&output_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template_manager() -> Arc<TemplateManager> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let tracker_dir = templates_dir.join("tracker");

        fs::create_dir_all(&tracker_dir).expect("Failed to create tracker dir");

        let template_content = r#"[metadata]
app = "torrust-tracker"
purpose = "configuration"

[core.database]
driver = "sqlite3"
path = "/var/lib/torrust/tracker/database/sqlite3.db"
"#;

        fs::write(tracker_dir.join("tracker.toml.tera"), template_content)
            .expect("Failed to write template");

        // Prevent temp_dir from being dropped
        std::mem::forget(temp_dir);

        Arc::new(TemplateManager::new(templates_dir))
    }

    #[test]
    fn it_should_render_tracker_template_successfully() {
        let template_manager = create_test_template_manager();
        let renderer = TrackerConfigRenderer::new(template_manager);

        let temp_output = TempDir::new().expect("Failed to create output dir");
        let ctx = TrackerContext::default_config();

        let result = renderer.render(&ctx, temp_output.path());
        assert!(result.is_ok());

        let output_file = temp_output.path().join("tracker.toml");
        assert!(output_file.exists());

        let file_content = fs::read_to_string(&output_file).expect("Failed to read output");
        assert!(file_content.contains("[metadata]"));
        assert!(file_content.contains("torrust-tracker"));
    }

    #[test]
    fn it_should_render_correct_database_path() {
        let template_manager = create_test_template_manager();
        let renderer = TrackerConfigRenderer::new(template_manager);

        let temp_output = TempDir::new().expect("Failed to create output dir");
        let ctx = TrackerContext::default_config();

        renderer
            .render(&ctx, temp_output.path())
            .expect("Rendering failed");

        let output_file = temp_output.path().join("tracker.toml");
        let file_content = fs::read_to_string(&output_file).expect("Failed to read output");

        assert!(file_content.contains("/var/lib/torrust/tracker/database/sqlite3.db"));
    }

    #[test]
    fn it_should_use_embedded_template_when_external_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let empty_templates_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&empty_templates_dir).expect("Failed to create dir");

        let template_manager = Arc::new(TemplateManager::new(empty_templates_dir));
        let renderer = TrackerConfigRenderer::new(template_manager);

        let temp_output = TempDir::new().expect("Failed to create output dir");
        let context = TrackerContext::default_config();

        // Should succeed because TemplateManager extracts from embedded resources
        let result = renderer.render(&context, temp_output.path());
        assert!(
            result.is_ok(),
            "Should succeed using embedded template: {:?}",
            result.err()
        );

        let output_file = temp_output.path().join("tracker.toml");
        assert!(
            output_file.exists(),
            "tracker.toml should be created from embedded template"
        );
    }

    #[test]
    fn it_should_create_renderer_with_template_manager() {
        let template_manager = create_test_template_manager();
        let _renderer = TrackerConfigRenderer::new(template_manager);
        // Should create without panicking
    }
}
