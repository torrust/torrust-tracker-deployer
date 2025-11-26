//! Black-box E2E testing tasks.
//!
//! This module provides reusable tasks for black-box E2E testing,
//! where tests execute CLI commands as external processes.
//!
//! ## Module Structure
//!
//! - `configure_services` - Configure services on provisioned infrastructure
//! - `create_environment` - Create environment from configuration file
//! - `destroy_infrastructure` - Destroy infrastructure for environment
//! - `generate_config` - Generate environment configuration files
//! - `preflight_cleanup` - Remove artifacts from previous test runs
//! - `provision_infrastructure` - Provision infrastructure for environment
//! - `validate_deployment` - Validate deployment by running test command
//! - `verify_dependencies` - Verify required system dependencies are installed

pub mod configure_services;
pub mod create_environment;
pub mod destroy_infrastructure;
pub mod generate_config;
pub mod preflight_cleanup;
pub mod provision_infrastructure;
pub mod validate_deployment;
pub mod verify_dependencies;

// Re-export commonly used items
pub use configure_services::configure_services;
pub use create_environment::create_environment;
pub use destroy_infrastructure::destroy_infrastructure;
pub use generate_config::generate_environment_config;
pub use preflight_cleanup::run_preflight_cleanup;
pub use provision_infrastructure::provision_infrastructure;
pub use validate_deployment::validate_deployment;
pub use verify_dependencies::verify_required_dependencies;
