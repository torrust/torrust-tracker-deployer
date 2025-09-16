use anyhow::Result;
use tracing::info;

use crate::container::Services;

/// Clean and prepare templates directory to ensure fresh embedded templates
///
/// # Errors
///
/// Returns an error if the template manager fails to reset the templates directory
pub fn clean_and_prepare_templates(services: &Services) -> Result<()> {
    // Clean templates directory to ensure we use fresh templates from embedded resources
    info!(
        operation = "clean_templates",
        "Cleaning templates directory to ensure fresh embedded templates"
    );

    services
        .template_manager
        .reset_templates_dir()
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}
