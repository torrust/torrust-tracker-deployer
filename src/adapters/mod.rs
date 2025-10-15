//! External Tool Adapters
//!
//! This module contains thin wrappers (adapters) for external CLI tools used throughout
//! the project. These adapters provide a consistent Rust interface for interacting with
//! external command-line tools.
//!
//! ## Design Philosophy
//!
//! All adapters in this module follow these principles:
//!
//! - **Thin wrappers**: Minimal business logic, primarily command builders
//! - **Consistent pattern**: All use `CommandExecutor` from `crate::shared::command`
//! - **Generic and reusable**: Designed to be usable across different projects
//! - **Infrastructure concerns**: Handle external system interactions
//!
//! ## Architecture
//!
//! Each adapter:
//!
//! 1. Wraps an external CLI tool (SSH, Docker, Ansible, LXD, `OpenTofu`)
//! 2. Uses `CommandExecutor` as a collaborator for actual command execution
//! 3. Provides domain-specific methods that return typed results
//! 4. Handles tool-specific error cases with structured error types
//!
//! ## Available Adapters
//!
//! - **`ansible`** - Ansible configuration management tool wrapper
//! - **`docker`** - Docker container platform wrapper
//! - **`lxd`** - LXD container and VM management wrapper
//! - **`ssh`** - SSH secure shell client wrapper
//! - **`tofu`** - `OpenTofu` infrastructure provisioning wrapper
//!
//! ## Example Usage
//!
//! ```ignore
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::adapters::ssh::{SshClient, SshConfig};
//! use torrust_tracker_deployer_lib::adapters::docker::DockerClient;
//!
//! // Create SSH client adapter
//! let ssh_config = SshConfig::default();
//! let ssh_client = SshClient::new(Arc::new(ssh_config));
//!
//! // Create Docker client adapter
//! let docker_client = DockerClient::new();
//! ```
//!
//! ## Relationship with Infrastructure Layer
//!
//! While these adapters live at the top level (`src/adapters/`), application-specific
//! logic for using these tools remains in `src/infrastructure/external_tools/`:
//!
//! - **`src/adapters/`**: Generic CLI wrappers (this module)
//! - **`src/infrastructure/external_tools/`**: Application-specific tool configuration
//!   (e.g., Ansible inventory rendering, `OpenTofu` template generation)
//!
//! This separation ensures adapters remain reusable while application-specific logic
//! stays in the infrastructure layer.

pub mod ansible;
pub mod docker;
pub mod lxd;
pub mod ssh;
pub mod tofu;

// Re-exports for migrated adapters
pub use docker::DockerClient;
pub use ssh::{SshClient, SshConfig, SshConnectionConfig, SshCredentials, SshPublicKey};

// Re-exports pending migration
// pub use ansible::AnsibleClient;
// pub use lxd::LxdClient;
// pub use tofu::OpenTofuClient;
