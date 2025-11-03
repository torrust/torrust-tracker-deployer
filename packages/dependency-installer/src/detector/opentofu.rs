use tracing::info;

use crate::command::command_exists;
use crate::detector::ToolDetector;
use crate::errors::DetectionError;

/// Detector for `OpenTofu` tool
pub struct OpenTofuDetector;

impl ToolDetector for OpenTofuDetector {
    fn name(&self) -> &'static str {
        "OpenTofu"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(tool = "opentofu", "Checking if OpenTofu is installed");

        let installed = command_exists("tofu").map_err(|e| DetectionError::DetectionFailed {
            tool: self.name().to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(tool = "opentofu", "OpenTofu is installed");
        } else {
            info!(tool = "opentofu", "OpenTofu is not installed");
        }

        Ok(installed)
    }
}
