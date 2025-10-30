pub mod ansible_host;
pub mod ansible_port;
pub mod ssh_private_key_file;

use serde::Serialize;
use thiserror::Error;

#[cfg(test)]
use std::str::FromStr;

pub use ansible_host::{AnsibleHost, AnsibleHostError};
pub use ansible_port::{AnsiblePort, AnsiblePortError};
pub use ssh_private_key_file::{SshPrivateKeyFile, SshPrivateKeyFileError};

/// Errors that can occur when creating an `InventoryContext`
#[derive(Debug, Error)]
pub enum InventoryContextError {
    #[error("Invalid ansible host: {0}")]
    InvalidAnsibleHost(#[from] AnsibleHostError),

    #[error("Invalid SSH private key file: {0}")]
    InvalidSshPrivateKeyFile(#[from] SshPrivateKeyFileError),

    #[error("Invalid ansible port: {0}")]
    InvalidAnsiblePort(#[from] AnsiblePortError),

    #[error("Missing ansible host - must be set before building")]
    MissingAnsibleHost,

    /// Missing SSH private key file in context
    #[error("Missing SSH private key file - must be set before building")]
    MissingSshPrivateKeyFile,

    /// Missing SSH port in context  
    #[error("Missing SSH port - must be set before building")]
    MissingSshPort,
}

#[derive(Serialize, Debug, Clone)]
#[allow(clippy::struct_field_names)] // Field names mirror Ansible inventory variables
pub struct InventoryContext {
    ansible_host: AnsibleHost,
    ansible_ssh_private_key_file: SshPrivateKeyFile,
    ansible_port: AnsiblePort,
    /// Alias for ansible_port used in playbook templates
    #[serde(rename = "ssh_port")]
    ssh_port: AnsiblePort,
}

/// Builder for `InventoryContext` with fluent interface
#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)] // Field names mirror Ansible inventory variables
pub struct InventoryContextBuilder {
    ansible_host: Option<AnsibleHost>,
    ansible_ssh_private_key_file: Option<SshPrivateKeyFile>,
    ansible_port: Option<AnsiblePort>,
}

impl InventoryContextBuilder {
    /// Creates a new empty builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Ansible host for the builder.
    #[must_use]
    pub fn with_host(mut self, ansible_host: AnsibleHost) -> Self {
        self.ansible_host = Some(ansible_host);
        self
    }

    /// Sets the SSH port for the builder.
    #[must_use]
    pub fn with_ssh_port(mut self, ansible_port: AnsiblePort) -> Self {
        self.ansible_port = Some(ansible_port);
        self
    }

    /// Sets the SSH private key file path for the builder.
    #[must_use]
    pub fn with_ssh_priv_key_path(mut self, ssh_private_key_file: SshPrivateKeyFile) -> Self {
        self.ansible_ssh_private_key_file = Some(ssh_private_key_file);
        self
    }

    /// Builds the `InventoryContext`
    ///
    /// # Errors
    ///
    /// Returns an error if any required field is missing
    pub fn build(self) -> Result<InventoryContext, InventoryContextError> {
        let ansible_host = self
            .ansible_host
            .ok_or(InventoryContextError::MissingAnsibleHost)?;

        let ansible_ssh_private_key_file = self
            .ansible_ssh_private_key_file
            .ok_or(InventoryContextError::MissingSshPrivateKeyFile)?;

        let ansible_port = self
            .ansible_port
            .ok_or(InventoryContextError::MissingSshPort)?;

        Ok(InventoryContext {
            ansible_host,
            ansible_ssh_private_key_file,
            ansible_port,
            ssh_port: ansible_port, // Same value for playbook templates
        })
    }
}

impl InventoryContext {
    /// Creates a new `InventoryContext` using typed parameters
    ///
    /// # Errors
    ///
    /// This method cannot fail with the current implementation since it takes
    /// already validated types, but returns Result for consistency with builder pattern
    pub fn new(
        ansible_host: AnsibleHost,
        ansible_ssh_private_key_file: SshPrivateKeyFile,
        ansible_port: AnsiblePort,
    ) -> Result<Self, InventoryContextError> {
        Ok(Self {
            ansible_host,
            ansible_ssh_private_key_file,
            ansible_port,
            ssh_port: ansible_port, // Same value for playbook templates
        })
    }

    /// Creates a new builder for `InventoryContext` with fluent interface
    #[must_use]
    pub fn builder() -> InventoryContextBuilder {
        InventoryContextBuilder::new()
    }

    /// Get the ansible host value as a string
    #[must_use]
    pub fn ansible_host(&self) -> String {
        self.ansible_host.as_str()
    }

    /// Get the ansible SSH private key file path as a string
    #[must_use]
    pub fn ansible_ssh_private_key_file(&self) -> String {
        self.ansible_ssh_private_key_file.as_str()
    }

    /// Get the ansible port
    #[must_use]
    pub fn ansible_port(&self) -> u16 {
        self.ansible_port.as_u16()
    }

    /// Get the ansible host wrapper
    #[must_use]
    pub fn ansible_host_wrapper(&self) -> &AnsibleHost {
        &self.ansible_host
    }

    /// Get the ansible SSH private key file wrapper
    #[must_use]
    pub fn ansible_ssh_private_key_file_wrapper(&self) -> &SshPrivateKeyFile {
        &self.ansible_ssh_private_key_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_access_to_wrapper_types() {
        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(AnsiblePort::new(22).unwrap())
            .build()
            .unwrap();

        // Test wrapper access
        let host_wrapper = context.ansible_host_wrapper();
        let key_wrapper = context.ansible_ssh_private_key_file_wrapper();

        assert_eq!(host_wrapper.as_str(), "10.0.0.1");
        assert_eq!(key_wrapper.as_str(), "/path/to/key");
    }

    #[test]
    fn it_should_support_builder_pattern_fluent_interface() {
        // Test the fluent builder interface as requested
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(AnsiblePort::new(22).unwrap())
            .build()
            .unwrap();

        assert_eq!(inventory_context.ansible_host(), "192.168.1.100");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/home/user/.ssh/id_rsa"
        );
        assert_eq!(inventory_context.ansible_port(), 22);
    }

    #[test]
    fn it_should_work_with_builder_typed_parameters() {
        // Test builder with typed parameters instead of strings
        let host = AnsibleHost::from_str("10.0.0.1").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();

        let inventory_context = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(AnsiblePort::new(22).unwrap())
            .build()
            .unwrap();

        assert_eq!(inventory_context.ansible_host(), "10.0.0.1");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/path/to/key"
        );
        assert_eq!(inventory_context.ansible_port(), 22);
    }

    #[test]
    fn it_should_fail_when_builder_missing_host() {
        // Test that builder fails when host is missing
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let result = InventoryContext::builder()
            .with_ssh_priv_key_path(ssh_key)
            .with_ssh_port(AnsiblePort::new(22).unwrap())
            .build();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing ansible host"));
    }

    #[test]
    fn it_should_fail_when_builder_missing_ssh_key() {
        // Test that builder fails when SSH key is missing
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let result = InventoryContext::builder()
            .with_host(host)
            .with_ssh_port(AnsiblePort::new(22).unwrap())
            .build();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing SSH private key file"));
    }

    #[test]
    fn it_should_fail_when_builder_missing_ssh_port() {
        // Test that builder fails when SSH port is missing
        let host = AnsibleHost::from_str("192.168.1.100").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/path/to/key").unwrap();
        let result = InventoryContext::builder()
            .with_host(host)
            .with_ssh_priv_key_path(ssh_key)
            .build();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing SSH port"));
    }

    #[test]
    fn it_should_create_new_inventory_context_with_typed_parameters() {
        // Test the new direct constructor with typed parameters
        let host = AnsibleHost::from_str("192.168.1.50").unwrap();
        let ssh_key = SshPrivateKeyFile::new("/etc/ssh/test_key").unwrap();
        let ssh_port = AnsiblePort::new(22).unwrap();

        let inventory_context = InventoryContext::new(host, ssh_key, ssh_port).unwrap();

        assert_eq!(inventory_context.ansible_host(), "192.168.1.50");
        assert_eq!(
            inventory_context.ansible_ssh_private_key_file(),
            "/etc/ssh/test_key"
        );
        assert_eq!(inventory_context.ansible_port(), 22);
    }
}
