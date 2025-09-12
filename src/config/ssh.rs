use std::path::PathBuf;

/// SSH connection configuration for remote instances.
///
/// Contains all SSH-related settings needed to establish connections
/// to deployed instances during the deployment process.
pub struct SshConfig {
    /// Path to the SSH private key file for remote connections.
    ///
    /// This key will be used by the SSH client to authenticate with remote
    /// instances created during deployment. The corresponding public key
    /// should be authorized on the target instances.
    pub ssh_key_path: PathBuf,

    /// Username for SSH connections to remote instances.
    ///
    /// This username will be used when establishing SSH connections to
    /// deployed instances. Common values include "ubuntu", "root", or "torrust".
    pub ssh_username: String,
}

impl SshConfig {
    /// Creates a new SSH configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::config::SshConfig;
    /// let ssh_config = SshConfig::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     "ubuntu".to_string(),
    /// );
    /// ```
    #[must_use]
    pub fn new(ssh_key_path: PathBuf, ssh_username: String) -> Self {
        Self {
            ssh_key_path,
            ssh_username,
        }
    }
}
