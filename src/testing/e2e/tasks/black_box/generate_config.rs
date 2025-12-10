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

use crate::testing::e2e::containers::E2eEnvironmentInfo;

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
    let env_info = generate_environment_config_with_port(environment_name)?;
    Ok(env_info.config_file_path)
}

/// Generates the environment configuration file with absolute SSH key paths.
///
/// Creates a complete E2E environment configuration including tracker ports,
/// SSH credentials, and provider settings. With host networking, the SSH port
/// is defined in the configuration and remains the same inside and outside the container.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
///
/// # Returns
///
/// Returns `E2eEnvironmentInfo` containing all necessary information for E2E testing:
/// - Environment name
/// - Path to the generated configuration file
/// - SSH port (extracted from tracker configuration)
/// - Tracker ports (extracted from tracker configuration)
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
/// let env_info = generate_environment_config_with_port("e2e-config")?;
/// let socket_addr = env_info.ssh_socket_addr();
/// ```
pub fn generate_environment_config_with_port(environment_name: &str) -> Result<E2eEnvironmentInfo> {
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

    // Create configuration JSON with absolute paths and tracker configuration
    // This must match the format expected by EnvironmentCreationConfig
    let config = serde_json::json!({
        "environment": {
            "name": environment_name
        },
        "ssh_credentials": {
            "private_key_path": private_key_path.to_string_lossy(),
            "public_key_path": public_key_path.to_string_lossy()
        },
        "provider": {
            "provider": "lxd",
            "profile_name": format!("torrust-profile-{}", environment_name)
        },
        "tracker": {
            "core": {
                "database": {
                    "driver": "sqlite3",
                    "database_name": "tracker.db"
                },
                "private": false
            },
            "udp_trackers": [
                {"bind_address": "0.0.0.0:6969"}
            ],
            "http_trackers": [
                {"bind_address": "0.0.0.0:7070"}
            ],
            "http_api": {
                "admin_token": "MyAccessToken"
            }
        }
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
        "Generated environment configuration with absolute SSH key paths"
    );

    // Create E2eEnvironmentInfo from the generated config
    E2eEnvironmentInfo::from_config_file(environment_name.to_string(), config_path, None)
}

/// Update the SSH port in an existing environment configuration file
///
/// This is needed when using bridge networking where Docker assigns a random
/// mapped port that differs from the configured port (22).
///
/// # Arguments
/// * `environment_name` - Name of the environment
/// * `mapped_ssh_port` - The actual SSH port mapped by Docker
///
/// # Errors
/// Returns an error if the config file cannot be read, parsed, or written
pub fn update_environment_ssh_port(environment_name: &str, mapped_ssh_port: u16) -> Result<()> {
    use std::fs;

    // Get project root from current directory
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {e}"))?;

    // Path to environment config file
    let config_path = project_root.join(format!("envs/{environment_name}.json"));

    // Read existing config
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {e}"))?;

    // Parse as JSON
    let mut config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config JSON: {e}"))?;

    // Update SSH port
    if let Some(created) = config.get_mut("Created") {
        if let Some(created_obj) = created.as_object_mut() {
            created_obj.insert("ssh_port".to_string(), serde_json::json!(mapped_ssh_port));
        }
    }

    // Write updated config
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .map_err(|e| anyhow::anyhow!("Failed to write updated config: {e}"))?;

    info!(
        environment_name = %environment_name,
        mapped_ssh_port = %mapped_ssh_port,
        config_path = %config_path.display(),
        "Updated environment configuration with mapped SSH port"
    );

    Ok(())
}
