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
//! - Examples: IP addresses, container IDs, timestamps
//!
//! Add new fields here when: Operations produce new data about the deployed infrastructure.
//!
//! ## Future Extensions
//!
//! This struct is expected to grow with fields like:
//! - `container_id: Option<String>` - Container/VM identifier
//! - `deployment_timestamp: Option<DateTime<Utc>>` - When the environment was deployed
//! - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
//! - `service_endpoints: Option<Vec<ServiceEndpoint>>` - HTTP/TCP service URLs

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

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
/// - `deployment_timestamp: Option<DateTime<Utc>>` - When the environment was deployed
/// - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
/// - `service_endpoints: Option<Vec<ServiceEndpoint>>` - HTTP/TCP service URLs
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::domain::environment::runtime_outputs::RuntimeOutputs;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let runtime_outputs = RuntimeOutputs {
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutputs {
    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    pub instance_ip: Option<IpAddr>,
}
