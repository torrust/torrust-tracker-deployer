//! `Ansible` dependency installer
//!
//! This module provides installation logic for the `Ansible` dependency.

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

/// Installer for `Ansible` dependency
pub struct AnsibleInstaller;

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

#[async_trait]
impl DependencyInstaller for AnsibleInstaller {
    fn name(&self) -> &'static str {
        "Ansible"
    }

    fn dependency(&self) -> Dependency {
        Dependency::Ansible
    }

    async fn install(&self) -> Result<(), InstallationError> {
        info!(dependency = "ansible", "Installing Ansible");

        // Install Ansible
        // Note: Assumes apt package lists are already updated (system pre-condition)
        debug!("Installing Ansible via apt-get");
        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", "ansible"])
            .output()
            .map_err(|e| InstallationError::CommandFailed {
                dependency: Dependency::Ansible,
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallationError::InstallationFailed {
                dependency: Dependency::Ansible,
                message: format!("apt-get install failed: {stderr}"),
            });
        }

        info!(
            dependency = "ansible",
            status = "installed",
            "Ansible installation completed"
        );

        Ok(())
    }

    fn requires_sudo(&self) -> bool {
        true
    }
}
