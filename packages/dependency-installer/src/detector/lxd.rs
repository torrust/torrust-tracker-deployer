use tracing::info;

use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

/// Detector for `LXD` dependency
pub struct LxdDetector;

impl DependencyDetector for LxdDetector {
    fn name(&self) -> &'static str {
        "LXD"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(dependency = "lxd", "Checking if LXD is installed");

        // Check for 'lxc' command (LXD client)
        let installed = command_exists("lxc").map_err(|e| DetectionError::DetectionFailed {
            dependency: Dependency::Lxd,
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(dependency = "lxd", status = "installed", "LXD is installed");
        } else {
            info!(
                dependency = "lxd",
                status = "not installed",
                "LXD is not installed"
            );
        }

        Ok(installed)
    }
}
