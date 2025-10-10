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
//! use torrust_tracker_deployer_lib::e2e::containers::{ContainerExecutor, actions::{SshKeySetupAction, SshWaitAction}};
//! use torrust_tracker_deployer_lib::shared::ssh::SshCredentials;
//! use std::time::Duration;
//! use std::net::SocketAddr;
//!
//! async fn setup_container_ssh<T: ContainerExecutor>(
//!     container: &T,
//!     ssh_credentials: &SshCredentials,
//!     socket_addr: SocketAddr,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     // Setup SSH keys inside the container
//!     let ssh_setup = SshKeySetupAction::new();
//!     ssh_setup.execute(container, ssh_credentials).await?;
//!     
//!     // Wait for SSH to be accessible
//!     let ssh_wait = SshWaitAction::new(Duration::from_secs(30), 10);
//!     ssh_wait.execute(socket_addr)?;
//!     
//!     Ok(())
//! }
//! ```

pub mod ssh_key_setup;
pub mod ssh_wait;

// Re-export action types for easy access
pub use ssh_key_setup::SshKeySetupAction;
pub use ssh_wait::SshWaitAction;
