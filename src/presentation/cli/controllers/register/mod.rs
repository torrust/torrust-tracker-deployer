//! Register Command Presentation Module
//!
//! This module implements the CLI presentation layer for the register command,
//! handling argument processing and user interaction.
//!
//! ## Architecture
//!
//! The register command presentation layer follows the DDD pattern, orchestrating
//! the application layer's `RegisterCommandHandler` while providing user-friendly
//! output and error handling.
//!
//! ## Components
//!
//! - `errors` - Presentation layer error types with `.help()` methods
//! - `handler` - Main command controller orchestrating the workflow

pub mod errors;
pub mod handler;
pub use handler::RegisterCommandController;
