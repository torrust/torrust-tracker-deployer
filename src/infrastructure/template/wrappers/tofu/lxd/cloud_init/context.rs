//! Context for Cloud Init template rendering
//!
//! This module provides the `CloudInitContext` and builder pattern for creating
//! template contexts with SSH public key information for cloud-init configuration.

use serde::Serialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when creating a `CloudInitContext`
#[derive(Error, Debug, Clone)]
pub enum CloudInitContextError {
    #[error("SSH public key is required but not provided")]
    MissingSshPublicKey,
    #[error("Failed to read SSH public key from file: {0}")]
    SshPublicKeyReadError(String),
}

/// Template context for Cloud Init configuration with SSH public key
#[derive(Debug, Clone, Serialize)]
pub struct CloudInitContext {
    /// SSH public key content to be injected into cloud-init configuration
    pub ssh_public_key: String,
}

/// Builder for `CloudInitContext` with fluent interface
#[derive(Debug, Default)]
pub struct CloudInitContextBuilder {
    ssh_public_key: Option<String>,
}

impl CloudInitContextBuilder {
    /// Set the SSH public key content directly
    #[must_use]
    pub fn with_ssh_public_key(mut self, ssh_public_key: String) -> Self {
        self.ssh_public_key = Some(ssh_public_key);
        self
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

        Ok(CloudInitContext { ssh_public_key })
    }
}

impl CloudInitContext {
    /// Creates a new `CloudInitContext` with SSH public key content
    #[must_use]
    pub fn new(ssh_public_key: String) -> Self {
        Self { ssh_public_key }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_cloud_init_context_with_ssh_key() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let context = CloudInitContext::new(ssh_key.to_string());

        assert_eq!(context.ssh_public_key(), ssh_key);
    }

    #[test]
    fn it_should_build_context_with_builder_pattern() {
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com";
        let context = CloudInitContext::builder()
            .with_ssh_public_key(ssh_key.to_string())
            .build()
            .unwrap();

        assert_eq!(context.ssh_public_key(), ssh_key);
    }

    #[test]
    fn it_should_read_ssh_key_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let key_file = temp_dir.path().join("test_key.pub");
        let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... test@example.com\n";

        fs::write(&key_file, ssh_key).unwrap();

        let context = CloudInitContext::builder()
            .with_ssh_public_key_from_file(&key_file)
            .unwrap()
            .build()
            .unwrap();

        // Should trim the trailing newline
        assert_eq!(context.ssh_public_key(), ssh_key.trim());
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
        let context = CloudInitContext::new(ssh_key.to_string());

        let json = serde_json::to_value(&context).unwrap();
        assert_eq!(json["ssh_public_key"], ssh_key);
    }
}
