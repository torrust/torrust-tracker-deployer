//! Infrastructure lifecycle management steps
//!
//! This module contains steps that manage infrastructure lifecycle using `OpenTofu`
//! (Terraform). These steps handle the complete infrastructure provisioning workflow
//! including initialization, planning, application, and information retrieval.
//!
//! ## Available Steps
//!
//! - `initialize` - `OpenTofu` initialization (tofu init)
//! - `plan` - Infrastructure planning and change preview (tofu plan)
//! - `apply` - Infrastructure provisioning and application (tofu apply)
//! - `get_instance_info` - Instance information retrieval from state
//!
//! ## Key Features
//!
//! - Complete `OpenTofu` workflow orchestration
//! - Infrastructure state management and tracking
//! - Instance information extraction and processing
//! - Integration with template rendering and configuration systems
//!
//! These steps provide the core infrastructure management capabilities for
//! provisioning and managing deployment environments.

pub mod apply;
pub mod get_instance_info;
pub mod initialize;
pub mod plan;

pub use apply::ApplyInfrastructureStep;
pub use get_instance_info::GetInstanceInfoStep;
pub use initialize::InitializeInfrastructureStep;
pub use plan::PlanInfrastructureStep;
