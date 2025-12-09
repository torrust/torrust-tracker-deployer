//! Tracker port configuration for E2E testing
//!
//! This module provides types for managing tracker port configurations in E2E tests.
//! These types are intentionally decoupled from production code to avoid tight coupling
//! with internal implementation details.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Complete E2E environment information including ports and configuration paths
///
/// This type encapsulates all the information needed for E2E container setup
/// and testing, avoiding coupling with production types.
#[derive(Debug, Clone)]
pub struct E2eEnvironmentInfo {
    /// Environment name (e.g., "e2e-config")
    pub environment_name: String,
    /// Path to the environment configuration JSON file
    pub config_file_path: PathBuf,
    /// SSH port for container access
    pub ssh_port: u16,
    /// Tracker port configuration
    pub tracker_ports: TrackerPorts,
}

impl E2eEnvironmentInfo {
    /// Create E2E environment info from configuration file
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

    /// Get the SSH socket address for this environment
    ///
    /// With host networking, the SSH port inside and outside the container
    /// is the same, so we bind to localhost with the configured SSH port.
    #[must_use]
    pub fn ssh_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], self.ssh_port))
    }
}

/// Tracker port configuration extracted from environment JSON file
///
/// This is a simplified E2E-specific type that extracts only the port numbers
/// needed for container setup, avoiding coupling with production types.
#[derive(Debug, Clone)]
pub struct TrackerPorts {
    /// HTTP API port (default: 1212)
    pub http_api_port: u16,
    /// HTTP tracker port (default: 7070)
    pub http_tracker_port: u16,
    /// UDP tracker port (default: 6969)
    pub udp_tracker_port: u16,
}

impl TrackerPorts {
    /// Extract tracker ports from an environment configuration JSON file
    ///
    /// This reads the environment JSON file and extracts the tracker port numbers
    /// without depending on production types.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be read
    /// - JSON parsing fails
    /// - Required tracker configuration is missing
    pub fn from_env_file(env_file_path: &Path) -> Result<Self> {
        let json_content = std::fs::read_to_string(env_file_path).with_context(|| {
            format!(
                "Failed to read environment file: {}",
                env_file_path.display()
            )
        })?;

        let env_json: EnvironmentJson =
            serde_json::from_str(&json_content).context("Failed to parse environment JSON")?;

        // Extract HTTP API port (from http_api.bind_address if present, otherwise default 1212)
        let http_api_port = env_json
            .user_inputs
            .tracker
            .http_api
            .as_ref()
            .and_then(|api| extract_port_from_bind_address(&api.bind_address))
            .unwrap_or(1212);

        // Extract HTTP tracker port from first HTTP tracker (or default 7070)
        let http_tracker_port = env_json
            .user_inputs
            .tracker
            .http_trackers
            .first()
            .and_then(|tracker| extract_port_from_bind_address(&tracker.bind_address))
            .unwrap_or(7070);

        // Extract UDP tracker port from first UDP tracker (or default 6969)
        let udp_tracker_port = env_json
            .user_inputs
            .tracker
            .udp_trackers
            .first()
            .and_then(|tracker| extract_port_from_bind_address(&tracker.bind_address))
            .unwrap_or(6969);

        Ok(Self {
            http_api_port,
            http_tracker_port,
            udp_tracker_port,
        })
    }

    /// Get all TCP ports that need to be exposed
    ///
    /// Returns HTTP API and HTTP tracker ports (UDP tracker is not exposed via TCP)
    #[must_use]
    pub fn tcp_ports(&self) -> Vec<u16> {
        vec![self.http_api_port, self.http_tracker_port]
    }

    /// Get all ports (TCP and UDP) that need to be exposed
    #[must_use]
    pub fn all_ports(&self) -> Vec<u16> {
        vec![
            self.http_api_port,
            self.http_tracker_port,
            self.udp_tracker_port,
        ]
    }
}

/// Extract port number from bind address (e.g., "0.0.0.0:7070" -> 7070)
fn extract_port_from_bind_address(bind_address: &str) -> Option<u16> {
    bind_address.split(':').nth(1)?.parse().ok()
}

/// Extract SSH port from environment configuration file
fn extract_ssh_port_from_file(env_file_path: &Path) -> Option<u16> {
    let json_content = std::fs::read_to_string(env_file_path).ok()?;
    let env_json: EnvironmentJson = serde_json::from_str(&json_content).ok()?;
    Some(env_json.user_inputs.ssh_port)
}

// E2E-specific JSON structure (minimal, only what we need)
#[derive(Debug, Deserialize, Serialize)]
struct EnvironmentJson {
    #[serde(rename = "Created")]
    user_inputs: UserInputs,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserInputs {
    #[serde(default = "default_ssh_port")]
    ssh_port: u16,
    tracker: TrackerConfig,
}

fn default_ssh_port() -> u16 {
    22
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackerConfig {
    #[serde(default)]
    udp_trackers: Vec<TrackerBinding>,
    #[serde(default)]
    http_trackers: Vec<TrackerBinding>,
    #[serde(default)]
    http_api: Option<HttpApiConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackerBinding {
    bind_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HttpApiConfig {
    #[serde(default = "default_api_bind_address")]
    bind_address: String,
}

fn default_api_bind_address() -> String {
    "0.0.0.0:1212".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_extract_port_from_bind_address() {
        assert_eq!(extract_port_from_bind_address("0.0.0.0:7070"), Some(7070));
        assert_eq!(extract_port_from_bind_address("127.0.0.1:1212"), Some(1212));
        assert_eq!(extract_port_from_bind_address("0.0.0.0:6969"), Some(6969));
        assert_eq!(extract_port_from_bind_address("invalid"), None);
        assert_eq!(extract_port_from_bind_address("0.0.0.0"), None);
    }

    #[test]
    fn it_should_use_default_ports_when_missing() {
        let json = r#"{
            "Created": {
                "tracker": {
                    "udp_trackers": [],
                    "http_trackers": []
                }
            }
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), json).unwrap();

        let ports = TrackerPorts::from_env_file(temp_file.path()).unwrap();

        assert_eq!(ports.http_api_port, 1212);
        assert_eq!(ports.http_tracker_port, 7070);
        assert_eq!(ports.udp_tracker_port, 6969);
    }

    #[test]
    fn it_should_extract_custom_ports() {
        let json = r#"{
            "Created": {
                "tracker": {
                    "udp_trackers": [
                        {"bind_address": "0.0.0.0:6969"}
                    ],
                    "http_trackers": [
                        {"bind_address": "0.0.0.0:7070"}
                    ],
                    "http_api": {
                        "bind_address": "0.0.0.0:1212"
                    }
                }
            }
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), json).unwrap();

        let ports = TrackerPorts::from_env_file(temp_file.path()).unwrap();

        assert_eq!(ports.http_api_port, 1212);
        assert_eq!(ports.http_tracker_port, 7070);
        assert_eq!(ports.udp_tracker_port, 6969);
    }

    #[test]
    fn it_should_return_tcp_ports() {
        let ports = TrackerPorts {
            http_api_port: 1212,
            http_tracker_port: 7070,
            udp_tracker_port: 6969,
        };

        let tcp_ports = ports.tcp_ports();
        assert_eq!(tcp_ports, vec![1212, 7070]);
    }

    #[test]
    fn it_should_return_all_ports() {
        let ports = TrackerPorts {
            http_api_port: 1212,
            http_tracker_port: 7070,
            udp_tracker_port: 6969,
        };

        let all_ports = ports.all_ports();
        assert_eq!(all_ports, vec![1212, 7070, 6969]);
    }
}
