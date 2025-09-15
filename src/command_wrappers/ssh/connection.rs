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
}

impl SshConnection {
    /// Creates a new SSH connection configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::command_wrappers::ssh::{SshCredentials, SshConnection};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let connection = SshConnection::new(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    /// );
    /// ```
    #[must_use]
    pub fn new(credentials: SshCredentials, host_ip: IpAddr) -> Self {
        Self {
            credentials,
            host_ip,
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
}
