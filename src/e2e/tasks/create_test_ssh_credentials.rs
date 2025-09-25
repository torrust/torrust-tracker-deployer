//! SSH credentials helpers for E2E testing
//!
//! This module provides helper functions for creating SSH credentials
//! used in E2E testing scenarios. It centralizes the creation of test
//! SSH credentials to eliminate code duplication.
//!
//! ## Key Operations
//!
//! - Creates SSH credentials using test keys from fixtures directory
//! - Provides consistent SSH credentials across different test scenarios
//! - Eliminates code duplication for credential creation

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::SshCredentials;

/// Create SSH credentials for E2E testing
///
/// This function creates SSH credentials using the test SSH keys from the
/// fixtures directory. It provides a centralized way to create consistent
/// SSH credentials across different E2E testing scenarios.
///
/// # Arguments
///
/// * `ssh_username` - Username for SSH connection
///
/// # Returns
///
/// Returns SSH credentials configured with test keys and the provided username.
///
/// # Errors
///
/// Returns an error if:
/// - Current directory cannot be determined
/// - SSH key files are not found in fixtures directory
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::create_test_ssh_credentials::create_test_ssh_credentials;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let ssh_credentials = create_test_ssh_credentials("testuser")?;
///     println!("SSH credentials created for user: {}", ssh_credentials.ssh_username);
///     Ok(())
/// }
/// ```
pub fn create_test_ssh_credentials(ssh_username: &str) -> Result<SshCredentials> {
    let project_root = env::current_dir().context("Failed to get current directory")?;

    let private_key_path: PathBuf = project_root.join("fixtures/testing_rsa");
    let public_key_path: PathBuf = project_root.join("fixtures/testing_rsa.pub");

    Ok(SshCredentials::new(
        private_key_path,
        public_key_path,
        ssh_username.to_string(),
    ))
}
