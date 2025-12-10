//! Black-box E2E testing tasks.
//!
//! This module provides reusable tasks for black-box E2E testing,
//! where tests execute CLI commands as external processes.
//!
//! ## Main Type
//!
//! - [`E2eTestRunner`] - High-level abstraction for running E2E test tasks
//!   against a specific environment. Provides methods for all test operations.
//!
//! ## Standalone Functions
//!
//! These functions are used for setup tasks that don't require an active test runner:
//!
//! - [`generate_environment_config`] - Generate environment configuration files
//! - [`run_preflight_cleanup`] - Remove artifacts from previous test runs
//! - [`verify_required_dependencies`] - Verify required system dependencies are installed
//!
//! ## Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::{
//!     E2eTestRunner, generate_environment_config, run_preflight_cleanup,
//!     verify_required_dependencies,
//! };
//! use torrust_dependency_installer::Dependency;
//!
//! // Setup tasks (before creating the test runner)
//! verify_required_dependencies(&[Dependency::OpenTofu, Dependency::Ansible])?;
//! run_preflight_cleanup("e2e-full")?;
//! let config_path = generate_environment_config("e2e-full")?;
//!
//! // Create test runner and execute tasks
//! let test_runner = E2eTestRunner::new("e2e-full")
//!     .with_cleanup_on_failure(true);
//!
//! test_runner.create_environment(&config_path)?;
//! test_runner.provision_infrastructure()?;
//! test_runner.configure_services()?;
//! test_runner.validate_deployment()?;
//! test_runner.destroy_infrastructure()?;
//! ```

mod generate_config;
mod preflight_cleanup;
mod test_runner;
mod verify_dependencies;

// Re-export the main test runner type
pub use test_runner::E2eTestRunner;

// Re-export standalone setup functions
pub use generate_config::{
    create_test_environment_config, generate_environment_config,
    generate_environment_config_with_port,
};
pub use preflight_cleanup::run_container_preflight_cleanup;
pub use preflight_cleanup::run_preflight_cleanup;
pub use verify_dependencies::verify_required_dependencies;
