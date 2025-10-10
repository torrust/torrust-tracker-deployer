//! Process ID type for cross-platform process management
//!
//! This module provides a type-safe wrapper around process IDs (PIDs) with
//! cross-platform support for Unix and Windows systems.
//!
//! # Design
//!
//! The `ProcessId` type is a newtype wrapper around `u32` that provides:
//! - Type safety: PIDs can't be confused with other numeric types
//! - Cross-platform compatibility: Works on both Unix and Windows
//! - Process liveness checking: Can verify if a process is still running
//!
//! # Usage
//!
//! ```rust
//! use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
//!
//! // Get the current process ID
//! let current_pid = ProcessId::current();
//! println!("Current process: {}", current_pid);
//!
//! // Check if a process is alive
//! assert!(current_pid.is_alive());
//!
//! // Parse from string (useful when reading from files)
//! let pid: ProcessId = "12345".parse().expect("Invalid PID");
//! ```

use std::process;

use super::platform;

/// Process ID newtype for type safety
///
/// Wraps a u32 process ID to provide type safety and prevent accidental misuse.
/// This ensures PIDs are only used in appropriate contexts and makes the code
/// more self-documenting.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
///
/// // Get current process ID
/// let pid = ProcessId::current();
///
/// // Create from raw value
/// let pid = ProcessId::from_raw(12345);
///
/// // Get raw value
/// let raw: u32 = pid.as_u32();
///
/// // Parse from string
/// let pid: ProcessId = "12345".parse().unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(u32);

impl ProcessId {
    /// Get the current process ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
    ///
    /// let current = ProcessId::current();
    /// assert!(current.is_alive());
    /// ```
    #[must_use]
    pub fn current() -> Self {
        Self(process::id())
    }

    /// Create a `ProcessId` from a raw u32
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
    ///
    /// let pid = ProcessId::from_raw(12345);
    /// assert_eq!(pid.as_u32(), 12345);
    /// ```
    #[must_use]
    pub fn from_raw(pid: u32) -> Self {
        Self(pid)
    }

    /// Get the raw u32 value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
    ///
    /// let pid = ProcessId::from_raw(12345);
    /// assert_eq!(pid.as_u32(), 12345);
    /// ```
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if this process is currently alive
    ///
    /// Uses platform-specific methods to check if the process exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer::infrastructure::persistence::filesystem::process_id::ProcessId;
    ///
    /// let current = ProcessId::current();
    /// assert!(current.is_alive());
    ///
    /// let fake_pid = ProcessId::from_raw(999_999);
    /// assert!(!fake_pid.is_alive());
    /// ```
    #[must_use]
    pub fn is_alive(&self) -> bool {
        platform::is_process_alive(*self)
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ProcessId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_get_current_process_id() {
        let pid = ProcessId::current();
        assert!(pid.as_u32() > 0, "Current PID should be positive");
    }

    #[test]
    fn it_should_create_from_raw_value() {
        let pid = ProcessId::from_raw(12345);
        assert_eq!(pid.as_u32(), 12345);
    }

    #[test]
    fn it_should_parse_from_string() {
        let pid: ProcessId = "12345".parse().expect("Should parse valid PID");
        assert_eq!(pid.as_u32(), 12345);
    }

    #[test]
    fn it_should_fail_to_parse_invalid_string() {
        let result: Result<ProcessId, _> = "not-a-number".parse();
        assert!(result.is_err(), "Should fail to parse invalid PID");
    }

    #[test]
    fn it_should_display_as_string() {
        let pid = ProcessId::from_raw(12345);
        assert_eq!(pid.to_string(), "12345");
    }

    #[test]
    fn it_should_detect_current_process_as_alive() {
        let current = ProcessId::current();
        assert!(current.is_alive(), "Current process should always be alive");
    }

    #[test]
    fn it_should_detect_fake_process_as_dead() {
        let fake_pid = ProcessId::from_raw(999_999);
        assert!(!fake_pid.is_alive(), "Fake PID 999999 should not be alive");
    }

    #[test]
    fn it_should_implement_equality() {
        let pid1 = ProcessId::from_raw(12345);
        let pid2 = ProcessId::from_raw(12345);
        let pid3 = ProcessId::from_raw(67890);

        assert_eq!(pid1, pid2);
        assert_ne!(pid1, pid3);
    }

    #[test]
    fn it_should_be_copyable() {
        let pid1 = ProcessId::from_raw(12345);
        let pid2 = pid1; // Copy
        assert_eq!(pid1, pid2);
    }
}
