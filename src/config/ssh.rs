use std::net::IpAddr;
use std::path::PathBuf;

/// SSH connection configuration for remote instances.
///
/// Contains all SSH-related settings needed to establish connections
/// to deployed instances during the deployment process.
#[derive(Clone)]
pub struct SshConfig {
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

    /// IP address of the target host for SSH connections.
    ///
    /// This is the IP address of the remote instance that the SSH client
    /// will connect to.
    pub host_ip: IpAddr,
}

impl SshConfig {
    /// Creates a new SSH configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::config::SshConfig;
    /// let ssh_config = SshConfig::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        ssh_priv_key_path: PathBuf,
        ssh_pub_key_path: PathBuf,
        ssh_username: String,
        host_ip: IpAddr,
    ) -> Self {
        Self {
            ssh_priv_key_path,
            ssh_pub_key_path,
            ssh_username,
            host_ip,
        }
    }
}
