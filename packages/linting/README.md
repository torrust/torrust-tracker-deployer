# Torrust Linting Package

This package provides a unified linting framework that can be easily integrated into any Rust project.

## Features

- **Multiple Linters**: Supports markdown, YAML, TOML, Rust (clippy + rustfmt), and shell script linting
- **CLI Ready**: Pre-built CLI components for easy binary creation
- **Extensible**: Easy to add new linters
- **Configurable**: Uses existing configuration files (.taplo.toml, .yamllint.yml, etc.)

## Usage

### Option 1: Use the Complete CLI (Easiest)

Create a simple binary that uses the complete CLI:

```rust
// src/bin/linter.rs
use anyhow::Result;

fn main() -> Result<()> {
    torrust_linting::run_cli()
}
```

This gives you a full-featured linter CLI with all commands and help text.

### Option 2: Custom CLI Implementation

For more control, you can use the individual components:

```rust
use anyhow::Result;
use clap::Parser;
use torrust_linting::{Cli, execute_command, init_tracing};

fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    execute_command(cli.command)?;

    Ok(())
}
```

### Option 3: Use Individual Linters

Use specific linters programmatically:

```rust
use anyhow::Result;
use torrust_linting::{run_rustfmt_linter, run_clippy_linter, run_all_linters};

fn main() -> Result<()> {
    // Run individual linters
    run_rustfmt_linter()?;
    run_clippy_linter()?;

    // Or run all linters at once
    run_all_linters()?;

    Ok(())
}
```

### Option 4: Custom Command Structure

Build your own CLI with custom commands:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use torrust_linting::{run_rustfmt_linter, run_clippy_linter};

#[derive(Parser)]
struct MyCli {
    #[command(subcommand)]
    command: MyCommands,
}

#[derive(Subcommand)]
enum MyCommands {
    /// Check Rust formatting
    Format,
    /// Run Rust linting
    Lint,
}

fn main() -> Result<()> {
    let cli = MyCli::parse();

    match cli.command {
        MyCommands::Format => run_rustfmt_linter()?,
        MyCommands::Lint => run_clippy_linter()?,
    }

    Ok(())
}
```

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
torrust-linting = { path = "path/to/torrust-linting" }
```

Or if using in a workspace:

```toml
[workspace]
members = ["packages/torrust-linting"]

[dependencies]
torrust-linting = { path = "packages/torrust-linting" }
```

## Available Linters

- **Markdown**: Uses markdownlint-cli
- **YAML**: Uses yamllint
- **TOML**: Uses taplo
- **Rust Clippy**: Uses cargo clippy
- **Rust Format**: Uses cargo fmt
- **Shell**: Uses shellcheck

## Configuration

The linting package respects existing configuration files:

- `.taplo.toml` for TOML formatting
- `.yamllint.yml` for YAML linting
- `.markdownlint.json` for Markdown linting
- Standard Rust tooling configuration for clippy and rustfmt
