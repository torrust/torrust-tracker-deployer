/*!
 * Infrastructure Steps
 *
 * This module contains steps that manage infrastructure lifecycle using `OpenTofu`.
 * These steps handle VM/container creation, modification, and destruction.
 *
 * Current steps:
 * - `OpenTofu` initialization (tofu init)
 * - Infrastructure planning (tofu plan)
 * - Infrastructure provisioning (tofu apply)
 * - Infrastructure information retrieval
 *
 * Future steps may include:
 * - Infrastructure destruction (tofu destroy)
 * - Infrastructure state management
 * - Multi-provider support
 */

pub mod apply;
pub mod get_instance_info;
pub mod initialize;
pub mod plan;

pub use apply::ApplyInfrastructureStep;
pub use get_instance_info::GetInstanceInfoStep;
pub use initialize::InitializeInfrastructureStep;
pub use plan::PlanInfrastructureStep;
