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

pub mod external_tools;
pub mod remote_actions;
