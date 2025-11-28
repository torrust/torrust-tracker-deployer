//! Environment configuration generation for black-box E2E tests.
//!
//! This module provides a shared function to generate environment configuration
//! files with absolute SSH key paths for E2E testing.
//!
//! # Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::generate_environment_config;
//!
//! // Generate config for "e2e-provision" environment
//! let config_path = generate_environment_config("e2e-provision")?;
//! ```

use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

/// Generates the environment configuration file with absolute SSH key paths.
///
/// This function creates a configuration file with absolute paths
/// to the SSH keys, ensuring they work correctly regardless of the directory
/// from which Ansible runs.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
///
/// # Returns
///
/// Returns the path to the generated configuration file.
///
/// # Errors
///
/// Returns an error if the configuration file cannot be created.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::generate_environment_config;
///
/// let config_path = generate_environment_config("e2e-provision")?;
/// let config_path = generate_environment_config("e2e-full")?;
/// ```
pub fn generate_environment_config(environment_name: &str) -> Result<PathBuf> {
    generate_environment_config_with_port(environment_name, None)
}

/// Generates the environment configuration file with absolute SSH key paths and optional SSH port.
///
/// This variant allows specifying a custom SSH port, which is useful for container-based
/// testing where the SSH port is dynamically mapped.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
/// * `ssh_port` - Optional SSH port (defaults to 22 if not specified)
///
/// # Returns
///
/// Returns the path to the generated configuration file.
///
/// # Errors
///
/// Returns an error if the configuration file cannot be created.
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::generate_environment_config_with_port;
///
/// // Use default port (22)
/// let config_path = generate_environment_config_with_port("e2e-provision", None)?;
///
/// // Use custom port for container testing
/// let config_path = generate_environment_config_with_port("e2e-config", Some(32808))?;
/// ```
pub fn generate_environment_config_with_port(
    environment_name: &str,
    ssh_port: Option<u16>,
) -> Result<PathBuf> {
    use std::fs;

    // Get project root from current directory (cargo run runs from project root)
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {e}"))?;

    // Build absolute paths to SSH keys
    let private_key_path = project_root.join("fixtures/testing_rsa");
    let public_key_path = project_root.join("fixtures/testing_rsa.pub");

    // Verify SSH keys exist
    if !private_key_path.exists() {
        return Err(anyhow::anyhow!(
            "SSH private key not found at: {}",
            private_key_path.display()
        ));
    }
    if !public_key_path.exists() {
        return Err(anyhow::anyhow!(
            "SSH public key not found at: {}",
            public_key_path.display()
        ));
    }

    // Create configuration JSON with absolute paths
    let mut ssh_credentials = serde_json::json!({
        "private_key_path": private_key_path.to_string_lossy(),
        "public_key_path": public_key_path.to_string_lossy()
    });

    // Add port if specified
    if let Some(port) = ssh_port {
        ssh_credentials["port"] = serde_json::json!(port);
    }

    let config = serde_json::json!({
        "environment": {
            "name": environment_name
        },
        "ssh_credentials": ssh_credentials
    });

    // Write to envs directory
    let config_path = project_root.join(format!("envs/{environment_name}.json"));

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {e}"))?;
    }

    fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .map_err(|e| anyhow::anyhow!("Failed to write config file: {e}"))?;

    info!(
        config_path = %config_path.display(),
        private_key = %private_key_path.display(),
        public_key = %public_key_path.display(),
        ssh_port = ?ssh_port,
        "Generated environment configuration with absolute SSH key paths"
    );

    Ok(config_path)
}
