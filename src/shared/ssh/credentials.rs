//! SSH credentials management for remote authentication
//!
//! This module provides the `SshCredentials` struct which manages SSH authentication
//! information including private/public key paths and username configuration.
//!
//! ## Key Features
//!
//! - SSH key pair management (private and public keys)
//! - Username configuration for remote connections
//! - Integration with SSH connection establishment
//! - Support for creating SSH connections with target IP addresses
//!
//! The credentials are typically configured at startup and used throughout
//! the deployment process for secure remote access to provisioned instances.

use std::net::IpAddr;
use std::path::PathBuf;

use super::SshConnection;

/// SSH credentials for remote instance authentication.
///
/// Contains the static SSH authentication information that is known
/// at program startup, before any instances are provisioned.
#[derive(Clone)]
pub struct SshCredentials {
    /// Path to the SSH private key file for remote connections.
    ///
    /// This key will be used by the SSH client to authenticate with remote
    /// instances created during deployment. The corresponding public key
    /// should be authorized on the target instances.
    pub ssh_priv_key_path: PathBuf,

    /// Path to the SSH public key file for remote connections.
    ///
    /// This public key will be used for authorization on target instances
    /// during the deployment process, typically injected into cloud-init
    /// configurations or `authorized_keys` files.
    pub ssh_pub_key_path: PathBuf,

    /// Username for SSH connections to remote instances.
    ///
    /// This username will be used when establishing SSH connections to
    /// deployed instances. Common values include "ubuntu", "root", or "torrust".
    pub ssh_username: String,
}

impl SshCredentials {
    /// Creates new SSH credentials with the provided parameters.
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::shared::ssh::SshCredentials;
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        ssh_priv_key_path: PathBuf,
        ssh_pub_key_path: PathBuf,
        ssh_username: String,
    ) -> Self {
        Self {
            ssh_priv_key_path,
            ssh_pub_key_path,
            ssh_username,
        }
    }

    /// Promote these credentials to a full SSH connection configuration.
    ///
    /// This method creates an `SshConnection` by combining these credentials
    /// with a specific host IP address.
    #[must_use]
    pub fn with_host(self, host_ip: IpAddr) -> SshConnection {
        SshConnection::with_default_port(self, host_ip)
    }

    /// Create an SSH connection with a custom port
    #[must_use]
    pub fn with_host_and_port(self, host_ip: IpAddr, port: u16) -> SshConnection {
        SshConnection::with_ip_and_port(self, host_ip, port)
    }
}
