//! SSH private key file wrapper type for path handling and serialization

use serde::Serialize;
use std::fmt;

/// Wrapper type for SSH private key file path using the newtype pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshPrivateKeyFile(String);

impl SshPrivateKeyFile {
    /// Create a new `SshPrivateKeyFile` from a string path
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self(path.into())
    }

    /// Get the inner path as a string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the inner path as a string
    #[must_use]
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for SshPrivateKeyFile {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for SshPrivateKeyFile {
    fn from(path: String) -> Self {
        Self(path)
    }
}

impl fmt::Display for SshPrivateKeyFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SshPrivateKeyFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}
