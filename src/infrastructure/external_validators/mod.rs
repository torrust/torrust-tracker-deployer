//! External validators module
//!
//! This module contains validators that perform end-to-end validation from
//! OUTSIDE the VM, testing services as an external user would access them.
//!
//! ## Execution Context
//!
//! Unlike `remote_actions` which execute commands INSIDE the VM via SSH,
//! external validators:
//! - Run from the test runner or deployment machine
//! - Test service accessibility via HTTP/HTTPS from outside
//! - Validate end-to-end functionality including network and firewall
//!
//! ## Distinction from Remote Actions
//!
//! **Remote Actions** (`infrastructure/remote_actions/`):
//! - Execute commands via SSH inside the VM
//! - Examples: cloud-init validation, Docker installation checks
//! - Scope: Internal VM state and configuration
//!
//! **External Validators** (this module):
//! - Make HTTP requests from outside the VM
//! - Examples: Service health checks, API accessibility tests
//! - Scope: End-to-end service validation including network/firewall
//!
//! ## Available Validators
//!
//! - `running_services` - Validates Docker Compose services via external HTTP/HTTPS health checks

pub mod running_services;

pub use running_services::RunningServicesValidator;
