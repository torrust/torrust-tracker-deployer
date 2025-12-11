//! Tracker port configuration for E2E testing
//!
//! This module provides types for managing tracker port configurations in E2E tests.
//! These types are intentionally decoupled from production code to avoid tight coupling
//! with internal implementation details.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
                }
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

/// E2E runtime environment - represents actual runtime state after container starts
///
/// This type contains the actual mapped ports returned by Docker when using
/// bridge networking mode. These may differ from the requested ports in the config.
#[derive(Debug, Clone)]
pub struct E2eRuntimeEnvironment {
    /// Configuration environment (what we requested)
    pub config: E2eConfigEnvironment,
    /// Actual mapped ports from Docker (what we got)
    pub container_ports: ContainerPorts,
}

impl E2eRuntimeEnvironment {
    /// Create a new runtime environment from config and container ports
    #[must_use]
    pub fn new(config: E2eConfigEnvironment, container_ports: ContainerPorts) -> Self {
        Self {
            config,
            container_ports,
        }
    }

    /// Get the SSH socket address using the mapped SSH port
    #[must_use]
    pub fn ssh_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], self.container_ports.ssh_port))
    }

    /// Get the tracker API URL for external access
    #[must_use]
    pub fn tracker_api_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.container_ports.http_api_port)
    }

    /// Get the HTTP tracker URL for external access
    #[must_use]
    pub fn http_tracker_url(&self) -> String {
        format!(
            "http://127.0.0.1:{}",
            self.container_ports.http_tracker_port
        )
    }
}

/// Container ports - actual mapped ports from Docker
///
/// With bridge networking, Docker dynamically assigns host ports that map to
/// the container's internal ports. This type holds those actual mapped ports.
#[derive(Debug, Clone)]
pub struct ContainerPorts {
    /// Mapped SSH port on the host
    pub ssh_port: u16,
    /// Mapped HTTP API port on the host
    pub http_api_port: u16,
    /// Mapped HTTP tracker port on the host
    pub http_tracker_port: u16,
    /// Mapped UDP tracker port on the host
    pub udp_tracker_port: u16,
}

impl ContainerPorts {
    /// Create container ports from a list of mapped ports
    ///
    /// # Arguments
    /// * `ssh_port` - Mapped SSH port
    /// * `additional_ports` - Mapped additional ports in order: [`http_api`, `http_tracker`, `udp_tracker`]
    ///
    /// # Panics
    /// Panics if `additional_ports` doesn't have exactly 3 elements
    #[must_use]
    pub fn from_mapped_ports(ssh_port: u16, additional_ports: &[u16]) -> Self {
        assert_eq!(
            additional_ports.len(),
            3,
            "Expected exactly 3 additional ports (http_api, http_tracker, udp_tracker)"
        );

        Self {
            ssh_port,
            http_api_port: additional_ports[0],
            http_tracker_port: additional_ports[1],
            udp_tracker_port: additional_ports[2],
        }
    }
}

// Deprecated: Keep for backward compatibility during migration
/// @deprecated Use `E2eConfigEnvironment` instead
pub type E2eEnvironmentInfo = E2eConfigEnvironment;

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

impl Default for TrackerPorts {
    /// Create tracker ports with default values
    ///
    /// Default ports match the standard test configuration:
    /// - HTTP API: 1212
    /// - HTTP tracker: 7070
    /// - UDP tracker: 6969
    fn default() -> Self {
        Self {
            http_api_port: 1212,
            http_tracker_port: 7070,
            udp_tracker_port: 6969,
        }
    }
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
        let json_content = std::fs::read(env_file_path).with_context(|| {
            format!(
                "Failed to read environment file: {}",
                env_file_path.display()
            )
        })?;

        // Try to parse as EnvironmentCreationConfig first (new format)
        if let Ok(config_json) = serde_json::from_slice::<ConfigJson>(&json_content) {
            // Extract HTTP API port (default 1212 - not configurable in user config)
            let http_api_port = 1212;

            // Extract HTTP tracker port from first HTTP tracker (or default 7070)
            let http_tracker_port = config_json
                .tracker
                .http_trackers
                .first()
                .and_then(|tracker| extract_port_from_bind_address(&tracker.bind_address))
                .unwrap_or(7070);

            // Extract UDP tracker port from first UDP tracker (or default 6969)
            let udp_tracker_port = config_json
                .tracker
                .udp_trackers
                .first()
                .and_then(|tracker| extract_port_from_bind_address(&tracker.bind_address))
                .unwrap_or(6969);

            return Ok(Self {
                http_api_port,
                http_tracker_port,
                udp_tracker_port,
            });
        }

        // Fallback to EnvironmentJson format (old saved state format)
        let env_json: EnvironmentJson =
            serde_json::from_slice(&json_content).context("Failed to parse environment JSON")?;

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

    // Try to parse as EnvironmentCreationConfig first (new format)
    if let Ok(config_json) = serde_json::from_str::<ConfigJson>(&json_content) {
        return Some(config_json.ssh_credentials.port.unwrap_or(22));
    }

    // Fallback to EnvironmentJson format (old saved state format)
    let env_json: EnvironmentJson = serde_json::from_str(&json_content).ok()?;
    Some(env_json.user_inputs.ssh_port)
}

// EnvironmentCreationConfig JSON structure (new format - configuration files)
#[derive(Debug, Deserialize, Serialize)]
struct ConfigJson {
    ssh_credentials: SshCredentialsConfig,
    tracker: TrackerConfigCreation,
}

#[derive(Debug, Deserialize, Serialize)]
struct SshCredentialsConfig {
    #[serde(default)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackerConfigCreation {
    core: TrackerCoreConfig,
    #[serde(default)]
    udp_trackers: Vec<TrackerBinding>,
    #[serde(default)]
    http_trackers: Vec<TrackerBinding>,
    http_api: HttpApiConfigCreation,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackerCoreConfig {
    // We don't need the fields, just need the struct to exist
}

#[derive(Debug, Deserialize, Serialize)]
struct HttpApiConfigCreation {
    admin_token: String,
}

// E2E-specific JSON structure (old format - saved environment state)
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
