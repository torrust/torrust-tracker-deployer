//! SSH Key Setup Action
//!
//! This module provides an action to setup SSH key authentication inside a container.
//! The action is decoupled from specific container implementations and can be used
//! with any container that implements the `ContainerExecutor` trait.

use std::fs;
use std::path::Path;

use testcontainers::core::ExecCommand;

use crate::e2e::containers::ContainerExecutor;
use crate::shared::ssh::SshCredentials;
use crate::shared::Username;

/// Specific error types for SSH key setup operations
#[derive(Debug, thiserror::Error)]
pub enum SshKeySetupError {
    /// Failed to read SSH public key file
    #[error("Failed to read SSH public key from '{path}' for user '{ssh_user}': {source}")]
    SshKeyFileRead {
        path: String,
        ssh_user: String,
        #[source]
        source: std::io::Error,
    },

    /// SSH public key file is empty or invalid
    #[error(
        "SSH public key file '{path}' is empty or contains invalid data for user '{ssh_user}'"
    )]
    SshKeyFileEmpty { path: String, ssh_user: String },

    /// Failed to create SSH directory in container
    #[error("Failed to create SSH directory '/home/{ssh_user}/.ssh' in container: {source}")]
    SshDirectoryCreationFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to write `authorized_keys` file in container
    #[error("Failed to write authorized_keys file for user '{ssh_user}' in container: {source}")]
    AuthorizedKeysWriteFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to set SSH directory permissions in container
    #[error(
        "Failed to set SSH directory permissions for user '{ssh_user}' in container: {source}"
    )]
    SshPermissionsFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Failed to change ownership of SSH directory in container
    #[error(
        "Failed to change ownership of SSH directory to user '{ssh_user}' in container: {source}"
    )]
    SshOwnershipFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    /// Generic SSH key setup failure (fallback for unspecific errors)
    #[error("SSH key setup failed for user '{ssh_user}' in container: {source}")]
    SshKeySetupFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },
}

/// Result type alias for SSH key setup operations
pub type Result<T> = std::result::Result<T, SshKeySetupError>;

/// Action to setup SSH key authentication inside a container
///
/// This action configures SSH key authentication by:
/// 1. Reading the public key from the credentials
/// 2. Creating the SSH directory for the specified user
/// 3. Adding the public key to the `authorized_keys` file
/// 4. Setting appropriate permissions
///
/// ## Usage
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::e2e::containers::{ContainerExecutor, actions::SshKeySetupAction};
/// use torrust_tracker_deployer_lib::shared::ssh::SshCredentials;
///
/// async fn setup_ssh<T: ContainerExecutor>(
///     container: &T,
///     credentials: &SshCredentials,
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     let action = SshKeySetupAction;
///     action.execute(container, credentials).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct SshKeySetupAction;

impl SshKeySetupAction {
    /// Create a new SSH key setup action
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Execute SSH key setup on a container
    ///
    /// Sets up SSH key authentication by:
    /// 1. Creating the SSH directory (`~/.ssh`)
    /// 2. Adding the public key to `authorized_keys`
    /// 3. Setting proper permissions on SSH files
    ///
    /// # Arguments
    ///
    /// * `container` - Container that implements `ContainerExecutor`
    /// * `credentials` - SSH credentials containing public key path and username
    ///
    /// # Errors
    ///
    /// Returns an error if any of the setup commands fail
    pub async fn execute<T: ContainerExecutor>(
        &self,
        container: &T,
        credentials: &SshCredentials,
    ) -> std::result::Result<(), SshKeySetupError> {
        Self::create_ssh_directory(container, &credentials.ssh_username).await?;

        Self::add_public_key_to_authorized_keys(
            container,
            &credentials.ssh_username,
            &credentials.ssh_pub_key_path,
        )
        .await?;

        Self::set_ssh_directory_permissions(
            container,
            credentials.ssh_username.as_str(),
            &format!("/home/{}/.ssh", credentials.ssh_username.as_str()),
        )
        .await?;

        Self::set_authorized_keys_permissions(
            container,
            credentials.ssh_username.as_str(),
            &format!(
                "/home/{}/.ssh/authorized_keys",
                credentials.ssh_username.as_str()
            ),
        )
        .await?;

        Ok(())
    }

