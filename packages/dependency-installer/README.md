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
- **Dependency Installation**: Install missing dependencies automatically
- **Extensible**: Easy to add new dependency detectors and installers
- **Structured Logging**: Built-in tracing support for observability and automation
- **Type-Safe**: Uses strongly-typed enums for dependencies
- **Error Handling**: Clear, actionable error messages
- **Async Support**: Asynchronous installation operations for better performance

## Supported Dependencies

This package can detect and install the following development dependencies:

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

# Install all dependencies
dependency-installer install

# Install specific dependency
dependency-installer install --dependency opentofu

# List all dependencies with status
dependency-installer list

# Control log level (off, error, warn, info, debug, trace)
dependency-installer check --log-level debug
dependency-installer install --log-level debug
dependency-installer check --log-level off   # Disable all logging

# Enable verbose logging (equivalent to --log-level debug)
dependency-installer check --verbose
dependency-installer install --verbose

# Get help
dependency-installer --help
dependency-installer check --help
dependency-installer install --help
```

#### Exit Codes

- **0**: Success (all checks or installations passed)
- **1**: Missing dependencies or installation failures
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

# Install all dependencies
$ dependency-installer install
2025-11-04T19:30:10.000000Z  INFO torrust_dependency_installer::handlers::install: Installing all dependencies
2025-11-04T19:30:10.100000Z  INFO torrust_dependency_installer::installer::cargo_machete: Installing cargo-machete dependency="cargo-machete"
2025-11-04T19:30:25.000000Z  INFO torrust_dependency_installer::handlers::install: Dependency installation result dependency="cargo-machete" status="installed"
2025-11-04T19:30:25.100000Z  INFO torrust_dependency_installer::installer::opentofu: Installing OpenTofu dependency="opentofu"
2025-11-04T19:30:40.000000Z  INFO torrust_dependency_installer::handlers::install: Dependency installation result dependency="OpenTofu" status="installed"
...
2025-11-04T19:31:00.000000Z  INFO torrust_dependency_installer::handlers::install: All dependencies installed successfully

# Install specific dependency with verbose logging
$ dependency-installer install --dependency opentofu --verbose
2025-11-04T19:30:10.000000Z  INFO torrust_dependency_installer::handlers::install: Installing specific dependency dependency=opentofu
2025-11-04T19:30:10.100000Z  INFO torrust_dependency_installer::installer::opentofu: Installing OpenTofu dependency="opentofu"
2025-11-04T19:30:10.200000Z DEBUG torrust_dependency_installer::installer::opentofu: Downloading OpenTofu installer script
2025-11-04T19:30:12.000000Z DEBUG torrust_dependency_installer::installer::opentofu: Making installer script executable
2025-11-04T19:30:12.100000Z DEBUG torrust_dependency_installer::installer::opentofu: Running OpenTofu installer with sudo
2025-11-04T19:30:25.000000Z DEBUG torrust_dependency_installer::installer::opentofu: Cleaning up installer script
2025-11-04T19:30:25.100000Z  INFO torrust_dependency_installer::handlers::install: Dependency installation completed dependency="OpenTofu" status="installed"

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

#### Installing Dependencies

```rust
use torrust_dependency_installer::{Dependency, DependencyManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt::init();

    let manager = DependencyManager::new();

    // Install specific dependency
    manager.install(Dependency::OpenTofu).await?;

    // Or install all dependencies
    let results = manager.install_all().await;

    for result in results {
        let installer = manager.get_installer(result.dependency);
        if result.success {
            tracing::info!(
                dependency = installer.name(),
                "Installation succeeded"
            );
        } else {
            tracing::error!(
                dependency = installer.name(),
                error = result.error.as_deref().unwrap_or("unknown"),
                "Installation failed"
            );
        }
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

## Testing

### Operating System Pre-conditions

The Docker-based integration tests use a pre-configured image (`docker/ubuntu-24.04.Dockerfile`) that documents the **operating system dependencies required** before using the installers. These pre-conditions include:

- **System packages**: `ca-certificates`, `sudo`, `curl`, `build-essential`
- **Rust toolchain**: `nightly-2025-10-15` via rustup (for cargo-machete installation)
- **PATH configuration**: Cargo binaries accessible in PATH

The Dockerfile serves as **explicit documentation** of what must be installed on the operating system before the dependency installers can work. This ensures:

- Clear expectations for production environments
- Fast test execution (OS dependencies installed once during image build)
- Confidence that installers work given the declared pre-conditions

### Running Tests

```bash
# Run all tests (unit tests run normally, Docker tests use pre-built image)
cargo test -p torrust-dependency-installer

# Run only Docker-based integration tests
cargo test -p torrust-dependency-installer --test docker_install_command
cargo test -p torrust-dependency-installer --test docker_check_command

# Run expensive tests (OpenTofu, Ansible, LXD)
cargo test -p torrust-dependency-installer -- --ignored

# Build the test Docker image
docker build -f docker/ubuntu-24.04.Dockerfile -t dependency-installer-test:ubuntu-24.04 .
```

See `docker/README.md` for detailed testing infrastructure documentation.

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
