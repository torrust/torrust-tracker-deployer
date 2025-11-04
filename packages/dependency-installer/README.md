# Torrust Dependency Installer Package

This package provides dependency detection and installation utilities for the Torrust Tracker Deployer project.

## Design Philosophy

**This is an internal automation tool** - Its primary purpose is to check dependencies in CI/CD pipelines and automated workflows. As such, it uses **structured logging only** (via the `tracing` crate) rather than user-facing console output.

This design choice offers several benefits:

- **Automation-friendly**: Structured logs are easy to parse and filter programmatically
- **Consistent**: Same output format as the rest of the Torrust ecosystem
- **Simple**: The tool is straightforward enough that logging output is sufficient even for manual use
- **Observable**: Rich contextual information through structured fields

For manual usage, you can control log verbosity with the `--verbose` flag or `RUST_LOG` environment variable.

## Features

- **Dependency Detection**: Check if required development tools are installed
- **Extensible**: Easy to add new dependency detectors
- **Structured Logging**: Built-in tracing support for observability and automation
- **Type-Safe**: Uses strongly-typed enums for dependencies
- **Error Handling**: Clear, actionable error messages

## Supported Dependencies

This package can detect the following development dependencies:

- **cargo-machete** - Detects unused Rust dependencies
- **OpenTofu** - Infrastructure provisioning tool
- **Ansible** - Configuration management tool
- **LXD** - VM-based testing infrastructure

## Usage

### CLI Binary

The package provides a `dependency-installer` binary for command-line usage:

```bash
# Check all dependencies (default: info log level)
dependency-installer check

# Check specific dependency
dependency-installer check --dependency opentofu

# List all dependencies with status
dependency-installer list

# Control log level (off, error, warn, info, debug, trace)
dependency-installer check --log-level debug
dependency-installer check --log-level off   # Disable all logging

# Enable verbose logging (equivalent to --log-level debug)
dependency-installer check --verbose

# Get help
dependency-installer --help
dependency-installer check --help
```

#### Exit Codes

- **0**: Success (all checks passed)
- **1**: Missing dependencies
- **2**: Invalid arguments
- **3**: Internal error

#### Output Format

The tool uses structured logging (via `tracing`) instead of plain text output:

```bash
# Check all dependencies (default log level shows INFO and above)
$ dependency-installer check
2025-11-04T17:33:20.959847Z  INFO torrust_dependency_installer::handlers::check: Checking all dependencies
2025-11-04T17:33:20.960126Z  INFO torrust_dependency_installer::handlers::check: Dependency check result dependency="cargo-machete" status="installed"
2025-11-04T17:33:20.960131Z  INFO torrust_dependency_installer::handlers::check: Dependency check result dependency="OpenTofu" status="not installed"
2025-11-04T17:33:20.960136Z  INFO torrust_dependency_installer::handlers::check: Dependency check result dependency="Ansible" status="not installed"
2025-11-04T17:33:20.960139Z  INFO torrust_dependency_installer::handlers::check: Dependency check result dependency="LXD" status="installed"
2025-11-04T17:33:20.960144Z  INFO torrust_dependency_installer::handlers::check: Missing dependencies missing_count=2 total_count=4
Error: Check command failed: Failed to check all dependencies: Missing 2 out of 4 required dependencies

# Check specific dependency
$ dependency-installer check --dependency opentofu
2025-11-04T17:33:20.959855Z  INFO torrust_dependency_installer::handlers::check: Checking specific dependency dependency=opentofu
2025-11-04T17:33:20.960473Z  INFO torrust_dependency_installer::detector::opentofu: OpenTofu is not installed dependency="opentofu"
2025-11-04T17:33:20.960482Z  INFO torrust_dependency_installer::handlers::check: Dependency is not installed dependency="OpenTofu" status="not installed"
Error: Check command failed: Failed to check specific dependency: opentofu: not installed

# List all dependencies
$ dependency-installer list
2025-11-04T17:33:20.960482Z  INFO torrust_dependency_installer::handlers::list: Available dependency dependency="cargo-machete" status="installed"
2025-11-04T17:33:20.960494Z  INFO torrust_dependency_installer::handlers::list: Available dependency dependency="OpenTofu" status="not installed"
2025-11-04T17:33:20.960962Z  INFO torrust_dependency_installer::handlers::list: Available dependency dependency="Ansible" status="not installed"
2025-11-04T17:33:20.961521Z  INFO torrust_dependency_installer::handlers::list: Available dependency dependency="LXD" status="installed"

# Enable verbose logging (includes DEBUG level)
$ dependency-installer check --verbose
2025-11-04T17:33:20.959872Z DEBUG torrust_dependency_installer::detector::cargo_machete: Checking if cargo-machete is installed dependency="cargo-machete"
...
```

#### Dependency Names

The CLI accepts the following dependency names:

- `cargo-machete` - Rust dependency analyzer
- `opentofu` - Infrastructure provisioning tool
- `ansible` - Configuration management tool
- `lxd` - Lightweight VM manager

### Library Usage

#### Checking Dependencies

```rust
use torrust_dependency_installer::DependencyManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt::init();

    let manager = DependencyManager::new();

    // Check all dependencies
    let results = manager.check_all()?;

    for result in results {
        let detector = manager.get_detector(result.dependency);
        tracing::info!(
            dependency = detector.name(),
            installed = result.installed,
            "Dependency status"
        );
    }

    Ok(())
}
```

#### Using Individual Detectors

```rust
use torrust_dependency_installer::{DependencyDetector, Dependency, DependencyManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let manager = DependencyManager::new();
    let detector = manager.get_detector(Dependency::OpenTofu);

    if detector.is_installed()? {
        tracing::info!(dependency = detector.name(), "Dependency is installed");
    } else {
        tracing::warn!(dependency = detector.name(), "Dependency is not installed");
    }

    Ok(())
}
```

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
torrust-dependency-installer = { path = "path/to/torrust-dependency-installer" }
```

Or if using in a workspace:

```toml
[workspace]
members = ["packages/torrust-dependency-installer"]

[dependencies]
torrust-dependency-installer = { path = "packages/torrust-dependency-installer" }
```
