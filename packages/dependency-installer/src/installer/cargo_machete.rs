//! `cargo-machete` dependency installer
//!
//! This module provides installation logic for the `cargo-machete` dependency.

// Standard library
use std::process::Command;

// External crates
use async_trait::async_trait;
use tracing::{debug, info};

// Internal crate
use crate::Dependency;

use super::{DependencyInstaller, InstallationError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Installer for `cargo-machete` dependency
pub struct CargoMacheteInstaller;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

#[async_trait]
impl DependencyInstaller for CargoMacheteInstaller {
    fn name(&self) -> &'static str {
        "cargo-machete"
    }

    fn dependency(&self) -> Dependency {
        Dependency::CargoMachete
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!(dependency = "cargo-machete", "Installing cargo-machete");

        debug!("Running: cargo install cargo-machete");

        let output = Command::new("cargo")
            .args(["install", "cargo-machete"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::CargoMachete,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::CargoMachete,
                message: format!("cargo install failed: {stderr}"),
            });
        }

        info!(
            dependency = "cargo-machete",
            status = "installed",
            "cargo-machete installation completed"
        );

        Ok(())
    }

    fn requires_sudo(&self) -> bool {
        false
    }
}
