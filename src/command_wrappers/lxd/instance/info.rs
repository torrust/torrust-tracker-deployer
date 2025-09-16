//! LXD instance information structures
//!
//! This module provides the `InstanceInfo` struct which contains runtime information
//! about LXD instances, including their names and network configuration.
//!
//! ## Key Features
//!
//! - Instance name and IP address tracking
//! - Optional IP address handling for instances that may not have network connectivity
//! - Integration with instance name validation
//! - Support for both containers and virtual machines
//!
//! The instance information is typically retrieved from LXD list commands and used
//! for connecting to and managing deployed instances.

use std::net::IpAddr;

use super::name::InstanceName;

/// Instance information from LXD
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceInfo {
    pub name: InstanceName,
    pub ip_address: Option<IpAddr>,
}
