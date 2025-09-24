//! Container Actions
//!
//! This module provides actions that can be performed on running containers.
//! Actions are decoupled from container implementations, making them reusable
//! and testable independently.
//!
//! ## Architecture
//!
//! Container actions follow a trait-based architecture:
//! - Actions that need to execute commands inside containers use `ContainerExecutor`
//! - Actions that interact with containers externally (like network connectivity checks) don't need the trait
//! - Each action is responsible for a single, well-defined operation
//!
//! ## Available Actions
//!
//! - **SSH Key Setup** - Configure SSH key authentication inside a container
//! - **SSH Wait** - Wait for SSH connectivity to become available
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::e2e::containers::{ContainerExecutor, actions::{SshKeySetupAction, SshWaitAction}};
//! use torrust_tracker_deploy::shared::ssh::SshCredentials;
//! use std::time::Duration;
//!
//! fn setup_container_ssh<T: ContainerExecutor>(
//!     container: &T,
//!     ssh_credentials: &SshCredentials,
//!     host: &str,
//!     port: u16,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     // Setup SSH keys inside the container
//!     let ssh_setup = SshKeySetupAction::new();
//!     ssh_setup.execute(container, ssh_credentials)?;
//!     
//!     // Wait for SSH to be accessible
//!     let ssh_wait = SshWaitAction::new(Duration::from_secs(30), 10);
//!     ssh_wait.execute(host, port)?;
//!     
//!     Ok(())
//! }
//! ```

pub mod ssh_key_setup;
pub mod ssh_wait;

// Re-export action types for easy access
pub use ssh_key_setup::SshKeySetupAction;
pub use ssh_wait::SshWaitAction;
