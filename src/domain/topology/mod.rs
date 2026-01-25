//! Docker Compose Topology Domain Types
//!
//! This module contains domain types for Docker Compose topology elements,
//! including networks, services, and their relationships.
//!
//! ## Design Principles
//!
//! These types represent business concepts with strong typing:
//! - Type-safe network references (no string typos)
//! - Single source of truth for network names
//! - Domain-level invariant enforcement
//!
//! ## Components
//!
//! - [`Network`] - Docker Compose network enum representing isolation boundaries

pub mod network;

// Re-export main types for convenience
pub use network::Network;
