//! Linting binary for Torrust Tracker Deployer
//!
//! This binary provides a thin wrapper around the torrust-linting crate,
//! making it easy to run all project linting checks in a standardized way.
//!
//! It supports linting for multiple file types including Rust, Markdown, YAML,
//! and shell scripts.

//! Linting binary for Torrust Tracker Deployer
//!
//! This binary provides comprehensive linting capabilities for the project,
//! including markdown, YAML, TOML, shell scripts, and Rust code.
//!
//! It supports running individual linters or all linters together,
//! making it easy to maintain code quality across all file types.
//!
//! Linting binary for Torrust Tracker Deployer

use anyhow::Result;

fn main() -> Result<()> {
    torrust_linting::run_cli()
}
