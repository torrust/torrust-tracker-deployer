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
    let env_info = build_e2e_test_config(environment_name);
    write_environment_config(&env_info)?;
    Ok(env_info.config_file_path)
}

/// Generates E2E environment configuration in-memory
///
/// Creates a complete E2E environment configuration including tracker ports,
/// SSH credentials, and provider settings. With host networking, the SSH port
/// is defined in the configuration and remains the same inside and outside the container.
///
/// This function builds the configuration structure directly without file I/O.
/// Use `write_environment_config()` to persist the configuration to disk when needed.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
///
/// # Returns
///
/// Returns `E2eEnvironmentInfo` containing all necessary information for E2E testing:
/// - Environment name
/// - Path where config should be written (if needed)
/// - SSH port (22 - default for test containers)
/// - Tracker ports (default test configuration)
///
/// # Panics
///
/// Panics if the current working directory cannot be determined (should never happen in normal operation).
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::build_e2e_test_config;
///
/// let env_info = build_e2e_test_config("e2e-config");
/// let socket_addr = env_info.ssh_socket_addr();
/// ```
#[must_use]
pub fn build_e2e_test_config(environment_name: &str) -> E2eEnvironmentInfo {
    use crate::testing::e2e::containers::TrackerPorts;

    let project_root = std::env::current_dir().expect("Failed to get current directory");

    let config_path = project_root.join(format!("envs/{environment_name}.json"));

    // Build E2eConfigEnvironment directly with default test values
    let ssh_port = 22; // Default SSH port for test containers
    let tracker_ports = TrackerPorts::default();

    info!(
        environment_name = %environment_name,
        ssh_port = %ssh_port,
        "Generated E2E environment configuration in-memory"
    );

    E2eEnvironmentInfo::new(
        environment_name.to_string(),
        config_path,
        ssh_port,
        tracker_ports,
    )
}

/// Writes E2E environment configuration to disk
///
/// Creates the configuration JSON file with absolute SSH key paths,
/// ensuring the environment can be used by CLI commands.
///
/// # Arguments
///
/// * `config_env` - The E2E configuration to write
///
/// # Errors
///
/// Returns an error if:
/// - Configuration directory cannot be created
/// - Configuration file cannot be written
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::{
///     build_e2e_test_config,
///     write_environment_config,
/// };
///
/// let env_info = build_e2e_test_config("e2e-config");
/// write_environment_config(&env_info)?;
/// ```
pub fn write_environment_config(config_env: &E2eEnvironmentInfo) -> Result<()> {
    use std::fs;

    let config_json = config_env.to_json_config();

    // Ensure parent directory exists
    if let Some(parent) = config_env.config_file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {e}"))?;
    }

    fs::write(&config_env.config_file_path, config_json)
        .map_err(|e| anyhow::anyhow!("Failed to write config file: {e}"))?;

    info!(
        config_path = %config_env.config_file_path.display(),
        "Wrote environment configuration to disk"
    );

    Ok(())
}

/// Creates a test environment configuration with absolute SSH key paths
///
/// Generates a JSON configuration string for E2E testing with:
/// - Absolute paths to SSH keys in fixtures/ (using `CARGO_MANIFEST_DIR`)
/// - LXD provider configuration
/// - Default tracker configuration (UDP 6969, HTTP 7070, API token)
///
/// This function uses `env!("CARGO_MANIFEST_DIR")` to locate the project root at compile time,
/// ensuring SSH keys are found regardless of the current working directory at runtime.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
///
/// # Returns
///
/// Returns a `String` containing the complete environment configuration as pretty-printed JSON
///
/// # Example
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;
///
/// let config = create_test_environment_config("test-env");
/// println!("{}", config);
/// ```
pub fn create_test_environment_config(environment_name: &str) -> String {
    // Use compile-time constant to get project root - more reliable than current_dir()
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    info!(
        private_key = %private_key_path,
        public_key = %public_key_path,
        environment_name = %environment_name,
        "Generated environment configuration with absolute SSH key paths"
    );

    // Create configuration JSON with absolute paths and tracker configuration
    // This must match the format expected by EnvironmentCreationConfig
    serde_json::json!({
        "environment": {
            "name": environment_name
        },
        "ssh_credentials": {
            "private_key_path": private_key_path,
            "public_key_path": public_key_path
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
    })
    .to_string()
}
