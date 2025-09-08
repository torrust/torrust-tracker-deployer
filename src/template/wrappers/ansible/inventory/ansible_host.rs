//! Ansible host wrapper type for IP address validation and serialization

use anyhow::{Context, Result};
use serde::Serialize;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

/// Wrapper type for Ansible host address using the newtype pattern
///
/// Ansible's `ansible_host` can contain:
/// - Hostnames (e.g., "server.example.com")
/// - FQDN (e.g., "www.example.com")
/// - IP addresses (IPv4/IPv6)
/// - Custom connection aliases
/// - SSH proxy configurations
///
/// For this implementation, we only support IP addresses (IPv4 and IPv6) for simplicity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsibleHost(IpAddr);

impl AnsibleHost {
    /// Create a new `AnsibleHost` from an IP address
    #[must_use]
    pub fn new(ip: IpAddr) -> Self {
        Self(ip)
    }

    /// Get the inner IP address
    #[must_use]
    pub fn as_ip_addr(&self) -> &IpAddr {
        &self.0
    }

    /// Convert to string representation
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for AnsibleHost {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip = IpAddr::from_str(s).with_context(|| format!("Invalid IP address format: {s}"))?;
        Ok(Self(ip))
    }
}

impl fmt::Display for AnsibleHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for AnsibleHost {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}
