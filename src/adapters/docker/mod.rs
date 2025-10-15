//! Docker CLI client for executing Docker commands
//!
//! This module provides a `DockerClient` that wraps Docker CLI operations using our
//! `CommandExecutor` abstraction. This enables testability and consistency with other
//! external tool clients (Ansible, `OpenTofu`, LXD).
//!
//! # Architecture
//!
//! - `client.rs` - Main Docker client with one method per Docker subcommand
//! - `error.rs` - Docker-specific error types with actionable messages
//!
//! # Example
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::adapters::docker::DockerClient;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let docker = DockerClient::new();
//!
//! // Build a Docker image
//! docker.build_image("docker/ssh-server", "my-app", "latest")?;
//!
//! // Check if image exists
//! let exists = docker.image_exists("my-app", "latest")?;
//! assert!(exists);
//!
//! // List all containers
//! let containers = docker.list_containers(true)?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;

pub use client::DockerClient;
pub use error::DockerError;
