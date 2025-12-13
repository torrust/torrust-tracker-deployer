//! JSON Schema Generation Infrastructure
//!
//! This module provides infrastructure for generating JSON Schemas from Rust types
//! using the Schemars library. It serves as a thin wrapper around the external
//! dependency, isolating it from the application and domain layers.
//!
//! ## Architecture
//!
//! This is an **Infrastructure Layer** component because:
//! - Wraps external dependency (Schemars)
//! - Pure technical mechanism with no business logic
//! - Provides serialization/export functionality
//!
//! See [docs/features/json-schema-generation/specification.md](../../../docs/features/json-schema-generation/specification.md)
//! for architectural rationale.

mod schema_generator;

pub use schema_generator::{SchemaGenerationError, SchemaGenerator};
