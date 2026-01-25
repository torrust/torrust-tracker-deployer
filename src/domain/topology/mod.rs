//! Docker Compose Topology Domain Types
//!
//! This module contains domain types for Docker Compose topology elements,
//! including networks, services, and their relationships.
//!
//! ## Design Principles
//!
//! These types represent business concepts with strong typing:
//! - Type-safe network references (no string typos)
//! - Type-safe service identification
//! - Single source of truth for network names
//! - Domain-level invariant enforcement
//!
//! ## Components
//!
//! - [`Network`] - Docker Compose network enum representing isolation boundaries
//! - [`Service`] - Docker Compose service enum for type-safe service identification
//! - [`DockerComposeTopology`] - Aggregate that derives required networks from services
//! - [`ServiceTopology`] - Topology information for a single service

pub mod aggregate;
pub mod network;
pub mod service;

// Re-export main types for convenience
pub use aggregate::{DockerComposeTopology, ServiceTopology};
pub use network::Network;
pub use service::Service;
