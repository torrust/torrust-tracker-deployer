//! Ansible host wrapper type for IP address validation and serialization

use derive_more::{Display, From};
use serde::Serialize;
use std::net::IpAddr;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur when working with Ansible hosts
#[derive(Debug, Error, PartialEq)]
pub enum AnsibleHostError {
    #[error("Invalid IP address format: {input}")]
    InvalidIpAddress { input: String },
}

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
#[derive(Debug, Clone, PartialEq, Eq, Display, From, Serialize)]
#[display(fmt = "{ip}")]
#[serde(transparent)]
pub struct AnsibleHost {
    ip: IpAddr,
}

impl AnsibleHost {
    /// Create a new `AnsibleHost` from an IP address
    #[must_use]
    pub fn new(ip: IpAddr) -> Self {
        Self { ip }
    }

    /// Get the inner IP address
    #[must_use]
    pub fn as_ip_addr(&self) -> &IpAddr {
        &self.ip
    }

    /// Convert to string representation
    #[must_use]
    pub fn as_str(&self) -> String {
        self.ip.to_string()
    }
}

impl FromStr for AnsibleHost {
    type Err = AnsibleHostError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip = IpAddr::from_str(s).map_err(|_| AnsibleHostError::InvalidIpAddress {
            input: s.to_string(),
        })?;
        Ok(Self::new(ip))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn it_should_create_ansible_host_with_ipv4() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let host = AnsibleHost::new(ip);
        assert_eq!(host.as_ip_addr(), &ip);
        assert_eq!(host.as_str(), "192.168.1.1");
    }

    #[test]
    fn it_should_create_ansible_host_with_ipv6() {
        let ip = IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x0db8, 0x85a3, 0, 0, 0x8a2e, 0x0370, 0x7334,
        ));
        let host = AnsibleHost::new(ip);
        assert_eq!(host.as_ip_addr(), &ip);
        assert_eq!(host.as_str(), "2001:db8:85a3::8a2e:370:7334");
    }

    #[test]
    fn it_should_parse_valid_ipv4_from_string() {
        let result = AnsibleHost::from_str("192.168.1.1");
        assert!(result.is_ok());
        let host = result.unwrap();
        assert_eq!(host.as_str(), "192.168.1.1");
    }

    #[test]
    fn it_should_parse_valid_ipv6_from_string() {
        let result = AnsibleHost::from_str("2001:db8:85a3::8a2e:370:7334");
        assert!(result.is_ok());
        let host = result.unwrap();
        assert_eq!(host.as_str(), "2001:db8:85a3::8a2e:370:7334");
    }

    #[test]
    fn it_should_parse_localhost_ipv4() {
        let result = AnsibleHost::from_str("127.0.0.1");
        assert!(result.is_ok());
        let host = result.unwrap();
        assert_eq!(host.as_str(), "127.0.0.1");
    }

    #[test]
    fn it_should_parse_localhost_ipv6() {
        let result = AnsibleHost::from_str("::1");
        assert!(result.is_ok());
        let host = result.unwrap();
        assert_eq!(host.as_str(), "::1");
    }

    #[test]
    fn it_should_fail_with_invalid_ip_address() {
        let result = AnsibleHost::from_str("invalid.ip.address");
        assert_eq!(
            result,
            Err(AnsibleHostError::InvalidIpAddress {
                input: "invalid.ip.address".to_string()
            })
        );
    }

    #[test]
    fn it_should_fail_with_invalid_ipv4_values() {
        let result = AnsibleHost::from_str("256.256.256.256");
        assert_eq!(
            result,
            Err(AnsibleHostError::InvalidIpAddress {
                input: "256.256.256.256".to_string()
            })
        );
    }

    #[test]
    fn it_should_fail_with_empty_string() {
        let result = AnsibleHost::from_str("");
        assert_eq!(
            result,
            Err(AnsibleHostError::InvalidIpAddress {
                input: String::new()
            })
        );
    }

    #[test]
    fn it_should_implement_display_trait() {
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        assert_eq!(format!("{host}"), "192.168.1.100");
    }

    #[test]
    fn it_should_serialize_ipv4_to_json() {
        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let json = serde_json::to_string(&host).unwrap();
        assert_eq!(json, "\"10.0.0.1\"");
    }

    #[test]
    fn it_should_serialize_ipv6_to_json() {
        let host = AnsibleHost::from_str("::1").unwrap();
        let json = serde_json::to_string(&host).unwrap();
        assert_eq!(json, "\"::1\"");
    }

    #[test]
    fn it_should_support_clone_and_equality() {
        let host1 = AnsibleHost::from_str("192.168.1.1").unwrap();
        let host2 = host1.clone();
        assert_eq!(host1, host2);
    }

    #[test]
    fn it_should_support_from_trait_for_ipv4() {
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
        let host = AnsibleHost::from(ip);
        assert_eq!(host.as_str(), "172.16.0.1");
    }

    #[test]
    fn it_should_support_from_trait_for_ipv6() {
        let ip = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let host = AnsibleHost::from(ip);
        assert_eq!(host.as_str(), "::1");
    }

    #[test]
    fn it_should_display_error_message_correctly() {
        let error = AnsibleHostError::InvalidIpAddress {
            input: "bad_input".to_string(),
        };
        assert_eq!(format!("{error}"), "Invalid IP address format: bad_input");
    }
}
