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
