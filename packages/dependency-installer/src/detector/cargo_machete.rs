use tracing::info;

use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

/// Detector for `cargo-machete` dependency
pub struct CargoMacheteDetector;

impl DependencyDetector for CargoMacheteDetector {
    fn name(&self) -> &'static str {
        "cargo-machete"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(
            dependency = "cargo-machete",
            "Checking if cargo-machete is installed"
        );

        let installed =
            command_exists("cargo-machete").map_err(|e| DetectionError::DetectionFailed {
                dependency: Dependency::CargoMachete,
                source: std::io::Error::other(e.to_string()),
            })?;

        if installed {
            info!(
                dependency = "cargo-machete",
                status = "installed",
                "cargo-machete is installed"
            );
        } else {
            info!(
                dependency = "cargo-machete",
                status = "not installed",
                "cargo-machete is not installed"
            );
        }

        Ok(installed)
    }
}
