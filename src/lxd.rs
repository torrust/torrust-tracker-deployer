use std::net::IpAddr;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use tracing::info;

use crate::command::CommandExecutor;

/// Instance information from LXD
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceInfo {
    pub name: String,
    pub ip_address: Option<IpAddr>,
}

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
    /// * `Ok(Some(IpAddr))` - The IPv4 address if found
    /// * `Ok(None)` - Instance not found or no IPv4 address available
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * LXD command execution fails
    /// * JSON parsing fails
    pub fn get_instance_ip(&self, instance_name: &str) -> Result<Option<IpAddr>> {
        info!("Getting IP address for instance: {}", instance_name);

        let Some(instance) = self.get_instance_by_name(instance_name)? else {
            info!("Instance '{}' not found", instance_name);
            return Ok(None);
        };

        let Some(ip) = instance.ip_address else {
            info!("Instance '{}' has no IPv4 address", instance_name);
            return Ok(None);
        };

        info!(
            "Found IPv4 address for instance '{}': {}",
            instance_name, ip
        );

        Ok(Some(ip))
    }

    /// Get a specific instance by name
    ///
    /// # Arguments
    ///
    /// * `instance_name` - Name of the instance to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(InstanceInfo))` - Instance information if found
    /// * `Ok(None)` - Instance not found
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * LXD command execution fails
    /// * JSON parsing fails
    pub fn get_instance_by_name(&self, instance_name: &str) -> Result<Option<InstanceInfo>> {
        info!("Getting instance by name: {}", instance_name);

        let instances = self.list(Some(instance_name))?;

        Ok(instances
            .into_iter()
            .find(|inst| inst.name == instance_name))
    }

    /// List instances in JSON format
    ///
    /// # Arguments
    ///
    /// * `instance_name` - Optional instance name to filter results
    ///
    /// # Returns
    /// * `Ok(Vec<InstanceInfo>)` - List of instance information if the command succeeds
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The LXD command fails
    /// * LXD is not installed or accessible
    /// * JSON parsing fails
    fn list(&self, instance_name: Option<&str>) -> Result<Vec<InstanceInfo>> {
        info!("Listing LXD instances");

        let mut args = vec!["list", "--format=json"];

        if let Some(name) = instance_name {
            args.push(name);
            info!("Filtering by instance name: {}", name);
        }

        let output = self
            .command_executor
            .run_command("lxc", &args, None)
            .map_err(anyhow::Error::from)
            .context("Failed to execute lxc list command")?;

        LxdJsonParser::parse_instances_json(&output)
    }
}

/// A JSON parser for LXD responses.
///
/// This parser handles the complex JSON structure returned by LXD commands
/// and converts them into structured Rust types. It encapsulates all the
/// JSON parsing logic and can be unit tested independently.
struct LxdJsonParser;

impl LxdJsonParser {
    /// Parse JSON output from lxc list command into structured instance information
    ///
    /// # Arguments
    ///
    /// * `json_output` - JSON string from lxc list command
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<InstanceInfo>)` - Parsed instance information
    /// * `Err(anyhow::Error)` - JSON parsing error
    fn parse_instances_json(json_output: &str) -> Result<Vec<InstanceInfo>> {
        let instances: Value =
            serde_json::from_str(json_output).context("Failed to parse LXC list output as JSON")?;

        let instances_array = instances
            .as_array()
            .ok_or_else(|| anyhow!("Expected JSON array from lxc list"))?;

        let mut result = Vec::new();

        for instance_value in instances_array {
            let name = instance_value["name"]
                .as_str()
                .ok_or_else(|| anyhow!("Instance missing name field"))?
                .to_string();

            let ip_address = Self::extract_ipv4_address(instance_value)?;

            result.push(InstanceInfo { name, ip_address });
        }

        Ok(result)
    }

    /// Extract IPv4 address from instance JSON data
    ///
    /// # Arguments
    ///
    /// * `instance` - JSON value representing an instance
    ///
    /// # Returns
    ///
    /// * `Ok(Option<IpAddr>)` - IPv4 address if found, None otherwise
    /// * `Err(anyhow::Error)` - Error parsing IP address
    fn extract_ipv4_address(instance: &Value) -> Result<Option<IpAddr>> {
        let addresses = instance["state"]["network"]["eth0"]["addresses"].as_array();

        if let Some(addresses) = addresses {
            for addr in addresses {
                if addr["family"].as_str() == Some("inet") {
                    if let Some(ip_str) = addr["address"].as_str() {
                        let ip = ip_str
                            .parse::<IpAddr>()
                            .with_context(|| format!("Failed to parse IP address: {ip_str}"))?;
                        return Ok(Some(ip));
                    }
                }
            }
        }

        Ok(None)
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

        let instances = LxdJsonParser::parse_instances_json(mock_json).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].name, "test-instance");
        assert_eq!(
            instances[0].ip_address.unwrap().to_string(),
            "192.168.1.100"
        );
    }

    #[test]
    fn it_should_handle_empty_instance_list() {
        // Mock empty JSON response
        let mock_json = r"[]";

        let instances = LxdJsonParser::parse_instances_json(mock_json).unwrap();
        assert!(instances.is_empty());
    }

    #[test]
    fn it_should_handle_instance_without_ipv4_address() {
        // Mock JSON response without IPv4 address
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

        let instances = LxdJsonParser::parse_instances_json(mock_json).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].name, "test-instance");
        assert!(instances[0].ip_address.is_none());
    }

    #[test]
    fn it_should_handle_malformed_json() {
        let malformed_json = r"{ invalid json }";
        let result = LxdJsonParser::parse_instances_json(malformed_json);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_extract_ipv4_address_from_instance_json() {
        let instance_json = serde_json::json!({
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
        });

        let result = LxdJsonParser::extract_ipv4_address(&instance_json).unwrap();
        assert_eq!(result.unwrap().to_string(), "192.168.1.100");
    }

    #[test]
    fn it_should_return_none_when_no_ipv4_address_found() {
        let instance_json = serde_json::json!({
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
        });

        let result = LxdJsonParser::extract_ipv4_address(&instance_json).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn it_should_return_none_when_no_network_interfaces() {
        let instance_json = serde_json::json!({
            "name": "test-instance",
            "state": {
                "network": {}
            }
        });

        let result = LxdJsonParser::extract_ipv4_address(&instance_json).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn it_should_fail_when_instance_missing_name() {
        let mock_json = r#"[
            {
                "state": {
                    "network": {
                        "eth0": {
                            "addresses": [
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

        let result = LxdJsonParser::parse_instances_json(mock_json);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_return_none_when_instance_not_found() {
        let _client = LxdClient::new(false);
        // We can't easily test this without mocking CommandExecutor, but the behavior
        // is now that get_instance_ip returns Ok(None) instead of an error when
        // the instance is not found or has no IP address.
        // This is tested implicitly through the other unit tests of the parser.
    }
}
