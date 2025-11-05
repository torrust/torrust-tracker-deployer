//! `cargo-machete` dependency detector
//!
//! This module provides detection logic for the `cargo-machete` dependency.

// External crates
use tracing::info;

// Internal crate
use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Detector for `cargo-machete` dependency
pub struct CargoMacheteDetector;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

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
                source: e.into(),
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
