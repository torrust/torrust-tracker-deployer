//! Infrastructure adapters for external deployment tools
//!
//! This module provides adapter interfaces for external tools used in the deployment
//! process. Each adapter encapsulates the complexity of command-line tool interaction
//! and provides clean Rust-native APIs with proper error handling.
//!
//! These adapters are part of the infrastructure layer in our DDD architecture,
//! providing technical capabilities for higher-level application services.
//!
//! ## Available Tool Adapters
//!
//! - `ansible` - Ansible playbook execution and configuration management
//! - `lxd` - LXD container/VM management and operations  
//! - `opentofu` - OpenTofu/Terraform infrastructure provisioning
//! - `ssh` - SSH connectivity and remote command execution
//!
//! All adapters follow consistent patterns for error handling, logging, and configuration.

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
