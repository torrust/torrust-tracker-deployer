//! Ansible port wrapper type for port validation and serialization

use derive_more::{Display, From};
use serde::Serialize;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur when working with Ansible ports
#[derive(Debug, Error, PartialEq)]
pub enum AnsiblePortError {
    #[error("Invalid port number: {input} (must be between 1 and 65535)")]
    InvalidPortNumber { input: String },

    #[error("Port number parse error: {input}")]
    ParseError { input: String },
}

/// Wrapper type for Ansible SSH port using the newtype pattern
///
/// Ansible's `ansible_port` represents the SSH port to connect to.
/// Valid port numbers are in the range 1-65535.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, From, Serialize)]
#[display("{port}")]
#[serde(transparent)]
pub struct AnsiblePort {
    port: u16,
}

impl AnsiblePort {
    /// Create a new `AnsiblePort` from a port number
    ///
    /// # Errors
    ///
    /// Returns an error if the port number is 0 (invalid for SSH connections)
    pub fn new(port: u16) -> Result<Self, AnsiblePortError> {
        if port == 0 {
            return Err(AnsiblePortError::InvalidPortNumber {
                input: port.to_string(),
            });
        }
        Ok(Self { port })
    }

    /// Get the inner port number
    #[must_use]
    pub fn as_u16(&self) -> u16 {
        self.port
    }

    /// Convert to string representation
    #[must_use]
    pub fn as_str(&self) -> String {
        self.port.to_string()
    }
}

impl FromStr for AnsiblePort {
    type Err = AnsiblePortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let port = s.parse::<u16>().map_err(|_| AnsiblePortError::ParseError {
            input: s.to_string(),
        })?;
        Self::new(port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn it_should_create_ansible_port_with_valid_port() {
        let port = AnsiblePort::new(22).unwrap();
        assert_eq!(port.as_u16(), 22);
        assert_eq!(port.as_str(), "22");
    }

    #[test]
    fn it_should_create_ansible_port_with_high_port() {
        let port = AnsiblePort::new(65535).unwrap();
        assert_eq!(port.as_u16(), 65535);
        assert_eq!(port.as_str(), "65535");
    }

    #[test]
    fn it_should_fail_with_port_zero() {
        let result = AnsiblePort::new(0);
        assert_eq!(
            result,
            Err(AnsiblePortError::InvalidPortNumber {
                input: "0".to_string()
            })
        );
    }

    #[test]
    fn it_should_parse_valid_port_from_string() {
        let result = AnsiblePort::from_str("22");
        assert!(result.is_ok());
        let port = result.unwrap();
        assert_eq!(port.as_u16(), 22);
    }

    #[test]
    fn it_should_parse_custom_ssh_port_from_string() {
        let result = AnsiblePort::from_str("2222");
        assert!(result.is_ok());
        let port = result.unwrap();
        assert_eq!(port.as_u16(), 2222);
    }

    #[test]
    fn it_should_fail_with_invalid_string() {
        let result = AnsiblePort::from_str("invalid_port");
        assert_eq!(
            result,
            Err(AnsiblePortError::ParseError {
                input: "invalid_port".to_string()
            })
        );
    }

    #[test]
    fn it_should_fail_with_negative_string() {
        let result = AnsiblePort::from_str("-22");
        assert_eq!(
            result,
            Err(AnsiblePortError::ParseError {
                input: "-22".to_string()
            })
        );
    }

    #[test]
    fn it_should_fail_with_port_zero_string() {
        let result = AnsiblePort::from_str("0");
        assert_eq!(
            result,
            Err(AnsiblePortError::InvalidPortNumber {
                input: "0".to_string()
            })
        );
    }

    #[test]
    fn it_should_fail_with_empty_string() {
        let result = AnsiblePort::from_str("");
        assert_eq!(
            result,
            Err(AnsiblePortError::ParseError {
                input: String::new()
            })
        );
    }

    #[test]
    fn it_should_implement_display_trait() {
        let port = AnsiblePort::new(8080).unwrap();
        assert_eq!(format!("{port}"), "8080");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let port = AnsiblePort::new(443).unwrap();
        let json = serde_json::to_string(&port).unwrap();
        assert_eq!(json, "443");
    }

    #[test]
    fn it_should_support_clone_and_equality() {
        let port1 = AnsiblePort::new(22).unwrap();
        let port2 = port1;
        assert_eq!(port1, port2);
    }

    #[test]
    fn it_should_display_error_message_correctly() {
        let error = AnsiblePortError::InvalidPortNumber {
            input: "0".to_string(),
        };
        assert_eq!(
            format!("{error}"),
            "Invalid port number: 0 (must be between 1 and 65535)"
        );
    }

    #[test]
    fn it_should_display_parse_error_message_correctly() {
        let error = AnsiblePortError::ParseError {
            input: "abc".to_string(),
        };
        assert_eq!(format!("{error}"), "Port number parse error: abc");
    }
}
