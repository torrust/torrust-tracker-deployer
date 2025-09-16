/*!
 * Steps Module - Level 2 of Three-Level Architecture
 *
 * This module organizes deployment steps by operation type for maximum reusability
 * and clear separation of concerns. Steps are reusable building blocks that can be
 * composed into commands (Level 1) and may use remote actions (Level 3).
 *
 * Organization by Operation Type:
 * - rendering/     - Template and configuration generation
 * - infrastructure/ - Infrastructure lifecycle (`OpenTofu` operations)
 * - system/        - System-level configuration and management
 * - software/      - Software installation and management  
 * - application/   - Application deployment and lifecycle
 * - connectivity/  - Network and connection operations
 * - validation/    - Testing and validation operations
 *
 * This organization supports the full planned command ecosystem while enabling
 * step reuse across multiple commands.
 */

pub mod application;
pub mod connectivity;
pub mod infrastructure;
pub mod rendering;
pub mod software;
pub mod system;
pub mod validation;

// Re-export all steps for easy access
pub use connectivity::WaitForSSHConnectivityStep;
pub use infrastructure::{
    ApplyInfrastructureStep, GetInstanceInfoStep, InitializeInfrastructureStep,
    PlanInfrastructureStep,
};
pub use rendering::{
    RenderAnsibleTemplatesError, RenderAnsibleTemplatesStep, RenderOpenTofuTemplatesStep,
};
pub use software::{InstallDockerComposeStep, InstallDockerStep};
pub use system::WaitForCloudInitStep;
pub use validation::{
    ValidateCloudInitCompletionStep, ValidateDockerComposeInstallationStep,
    ValidateDockerInstallationStep,
};
