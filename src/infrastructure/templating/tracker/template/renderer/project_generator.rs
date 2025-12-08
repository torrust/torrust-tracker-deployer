//! Tracker Project Generator
//!
//! Orchestrates the rendering of all Tracker configuration templates following
//! the Project Generator pattern.
//!
//! ## Architecture
//!
//! This follows the three-layer Project Generator pattern:
//! - **Context** (`TrackerContext`) - Defines variables needed by templates
//! - **Template** (`TrackerTemplate`) - Wraps template file with context
//! - **Renderer** (`TrackerConfigRenderer`) - Renders specific .tera templates
//! - **`ProjectGenerator`** (this file) - Orchestrates all renderers
//!
//! ## Phase 4 Implementation
//!
//! In Phase 4, all tracker configuration values are hardcoded in the tracker.toml.tera
//! template file. The `TrackerContext` is empty - no variable substitution occurs.
//!
//! ## Phase 6 Future
//!
//! Phase 6 will populate `TrackerContext` with dynamic configuration values from
//! the environment configuration.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::tracker::template::{
    renderer::{TrackerConfigRenderer, TrackerConfigRendererError},
    TrackerContext,
};

/// Errors that can occur during Tracker project generation
#[derive(Error, Debug)]
pub enum TrackerProjectGeneratorError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to render tracker configuration
    #[error("Failed to render tracker configuration: {0}")]
    RendererFailed(#[from] TrackerConfigRendererError),
}

/// Orchestrates Tracker configuration template rendering
///
/// This is the Project Generator that coordinates all tracker template rendering.
/// It follows the standard pattern:
/// 1. Create build directory structure
/// 2. Call `TrackerConfigRenderer` to render tracker.toml.tera
/// 3. (Future) Copy any static files if needed
///
/// ## Phase 4: Hardcoded Configuration
///
/// Uses an empty `TrackerContext`. All values are hardcoded in the template.
///
/// ## Phase 6: Dynamic Configuration
///
/// Will accept configuration parameters and populate `TrackerContext` with
/// user-provided values for database, trackers, API settings, etc.
pub struct TrackerProjectGenerator {
    build_dir: PathBuf,
    tracker_renderer: TrackerConfigRenderer,
}

impl TrackerProjectGenerator {
    /// Default relative path for Tracker configuration files
    const TRACKER_BUILD_PATH: &'static str = "tracker";

    /// Creates a new Tracker project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let tracker_renderer = TrackerConfigRenderer::new(template_manager);

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            tracker_renderer,
        }
    }

    /// Renders Tracker configuration templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Tracker config
    /// 2. Renders tracker.toml.tera template with provided or default configuration
    /// 3. Writes the rendered content to tracker.toml
    ///
    /// # Arguments
    ///
    /// * `tracker_config` - Optional tracker configuration. If None, uses default hardcoded values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Build directory creation fails
    /// - Template loading fails
    /// - Template rendering fails
    /// - Writing output file fails
    #[instrument(
        name = "tracker_project_generator_render",
        skip(self, tracker_config),
        fields(
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn render(
        &self,
        tracker_config: Option<&crate::domain::environment::TrackerConfig>,
    ) -> Result<(), TrackerProjectGeneratorError> {
        // Create build directory for tracker templates
        let tracker_build_dir = self.build_dir.join(Self::TRACKER_BUILD_PATH);
        std::fs::create_dir_all(&tracker_build_dir).map_err(|source| {
            TrackerProjectGeneratorError::DirectoryCreationFailed {
                directory: tracker_build_dir.display().to_string(),
                source,
            }
        })?;

        // Create context from tracker config or use defaults
        let context = match tracker_config {
            Some(config) => TrackerContext::from_config(config),
            None => TrackerContext::default_config(),
        };

        // Render tracker.toml using TrackerRenderer
        self.tracker_renderer.render(&context, &tracker_build_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn it_should_create_tracker_build_directory() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = TrackerProjectGenerator::new(&build_dir, template_manager);

        generator.render(None).expect("Failed to render templates");

        let tracker_dir = build_dir.join("tracker");
        assert!(
            tracker_dir.exists(),
            "Tracker build directory should be created"
        );
        assert!(
            tracker_dir.is_dir(),
            "Tracker build path should be a directory"
        );
    }

    #[test]
    fn it_should_render_tracker_toml_with_hardcoded_values() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = TrackerProjectGenerator::new(&build_dir, template_manager);

        generator.render(None).expect("Failed to render templates");

        let tracker_toml_path = build_dir.join("tracker/tracker.toml");
        assert!(tracker_toml_path.exists(), "tracker.toml should be created");

        let content = fs::read_to_string(&tracker_toml_path).expect("Failed to read tracker.toml");

        // Verify hardcoded values in template
        assert!(content.contains(r#"app = "torrust-tracker""#));
        assert!(content.contains(r#"schema_version = "2.0.0""#));
        assert!(content.contains(r#"threshold = "info""#));
        assert!(content.contains("listed = false"));
        assert!(content.contains("private = false"));
        assert!(content.contains(r#"driver = "sqlite3""#));
        assert!(content.contains(r#"bind_address = "0.0.0.0:6868""#));
        assert!(content.contains(r#"bind_address = "0.0.0.0:6969""#));
        assert!(content.contains(r#"bind_address = "0.0.0.0:7070""#));
        assert!(content.contains(r#"bind_address = "0.0.0.0:1212""#));
    }

    #[test]
    fn it_should_use_embedded_template_when_not_in_external_dir() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        // Create template manager with empty templates directory
        let templates_dir = temp_dir.path().join("empty_templates");
        fs::create_dir_all(&templates_dir).expect("Failed to create templates dir");

        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        let generator = TrackerProjectGenerator::new(&build_dir, template_manager);

        // Should succeed because TemplateManager extracts from embedded resources
        let result = generator.render(None);
        assert!(
            result.is_ok(),
            "Should succeed using embedded template: {:?}",
            result.err()
        );

        let tracker_toml = build_dir.join("tracker/tracker.toml");
        assert!(
            tracker_toml.exists(),
            "tracker.toml should be created from embedded template"
        );
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path();

        let error = TrackerProjectGeneratorError::DirectoryCreationFailed {
            directory: build_dir.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test error"),
        };

        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("DirectoryCreationFailed"));
        assert!(debug_output.contains("PermissionDenied"));
    }

    // Helper function to create a test template manager with tracker.toml.tera
    fn create_test_template_manager() -> Arc<TemplateManager> {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let tracker_dir = templates_dir.join("tracker");

        fs::create_dir_all(&tracker_dir).expect("Failed to create tracker dir");

        // Create tracker.toml.tera with hardcoded test content
        let tracker_template_content = r#"[metadata]
app = "torrust-tracker"
purpose = "configuration"
schema_version = "2.0.0"

[logging]
threshold = "info"

[core]
listed = false
private = false

[core.tracker_policy]
persistent_torrent_completed_stat = true

[core.announce_policy]
interval = 300
interval_min = 300

[core.net]
on_reverse_proxy = true

[core.database]
driver = "sqlite3"
path = "/var/lib/torrust/tracker/database/sqlite3.db"

[[udp_trackers]]
bind_address = "0.0.0.0:6868"

[[udp_trackers]]
bind_address = "0.0.0.0:6969"

[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:1212"
"#;

        fs::write(
            tracker_dir.join("tracker.toml.tera"),
            tracker_template_content,
        )
        .expect("Failed to write tracker template");

        // Prevent temp_dir from being dropped
        std::mem::forget(temp_dir);

        Arc::new(TemplateManager::new(templates_dir))
    }
}
