use tracing::info;

use crate::command::command_exists;

use super::{DetectionError, ToolDetector};

/// Detector for `cargo-machete` tool
pub struct CargoMacheteDetector;

impl ToolDetector for CargoMacheteDetector {
    fn name(&self) -> &'static str {
        "cargo-machete"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(
            tool = "cargo-machete",
            "Checking if cargo-machete is installed"
        );

        let installed =
            command_exists("cargo-machete").map_err(|e| DetectionError::DetectionFailed {
                tool: self.name().to_string(),
                source: std::io::Error::other(e.to_string()),
            })?;

        if installed {
            info!(tool = "cargo-machete", "cargo-machete is installed");
        } else {
            info!(tool = "cargo-machete", "cargo-machete is not installed");
        }

        Ok(installed)
    }
}
