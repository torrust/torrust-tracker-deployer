//! SSH connection configuration and management
//!
//! This module provides the `SshConnection` struct which encapsulates all the information
//! needed to establish an SSH connection to a remote host, including credentials and
//! target host socket address.
//!
//! ## Key Components
//!
//! - Connection configuration combining credentials and target host socket address
//! - Socket address management (IP address and port) for remote instances
//! - Integration with SSH credentials for authentication
//!
//! The connection configuration is used by SSH clients to establish secure
//! connections for remote command execution.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use super::SshCredentials;

/// Default SSH port number.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// SSH connection configuration for a specific remote instance.
///
/// Contains both the SSH credentials and the target host socket address,
/// representing everything needed to establish an SSH connection.
#[derive(Clone)]
pub struct SshConnection {
    /// SSH authentication credentials.
    pub credentials: SshCredentials,

    /// Socket address (IP address and port) of the target host for SSH connections.
    ///
    /// This contains both the IP address and port number of the remote instance
    /// that the SSH client will connect to.
    pub socket_addr: SocketAddr,
}

impl SshConnection {
    /// Creates a new SSH connection configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::shared::{Username, ssh::{SshCredentials, SshConnection}};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     Username::new("ubuntu").unwrap(),
    /// );
    /// let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 22);
    /// let connection = SshConnection::new(
    ///     credentials,
    ///     socket_addr,
    /// );
    /// ```
    #[must_use]
    pub fn new(credentials: SshCredentials, ssh_socket_addr: SocketAddr) -> Self {
        Self {
            credentials,
            socket_addr: ssh_socket_addr,
        }
    }

    /// Creates a new SSH connection configuration with the default port (22).
    ///
    /// This is a convenience method for when you want to use the standard SSH port.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::shared::{Username, ssh::{SshCredentials, SshConnection}};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     Username::new("ubuntu").unwrap(),
    /// );
    /// let connection = SshConnection::with_default_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    /// );
    /// ```
    #[must_use]
    pub fn with_default_port(credentials: SshCredentials, host_ip: IpAddr) -> Self {
        let socket_addr = SocketAddr::new(host_ip, DEFAULT_SSH_PORT);
        Self::new(credentials, socket_addr)
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
        self.credentials.ssh_username.as_str()
    }

    /// Access the SSH port.
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.socket_addr.port()
    }

    /// Access the host IP address.
    #[must_use]
    pub fn host_ip(&self) -> IpAddr {
        self.socket_addr.ip()
    }

    /// Access the socket address.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }
}
