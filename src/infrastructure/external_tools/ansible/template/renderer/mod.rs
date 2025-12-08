//! # Ansible Template Renderer
//!
//! This module handles `Ansible` template rendering for deployment workflows.
//! It manages the creation of build directories, copying static template files (playbooks and configs),
//! and processing dynamic Tera templates with runtime variables (like inventory.yml.tera).
//!
//! ## Key Features
//!
//! - **Static file copying**: Handles Ansible playbooks and configuration files that don't need templating
//! - **Dynamic template rendering**: Processes Tera templates with runtime variables like IP addresses and SSH keys
//! - **Structured error handling**: Provides specific error types with detailed context and source chaining
//! - **Tracing integration**: Comprehensive logging for debugging and monitoring deployment processes
//! - **Testable design**: Modular structure that allows for comprehensive unit testing
//!
//! ## Usage
//!
//! ```rust
//! # use std::str::FromStr;
//! # use std::sync::Arc;
//! # use tempfile::TempDir;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::renderer::AnsibleProjectGenerator;
//! use torrust_tracker_deployer_lib::domain::template::TemplateManager;
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::ansible::template::wrappers::inventory::{
//!     InventoryContext, AnsibleHost, AnsiblePort, SshPrivateKeyFile
//! };
//!
//! let temp_dir = TempDir::new()?;
//! let template_manager = Arc::new(TemplateManager::new("/path/to/templates"));
//! let renderer = AnsibleProjectGenerator::new(temp_dir.path(), template_manager);
//!
//! let host = AnsibleHost::from_str("192.168.1.100")?;
//! let ssh_key = SshPrivateKeyFile::new("/path/to/ssh/key")?;
//! let ssh_port = AnsiblePort::new(22)?;
//! let inventory_context = InventoryContext::builder()
//!     .with_host(host)
//!     .with_ssh_priv_key_path(ssh_key)
//!     .with_ssh_port(ssh_port)
//!     .with_ansible_user("torrust".to_string())
//!     .build()?;
//!
//! // Note: This would require actual template files to work
//! // renderer.render(&inventory_context).await?;
//! # Ok(())
//! # }
//! ```

pub mod inventory;
mod project_generator;
pub mod variables;

pub use inventory::InventoryRenderer;
pub use project_generator::{AnsibleProjectGenerator, AnsibleProjectGeneratorError};
pub use variables::VariablesRenderer;
