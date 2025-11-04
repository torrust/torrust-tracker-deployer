# Torrust Dependency Installer Package

This package provides dependency detection and installation utilities for the Torrust Tracker Deployer project.

## Features

- **Tool Detection**: Check if required development tools are installed
- **Extensible**: Easy to add new tool detectors
- **Logging**: Built-in tracing support for observability
- **Error Handling**: Clear, actionable error messages

## Required Tools

This package can detect the following development dependencies:

- **cargo-machete** - Detects unused Rust dependencies
- **OpenTofu** - Infrastructure provisioning tool
- **Ansible** - Configuration management tool
- **LXD** - VM-based testing infrastructure

## Usage

### CLI Binary

The package provides a `dependency-installer` binary for command-line usage:

```bash
# Check all dependencies
dependency-installer check

# Check specific tool
dependency-installer check --tool opentofu

# List all tools with status
dependency-installer list

# Enable verbose logging
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

#### Examples

```bash
# Check all dependencies
$ dependency-installer check
Checking dependencies...

✓ cargo-machete: installed
✗ OpenTofu: not installed
✗ Ansible: not installed
✓ LXD: installed

Missing 2 out of 4 required dependencies

# Check specific tool (with aliases)
$ dependency-installer check --tool tofu
✗ OpenTofu: not installed

# List all tools
$ dependency-installer list
Available tools:

- cargo-machete (installed)
- OpenTofu (not installed)
- Ansible (not installed)
- LXD (installed)
```

#### Tool Aliases

The CLI accepts multiple aliases for tools:

- `cargo-machete` or `machete`
- `opentofu` or `tofu`
- `ansible`
- `lxd`

### Library Usage

#### Checking Dependencies

```rust
use torrust_dependency_installer::DependencyManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = DependencyManager::new();
    
    // Check all dependencies
    let results = manager.check_all()?;
    
    for result in results {
        println!("{}: {}", result.tool, if result.installed { "✓" } else { "✗" });
    }
    
    Ok(())
}
```

#### Using Individual Detectors

```rust
use torrust_dependency_installer::{ToolDetector, OpenTofuDetector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = OpenTofuDetector;
    
    if detector.is_installed()? {
        println!("{} is installed", detector.name());
    } else {
        println!("{} is not installed", detector.name());
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
