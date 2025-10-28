//! Repository module for environment persistence
//!
//! This module provides the abstractions and implementations for persisting
//! environment state across different storage backends.

mod environment_repository;
mod repository_error;
mod typed_repository;

// Re-export public API
pub use environment_repository::EnvironmentRepository;
pub use repository_error::RepositoryError;
pub use typed_repository::TypedEnvironmentRepository;
