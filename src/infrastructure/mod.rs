//! Infrastructure Layer (DDD)
//!
//! This module contains infrastructure concerns including low-level command execution,
//! external tool adapters, and repository-like implementations. The infrastructure layer
//! provides technical capabilities and delivery mechanisms for the application layer.
//!
//! ## Components
//!
//! - `external_tools` - Integration and delivery mechanisms for third-party console tools
//!   - `adapters` - External tool integration adapters (Ansible, LXD, `OpenTofu`, SSH)
//!   - `ansible` - Ansible delivery mechanism and implementation details
//!   - `tofu` - `OpenTofu` delivery mechanism and implementation details
//!   - `template` - Template rendering delivery mechanisms (wrappers)
//! - `remote_actions` - Repository-like implementations for remote system operations
//! - `persistence` - Persistence infrastructure (repositories, file locking, storage)
//! - `templates` - Configuration template generation for user-facing configuration files
//! - `trace` - Trace file generation for error analysis

pub mod external_tools;
pub mod persistence;
pub mod remote_actions;
pub mod templates;
pub mod trace;
