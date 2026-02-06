//! Tracker port configuration extraction for E2E testing
//!
//! Extracts tracker port numbers from environment configuration files.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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

// JSON deserialization types for configuration formats

// EnvironmentCreationConfig JSON structure (new format - configuration files)
#[derive(Debug, Deserialize, Serialize)]
struct ConfigJson {
    tracker: TrackerConfigCreation,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackerConfigCreation {
    #[serde(default)]
    udp_trackers: Vec<TrackerBinding>,
    #[serde(default)]
    http_trackers: Vec<TrackerBinding>,
}

// E2E-specific JSON structure (old format - saved environment state)
#[derive(Debug, Deserialize, Serialize)]
struct EnvironmentJson {
    #[serde(rename = "Created")]
    user_inputs: UserInputs,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserInputs {
    tracker: TrackerConfig,
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
    bind_address: String,
}

/// Extract port number from bind address (e.g., "0.0.0.0:7070" -> 7070)
fn extract_port_from_bind_address(bind_address: &str) -> Option<u16> {
    bind_address.split(':').nth(1)?.parse().ok()
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
}
