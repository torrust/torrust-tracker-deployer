//! Torrust Tracker Deploy
//!
//! Main library for torrust-tracker-deploy with layer-based architecture following DDD principles.
//!
//! ## Architecture Layers
//!
//! - `application` - Application Layer: High-level commands and use case orchestration
//! - `domain` - Domain Layer: Business logic and entities (future expansion)
//! - `infrastructure` - Infrastructure Layer: Technical capabilities and external integrations
//!   - `executor` - Low-level command execution utilities
//!   - `adapters` - External tool adapters (Ansible, LXD, `OpenTofu`, SSH)
//!
//! ## Other Modules
//! - `template` - Template rendering system with modular organization
//! - `steps` - Mid-level deployment steps (Level 2 of three-level architecture)
//! - `remote_actions` - Low-level remote operations (Level 3 of three-level architecture)
//! - `config` - Configuration management for deployment environments
//! - `container` - Service container for dependency injection
//! - `e2e` - End-to-end testing utilities
//! - `logging` - Logging configuration and utilities
//! - `ansible` - Ansible-specific template utilities
//! - `tofu` - OpenTofu-specific template utilities

pub mod ansible;
pub mod application;
pub mod config;
pub mod container;
pub mod domain;
pub mod e2e;
pub mod infrastructure;
pub mod logging;
pub mod remote_actions;
pub mod steps;
pub mod template;
pub mod tofu;
