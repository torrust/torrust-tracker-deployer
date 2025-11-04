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

### Checking Dependencies

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

### Using Individual Detectors

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
