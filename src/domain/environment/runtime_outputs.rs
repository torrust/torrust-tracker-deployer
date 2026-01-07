//! Runtime Outputs Module
//!
//! This module contains the `RuntimeOutputs` struct which holds data generated
//! during deployment operations.
//!
//! ## Purpose
//!
//! Runtime outputs represent data that is produced as deployment operations
//! execute. These fields are mutable and grow as the deployment progresses.
//!
//! ## Semantic Category
//!
//! **Runtime Outputs** are:
//! - Generated during deployment operations
//! - Mutable as operations progress
//! - Examples: IP addresses, container IDs
//!
//! Add new fields here when: Operations produce new data about the deployed infrastructure.
//!
//! ## Future Extensions
//!
//! This struct is expected to grow with fields like:
//! - `container_id: Option<String>` - Container/VM identifier
//! - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
//! - `service_endpoints: Option<Vec<ServiceEndpoint>>` - HTTP/TCP service URLs

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// How the infrastructure instance was provisioned
///
/// This enum tracks the method used to provision the infrastructure, which
/// affects how the environment can be destroyed and other lifecycle operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionMethod {
    /// Instance was provisioned using `OpenTofu` (infrastructure as code)
    ///
    /// This method creates new infrastructure that can be destroyed using
    /// `tofu destroy`. The infrastructure lifecycle is fully managed.
    #[default]
    Provisioned,

    /// Instance was registered from existing infrastructure
    ///
    /// This method connects to existing infrastructure (VMs, containers, physical servers)
    /// that was created externally. The infrastructure cannot be destroyed by this tool;
    /// the `destroy` command will only clean up local state, not the actual instance.
    Registered,
}

impl std::fmt::Display for ProvisionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provisioned => write!(f, "provisioned"),
            Self::Registered => write!(f, "registered"),
        }
    }
}

/// Runtime outputs generated during deployment operations
///
/// This struct contains fields that are generated during deployment operations
/// and represent the runtime state of deployed infrastructure. These fields
/// are mutable as operations progress.
///
/// # Future Fields
///
/// This struct is expected to grow as deployment operations become more complex:
/// - `container_id: Option<String>` - Container/VM identifier
/// - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
/// - `service_endpoints: Option<Vec<ServiceEndpoint>>` - HTTP/TCP service URLs
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::runtime_outputs::{RuntimeOutputs, ProvisionMethod};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let runtime_outputs = RuntimeOutputs {
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
///     provision_method: Some(ProvisionMethod::Provisioned),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutputs {
    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    pub instance_ip: Option<IpAddr>,

    /// How the instance was provisioned
    ///
    /// This field tracks whether the instance was created via `OpenTofu` (`Provisioned`)
    /// or registered from existing infrastructure (`Registered`). This affects
    /// lifecycle operations like `destroy`.
    ///
    /// - `None`: Unknown or legacy state (before this field was added)
    /// - `Some(Provisioned)`: Instance was created via `provision` command
    /// - `Some(Registered)`: Instance was connected via `register` command
    #[serde(default)]
    pub provision_method: Option<ProvisionMethod>,
}
