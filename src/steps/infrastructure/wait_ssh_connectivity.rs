use std::net::IpAddr;
use std::path::PathBuf;

use tracing::info;

use crate::command_wrappers::ssh::{SshClient, SshError};

/// Step that waits for SSH connectivity to be established on a remote host
pub struct WaitForSSHConnectivityStep {
    ssh_key_path: PathBuf,
    username: String,
    instance_ip: IpAddr,
}

impl WaitForSSHConnectivityStep {
    #[must_use]
    pub fn new(ssh_key_path: PathBuf, username: String, instance_ip: IpAddr) -> Self {
        Self {
            ssh_key_path,
            username,
            instance_ip,
        }
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
            instance_ip = %self.instance_ip,
            username = %self.username,
            "Waiting for SSH connectivity to be established"
        );

        // Create SSH client
        let ssh_client = SshClient::new(&self.ssh_key_path, &self.username, self.instance_ip);

        // Wait for connectivity
        ssh_client.wait_for_connectivity().await?;

        info!(
            step = "wait_ssh_connectivity",
            instance_ip = %self.instance_ip,
            status = "success",
            "SSH connectivity successfully established"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_should_create_wait_ssh_connectivity_step() {
        let ssh_key_path = PathBuf::from("/tmp/test_key");
        let username = "testuser".to_string();
        let instance_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        let step =
            WaitForSSHConnectivityStep::new(ssh_key_path.clone(), username.clone(), instance_ip);

        assert_eq!(step.ssh_key_path, ssh_key_path);
        assert_eq!(step.username, username);
        assert_eq!(step.instance_ip, instance_ip);
    }

    #[test]
    fn it_should_create_step_with_ipv6_address() {
        let ssh_key_path = PathBuf::from("/home/user/.ssh/id_rsa");
        let username = "torrust".to_string();
        let instance_ip = "::1".parse::<IpAddr>().unwrap();

        let step =
            WaitForSSHConnectivityStep::new(ssh_key_path.clone(), username.clone(), instance_ip);

        assert_eq!(step.ssh_key_path, ssh_key_path);
        assert_eq!(step.username, username);
        assert_eq!(step.instance_ip, instance_ip);
    }

    #[test]
    fn it_should_store_step_parameters_correctly() {
        let ssh_key_path = PathBuf::from("/path/to/ssh/key");
        let username = "admin".to_string();
        let instance_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        let step =
            WaitForSSHConnectivityStep::new(ssh_key_path.clone(), username.clone(), instance_ip);

        assert_eq!(step.ssh_key_path, ssh_key_path);
        assert_eq!(step.username, username);
        assert_eq!(step.instance_ip, instance_ip);
    }
}
