//! Example: Custom CLI with individual linters
//!
//! This example shows how to create a custom CLI that uses individual
//! linter functions from the torrust-linting package.

use anyhow::Result;
use clap::{Parser, Subcommand};
use torrust_linting::{
    run_clippy_linter, run_markdown_linter, run_rustfmt_linter, run_shellcheck_linter,
    run_toml_linter, run_yaml_linter,
};

#[derive(Parser)]
#[command(name = "custom-linter")]
#[command(about = "A custom linter CLI using torrust-linting")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint only Rust code (clippy + rustfmt)
    Rust,
    /// Lint only markup files (markdown + yaml + toml)
    Markup,
    /// Lint only shell scripts
    Shell,
}

fn main() -> Result<()> {
    // Initialize logging (you can customize this)
    torrust_linting::init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Rust => {
            println!("🦀 Running Rust linters...");
            run_clippy_linter()?;
            run_rustfmt_linter()?;
        }
        Commands::Markup => {
            println!("📄 Running markup linters...");
            run_markdown_linter()?;
            run_yaml_linter()?;
            run_toml_linter()?;
        }
        Commands::Shell => {
            println!("🐚 Running shell script linter...");
            run_shellcheck_linter()?;
        }
    }

    println!("✅ All selected linters passed!");
    Ok(())
}
