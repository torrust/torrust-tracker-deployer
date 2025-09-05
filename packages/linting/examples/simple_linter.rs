//! Example: Simple linter binary
//!
//! This example shows how to create the simplest possible linter binary
//! using the torrust-linting package.

use anyhow::Result;

fn main() -> Result<()> {
    // This single line gives you a complete linter CLI!
    torrust_linting::run_cli()
}
