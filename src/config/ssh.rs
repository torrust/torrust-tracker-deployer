use std::net::IpAddr;
use std::path::PathBuf;

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
    /// # use torrust_tracker_deploy::config::ssh::SshCredentials;
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
        SshConnection {
            credentials: self,
            host_ip,
        }
    }
}

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
    /// # use torrust_tracker_deploy::config::ssh::{SshCredentials, SshConnection};
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
