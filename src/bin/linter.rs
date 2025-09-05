use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use torrust_tracker_deploy::linting::linters::{
    run_clippy_linter, run_markdown_linter, run_rustfmt_linter, run_shellcheck_linter,
    run_toml_linter, run_yaml_linter,
};
use tracing::{error, info, Level};

#[derive(Parser)]
#[command(name = "linter")]
#[command(about = "Unified linting tool for the Torrust Tracker Deploy project")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run markdown linter
    #[command(alias = "md")]
    Markdown,

    /// Run YAML linter
    Yaml,

    /// Run TOML linter using Taplo
    Toml,

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
fn run_all_linters() -> Result<()> {
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

fn main() -> Result<()> {
    // Initialize tracing with a format similar to the bash scripts
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Markdown) => {
            run_markdown_linter()?;
        }
        Some(Commands::Yaml) => {
            run_yaml_linter()?;
        }
        Some(Commands::Toml) => {
            run_toml_linter()?;
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
            println!("\n");
            println!("Examples:");
            println!("  cargo run --bin linter markdown   # Run markdown linter");
            println!("  cargo run --bin linter yaml       # Run YAML linter");
            println!("  cargo run --bin linter toml       # Run TOML linter");
            println!("  cargo run --bin linter clippy     # Run Rust clippy linter");
            println!("  cargo run --bin linter rustfmt    # Run Rust formatter check");
            println!("  cargo run --bin linter shellcheck # Run ShellCheck linter");
            println!("  cargo run --bin linter all        # Run all linters");
        }
    }

    Ok(())
}
