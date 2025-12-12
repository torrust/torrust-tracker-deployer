//! Infrastructure Layer (DDD)
//!
//! This module contains infrastructure concerns including low-level command execution,
//! external tool adapters, and repository-like implementations. The infrastructure layer
//! provides technical capabilities and delivery mechanisms for the application layer.
//!
//! ## Components
//!
//! - `templating` - Template rendering and delivery mechanisms for deployment tools
//!   - `ansible` - Ansible template generation and project structure
//!   - `docker_compose` - Docker Compose template generation
//!   - `tofu` - `OpenTofu` template generation and project structure
//!   - `tracker` - Torrust Tracker configuration templates
//! - `remote_actions` - SSH-based operations executed inside VMs
//! - `external_validators` - E2E validation from outside VMs (HTTP health checks)
//! - `persistence` - Persistence infrastructure (repositories, file locking, storage)
//! - `trace` - Trace file generation for error analysis
//! - `schema` - JSON Schema generation from Rust types

pub mod external_validators;
pub mod persistence;
pub mod remote_actions;
pub mod schema;
pub mod templating;
pub mod trace;
