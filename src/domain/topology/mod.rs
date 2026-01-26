//! Docker Compose Topology Domain Types
//!
//! This module contains domain types for Docker Compose topology elements,
//! including networks, ports, services, and their relationships.
//!
//! ## Design Principles
//!
//! These types represent business concepts with strong typing:
//! - Type-safe network references (no string typos)
//! - Type-safe port bindings with protocol awareness
//! - Type-safe service identification
//! - Single source of truth for network names
//! - Domain-level invariant enforcement
//!
//! ## Components
//!
//! - [`Network`] - Docker Compose network enum representing isolation boundaries
//! - [`PortBinding`] - Port mapping with protocol and description
//! - [`Service`] - Docker Compose service enum for type-safe service identification
//! - [`DockerComposeTopology`] - Aggregate that derives required networks from services
//! - [`ServiceTopology`] - Topology information for a single service
//! - [`TopologyError`] - Validation errors (e.g., port conflicts)
//! - [`PortDerivation`] - Trait for services that derive their port bindings
//! - [`NetworkDerivation`] - Trait for services that derive their network assignments

pub mod aggregate;
pub mod enabled_services;
pub mod error;
pub mod network;
pub mod port;
pub mod service;
pub mod traits;

// Re-export main types for convenience
pub use aggregate::{DockerComposeTopology, ServiceTopology};
pub use enabled_services::EnabledServices;
pub use error::{PortConflict, TopologyError};
pub use network::Network;
pub use port::PortBinding;
pub use service::Service;
pub use traits::{NetworkDerivation, PortDerivation};
