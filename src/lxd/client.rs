use std::net::IpAddr;

use anyhow::{Context, Result};
use tracing::info;

use crate::command::CommandExecutor;

use super::instance::InstanceInfo;
use super::json_parser::LxdJsonParser;

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
            .find(|inst| inst.name.as_str() == instance_name))
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
    fn it_should_return_none_when_instance_not_found() {
        let _client = LxdClient::new(false);
        // We can't easily test this without mocking CommandExecutor, but the behavior
        // is now that get_instance_ip returns Ok(None) instead of an error when
        // the instance is not found or has no IP address.
        // This is tested implicitly through the other unit tests of the parser.
    }
}
