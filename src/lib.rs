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
//!
//! ## Other Modules
//! - `adapters` - External tool adapters (thin CLI wrappers)
//! - `config` - Configuration management for deployment environments
//! - `container` - Service container for dependency injection
//! - `logging` - Logging configuration and utilities
//! - `shared` - Shared modules used across different layers
//! - `testing` - Testing utilities (unit, integration, and end-to-end)

pub mod adapters;
pub mod application;
pub mod config;
pub mod container;
pub mod domain;
pub mod infrastructure;
pub mod logging;
pub mod shared;
pub mod testing;
