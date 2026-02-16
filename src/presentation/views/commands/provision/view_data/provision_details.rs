//! Provision Details Data Transfer Object
//!
//! This module contains the presentation DTO for provision command details.
//! It serves as the data structure passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::IpAddr;
use std::path::PathBuf;

use crate::application::command_handlers::show::info::ServiceInfo;
use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;

/// Provision details data for rendering
///
/// This struct holds all the data needed to render provision command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples domain models from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
#[derive(Debug, Clone, Serialize)]
pub struct ProvisionDetailsData {
    /// Name of the provisioned environment
    pub environment_name: String,
    /// Name of the provisioned instance
    pub instance_name: String,
    /// IP address of the provisioned instance
    pub instance_ip: Option<IpAddr>,
    /// SSH username for connecting to the instance
    pub ssh_username: String,
    /// SSH port for connections
    pub ssh_port: u16,
    /// Path to SSH private key
    pub ssh_private_key_path: PathBuf,
    /// Infrastructure provider (lowercase: "lxd" or "hetzner")
    pub provider: String,
    /// Timestamp when the environment was provisioned (ISO 8601 format in JSON)
    pub provisioned_at: DateTime<Utc>,
    /// Configured domain names (empty array for non-HTTPS configurations)
    pub domains: Vec<String>,
}

/// Conversion from domain model to presentation DTO
///
/// This `From` trait implementation is placed in the presentation layer
/// (not in the domain layer) to maintain proper DDD layering:
///
/// - Domain layer should not depend on presentation layer DTOs
/// - Presentation layer can depend on domain models (allowed)
/// - This keeps the domain clean and focused on business logic
///
/// Alternative approaches considered:
/// - Adding method to `Environment<Provisioned>`: Would violate DDD by making
///   domain depend on presentation DTOs
/// - Keeping mapping in controller: Works but less idiomatic than `From` trait
impl From<&Environment<Provisioned>> for ProvisionDetailsData {
    fn from(environment: &Environment<Provisioned>) -> Self {
        // Extract domains from tracker configuration via ServiceInfo
        let domains = if let Some(ip) = environment.instance_ip() {
            let tracker_config = environment.tracker_config();
            let grafana_config = environment.grafana_config();
            let services = ServiceInfo::from_tracker_config(tracker_config, ip, grafana_config);
            services
                .tls_domain_names()
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        } else {
            vec![]
        };

        Self {
            environment_name: environment.name().as_str().to_string(),
            instance_name: environment.instance_name().as_str().to_string(),
            instance_ip: environment.instance_ip(),
            ssh_username: environment.ssh_username().as_str().to_string(),
            ssh_port: environment.ssh_port(),
            ssh_private_key_path: environment.ssh_private_key_path().clone(),
            provider: environment.provider_config().provider_name().to_string(),
            provisioned_at: environment.created_at(),
            domains,
        }
    }
}
