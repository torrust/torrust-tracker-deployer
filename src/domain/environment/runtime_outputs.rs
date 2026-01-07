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
//! - Examples: IP addresses, container IDs, service URLs
//!
//! Add new fields here when: Operations produce new data about the deployed infrastructure.
//!
//! ## Future Extensions
//!
//! This struct is expected to grow with fields like:
//! - `container_id: Option<String>` - Container/VM identifier
//! - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use url::Url;

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

/// Service endpoints for deployed tracker services
///
/// This struct stores the URLs for all deployed tracker services. These URLs
/// are computed from the tracker configuration and instance IP after the
/// `run` command successfully starts the services.
///
/// # Purpose
///
/// Having service endpoints as first-class data allows:
/// - Displaying service URLs without recomputation
/// - Sharing URLs with external tools/integrations
/// - Validating service availability against stored endpoints
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::runtime_outputs::ServiceEndpoints;
/// use url::Url;
///
/// let endpoints = ServiceEndpoints {
///     udp_trackers: vec![
///         Url::parse("udp://10.0.0.1:6969/announce").unwrap(),
///     ],
///     http_trackers: vec![
///         Url::parse("http://10.0.0.1:7070/announce").unwrap(),
///     ],
///     api_endpoint: Some(Url::parse("http://10.0.0.1:1212/api").unwrap()),
///     health_check_url: Some(Url::parse("http://10.0.0.1:1313/health_check").unwrap()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// UDP tracker announce URLs (e.g., `udp://10.0.0.1:6969/announce`)
    #[serde(default)]
    pub udp_trackers: Vec<Url>,

    /// HTTP tracker announce URLs (e.g., `http://10.0.0.1:7070/announce`)
    #[serde(default)]
    pub http_trackers: Vec<Url>,

    /// HTTP API endpoint URL (e.g., `http://10.0.0.1:1212/api`)
    pub api_endpoint: Option<Url>,

    /// Health check API URL (e.g., `http://10.0.0.1:1313/health_check`)
    pub health_check_url: Option<Url>,
}

impl ServiceEndpoints {
    /// Create new `ServiceEndpoints` from the provided URLs
    #[must_use]
    pub fn new(
        udp_trackers: Vec<Url>,
        http_trackers: Vec<Url>,
        api_endpoint: Option<Url>,
        health_check_url: Option<Url>,
    ) -> Self {
        Self {
            udp_trackers,
            http_trackers,
            api_endpoint,
            health_check_url,
        }
    }

    /// Build `ServiceEndpoints` from tracker configuration and instance IP
    ///
    /// Constructs service URLs by combining the configured bind addresses
    /// with the actual instance IP address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::runtime_outputs::ServiceEndpoints;
    /// use torrust_tracker_deployer_lib::domain::tracker::TrackerConfig;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let tracker_config = TrackerConfig::default();
    /// let instance_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    ///
    /// let endpoints = ServiceEndpoints::from_tracker_config(&tracker_config, instance_ip);
    /// ```
    #[must_use]
    pub fn from_tracker_config(
        tracker_config: &crate::domain::tracker::TrackerConfig,
        instance_ip: IpAddr,
    ) -> Self {
        let udp_trackers = Self::build_udp_tracker_urls(&tracker_config.udp_trackers, instance_ip);
        let http_trackers =
            Self::build_http_tracker_urls(&tracker_config.http_trackers, instance_ip);
        let api_endpoint =
            Self::build_api_endpoint_url(tracker_config.http_api.bind_address, instance_ip);
        let health_check_url =
            Self::build_health_check_url(tracker_config.health_check_api.bind_address, instance_ip);

        Self::new(udp_trackers, http_trackers, api_endpoint, health_check_url)
    }

    fn build_udp_tracker_urls(
        udp_trackers: &[crate::domain::tracker::UdpTrackerConfig],
        instance_ip: IpAddr,
    ) -> Vec<Url> {
        udp_trackers
            .iter()
            .filter_map(|udp| {
                Url::parse(&format!(
                    "udp://{}:{}/announce",
                    instance_ip,
                    udp.bind_address.port()
                ))
                .ok()
            })
            .collect()
    }

    fn build_http_tracker_urls(
        http_trackers: &[crate::domain::tracker::HttpTrackerConfig],
        instance_ip: IpAddr,
    ) -> Vec<Url> {
        http_trackers
            .iter()
            .filter_map(|http| {
                Url::parse(&format!(
                    "http://{}:{}/announce", // DevSkim: ignore DS137138
                    instance_ip,
                    http.bind_address.port()
                ))
                .ok()
            })
            .collect()
    }

    fn build_api_endpoint_url(
        bind_address: std::net::SocketAddr,
        instance_ip: IpAddr,
    ) -> Option<Url> {
        Url::parse(&format!(
            "http://{}:{}/api", // DevSkim: ignore DS137138
            instance_ip,
            bind_address.port()
        ))
        .ok()
    }

    fn build_health_check_url(
        bind_address: std::net::SocketAddr,
        instance_ip: IpAddr,
    ) -> Option<Url> {
        Url::parse(&format!(
            "http://{}:{}/health_check", // DevSkim: ignore DS137138
            instance_ip,
            bind_address.port()
        ))
        .ok()
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
///     service_endpoints: None,
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

    /// Service endpoints populated after services are started
    ///
    /// This field stores the URLs for all deployed tracker services. It is
    /// populated by the `run` command after services start successfully.
    ///
    /// - `None`: Services not yet started or legacy state
    /// - `Some(endpoints)`: URLs for all running services
    #[serde(default)]
    pub service_endpoints: Option<ServiceEndpoints>,
}
