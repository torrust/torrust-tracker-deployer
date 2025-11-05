//! `OpenTofu` dependency installer
//!
//! This module provides installation logic for the `OpenTofu` dependency.

// Standard library
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::process::Command;

// External crates
use async_trait::async_trait;
#[cfg(unix)]
use tracing::{debug, info};

// Internal crate
use crate::Dependency;

use super::{DependencyInstaller, InstallationError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Installer for `OpenTofu` dependency
pub struct OpenTofuInstaller;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

#[async_trait]
impl DependencyInstaller for OpenTofuInstaller {
    fn name(&self) -> &'static str {
        "OpenTofu"
    }

    fn dependency(&self) -> Dependency {
        Dependency::OpenTofu
    }

    async fn install(&self) -> Result<(), InstallationError> {
        #[cfg(not(unix))]
        {
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::OpenTofu,
                message: "OpenTofu installation is only supported on Unix-like systems".to_string(),
            });
        }

        #[cfg(unix)]
        {
            info!(dependency = "opentofu", "Installing OpenTofu");

            let script_path = "/tmp/install-opentofu.sh";

            // Download installer script
            debug!("Downloading OpenTofu installer script");
            let output = Command::new("curl")
                .args([
                    "--proto",
                    "=https",
                    "--tlsv1.2",
                    "-fsSL",
                    "https://get.opentofu.org/install-opentofu.sh",
                    "-o",
                    script_path,
                ])
                .output()
                .map_err(|e| InstallationError::command_failed(Dependency::OpenTofu, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(InstallationError::InstallationFailed {
                    dependency: Dependency::OpenTofu,
                    message: format!("Failed to download installer: {stderr}"),
                });
            }

            // Make script executable
            debug!("Making installer script executable");
            fs::set_permissions(
                script_path,
                std::os::unix::fs::PermissionsExt::from_mode(0o755),
            )
            .map_err(|e| InstallationError::command_failed(Dependency::OpenTofu, e))?;

            // Run installer with sudo
            debug!("Running OpenTofu installer with sudo");
            let output = Command::new("sudo")
                .args([script_path, "--install-method", "deb"])
                .output()
                .map_err(|e| InstallationError::command_failed(Dependency::OpenTofu, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // Clean up script before returning error (ignore cleanup errors)
                fs::remove_file(script_path).ok();
                return Err(InstallationError::InstallationFailed {
                    dependency: Dependency::OpenTofu,
                    message: format!("Installer script failed: {stderr}"),
                });
            }

            // Clean up installer script
            debug!("Cleaning up installer script");
            fs::remove_file(script_path)
                .map_err(|e| InstallationError::command_failed(Dependency::OpenTofu, e))?;

            info!(
                dependency = "opentofu",
                status = "installed",
                "OpenTofu installation completed"
            );

            Ok(())
        }
    }

    fn requires_sudo(&self) -> bool {
        true
    }
}
