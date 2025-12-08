//! Tracker template rendering step
//!
//! This module provides the `RenderTrackerTemplatesStep` which handles rendering
//! of Tracker configuration templates to the build directory. This step prepares
//! tracker.toml configuration file for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Tracker configuration
//! - Integration with the `TrackerProjectGenerator` for file generation
//! - Build directory preparation for deployment operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the Tracker configuration files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderTrackerTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed
//!
//! ## Phase 4 Implementation
//!
//! For Phase 4, all tracker configuration values are hardcoded in the tracker.toml.tera
//! template. No environment configuration is used yet.
//!
//! In Phase 6, this will be extended to extract configuration from `EnvironmentConfig`.

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::tracker::{
    TrackerProjectGenerator, TrackerProjectGeneratorError,
};

/// Step that renders Tracker configuration templates to the build directory
///
/// This step handles the preparation of Tracker configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host by the `DeployTrackerConfigStep`.
pub struct RenderTrackerTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderTrackerTemplatesStep<S> {
    /// Creates a new `RenderTrackerTemplatesStep`
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
    /// This will render Tracker configuration templates to the build directory.
    ///
    /// # Returns
    ///
    /// Returns the path to the tracker build directory on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File writing fails
    #[instrument(
        name = "render_tracker_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "tracker",
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn execute(&self) -> Result<PathBuf, TrackerProjectGeneratorError> {
        info!(
            step = "render_tracker_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Tracker configuration templates"
        );

        let generator =
            TrackerProjectGenerator::new(&self.build_dir, self.template_manager.clone());

        // Extract tracker config from environment (Phase 6)
        let tracker_config = &self.environment.context().user_inputs.tracker;
        generator.render(Some(tracker_config))?;

        let tracker_build_dir = self.build_dir.join("tracker");

        info!(
            step = "render_tracker_templates",
            tracker_build_dir = %tracker_build_dir.display(),
            status = "success",
            "Tracker configuration templates rendered successfully"
        );

        Ok(tracker_build_dir)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    #[test]
    fn it_should_render_tracker_templates_to_build_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let build_dir = temp_dir.path().join("build");
        let tracker_templates_dir = templates_dir.join("tracker");

        fs::create_dir_all(&tracker_templates_dir).expect("Failed to create tracker templates dir");

        // Create test tracker.toml.tera template
        let tracker_template = r#"[metadata]
app = "torrust-tracker"
schema_version = "2.0.0"

[logging]
threshold = "info"
"#;
        fs::write(
            tracker_templates_dir.join("tracker.toml.tera"),
            tracker_template,
        )
        .expect("Failed to write tracker template");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = TemplateManager::new(&templates_dir);

        let step = RenderTrackerTemplatesStep::new(
            environment,
            Arc::new(template_manager),
            build_dir.clone(),
        );

        let result = step.execute();
        assert!(
            result.is_ok(),
            "Template rendering should succeed: {:?}",
            result.err()
        );

        let tracker_build_dir = result.unwrap();
        assert_eq!(tracker_build_dir, build_dir.join("tracker"));

        // Verify tracker.toml was created
        let tracker_toml = tracker_build_dir.join("tracker.toml");
        assert!(
            tracker_toml.exists(),
            "tracker.toml should be created in build directory"
        );

        let content = fs::read_to_string(&tracker_toml).expect("Failed to read tracker.toml");
        assert!(content.contains(r#"app = "torrust-tracker""#));
        assert!(content.contains(r#"schema_version = "2.0.0""#));
        assert!(content.contains(r#"threshold = "info""#));
    }

    #[test]
    fn it_should_use_embedded_template_when_not_in_external_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let build_dir = temp_dir.path().join("build");

        // Create empty templates directory (no tracker templates)
        fs::create_dir_all(&templates_dir).expect("Failed to create templates dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = TemplateManager::new(&templates_dir);

        let step = RenderTrackerTemplatesStep::new(
            environment,
            Arc::new(template_manager),
            build_dir.clone(),
        );

        let result = step.execute();
        assert!(
            result.is_ok(),
            "Should succeed using embedded template: {:?}",
            result.err()
        );

        // Verify tracker.toml was created using embedded template
        let tracker_toml = build_dir.join("tracker/tracker.toml");
        assert!(
            tracker_toml.exists(),
            "tracker.toml should be created from embedded template"
        );
    }

    #[test]
    fn it_should_create_tracker_subdirectory_in_build_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let build_dir = temp_dir.path().join("build");
        let tracker_templates_dir = templates_dir.join("tracker");

        fs::create_dir_all(&tracker_templates_dir).expect("Failed to create tracker templates dir");

        let tracker_template = "[metadata]\napp = \"torrust-tracker\"";
        fs::write(
            tracker_templates_dir.join("tracker.toml.tera"),
            tracker_template,
        )
        .expect("Failed to write tracker template");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = TemplateManager::new(&templates_dir);

        let step = RenderTrackerTemplatesStep::new(
            environment,
            Arc::new(template_manager),
            build_dir.clone(),
        );

        step.execute().expect("Template rendering should succeed");

        let tracker_dir = build_dir.join("tracker");
        assert!(tracker_dir.exists(), "tracker/ subdirectory should exist");
        assert!(tracker_dir.is_dir(), "tracker/ should be a directory");
    }
}
