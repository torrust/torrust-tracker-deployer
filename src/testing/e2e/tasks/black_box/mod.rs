//! Black-box E2E testing tasks.
//!
//! This module provides reusable tasks for black-box E2E testing,
//! where tests execute CLI commands as external processes.
//!
//! ## Module Structure
//!
//! - `generate_config` - Generate environment configuration files
//! - `preflight_cleanup` - Remove artifacts from previous test runs
//! - `verify_dependencies` - Verify required system dependencies are installed

pub mod generate_config;
pub mod preflight_cleanup;
pub mod verify_dependencies;

// Re-export commonly used items
pub use generate_config::generate_environment_config;
pub use preflight_cleanup::run_preflight_cleanup;
pub use verify_dependencies::verify_required_dependencies;
