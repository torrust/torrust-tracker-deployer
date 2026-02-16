//! Next Step Guidance View
//!
//! This module provides a view for rendering state-aware guidance
//! about what the user should do next.

/// View for rendering next step guidance
///
/// This view provides context-aware guidance to users about what
/// command they should run next based on the current environment state.
pub struct NextStepGuidanceView;

impl NextStepGuidanceView {
    /// Render next step guidance as formatted lines
    ///
    /// # Arguments
    ///
    /// * `state_name` - Internal state name (e.g., "created", "provisioned")
    ///
    /// # Returns
    ///
    /// A vector of formatted lines ready to be joined
    #[must_use]
    pub fn render(state_name: &str) -> Vec<String> {
        vec![
            String::new(), // blank line
            Self::get_guidance(state_name),
        ]
    }

    /// Get guidance text based on current state
    fn get_guidance(state_name: &str) -> String {
        match state_name {
            "created" => "Run 'provision' to create infrastructure.".to_string(),
            "provisioning" => {
                "Provisioning in progress. Wait for completion or check logs.".to_string()
            }
            "provisioned" => "Run 'configure' to set up the system.".to_string(),
            "configuring" => {
                "Configuration in progress. Wait for completion or check logs.".to_string()
            }
            "configured" => "Run 'release' to deploy the tracker software.".to_string(),
            "releasing" => "Release in progress. Wait for completion or check logs.".to_string(),
            "released" => "Run 'run' to start the tracker services.".to_string(),
            "running" => "Services are running. Use 'test' to verify health.".to_string(),
            "destroying" => "Destruction in progress. Wait for completion.".to_string(),
            "destroyed" => {
                "Environment has been destroyed. Create a new environment to redeploy.".to_string()
            }
            "provision_failed" => {
                "Provisioning failed. Run 'destroy' and create a new environment.".to_string()
            }
            "configure_failed" => {
                "Configuration failed. Run 'destroy' and create a new environment.".to_string()
            }
            "release_failed" => {
                "Release failed. Run 'destroy' and create a new environment.".to_string()
            }
            "run_failed" => "Run failed. Run 'destroy' and create a new environment.".to_string(),
            "destroy_failed" => {
                "Destruction failed. Check error details and retry 'destroy'.".to_string()
            }
            _ => format!("Unknown state: {state_name}. Check environment state file."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_guide_from_created_state() {
        let lines = NextStepGuidanceView::render("created");
        let text = lines.join("\n");
        assert!(text.contains("provision"));
    }

    #[test]
    fn it_should_guide_from_provisioned_state() {
        let lines = NextStepGuidanceView::render("provisioned");
        let text = lines.join("\n");
        assert!(text.contains("configure"));
    }

    #[test]
    fn it_should_guide_from_configured_state() {
        let lines = NextStepGuidanceView::render("configured");
        let text = lines.join("\n");
        assert!(text.contains("release"));
    }

    #[test]
    fn it_should_guide_from_released_state() {
        let lines = NextStepGuidanceView::render("released");
        let text = lines.join("\n");
        assert!(text.contains("run"));
    }

    #[test]
    fn it_should_guide_from_running_state() {
        let lines = NextStepGuidanceView::render("running");
        let text = lines.join("\n");
        assert!(text.contains("test"));
    }

    #[test]
    fn it_should_guide_from_destroyed_state() {
        let lines = NextStepGuidanceView::render("destroyed");
        let text = lines.join("\n");
        assert!(text.contains("destroyed"));
        assert!(text.contains("new environment"));
    }

    #[test]
    fn it_should_handle_provision_failed_state() {
        let lines = NextStepGuidanceView::render("provision_failed");
        let text = lines.join("\n");
        assert!(text.contains("failed"));
        assert!(text.contains("destroy"));
    }

    #[test]
    fn it_should_handle_configure_failed_state() {
        let lines = NextStepGuidanceView::render("configure_failed");
        let text = lines.join("\n");
        assert!(text.contains("failed"));
        assert!(text.contains("destroy"));
    }

    #[test]
    fn it_should_handle_release_failed_state() {
        let lines = NextStepGuidanceView::render("release_failed");
        let text = lines.join("\n");
        assert!(text.contains("failed"));
        assert!(text.contains("destroy"));
    }

    #[test]
    fn it_should_handle_run_failed_state() {
        let lines = NextStepGuidanceView::render("run_failed");
        let text = lines.join("\n");
        assert!(text.contains("failed"));
        assert!(text.contains("destroy"));
    }

    #[test]
    fn it_should_handle_destroy_failed_state() {
        let lines = NextStepGuidanceView::render("destroy_failed");
        let text = lines.join("\n");
        assert!(text.contains("failed"));
        assert!(text.contains("retry"));
    }

    #[test]
    fn it_should_handle_unknown_state() {
        let lines = NextStepGuidanceView::render("unknown_state");
        let text = lines.join("\n");
        assert!(text.contains("Unknown state"));
    }

    #[test]
    fn it_should_start_with_blank_line() {
        let lines = NextStepGuidanceView::render("created");
        assert!(lines.first().is_some_and(String::is_empty));
    }
}
