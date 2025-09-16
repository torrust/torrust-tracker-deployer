//! Linting binary for Torrust Tracker Deploy
//!
//! This binary provides a thin wrapper around the torrust-linting crate,
//! making it easy to run all project linting checks in a standardized way.
//!
//! It supports linting for multiple file types including Rust, Markdown, YAML,
//! and shell scripts.

//! Linting binary for Torrust Tracker Deploy
//!
//! This binary provides code quality and style checking functionality by
//! delegating to the external `torrust_linting` crate. It serves as a
//! convenient entry point for running all configured linting checks.
//!
//! ## Usage
//!
//! Run via cargo: `cargo run --bin linter`
//!
//! The actual linting implementation is provided by the `torrust_linting`
//! package which contains the project's linting rules and configurations.

use anyhow::Result;

fn main() -> Result<()> {
    torrust_linting::run_cli()
}
