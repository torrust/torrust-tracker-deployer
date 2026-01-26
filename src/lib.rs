//! Torrust Tracker Deployer
//!
//! Main library for torrust-tracker-deploy with complete DDD layer-based architecture.
//!
//! ## Architecture Layers
//!
//! - `domain` - Domain Layer: Pure business logic and domain models
//!   - `template` - Core template domain models and business logic
//! - `application` - Application Layer: Use case orchestration and workflow coordination  
//!   - `commands` - High-level deployment commands using Command pattern
//!   - `steps` - Workflow orchestration and business process steps
//! - `infrastructure` - Infrastructure Layer: Technical capabilities and delivery mechanisms
//!   - `executor` - Low-level command execution utilities
//!   - `adapters` - External tool integration adapters (Ansible, LXD, `OpenTofu`, SSH)
//!   - `remote_actions` - Repository-like implementations for remote system operations
//!   - `ansible` - Ansible delivery mechanism and implementation details
//!   - `tofu` - `OpenTofu` delivery mechanism and implementation details
//!   - `template` - Template rendering delivery mechanisms (wrappers)
//! - `presentation` - Presentation Layer: User-facing output and presentation concerns
//!
//! ## Other Modules
//! - `adapters` - External tool adapters (thin CLI wrappers)
//! - `bootstrap` - Application initialization and bootstrap concerns
//! - `config` - Configuration management for deployment environments
//! - `shared` - Shared modules used across different layers
//! - `testing` - Testing utilities (unit, integration, and end-to-end)

// False positive: clippy reports large_stack_arrays for vec![] macro with ServiceTopology
// This is a known upstream issue: https://github.com/rust-lang/rust-clippy/issues/12586
// Tracking issue: https://github.com/torrust/torrust-tracker-deployer/issues/304
// See: docs/issues/304-clippy-large-stack-arrays-false-positive.md
#![allow(clippy::large_stack_arrays)]

pub mod adapters;
pub mod application;
pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;
pub mod testing;
