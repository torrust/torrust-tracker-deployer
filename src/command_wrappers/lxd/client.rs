//! LXD client for container and VM instance management
//!
//! This module provides the `LxdClient` which wraps LXD command-line tools to provide
//! a Rust-native interface for managing LXD containers and virtual machines.
//!
//! ## Key Features
//!
//! - Instance lifecycle management (list, inspect, control)
//! - IP address retrieval and network information
//! - JSON output parsing for structured data access
//! - Integration with the command execution framework
//! - Support for both containers and virtual machines
//!
//! The client abstracts the complexity of LXD command-line interaction and provides
//! type-safe APIs for common instance management tasks.

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

impl Default for LxdClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LxdClient {
    /// Creates a new `LxdClient`
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_executor: CommandExecutor::new(),
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

    /// Wait for an instance to get an IP address (useful for VMs that take time to boot)
    ///
    /// # Arguments
    ///
    /// * `instance_name` - Name of the instance to wait for
    /// * `timeout_seconds` - Maximum time to wait in seconds
    /// * `poll_interval_seconds` - How often to check in seconds
    ///
    /// # Returns
    /// * `Ok(IpAddr)` - The IP address when found
    /// * `Err(anyhow::Error)` - Timeout or other error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * Timeout is reached without getting an IP
    /// * LXD command execution fails
    /// * JSON parsing fails
    pub fn wait_for_instance_ip(
        &self,
        instance_name: &str,
        timeout_seconds: u64,
        poll_interval_seconds: u64,
    ) -> Result<IpAddr> {
        use std::time::{Duration, Instant};

        info!(
            "Waiting for instance '{}' to get IP address (timeout: {}s, poll interval: {}s)",
            instance_name, timeout_seconds, poll_interval_seconds
        );

        let start_time = Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);
        let poll_interval = Duration::from_secs(poll_interval_seconds);

        loop {
            if let Some(ip) = self.get_instance_ip(instance_name)? {
                info!(
                    "Instance '{}' got IP address: {} (waited {:?})",
                    instance_name,
                    ip,
                    start_time.elapsed()
                );
                return Ok(ip);
            }

            if start_time.elapsed() >= timeout {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for instance '{}' to get IP address after {:?}",
                    instance_name,
                    timeout
                ));
            }

            std::thread::sleep(poll_interval);
        }
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

    /// Delete an LXD instance
    ///
    /// # Arguments
    ///
    /// * `instance_name` - Name of the instance to delete
    /// * `force` - Whether to force deletion (stop running instances)
    ///
    /// # Returns
    /// * `Ok(())` - Instance deleted successfully or didn't exist
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The LXD command fails with an unexpected error
    /// * LXD is not installed or accessible
    pub fn delete_instance(&self, instance_name: &str, force: bool) -> Result<()> {
        info!("Deleting LXD instance: {}", instance_name);

        let mut args = vec!["delete", instance_name];
        if force {
            args.push("--force");
        }

        let result = self.command_executor.run_command("lxc", &args, None);

        match result {
            Ok(_) => {
                info!("LXD instance '{}' deleted successfully", instance_name);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Instance not found is not an error for cleanup operations
                if error_msg.contains("not found") || error_msg.contains("does not exist") {
                    info!(
                        "LXD instance '{}' doesn't exist, skipping deletion",
                        instance_name
                    );
                    Ok(())
                } else {
                    Err(anyhow::Error::from(e)
                        .context(format!("Failed to delete LXD instance '{instance_name}'")))
                }
            }
        }
    }

    /// Delete an LXD profile
    ///
    /// # Arguments
    ///
    /// * `profile_name` - Name of the profile to delete
    ///
    /// # Returns
    /// * `Ok(())` - Profile deleted successfully or didn't exist
    /// * `Err(anyhow::Error)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The LXD command fails with an unexpected error
    /// * LXD is not installed or accessible
    /// * Profile is in use by existing instances
    pub fn delete_profile(&self, profile_name: &str) -> Result<()> {
        info!("Deleting LXD profile: {}", profile_name);

        let args = vec!["profile", "delete", profile_name];

        let result = self.command_executor.run_command("lxc", &args, None);

        match result {
            Ok(_) => {
                info!("LXD profile '{}' deleted successfully", profile_name);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Profile not found is not an error for cleanup operations
                if error_msg.contains("not found") || error_msg.contains("does not exist") {
                    info!(
                        "LXD profile '{}' doesn't exist, skipping deletion",
                        profile_name
                    );
                    Ok(())
                } else {
                    Err(anyhow::Error::from(e)
                        .context(format!("Failed to delete LXD profile '{profile_name}'")))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_lxd_client_successfully() {
        let _client = LxdClient::new();
        // Client should be created successfully
        // Note: Logging is handled by the tracing crate via CommandExecutor
    }

    #[test]
    fn it_should_create_lxd_client_with_default_implementation() {
        let _client = LxdClient::default();
        // Client should be created successfully using Default trait
    }

    #[test]
    fn it_should_return_none_when_instance_not_found() {
        let _client = LxdClient::new();
        // We can't easily test this without mocking CommandExecutor, but the behavior
        // is now that get_instance_ip returns Ok(None) instead of an error when
        // the instance is not found or has no IP address.
        // This is tested implicitly through the other unit tests of the parser.
    }
}
