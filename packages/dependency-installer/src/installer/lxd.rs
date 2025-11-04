//! `LXD` dependency installer
//!
//! This module provides installation logic for the `LXD` dependency.

// Standard library
use std::process::Command;
use std::thread;
use std::time::Duration;

// External crates
use async_trait::async_trait;
use tracing::{debug, info};

// Internal crate
use crate::Dependency;

use super::{DependencyInstaller, InstallationError};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Installer for `LXD` dependency
pub struct LxdInstaller;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

#[async_trait]
impl DependencyInstaller for LxdInstaller {
    fn name(&self) -> &'static str {
        "LXD"
    }

    fn dependency(&self) -> Dependency {
        Dependency::Lxd
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!(dependency = "lxd", "Installing LXD");

        // Install LXD via snap
        debug!("Installing LXD via snap");
        let output = Command::new("sudo")
            .args(["snap", "install", "lxd"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::Lxd,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::Lxd,
                message: format!("snap install failed: {stderr}"),
            });
        }

        // Wait for LXD daemon to start
        debug!("Waiting for LXD daemon to initialize");
        thread::sleep(Duration::from_secs(15));

        // Initialize LXD with default settings
        debug!("Initializing LXD with default settings");
        let output = Command::new("sudo")
            .args(["lxd", "init", "--auto"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::Lxd,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::Lxd,
                message: format!("lxd init failed: {stderr}"),
            });
        }

        // Add current user to lxd group
        debug!("Adding current user to lxd group");
        if let Ok(username) = std::env::var("USER") {
            let output = Command::new("sudo")
                .args(["usermod", "-a", "-G", "lxd", &username])
                .output()
                .map_err(|e| InstallationError::CommandFailed {
                    dependency: Dependency::Lxd,
                    source: e,
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!(
                    "Warning: Failed to add user to lxd group: {}. You may need to do this manually.",
                    stderr
                );
            }
        }

        // Fix socket permissions for CI environment
        debug!("Setting socket permissions for CI compatibility");
        let output = Command::new("sudo")
            .args(["chmod", "666", "/var/snap/lxd/common/lxd/unix.socket"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::Lxd,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!(
                "Warning: Failed to set socket permissions: {}. This may be okay in non-CI environments.",
                stderr
            );
        }

        // Test basic LXD functionality
        debug!("Testing LXD installation");
        let output = Command::new("sudo")
            .args(["lxc", "list"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::Lxd,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::Lxd,
                message: format!("lxc list test failed: {stderr}"),
            });
        }

        info!(
            dependency = "lxd",
            status = "installed",
            "LXD installation completed"
        );

        Ok(())
    }

    fn requires_sudo(&self) -> bool {
        true
    }
}
