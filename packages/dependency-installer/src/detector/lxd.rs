//! `LXD` dependency detector
//!
//! This module provides detection logic for the `LXD` dependency.

// External crates
use tracing::info;

// Internal crate
use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Detector for `LXD` dependency
pub struct LxdDetector;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

impl DependencyDetector for LxdDetector {
    fn name(&self) -> &'static str {
        "LXD"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(dependency = "lxd", "Checking if LXD is installed");

        // Check for 'lxc' command (LXD client)
        let installed = command_exists("lxc").map_err(|e| DetectionError::DetectionFailed {
            dependency: Dependency::Lxd,
            source: e.into(),
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
