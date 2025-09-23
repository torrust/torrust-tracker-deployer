//! SSH connection configuration and management
//!
//! This module provides the `SshConnection` struct which encapsulates all the information
//! needed to establish an SSH connection to a remote host, including credentials and
//! target host information.
//!
//! ## Key Components
//!
//! - Connection configuration combining credentials and target host
//! - IP address management for remote instances
//! - Integration with SSH credentials for authentication
//!
//! The connection configuration is used by SSH clients to establish secure
//! connections for remote command execution.

use std::net::IpAddr;
use std::path::PathBuf;

use super::SshCredentials;

/// SSH connection configuration for a specific remote instance.
///
/// Contains both the SSH credentials and the target host IP address,
/// representing everything needed to establish an SSH connection.
#[derive(Clone)]
pub struct SshConnection {
    /// SSH authentication credentials.
    pub credentials: SshCredentials,

    /// IP address of the target host for SSH connections.
    ///
    /// This is the IP address of the remote instance that the SSH client
    /// will connect to.
    pub host_ip: IpAddr,

    /// Port number for SSH connections.
    ///
    /// Defaults to 22 (standard SSH port) but can be customized for
    /// containerized environments or non-standard SSH configurations.
    pub port: u16,
}

impl SshConnection {
    /// Creates a new SSH connection configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::infrastructure::adapters::ssh::{SshCredentials, SshConnection};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let connection = SshConnection::new(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    ///     22,
    /// );
    /// ```
    #[must_use]
    pub fn new(credentials: SshCredentials, host_ip: IpAddr, port: u16) -> Self {
        Self::new_with_port(credentials, host_ip, port)
    }

    /// Creates a new SSH connection configuration with the default port (22).
    ///
    /// This is a convenience method for when you want to use the standard SSH port.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::infrastructure::adapters::ssh::{SshCredentials, SshConnection};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let connection = SshConnection::with_default_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    /// );
    /// ```
    #[must_use]
    pub fn with_default_port(credentials: SshCredentials, host_ip: IpAddr) -> Self {
        Self::new(credentials, host_ip, 22)
    }

    /// Creates a new SSH connection configuration with a custom port.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::infrastructure::adapters::ssh::{SshCredentials, SshConnection};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let connection = SshConnection::new_with_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    ///     2222,
    /// );
    /// ```
    #[must_use]
    pub fn new_with_port(credentials: SshCredentials, host_ip: IpAddr, port: u16) -> Self {
        Self {
            credentials,
            host_ip,
            port,
        }
    }

    /// Access the SSH private key path.
    #[must_use]
    pub fn ssh_priv_key_path(&self) -> &PathBuf {
        &self.credentials.ssh_priv_key_path
    }

    /// Access the SSH public key path.
    #[must_use]
    pub fn ssh_pub_key_path(&self) -> &PathBuf {
        &self.credentials.ssh_pub_key_path
    }

    /// Access the SSH username.
    #[must_use]
    pub fn ssh_username(&self) -> &str {
        &self.credentials.ssh_username
    }

    /// Access the SSH port.
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.port
    }
}
