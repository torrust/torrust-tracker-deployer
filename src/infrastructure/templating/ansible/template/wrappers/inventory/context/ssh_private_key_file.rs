//! SSH private key file wrapper type for path handling and serialization

use derive_more::{Display, From};
use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when working with SSH private key files
#[derive(Debug, Error, PartialEq)]
pub enum SshPrivateKeyFileError {
    #[error("Path is empty")]
    EmptyPath,

    #[error("Path contains invalid characters")]
    InvalidPath,
}

/// Wrapper type for SSH private key file path using the newtype pattern
#[derive(Debug, Clone, PartialEq, Eq, Display, From, Serialize)]
#[display(fmt = "{}", "path.display()")]
#[serde(transparent)]
pub struct SshPrivateKeyFile {
    path: PathBuf,
}

impl SshPrivateKeyFile {
    /// Create a new `SshPrivateKeyFile` from a path
    ///
    /// # Errors
    ///
    /// Returns `SshPrivateKeyFileError::EmptyPath` if the path is empty.
    /// Returns `SshPrivateKeyFileError::InvalidPath` if the path contains null bytes.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, SshPrivateKeyFileError> {
        let path_buf = path.as_ref().to_path_buf();

        if path_buf.as_os_str().is_empty() {
            return Err(SshPrivateKeyFileError::EmptyPath);
        }

        // Basic validation - check if path contains null bytes (invalid in most filesystems)
        if path_buf.to_string_lossy().contains('\0') {
            return Err(SshPrivateKeyFileError::InvalidPath);
        }

        Ok(Self { path: path_buf })
    }

    /// Get the inner path as a `PathBuf` reference
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// Get the inner path as a string reference
    #[must_use]
    pub fn as_str(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    /// Get the inner path as a `PathBuf`
    #[must_use]
    pub fn as_path_buf(&self) -> PathBuf {
        self.path.clone()
    }
}

impl TryFrom<&str> for SshPrivateKeyFile {
    type Error = SshPrivateKeyFileError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl TryFrom<String> for SshPrivateKeyFile {
    type Error = SshPrivateKeyFileError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn it_should_create_ssh_private_key_file_with_valid_path() {
        let result = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa");
        assert!(result.is_ok());

        let ssh_key = result.unwrap();
        assert_eq!(ssh_key.as_str(), "/home/user/.ssh/id_rsa");
    }

    #[test]
    fn it_should_fail_with_empty_path() {
        let result = SshPrivateKeyFile::new("");
        assert_eq!(result, Err(SshPrivateKeyFileError::EmptyPath));
    }

    #[test]
    fn it_should_fail_with_invalid_path_containing_null() {
        let result = SshPrivateKeyFile::new("/path/with/\0/null");
        assert_eq!(result, Err(SshPrivateKeyFileError::InvalidPath));
    }

    #[test]
    fn it_should_implement_display_trait() {
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        assert_eq!(format!("{ssh_key}"), "/home/user/.ssh/id_rsa");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let json = serde_json::to_string(&ssh_key).unwrap();
        assert_eq!(json, "\"/home/user/.ssh/id_rsa\"");
    }

    #[test]
    fn it_should_support_try_from_string() {
        let result = SshPrivateKeyFile::try_from("/path/to/key".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "/path/to/key");
    }

    #[test]
    fn it_should_support_try_from_str() {
        let result = SshPrivateKeyFile::try_from("/path/to/key");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "/path/to/key");
    }

    #[test]
    fn it_should_fail_try_from_empty_string() {
        let result = SshPrivateKeyFile::try_from("");
        assert_eq!(result, Err(SshPrivateKeyFileError::EmptyPath));
    }

    #[test]
    fn it_should_support_clone_and_equality() {
        let ssh_key1 = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        let ssh_key2 = ssh_key1.clone();
        assert_eq!(ssh_key1, ssh_key2);
    }

    #[test]
    fn it_should_provide_access_to_path() {
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        assert_eq!(ssh_key.as_path(), Path::new("/home/user/.ssh/id_rsa"));
    }

    #[test]
    fn it_should_provide_access_to_path_buf() {
        let ssh_key = SshPrivateKeyFile::new("/home/user/.ssh/id_rsa").unwrap();
        assert_eq!(
            ssh_key.as_path_buf(),
            PathBuf::from("/home/user/.ssh/id_rsa")
        );
    }
}
