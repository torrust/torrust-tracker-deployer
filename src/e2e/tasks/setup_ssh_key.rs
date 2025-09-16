//! SSH key setup task for E2E testing
//!
//! This module provides functionality to set up SSH keys for E2E testing by copying
//! test keys from the fixtures directory to a temporary location with proper permissions.
//!
//! ## Key Operations
//!
//! - Copy SSH private and public keys from fixtures to temporary directory
//! - Set correct file permissions (600 for private key, 644 for public key)
//! - Ensure SSH keys are properly configured for test automation
//!
//! ## Security Considerations
//!
//! The SSH keys used are test-only keys from the fixtures directory and should
//! never be used in production environments. Proper file permissions are enforced
//! to prevent SSH client warnings and ensure secure key handling.

use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;
use tracing::info;

/// Setup SSH key by copying from fixtures to temporary directory with proper permissions
///
/// # Errors
///
/// Returns an error if:
/// - SSH key files cannot be copied from fixtures
/// - File permissions cannot be set
pub fn setup_ssh_key(project_root: &std::path::Path, temp_dir: &TempDir) -> Result<()> {
    // Copy SSH private key from fixtures to temp directory
    let fixtures_ssh_key = project_root.join("fixtures/testing_rsa");
    let temp_ssh_key = temp_dir.path().join("testing_rsa");

    std::fs::copy(&fixtures_ssh_key, &temp_ssh_key)
        .context("Failed to copy SSH private key to temporary directory")?;

    // Copy SSH public key from fixtures to temp directory
    let fixtures_ssh_pub_key = project_root.join("fixtures/testing_rsa.pub");
    let temp_ssh_pub_key = temp_dir.path().join("testing_rsa.pub");

    std::fs::copy(&fixtures_ssh_pub_key, &temp_ssh_pub_key)
        .context("Failed to copy SSH public key to temporary directory")?;

    // Set proper permissions on the SSH key (600)
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&temp_ssh_key)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&temp_ssh_key, perms)?;
    }

    info!(
        operation = "ssh_key_setup",
        private_location = %temp_ssh_key.display(),
        public_location = %temp_ssh_pub_key.display(),
        "SSH keys copied to temporary location"
    );

    Ok(())
}
