use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use tracing::{error, info, Level};

use crate::linters::{
    run_clippy_linter, run_cspell_linter, run_markdown_linter, run_rustfmt_linter,
    run_shellcheck_linter, run_toml_linter, run_yaml_linter,
};

/// Initialize tracing with default configuration
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .with_max_level(Level::INFO)
        .init();
}

/// Run the complete linter CLI application
///
/// This function initializes tracing, parses CLI arguments, and executes the appropriate linting commands.
///
/// # Errors
///
/// Returns an error if any linter fails or if CLI parsing fails.
pub fn run_cli() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    execute_command(cli.command.as_ref())?;

    Ok(())
}
#[derive(Parser)]
#[command(name = "linter")]
#[command(about = "Unified linting tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run markdown linter
    #[command(alias = "md")]
    Markdown,

    /// Run YAML linter
    Yaml,

    /// Run TOML linter using Taplo
    Toml,

    /// Run `CSpell` spell checker
    #[command(alias = "spell")]
    Cspell,

    /// Run Rust clippy linter
    Clippy,

    /// Run Rust formatter check
    #[command(alias = "fmt")]
    Rustfmt,

    /// Run `ShellCheck` linter
    #[command(alias = "shell")]
    Shellcheck,

    /// Run all linters
    All,
}

/// Run all linters and collect results
///
/// # Errors
///
/// Returns an error if any linter fails.
pub fn run_all_linters() -> Result<()> {
    info!("Running All Linters");

    let mut failed = false;

    // Run markdown linter
    match run_markdown_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Markdown linting failed: {e}");
            failed = true;
        }
    }

    // Run YAML linter
    match run_yaml_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("YAML linting failed: {e}");
            failed = true;
        }
    }

    // Run TOML linter
    match run_toml_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("TOML linting failed: {e}");
            failed = true;
        }
    }

    // Run CSpell spell checker
    match run_cspell_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Spell checking failed: {e}");
            failed = true;
        }
    }

    // Run Rust clippy linter
    match run_clippy_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Rust clippy linting failed: {e}");
            failed = true;
        }
    }

    // Run Rust formatter check
    match run_rustfmt_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Rust formatting failed: {e}");
            failed = true;
        }
    }

    // Run ShellCheck linter
    match run_shellcheck_linter() {
        Ok(()) => {}
        Err(e) => {
            error!("Shell script linting failed: {e}");
            failed = true;
        }
    }

    if failed {
        error!("Some linters failed");
        return Err(anyhow::anyhow!("Some linters failed"));
    }
    info!("All linters passed");
    Ok(())
}

/// Execute the linting command
///
/// # Errors
///
/// Returns an error if any linter fails.
pub fn execute_command(command: Option<&Commands>) -> Result<()> {
    match command {
        Some(Commands::Markdown) => {
            run_markdown_linter()?;
        }
        Some(Commands::Yaml) => {
            run_yaml_linter()?;
        }
        Some(Commands::Toml) => {
            run_toml_linter()?;
        }
        Some(Commands::Cspell) => {
            run_cspell_linter()?;
        }
        Some(Commands::Clippy) => {
            run_clippy_linter()?;
        }
        Some(Commands::Rustfmt) => {
            run_rustfmt_linter()?;
        }
        Some(Commands::Shellcheck) => {
            run_shellcheck_linter()?;
        }
        Some(Commands::All) => {
            run_all_linters()?;
        }
        None => {
            // Show help when no command is provided
            let mut cmd = Cli::command();
            cmd.print_help()?;
            print_usage_examples();
        }
    }

    Ok(())
}

/// Print usage examples
pub fn print_usage_examples() {
    println!("\n");
    println!("Examples:");
    println!("  cargo run --bin linter markdown   # Run markdown linter");
    println!("  cargo run --bin linter yaml       # Run YAML linter");
    println!("  cargo run --bin linter toml       # Run TOML linter");
    println!("  cargo run --bin linter cspell     # Run CSpell spell checker");
    println!("  cargo run --bin linter clippy     # Run Rust clippy linter");
    println!("  cargo run --bin linter rustfmt    # Run Rust formatter check");
    println!("  cargo run --bin linter shellcheck # Run ShellCheck linter");
    println!("  cargo run --bin linter all        # Run all linters");
}
