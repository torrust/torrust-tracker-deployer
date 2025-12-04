//! Application deployment and lifecycle steps
//!
//! This module contains steps that manage application deployment and lifecycle
//! operations. These steps handle application-specific operations like deployment,
//! service management, configuration, and application health monitoring.
//!
//! ## Available Steps
//!
//! - `deploy_compose_files` - Deploys Docker Compose files to remote host via Ansible
//! - `run` - Starts the Docker Compose application stack
//!
//! ## Future Steps
//!
//! This module is prepared for future application deployment steps such as:
//! - Application health checks and validation
//! - Service stop and restart operations
//! - Status monitoring and reporting
//!
//! ## Integration
//!
//! Application steps integrate with the existing infrastructure and
//! software installation steps to provide complete deployment workflows
//! from infrastructure provisioning to application operation.

pub mod deploy_compose_files;
pub mod run;

pub use deploy_compose_files::{DeployComposeFilesStep, DeployComposeFilesStepError};
pub use run::{RunStep, RunStepError};
