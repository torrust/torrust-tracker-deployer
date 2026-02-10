//! Render Command Module
//!
//! This module implements the delivery-agnostic `RenderCommandHandler`
//! for generating deployment artifacts without executing deployment operations.
//!
//! ## Architecture
//!
//! The `RenderCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Loads environment state via `EnvironmentRepository` (env-name mode)
//! - **Template Generation**: Renders all deployment artifacts to build directory
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Read-Only Operations**: Does not modify environment state or execute deployments
//! - **Dual Input Modes**: Supports both existing environments and direct config files
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//! - **Output Separation**: Requires explicit output directory to prevent conflicts with provision artifacts
//!
//! ## State Constraints
//!
//! - **Created State Only**: Command only works for environments in "Created" state
//! - **IP Always Required**: User must provide target IP via --instance-ip flag
//! - **Output Directory Required**: User must provide output directory via --output-dir flag
//! - **Force Flag**: Use --force to overwrite existing output directory
//!
//! ## Dual Input Modes
//!
//! 1. **Environment Name Mode** (`--env-name`):
//!    - Loads existing environment from repository
//!    - Validates state is "Created"
//!    - Uses environment configuration
//!
//! 2. **Config File Mode** (`--env-file`):
//!    - Parses configuration file directly
//!    - No environment creation or persistence
//!    - Validates configuration only

pub mod errors;
pub mod handler;

pub use errors::RenderCommandHandlerError;
pub use handler::{RenderCommandHandler, RenderInputMode, RenderResult};
