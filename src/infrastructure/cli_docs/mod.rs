//! CLI JSON Documentation Generation Infrastructure
//!
//! This module provides infrastructure for generating JSON documentation from Clap CLI
//! structures using runtime introspection. It provides a machine-readable,
//! versionable specification of the CLI interface.
//!
//! ## Architecture
//!
//! This is an **Infrastructure Layer** component because:
//! - Uses external dependency (Clap) introspection APIs
//! - Pure technical mechanism with no business logic
//! - Provides serialization/export functionality
//!
//! ## Usage
//!
//! ```rust
//! use clap::Parser;
//! use torrust_tracker_deployer_lib::infrastructure::cli_docs::CliDocsGenerator;
//!
//! #[derive(Parser)]
//! struct MyCli {
//!     #[arg(short, long)]
//!     verbose: bool,
//! }
//!
//! let docs = CliDocsGenerator::generate::<MyCli>()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Module Structure
//!
//! - `generator` - Main documentation generator (`CliDocsGenerator`)
//! - `schema_builder` - JSON building utilities
//! - `errors` - Error types

mod errors;
mod generator;
mod schema_builder;

pub use errors::CliDocsGenerationError;
pub use generator::CliDocsGenerator;
