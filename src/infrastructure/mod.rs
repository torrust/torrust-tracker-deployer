//! Infrastructure Layer (DDD)
//!
//! This module contains infrastructure concerns including low-level command execution,
//! external tool adapters, and repository-like implementations. The infrastructure layer
//! provides technical capabilities and delivery mechanisms for the application layer.
//!
//! ## Components
//!
//! - `adapters` - External tool integration adapters (Ansible, LXD, `OpenTofu`, SSH)
//! - `remote_actions` - Repository-like implementations for remote system operations
//! - `ansible` - Ansible delivery mechanism and implementation details
//! - `tofu` - `OpenTofu` delivery mechanism and implementation details
//! - `template` - Template rendering delivery mechanisms (wrappers)

pub mod adapters;
pub mod ansible;
pub mod remote_actions;
pub mod template;
pub mod tofu;
