use std::net::IpAddr;

use super::name::InstanceName;

/// Instance information from LXD
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceInfo {
    pub name: InstanceName,
    pub ip_address: Option<IpAddr>,
}