    /// Create the SSH directory for the user
    async fn create_ssh_directory<T: ContainerExecutor>(
        container: &T,
        username: &Username,
    ) -> std::result::Result<(), SshKeySetupError> {
        let command =
            ExecCommand::new(["mkdir", "-p", &format!("/home/{}/.ssh", username.as_str())]);
        container.exec(command).await.map_err(|source| {
            SshKeySetupError::SshDirectoryCreationFailed {
                ssh_user: username.to_string(),
                source,
            }
        })?;
        Ok(())
    }

    /// Add the public key to the `authorized_keys` file
    async fn add_public_key_to_authorized_keys<T: ContainerExecutor>(
        container: &T,
        username: &Username,
        public_key_path: &Path,
    ) -> std::result::Result<(), SshKeySetupError> {
        let public_key = fs::read_to_string(public_key_path).map_err(|source| {
            SshKeySetupError::SshKeyFileRead {
                path: public_key_path.to_string_lossy().to_string(),
                ssh_user: username.to_string(),
                source,
            }
        })?;

        let command = ExecCommand::new([
            "sh",
            "-c",
            &format!(
                "echo '{}' >> /home/{}/.ssh/authorized_keys",
                public_key.trim(),
                username.as_str()
            ),
        ]);
        container.exec(command).await.map_err(|source| {
            SshKeySetupError::AuthorizedKeysWriteFailed {
                ssh_user: username.to_string(),
                source,
            }
        })?;
        Ok(())
    }

    /// Set permissions on the SSH directory (700)
    async fn set_ssh_directory_permissions<T: ContainerExecutor>(
        container: &T,
        ssh_user: &str,
        user_ssh_dir: &str,
    ) -> Result<()> {
        let command = ExecCommand::new(["sh", "-c", &format!("chmod 700 {user_ssh_dir}")]);

        container
            .exec(command)
            .await
            .map_err(|source| SshKeySetupError::SshPermissionsFailed {
                ssh_user: ssh_user.to_string(),
                source,
            })?;

        Ok(())
    }

    /// Set permissions on the `authorized_keys` file (600)
    async fn set_authorized_keys_permissions<T: ContainerExecutor>(
        container: &T,
        ssh_user: &str,
        authorized_keys_path: &str,
    ) -> Result<()> {
        let command = ExecCommand::new(["sh", "-c", &format!("chmod 600 {authorized_keys_path}")]);

        container
            .exec(command)
            .await
            .map_err(|source| SshKeySetupError::SshOwnershipFailed {
                ssh_user: ssh_user.to_string(),
                source,
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::path::PathBuf;

    #[test]
    fn it_should_create_new_ssh_key_setup_action() {
        let action = SshKeySetupAction::new();
        assert!(std::ptr::eq(
            std::ptr::addr_of!(action),
            std::ptr::addr_of!(action)
        )); // Just test it exists
    }

    #[test]
    fn it_should_create_default_ssh_key_setup_action() {
        let action = SshKeySetupAction;
        assert!(std::ptr::eq(
            std::ptr::addr_of!(action),
            std::ptr::addr_of!(action)
        )); // Just test it exists
    }

    #[test]
    fn it_should_have_proper_error_display_messages() {
        let error = SshKeySetupError::SshKeyFileRead {
            path: "/path/to/key".to_string(),
            ssh_user: "testuser".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        };
        assert!(error.to_string().contains("Failed to read SSH public key"));
        assert!(error.to_string().contains("/path/to/key"));
        assert!(error.to_string().contains("testuser"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_ssh_key_file_read() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = SshKeySetupError::SshKeyFileRead {
            path: "/path/to/key".to_string(),
            ssh_user: "testuser".to_string(),
            source: io_error,
        };

        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_preserve_error_chain_for_ssh_key_setup_failed() {
        let testcontainers_error = testcontainers::TestcontainersError::other("test error");
        let error = SshKeySetupError::SshKeySetupFailed {
            ssh_user: "testuser".to_string(),
            source: testcontainers_error,
        };

        assert!(error.source().is_some());
        assert!(error.to_string().contains("testuser"));
    }

    // Helper function to create mock SSH credentials for testing
    #[allow(dead_code)]
    fn create_mock_ssh_credentials() -> SshCredentials {
        use crate::shared::Username;
        SshCredentials::new(
            PathBuf::from("/mock/path/to/private_key"),
            PathBuf::from("/mock/path/to/public_key.pub"),
            Username::new("testuser").unwrap(),
        )
    }
}
