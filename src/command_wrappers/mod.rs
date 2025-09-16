//! Command wrappers for external deployment tools
//!
//! This module provides high-level client interfaces for external tools used in the
//! deployment process. Each wrapper encapsulates the complexity of command-line tool
//! interaction and provides Rust-native APIs with proper error handling.
//!
//! ## Available Tool Wrappers
//!
//! - `ansible` - Ansible playbook execution and configuration management
//! - `lxd` - LXD container/VM management and operations
//! - `opentofu` - OpenTofu/Terraform infrastructure provisioning
//! - `ssh` - SSH connectivity and remote command execution
//!
//! All wrappers follow consistent patterns for error handling, logging, and configuration.

pub mod ansible;
pub mod lxd;
pub mod opentofu;
pub mod ssh;

// Re-export public types for external use
pub use ansible::AnsibleClient;
pub use lxd::{InstanceInfo as LxdInstanceInfo, InstanceName, LxdClient};
pub use opentofu::{
    InstanceInfo as OpenTofuInstanceInfo, OpenTofuClient, OpenTofuError, ParseError,
};
pub use ssh::{SshClient, SshError};
