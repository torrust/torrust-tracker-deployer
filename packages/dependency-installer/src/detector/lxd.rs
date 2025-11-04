use tracing::info;

use crate::command::command_exists;

use super::{DetectionError, ToolDetector};

/// Detector for `LXD` tool
pub struct LxdDetector;

impl ToolDetector for LxdDetector {
    fn name(&self) -> &'static str {
        "LXD"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(tool = "lxd", "Checking if LXD is installed");

        // Check for 'lxc' command (LXD client)
        let installed = command_exists("lxc").map_err(|e| DetectionError::DetectionFailed {
            tool: self.name().to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(tool = "lxd", "LXD is installed");
        } else {
            info!(tool = "lxd", "LXD is not installed");
        }

        Ok(installed)
    }
}
