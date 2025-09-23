//! End-to-End testing infrastructure
//!
//! This module provides comprehensive testing infrastructure for validating
//! the complete deployment workflow from infrastructure provisioning through
//! application deployment and validation.
//!
//! ## Module Structure
//!
//! - `environment` - Test environment configuration and management
//! - `provisioned_container` - Docker container state machine for E2E testing
//! - `tasks` - High-level testing tasks and workflows
//!
//! ## Testing Workflow
//!
//! The E2E testing system orchestrates complete deployment scenarios including
//! provisioning, configuration, validation, and cleanup phases to ensure
//! the entire deployment system works correctly.

pub mod environment;
pub mod provisioned_container;
pub mod tasks;
