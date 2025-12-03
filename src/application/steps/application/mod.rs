//! Application deployment and lifecycle steps
//!
//! This module contains steps that manage application deployment and lifecycle
//! operations. These steps handle application-specific operations like deployment,
//! service management, configuration, and application health monitoring.
//!
//! ## Available Steps
//!
//! - `release` - Deploys configuration and Docker Compose files to remote host
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

pub mod release;
pub mod run;

pub use release::{ReleaseStep, ReleaseStepError};
pub use run::{RunStep, RunStepError};
