use tracing::info;

use crate::command_wrappers::ssh::{SshClient, SshError};
use crate::config::SshConfig;

/// Step that waits for SSH connectivity to be established on a remote host
pub struct WaitForSSHConnectivityStep {
    ssh_config: SshConfig,
}

impl WaitForSSHConnectivityStep {
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        Self { ssh_config }
    }

    /// Execute the SSH connectivity wait step
    ///
    /// This will create an SSH client and wait until connectivity is established
    /// with the remote host before returning.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * SSH connectivity cannot be established within the timeout period
    /// * The SSH client fails to initialize
    /// * The SSH command execution fails
    pub async fn execute(&self) -> Result<(), SshError> {
        info!(
            step = "wait_ssh_connectivity",
            instance_ip = %self.ssh_config.host_ip,
            username = %self.ssh_config.ssh_username,
            "Waiting for SSH connectivity to be established"
        );

        // Create SSH client
        let ssh_client = SshClient::new(self.ssh_config.clone());

        // Wait for connectivity
        ssh_client.wait_for_connectivity().await?;

        info!(
            step = "wait_ssh_connectivity",
            instance_ip = %self.ssh_config.host_ip,
            status = "success",
            "SSH connectivity successfully established"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::config::SshConfig;

    use super::*;

    #[test]
    fn it_should_create_wait_ssh_connectivity_step() {
        let instance_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let ssh_config = SshConfig::new(
            "/tmp/test_key".into(),
            "/tmp/test_key.pub".into(),
            "testuser".to_string(),
            instance_ip,
        );

        let step = WaitForSSHConnectivityStep::new(ssh_config);

        assert_eq!(
            step.ssh_config.ssh_priv_key_path.to_string_lossy(),
            "/tmp/test_key"
        );
        assert_eq!(step.ssh_config.ssh_username, "testuser");
        assert_eq!(step.ssh_config.host_ip, instance_ip);
    }

    #[test]
    fn it_should_create_step_with_ipv6_address() {
        let instance_ip = "::1".parse::<IpAddr>().unwrap();
        let ssh_config = SshConfig::new(
            "/home/user/.ssh/id_rsa".into(),
            "/home/user/.ssh/id_rsa.pub".into(),
            "torrust".to_string(),
            instance_ip,
        );

        let step = WaitForSSHConnectivityStep::new(ssh_config);

        assert_eq!(
            step.ssh_config.ssh_priv_key_path.to_string_lossy(),
            "/home/user/.ssh/id_rsa"
        );
        assert_eq!(step.ssh_config.ssh_username, "torrust");
        assert_eq!(step.ssh_config.host_ip, instance_ip);
    }

    #[test]
    fn it_should_store_step_parameters_correctly() {
        let instance_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ssh_config = SshConfig::new(
            "/path/to/ssh/key".into(),
            "/path/to/ssh/key.pub".into(),
            "admin".to_string(),
            instance_ip,
        );

        let step = WaitForSSHConnectivityStep::new(ssh_config);

        assert_eq!(
            step.ssh_config.ssh_priv_key_path.to_string_lossy(),
            "/path/to/ssh/key"
        );
        assert_eq!(step.ssh_config.ssh_username, "admin");
        assert_eq!(step.ssh_config.host_ip, instance_ip);
    }
}
