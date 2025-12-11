//! Tracker template wrapper
//!
//! Wraps the tracker.toml.tera template file with its context for rendering.

use std::path::Path;

use tera::Tera;
use thiserror::Error;

use super::context::TrackerContext;

/// Errors that can occur during tracker template operations
#[derive(Error, Debug)]
pub enum TrackerTemplateError {
    /// Failed to create Tera instance
    #[error("Failed to create Tera template engine: {0}")]
    TeraCreationFailed(#[from] tera::Error),

    /// Failed to render template
    #[error("Failed to render tracker template: {0}")]
    RenderingFailed(String),

    /// Failed to write rendered content to file
    #[error("Failed to write tracker configuration to '{path}': {source}")]
    WriteFileFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Wrapper for tracker.toml template with rendering context
///
/// This type encapsulates the tracker configuration template and provides
/// methods to render it with the given context.
///
/// ## Phase 4 Implementation
///
/// In Phase 4, the context is empty and the template contains hardcoded values.
/// The rendering process still works but performs no variable substitution.
///
/// ## Phase 6 Future
///
/// In Phase 6, the context will contain dynamic configuration values that
/// will be substituted into the template during rendering.
pub struct TrackerTemplate {
    /// The template content
    content: String,
    /// The rendering context (empty in Phase 4)
    context: TrackerContext,
}

impl TrackerTemplate {
    /// Creates a new tracker template with the given content and context
    ///
    /// # Arguments
    ///
    /// * `content` - The raw template content (tracker.toml.tera)
    /// * `context` - The rendering context (empty in Phase 4)
    ///
    /// # Errors
    ///
    /// Returns an error if the template content is invalid Tera syntax
    pub fn new(
        template_content: String,
        context: TrackerContext,
    ) -> Result<Self, TrackerTemplateError> {
        // Validate template syntax by attempting to create a Tera instance
        // Phase 4: Template has no variables, but we still validate syntax
        let mut tera = Tera::default();
        tera.add_raw_template("tracker.toml", &template_content)?;

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
    ///
    /// # Phase 4 Behavior
    ///
    /// In Phase 4, since the context is empty and the template has no variables,
    /// this effectively returns the template content unchanged.
    pub fn render(&self) -> Result<String, TrackerTemplateError> {
        let mut tera = Tera::default();
        tera.add_raw_template("tracker.toml", &self.content)
            .map_err(|e| TrackerTemplateError::RenderingFailed(e.to_string()))?;

        let context = tera::Context::from_serialize(&self.context)
            .map_err(|e| TrackerTemplateError::RenderingFailed(e.to_string()))?;

        tera.render("tracker.toml", &context)
            .map_err(|e| TrackerTemplateError::RenderingFailed(e.to_string()))
    }

    /// Renders the template and writes it to a file
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the rendered tracker.toml should be written
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails or if writing to the file fails
    pub fn render_to_file(&self, output_path: &Path) -> Result<(), TrackerTemplateError> {
        let rendered = self.render()?;

        std::fs::write(output_path, rendered).map_err(|source| {
            TrackerTemplateError::WriteFileFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }

    /// Returns the raw template content
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns a reference to the rendering context
    #[must_use]
    pub fn context(&self) -> &TrackerContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_template_content() -> String {
        r#"[metadata]
app = "torrust-tracker"
purpose = "configuration"

[core.database]
driver = "sqlite3"
path = "/var/lib/torrust/tracker/database/sqlite3.db"
"#
        .to_string()
    }

    #[test]
    fn it_should_create_template_with_valid_content() {
        let template_str = sample_template_content();
        let ctx = TrackerContext::default_config();

        let template = TrackerTemplate::new(template_str.clone(), ctx);
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.content(), template_str);
    }

    #[test]
    fn it_should_reject_invalid_tera_syntax() {
        let invalid_str = r"{{ unclosed_variable".to_string();
        let ctx = TrackerContext::default_config();

        let result = TrackerTemplate::new(invalid_str, ctx);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_render_template_unchanged_in_phase_4() {
        let template_str = sample_template_content();
        let ctx = TrackerContext::default_config();

        let template = TrackerTemplate::new(template_str.clone(), ctx).unwrap();
        let rendered = template.render().unwrap();

        // Phase 4: No variables, so rendered content should match original
        assert_eq!(rendered, template_str);
    }

    #[test]
    fn it_should_render_to_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("tracker.toml");

        let template_str = sample_template_content();
        let ctx = TrackerContext::default_config();

        let template = TrackerTemplate::new(template_str.clone(), ctx).unwrap();
        let result = template.render_to_file(&output_path);

        assert!(result.is_ok());
        assert!(output_path.exists());

        let written_content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(written_content, template_str);
    }

    #[test]
    fn it_should_provide_context_accessor() {
        let file_content = sample_template_content();
        let ctx = TrackerContext::default_config();

        let template = TrackerTemplate::new(file_content, ctx).unwrap();
        let retrieved_context = template.context();

        // Should return the same context
        let json1 = serde_json::to_value(retrieved_context).unwrap();
        let json2 = serde_json::to_value(TrackerContext::default_config()).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn it_should_handle_write_errors_gracefully() {
        let template_str = sample_template_content();
        let ctx = TrackerContext::default_config();
        let template = TrackerTemplate::new(template_str, ctx).unwrap();

        // Try to write to an invalid path
        let invalid_path = Path::new("/invalid/nonexistent/path/tracker.toml");
        let result = template.render_to_file(invalid_path);

        assert!(result.is_err());
        match result {
            Err(TrackerTemplateError::WriteFileFailed { path, .. }) => {
                assert_eq!(path, invalid_path.display().to_string());
            }
            _ => panic!("Expected WriteFileFailed error"),
        }
    }
}
