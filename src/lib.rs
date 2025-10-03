//! Torrust Tracker Deploy
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
//!
//! ## Other Modules
//! - `config` - Configuration management for deployment environments
//! - `container` - Service container for dependency injection
//! - `e2e` - End-to-end testing utilities
//! - `logging` - Logging configuration and utilities
//! - `shared` - Shared modules used across different layers

pub mod application;
pub mod config;
pub mod container;
pub mod domain;
pub mod e2e;
pub mod infrastructure;
pub mod logging;
pub mod shared;

#[cfg(test)]
pub mod testing;
