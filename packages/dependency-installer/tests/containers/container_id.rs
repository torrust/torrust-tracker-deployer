//! Docker container identifier type

use std::ffi::OsStr;
use std::fmt;

/// Docker container identifier
///
/// A validated Docker container ID, which is a hexadecimal string.
/// Docker generates these IDs and guarantees they contain only hex characters (0-9, a-f).
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// // Container IDs come from Docker/testcontainers and are always valid hex strings
/// let id = ContainerId::new("a1b2c3d4e5f6".to_string()).expect("valid hex string");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerId(String);

impl ContainerId {
    /// Create a new container ID with validation
    ///
    /// # Arguments
    ///
    /// * `id` - The container ID string (must be hexadecimal)
    ///
    /// # Returns
    ///
    /// `Ok(ContainerId)` if valid, `Err` with error message if invalid
    ///
    /// # Validation Rules
    ///
    /// - Must not be empty
    /// - Must contain only hexadecimal characters (0-9, a-f, A-F)
    /// - Typically 12 characters (short form) or 64 characters (full SHA256)
    pub fn new(id: String) -> Result<Self, String> {
        if id.is_empty() {
            return Err("Container ID cannot be empty".to_string());
        }

        if !id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Container ID must contain only hexadecimal characters, got: '{id}'"
            ));
        }

        Ok(Self(id))
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<OsStr> for ContainerId {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}
