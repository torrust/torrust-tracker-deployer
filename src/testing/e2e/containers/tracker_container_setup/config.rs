//! E2E configuration environment
//!
//! Represents the desired configuration for E2E testing.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::tracker_ports::TrackerPorts;

/// E2E configuration environment - represents the desired configuration
///
/// This type contains the configuration we want to use for E2E testing,
/// including the ports we request from the environment config.
#[derive(Debug, Clone)]
pub struct E2eConfigEnvironment {
    /// Environment name (e.g., "e2e-config")
    pub environment_name: String,
    /// Path to the environment configuration JSON file
    pub config_file_path: PathBuf,
    /// SSH port for container access (from config)
    pub ssh_port: u16,
    /// Tracker port configuration (from config)
    pub tracker_ports: TrackerPorts,
}

impl E2eConfigEnvironment {
    /// Create E2E config environment directly from values
    ///
    /// This is the primary constructor that builds the configuration in-memory
    /// without requiring file I/O. Use this when you want to work with the
    /// configuration before writing it to disk.
    ///
    /// # Arguments
    /// * `environment_name` - Name of the environment
    /// * `config_file_path` - Path where config will be written (if needed)
    /// * `ssh_port` - SSH port to use
    /// * `tracker_ports` - Tracker port configuration
    #[must_use]
    pub fn new(
        environment_name: String,
        config_file_path: PathBuf,
        ssh_port: u16,
        tracker_ports: TrackerPorts,
    ) -> Self {
        Self {
            environment_name,
            config_file_path,
            ssh_port,
            tracker_ports,
        }
    }

    /// Generate JSON configuration string from this E2E environment
    ///
    /// Creates a complete environment configuration JSON using the values
    /// from this struct, with absolute paths to SSH keys.
    ///
    /// # Returns
    ///
    /// Returns a JSON string ready to be written to the environment config file.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let env_info = E2eConfigEnvironment::new(...);
    /// let json = env_info.to_json_config();
    /// ```
    #[must_use]
    pub fn to_json_config(&self) -> String {
        // Use compile-time constant to get project root - more reliable than current_dir()
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Create configuration JSON with absolute paths and tracker configuration
        // This must match the format expected by EnvironmentCreationConfig
        serde_json::json!({
            "environment": {
                "name": &self.environment_name
            },
            "ssh_credentials": {
                "private_key_path": private_key_path,
                "public_key_path": public_key_path
            },
            "provider": {
                "provider": "lxd",
                "profile_name": format!("torrust-profile-{}", &self.environment_name)
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
                    {"bind_address": format!("0.0.0.0:{}", self.tracker_ports.udp_tracker_port)}
                ],
                "http_trackers": [
                    {"bind_address": format!("0.0.0.0:{}", self.tracker_ports.http_tracker_port)}
                ],
                "http_api": {
                    "bind_address": format!("0.0.0.0:{}", self.tracker_ports.http_api_port),
                    "admin_token": "MyAccessToken"
                },
                "health_check_api": {
                    "bind_address": "127.0.0.1:1313"
                }
            },
            "prometheus": {
                "scrape_interval_in_secs": 15
            },
            "grafana": {
                "admin_user": "admin",
                "admin_password": "e2e-test-password"
            }
        })
        .to_string()
    }

    /// Create E2E config environment from configuration file
    ///
    /// # Arguments
    /// * `environment_name` - Name of the environment
    /// * `config_file_path` - Path to the environment JSON configuration file
    /// * `ssh_port` - SSH port to use (or None to extract from config)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file cannot be read or parsed
    /// - Tracker configuration is invalid
    pub fn from_config_file(
        environment_name: String,
        config_file_path: PathBuf,
        ssh_port: Option<u16>,
    ) -> Result<Self> {
        let tracker_ports = TrackerPorts::from_env_file(&config_file_path)?;

        // Extract SSH port from config or use provided value
        let ssh_port =
            ssh_port.unwrap_or_else(|| extract_ssh_port_from_file(&config_file_path).unwrap_or(22));

        Ok(Self {
            environment_name,
            config_file_path,
            ssh_port,
            tracker_ports,
        })
    }
}

/// Extract SSH port from environment configuration file
fn extract_ssh_port_from_file(env_file_path: &Path) -> Option<u16> {
    let json_content = std::fs::read_to_string(env_file_path).ok()?;

    // Try to parse as EnvironmentCreationConfig first (new format)
    if let Ok(config_json) = serde_json::from_str::<ConfigJson>(&json_content) {
        return Some(config_json.ssh_credentials.port.unwrap_or(22));
    }

    // Fallback to EnvironmentJson format (old saved state format)
    let env_json: EnvironmentJson = serde_json::from_str(&json_content).ok()?;
    Some(env_json.user_inputs.ssh_port)
}

// JSON deserialization types for SSH port extraction

#[derive(Debug, Deserialize, Serialize)]
struct ConfigJson {
    ssh_credentials: SshCredentialsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct SshCredentialsConfig {
    #[serde(default)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EnvironmentJson {
    #[serde(rename = "Created")]
    user_inputs: UserInputs,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserInputs {
    #[serde(default = "default_ssh_port")]
    ssh_port: u16,
}

fn default_ssh_port() -> u16 {
    22
}
