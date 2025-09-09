use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use tracing::info;

use crate::command::{CommandError, CommandExecutor};

/// A specialized LXD client for instance management.
///
/// This client provides a consistent interface for LXD operations:
/// - List instances (containers and virtual machines) and their information
/// - Retrieve instance IP addresses
/// - Execute LXD commands with proper error handling
///
/// Uses `CommandExecutor` as a collaborator for actual command execution.
pub struct LxdClient {
    command_executor: CommandExecutor,
}

impl LxdClient {
    /// Creates a new `LxdClient`
    ///
    /// # Arguments
    /// 
    /// * `verbose` - Whether to log commands being executed
    #[must_use]
    pub fn new(verbose: bool) -> Self {
        Self {
            command_executor: CommandExecutor::new(verbose),
        }
    }

    /// Get the IPv4 address of a specific instance
    ///
    /// # Arguments
    /// 
    /// * `instance_name` - Name of the instance to get the IP address for
    ///
    /// # Returns
    /// * `Ok(String)` - The IPv4 address if found
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The instance is not found
    /// * The instance doesn't have an IPv4 address
    /// * LXD command execution fails
    /// * JSON parsing fails
    pub fn get_instance_ip(&self, instance_name: &str) -> Result<String> {
        info!("Getting IP address for instance: {}", instance_name);

        let output = self
            .list(Some(instance_name))
            .map_err(anyhow::Error::from)
            .with_context(|| format!("Failed to list instance: {instance_name}"))?;

        let instances: Value =
            serde_json::from_str(&output).context("Failed to parse LXC list output as JSON")?;

        let instance = instances
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow!("Instance '{}' not found", instance_name))?;

        let ip = instance["state"]["network"]["eth0"]["addresses"]
            .as_array()
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr["family"].as_str() == Some("inet"))
            })
            .and_then(|addr| addr["address"].as_str())
            .ok_or_else(|| {
                anyhow!(
                    "Could not find IPv4 address for instance '{}'",
                    instance_name
                )
            })?;

        info!(
            "Found IPv4 address for instance '{}': {}",
            instance_name, ip
        );

        Ok(ip.to_string())
    }

    /// List instances in JSON format
    ///
    /// # Arguments
    /// 
    /// * `instance_name` - Optional instance name to filter results
    ///
    /// # Returns
    /// * `Ok(String)` - The JSON output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The LXD command fails
    /// * LXD is not installed or accessible
    fn list(&self, instance_name: Option<&str>) -> Result<String, CommandError> {
        info!("Listing LXD instances");

        let mut args = vec!["list", "--format=json"];
        if let Some(name) = instance_name {
            args.push(name);
            info!("Filtering by instance name: {}", name);
        }

        self.command_executor.run_command("lxc", &args, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_lxd_client_with_verbose_disabled() {
        let _client = LxdClient::new(false);
        // Client should be created successfully
        // Note: We can't directly test the internal state since CommandExecutor
        // encapsulates the verbose setting
    }

    #[test]
    fn it_should_create_lxd_client_with_verbose_enabled() {
        let _client = LxdClient::new(true);
        // Client should be created successfully
    }
    #[test]
    fn it_should_parse_instance_ip_from_valid_json() {
        let _client = LxdClient::new(false);

        // Mock JSON response similar to what LXD returns
        let mock_json = r#"[
            {
                "name": "test-instance",
                "state": {
                    "network": {
                        "eth0": {
                            "addresses": [
                                {
                                    "family": "inet6",
                                    "address": "fe80::1"
                                },
                                {
                                    "family": "inet", 
                                    "address": "192.168.1.100"
                                }
                            ]
                        }
                    }
                }
            }
        ]"#;

        // Test the JSON parsing logic directly
        let instances: Value = serde_json::from_str(mock_json).unwrap();
        let instance = instances.as_array().unwrap().first().unwrap();

        let ip = instance["state"]["network"]["eth0"]["addresses"]
            .as_array()
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr["family"].as_str() == Some("inet"))
            })
            .and_then(|addr| addr["address"].as_str())
            .unwrap();

        assert_eq!(ip, "192.168.1.100");
    }

    #[test]
    fn it_should_handle_empty_instance_list() {
        let _client = LxdClient::new(false);

        // Mock empty JSON response
        let mock_json = r"[]";

        let instances: Value = serde_json::from_str(mock_json).unwrap();
        let result = instances.as_array().and_then(|arr| arr.first());

        assert!(result.is_none());
    }

    #[test]
    fn it_should_handle_instance_without_ipv4_address() {
        let _client = LxdClient::new(false); // Mock JSON response without IPv4 address
        let mock_json = r#"[
            {
                "name": "test-instance",
                "state": {
                    "network": {
                        "eth0": {
                            "addresses": [
                                {
                                    "family": "inet6",
                                    "address": "fe80::1"
                                }
                            ]
                        }
                    }
                }
            }
        ]"#;

        let instances: Value = serde_json::from_str(mock_json).unwrap();
        let instance = instances.as_array().unwrap().first().unwrap();

        let result = instance["state"]["network"]["eth0"]["addresses"]
            .as_array()
            .and_then(|addresses| {
                addresses
                    .iter()
                    .find(|addr| addr["family"].as_str() == Some("inet"))
            })
            .and_then(|addr| addr["address"].as_str());

        assert!(result.is_none());
    }

    #[test]
    fn it_should_handle_malformed_json() {
        let _client = LxdClient::new(false);

        let malformed_json = r"{ invalid json }";
        let result = serde_json::from_str::<Value>(malformed_json);

        assert!(result.is_err());
    }
}
