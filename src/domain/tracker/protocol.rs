//! Network protocol types for tracker services
//!
//! This module defines the protocol types used by tracker services
//! to distinguish between UDP and TCP based services.

use std::fmt;
use std::str::FromStr;

/// Network protocol used by tracker services
///
/// Distinguishes between UDP and TCP protocols for socket binding validation.
/// UDP and TCP maintain separate port spaces in the operating system, allowing
/// the same port number to be used by both protocols simultaneously.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::Protocol;
///
/// let udp = Protocol::Udp;
/// let tcp = Protocol::Tcp;
///
/// assert_eq!(udp.to_string(), "UDP");
/// assert_eq!(tcp.to_string(), "TCP");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    /// User Datagram Protocol - connectionless protocol
    Udp,
    /// Transmission Control Protocol - connection-oriented protocol
    Tcp,
}

/// Error type for protocol parsing failures
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolParseError {
    /// Unknown protocol string provided
    UnknownProtocol(String),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Udp => write!(f, "UDP"),
            Self::Tcp => write!(f, "TCP"),
        }
    }
}

impl FromStr for Protocol {
    type Err = ProtocolParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "UDP" => Ok(Self::Udp),
            "TCP" => Ok(Self::Tcp),
            _ => Err(ProtocolParseError::UnknownProtocol(s.to_string())),
        }
    }
}

impl fmt::Display for ProtocolParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProtocol(proto) => {
                write!(f, "Unknown protocol: '{proto}'. Expected 'UDP' or 'TCP'")
            }
        }
    }
}

impl std::error::Error for ProtocolParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    mod protocol_enum {
        use super::*;

        #[test]
        fn it_should_display_udp_as_uppercase_string() {
            assert_eq!(Protocol::Udp.to_string(), "UDP");
        }

        #[test]
        fn it_should_display_tcp_as_uppercase_string() {
            assert_eq!(Protocol::Tcp.to_string(), "TCP");
        }

        #[test]
        fn it_should_parse_udp_from_uppercase_string() {
            assert_eq!("UDP".parse::<Protocol>().unwrap(), Protocol::Udp);
        }

        #[test]
        fn it_should_parse_udp_from_lowercase_string() {
            assert_eq!("udp".parse::<Protocol>().unwrap(), Protocol::Udp);
        }

        #[test]
        fn it_should_parse_udp_from_mixed_case_string() {
            assert_eq!("Udp".parse::<Protocol>().unwrap(), Protocol::Udp);
        }

        #[test]
        fn it_should_parse_tcp_from_uppercase_string() {
            assert_eq!("TCP".parse::<Protocol>().unwrap(), Protocol::Tcp);
        }

        #[test]
        fn it_should_parse_tcp_from_lowercase_string() {
            assert_eq!("tcp".parse::<Protocol>().unwrap(), Protocol::Tcp);
        }

        #[test]
        fn it_should_parse_tcp_from_mixed_case_string() {
            assert_eq!("Tcp".parse::<Protocol>().unwrap(), Protocol::Tcp);
        }

        #[test]
        fn it_should_return_error_when_parsing_unknown_protocol() {
            let result = "HTTP".parse::<Protocol>();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                ProtocolParseError::UnknownProtocol("HTTP".to_string())
            );
        }

        #[test]
        fn it_should_return_error_when_parsing_empty_string() {
            let result = "".parse::<Protocol>();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                ProtocolParseError::UnknownProtocol(String::new())
            );
        }

        #[test]
        fn it_should_be_equal_when_same_protocol() {
            assert_eq!(Protocol::Udp, Protocol::Udp);
            assert_eq!(Protocol::Tcp, Protocol::Tcp);
        }

        #[test]
        fn it_should_not_be_equal_when_different_protocols() {
            assert_ne!(Protocol::Udp, Protocol::Tcp);
        }

        #[test]
        fn it_should_be_hashable() {
            use std::collections::HashSet;

            let mut set = HashSet::new();
            set.insert(Protocol::Udp);
            set.insert(Protocol::Tcp);
            set.insert(Protocol::Udp); // Duplicate

            assert_eq!(set.len(), 2); // Only two unique protocols
        }
    }

    mod protocol_parse_error {
        use super::*;

        #[test]
        fn it_should_display_helpful_error_message_for_unknown_protocol() {
            let error = ProtocolParseError::UnknownProtocol("HTTP".to_string());
            assert_eq!(
                error.to_string(),
                "Unknown protocol: 'HTTP'. Expected 'UDP' or 'TCP'"
            );
        }
    }
}
