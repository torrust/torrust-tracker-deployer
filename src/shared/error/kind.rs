//! Generic error kind classification
//!
//! Provides high-level categorization of errors across all commands for
//! debugging and potential recovery strategies. Error kinds appear in trace
//! files and failure contexts but are not directly user-facing.

use serde::{Deserialize, Serialize};

/// Generic error categories for command failures
///
/// These categories provide high-level classification for debugging
/// and potential recovery strategies. They are used in trace files
/// and failure context but are not directly user-facing.
///
/// Error kinds were introduced to provide an easy way to understand
/// what type of error occurred without parsing detailed trace log files.
/// They serve as a high-level summary that can be:
///
/// - Displayed to users without technical details
/// - Used for filtering/grouping errors
/// - Foundation for future retry/recovery strategies based on error category
///
/// # Examples
///
/// ```
/// use torrust_tracker_deployer::shared::ErrorKind;
///
/// let kind = ErrorKind::TemplateRendering;
/// assert_eq!(format!("{kind:?}"), "TemplateRendering");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Template rendering or generation failed
    ///
    /// Examples: Tera template errors, invalid template variables,
    /// missing template files
    TemplateRendering,

    /// Infrastructure operations failed
    ///
    /// Examples: `OpenTofu` operations (init, plan, apply), LXD operations,
    /// container/VM management failures
    InfrastructureOperation,

    /// Network connectivity or communication failed
    ///
    /// Examples: SSH connection timeouts, unreachable hosts,
    /// network configuration issues
    NetworkConnectivity,

    /// External tool or command execution failed
    ///
    /// Examples: Ansible playbook failures, shell command errors,
    /// tool installation failures
    CommandExecution,

    /// Timeout or deadline exceeded
    ///
    /// Examples: Operation timeouts, cloud-init wait exceeded,
    /// long-running process killed
    Timeout,

    /// File system or I/O operation failed
    ///
    /// Examples: File read/write errors, permission denied,
    /// disk space issues, path not found
    FileSystem,

    /// Configuration validation or parsing failed
    ///
    /// Examples: Invalid YAML/TOML, missing required fields,
    /// configuration value out of range
    Configuration,

    /// State persistence operation failed
    ///
    /// Examples: Failed to save environment state, repository errors,
    /// serialization/deserialization failures, storage access issues
    StatePersistence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_serialize_error_kind_to_json() {
        let kind = ErrorKind::TemplateRendering;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"TemplateRendering\"");
    }

    #[test]
    fn it_should_deserialize_error_kind_from_json() {
        let json = "\"NetworkConnectivity\"";
        let kind: ErrorKind = serde_json::from_str(json).unwrap();
        assert_eq!(kind, ErrorKind::NetworkConnectivity);
    }

    #[test]
    fn it_should_be_copy_and_clone() {
        let kind1 = ErrorKind::CommandExecution;
        let kind2 = kind1; // Copy
        let kind3 = kind1; // Copy (no need for .clone() on Copy types)

        assert_eq!(kind1, kind2);
        assert_eq!(kind1, kind3);
    }

    #[test]
    fn it_should_support_equality_comparison() {
        assert_eq!(ErrorKind::Timeout, ErrorKind::Timeout);
        assert_ne!(ErrorKind::Timeout, ErrorKind::FileSystem);
    }

    #[test]
    fn it_should_have_descriptive_debug_output() {
        let kind = ErrorKind::InfrastructureOperation;
        let debug_output = format!("{kind:?}");
        assert_eq!(debug_output, "InfrastructureOperation");
    }
}
