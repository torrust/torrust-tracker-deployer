use std::net::IpAddr;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::instance::{InstanceInfo, InstanceName};

/// A JSON parser for LXD responses.
///
/// This parser handles the complex JSON structure returned by LXD commands
/// and converts them into structured Rust types. It encapsulates all the
/// JSON parsing logic and can be unit tested independently.
pub(crate) struct LxdJsonParser;

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
    pub fn parse_instances_json(json_output: &str) -> Result<Vec<InstanceInfo>> {
        let instances: Value =
            serde_json::from_str(json_output).context("Failed to parse LXC list output as JSON")?;

        let instances_array = instances
            .as_array()
            .ok_or_else(|| anyhow!("Expected JSON array from lxc list"))?;

        let mut result = Vec::new();

        for instance_value in instances_array {
            let name_str = instance_value["name"]
                .as_str()
                .ok_or_else(|| anyhow!("Instance missing name field"))?;

            let name = InstanceName::new(name_str.to_string())
                .with_context(|| format!("Invalid instance name: {name_str}"))?;

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
        let network = instance["state"]["network"].as_object();

        if let Some(network) = network {
            // Iterate through all network interfaces (eth0, enp5s0, etc.)
            for (interface_name, interface_data) in network {
                // Skip loopback interface
                if interface_name == "lo" {
                    continue;
                }

                if let Some(addresses) = interface_data["addresses"].as_array() {
                    for addr in addresses {
                        if addr["family"].as_str() == Some("inet") {
                            if let Some(ip_str) = addr["address"].as_str() {
                                let ip = ip_str.parse::<IpAddr>().with_context(|| {
                                    format!("Failed to parse IP address: {ip_str}")
                                })?;
                                return Ok(Some(ip));
                            }
                        }
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
        assert_eq!(instances[0].name.as_str(), "test-instance");
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
        assert_eq!(instances[0].name.as_str(), "test-instance");
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
}
