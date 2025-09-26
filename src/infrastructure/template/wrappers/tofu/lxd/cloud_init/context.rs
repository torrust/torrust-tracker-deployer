//! Context for Cloud Init template rendering
//!
//! This module provides the `CloudInitContext` and builder pattern for creating
//! template contexts with SSH public key information for cloud-init configuration.

use serde::Serialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::shared::Username;

/// Errors that can occur when creating a `CloudInitContext`
#[derive(Error, Debug, Clone)]
pub enum CloudInitContextError {
    #[error("SSH public key is required but not provided")]
    MissingSshPublicKey,

    #[error("Username is required but not provided")]
    MissingUsername,

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Failed to read SSH public key from file: {0}")]
    SshPublicKeyReadError(String),
}

/// Template context for Cloud Init configuration with SSH public key and username
#[derive(Debug, Clone, Serialize)]
pub struct CloudInitContext {
    /// SSH public key content to be injected into cloud-init configuration
    pub ssh_public_key: String,
    /// Username to be created in the cloud-init configuration
    pub username: Username,
}

/// Builder for `CloudInitContext` with fluent interface
#[derive(Debug, Default)]
pub struct CloudInitContextBuilder {
    ssh_public_key: Option<String>,
    username: Option<Username>,
}

impl CloudInitContextBuilder {
    /// Set the SSH public key content directly
    #[must_use]
    pub fn with_ssh_public_key(mut self, ssh_public_key: String) -> Self {
        self.ssh_public_key = Some(ssh_public_key);
        self
    }

    /// Set the username for the cloud-init configuration
    ///
    /// # Errors
    /// Returns an error if the username is invalid according to Linux naming requirements
    pub fn with_username<S: Into<String>>(
        mut self,
        username: S,
    ) -> Result<Self, CloudInitContextError> {
        let username = Username::new(username)
            .map_err(|e| CloudInitContextError::InvalidUsername(e.to_string()))?;
        self.username = Some(username);
        Ok(self)
    }

    /// Set the SSH public key by reading from a file path
    ///
    /// # Errors
    /// Returns an error if the file cannot be read
    pub fn with_ssh_public_key_from_file<P: AsRef<Path>>(
        mut self,
        ssh_public_key_path: P,
    ) -> Result<Self, CloudInitContextError> {
        let content = fs::read_to_string(ssh_public_key_path.as_ref()).map_err(|e| {
            CloudInitContextError::SshPublicKeyReadError(format!(
                "Failed to read SSH public key from {}: {}",
                ssh_public_key_path.as_ref().display(),
                e
            ))
        })?;

        // Trim any trailing newlines or whitespace from the SSH key
        self.ssh_public_key = Some(content.trim().to_string());
        Ok(self)
    }

    /// Builds the `CloudInitContext`
    ///
    /// # Errors
    /// Returns an error if required fields are missing
    pub fn build(self) -> Result<CloudInitContext, CloudInitContextError> {
        let ssh_public_key = self
            .ssh_public_key
            .ok_or(CloudInitContextError::MissingSshPublicKey)?;

        let username = self
            .username
            .ok_or(CloudInitContextError::MissingUsername)?;

        Ok(CloudInitContext {
            ssh_public_key,
            username,
        })
    }
}

impl CloudInitContext {
    /// Creates a new `CloudInitContext` with SSH public key content and username
    ///
    /// # Errors
    /// Returns an error if the username is invalid according to Linux naming requirements
    pub fn new<S: Into<String>>(
        ssh_public_key: String,
        username: S,
    ) -> Result<Self, CloudInitContextError> {
        let username = Username::new(username)
            .map_err(|e| CloudInitContextError::InvalidUsername(e.to_string()))?;
        Ok(Self {
            ssh_public_key,
            username,
        })
    }

    /// Creates a new builder for `CloudInitContext` with fluent interface
    #[must_use]
    pub fn builder() -> CloudInitContextBuilder {
        CloudInitContextBuilder::default()
    }

    /// Get the SSH public key content
    #[must_use]
    pub fn ssh_public_key(&self) -> &str {
        &self.ssh_public_key
    }

    /// Get the username
    #[must_use]
    pub fn username(&self) -> &str {
        self.username.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_cloud_init_context_with_ssh_key() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let username = "testuser";
        let context = CloudInitContext::new(ssh_key.to_string(), username).unwrap();

        assert_eq!(context.ssh_public_key(), ssh_key);
        assert_eq!(context.username(), username);
    }

    #[test]
    fn it_should_build_context_with_builder_pattern() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let username = "testuser";
        let context = CloudInitContext::builder()
            .with_ssh_public_key(ssh_key.to_string())
            .with_username(username)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(context.ssh_public_key(), ssh_key);
        assert_eq!(context.username(), username);
    }

    #[test]
    fn it_should_read_ssh_key_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let key_file = temp_dir.path().join("test_key.pub");
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com\n";
        let username = "testuser";

        fs::write(&key_file, ssh_key).unwrap();

        let context = CloudInitContext::builder()
            .with_ssh_public_key_from_file(&key_file)
            .unwrap()
            .with_username(username)
            .unwrap()
            .build()
            .unwrap();

        // Should trim the trailing newline
        assert_eq!(context.ssh_public_key(), ssh_key.trim());
        assert_eq!(context.username(), username);
    }

    #[test]
    fn it_should_fail_when_ssh_key_is_missing() {
        let result = CloudInitContext::builder().build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CloudInitContextError::MissingSshPublicKey
        ));
    }

    #[test]
    fn it_should_fail_when_username_is_missing() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let result = CloudInitContext::builder()
            .with_ssh_public_key(ssh_key.to_string())
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CloudInitContextError::MissingUsername
        ));
    }

    #[test]
    fn it_should_fail_when_ssh_key_file_does_not_exist() {
        let result =
            CloudInitContext::builder().with_ssh_public_key_from_file("/nonexistent/path/key.pub");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CloudInitContextError::SshPublicKeyReadError(_)
        ));
    }

    #[test]
    fn it_should_serialize_to_json() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let username = "testuser";
        let context = CloudInitContext::new(ssh_key.to_string(), username).unwrap();

        let json = serde_json::to_value(&context).unwrap();
        assert_eq!(json["ssh_public_key"], ssh_key);
        assert_eq!(json["username"], username);
    }

    #[test]
    fn it_should_fail_with_invalid_username() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let invalid_username = "123invalid"; // starts with digit

        let result = CloudInitContext::new(ssh_key.to_string(), invalid_username);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CloudInitContextError::InvalidUsername(_)
        ));
    }

    #[test]
    fn it_should_fail_with_builder_when_username_is_invalid() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let invalid_username = "@invalid"; // contains @ symbol

        let result = CloudInitContext::builder()
            .with_ssh_public_key(ssh_key.to_string())
            .with_username(invalid_username);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CloudInitContextError::InvalidUsername(_)
        ));
    }
}
