//! `OpenTofu` dependency detector
//!
//! This module provides detection logic for the `OpenTofu` dependency.

// External crates
use tracing::info;

// Internal crate
use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Detector for `OpenTofu` dependency
pub struct OpenTofuDetector;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

impl DependencyDetector for OpenTofuDetector {
    fn name(&self) -> &'static str {
        "OpenTofu"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(dependency = "opentofu", "Checking if OpenTofu is installed");

        let installed = command_exists("tofu").map_err(|e| DetectionError::DetectionFailed {
            dependency: Dependency::OpenTofu,
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(
                dependency = "opentofu",
                status = "installed",
                "OpenTofu is installed"
            );
        } else {
            info!(
                dependency = "opentofu",
                status = "not installed",
                "OpenTofu is not installed"
            );
        }

        Ok(installed)
    }
}
