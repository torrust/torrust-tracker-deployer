# Module Organization

This document outlines the conventions for organizing items within Rust modules in the Torrust Tracker Deployer project.

## 📚 Background

While Rust doesn't enforce strict ordering rules for items within modules, following consistent organization principles makes code more maintainable, readable, and easier to navigate. The approach described here aligns with common Rust community practices and emphasizes a **top-down, public-first** organization style.

### Formal Conventions

The Rust community commonly refers to these practices as:

- **Top-down organization**: High-level abstractions before low-level details
- **Visibility-first ordering**: Public items before private items
- **Importance-based ordering**: Main responsibilities before secondary concerns

While not formally standardized in official Rust guidelines, these patterns are widely adopted in well-maintained Rust projects and align with principles of progressive disclosure and cognitive load reduction.

## 🎯 Core Principles

### 1. Imports Always First

Keep all imports at the top of the file, organized in groups:

```rust
// Standard library imports
use std::path::{Path, PathBuf};
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Internal crate imports - absolute paths
use crate::domain::Environment;
use crate::shared::Clock;

// Internal crate imports - relative paths (if needed)
use super::config::Config;
```

**Why**: This follows universal Rust conventions and makes dependencies immediately visible.

### 2. Public Before Private

Place public items before private items:

```rust
// ✅ Good: Public API first
pub struct Environment {
    name: String,
    data_dir: PathBuf,
}

impl Environment {
    pub fn new(name: String) -> Self {
        let data_dir = calculate_data_dir(&name);
        Self { name, data_dir }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Private helpers come after
fn calculate_data_dir(name: &str) -> PathBuf {
    PathBuf::from("data").join(name)
}
```

**Why**: Users of the module see the public interface first without wading through implementation details.

### 3. High-Level Before Low-Level

Organize abstractions from high-level (business logic) to low-level (implementation details):

```rust
// ✅ Good: High-level abstraction first
pub trait CommandExecutor {
    fn execute(&self, command: Command) -> Result<State>;
}

// Mid-level implementation
pub struct DefaultCommandExecutor {
    step_runner: Arc<dyn StepRunner>,
}

impl CommandExecutor for DefaultCommandExecutor {
    fn execute(&self, command: Command) -> Result<State> {
        self.step_runner.run_steps(command.steps())
    }
}

// Low-level details
trait StepRunner {
    fn run_steps(&self, steps: Vec<Step>) -> Result<State>;
}
```

**Why**: Readers can understand what the module does before diving into how it works.

### 4. Important Before Secondary

Place primary responsibilities before secondary concerns (like error types, constants, helpers):

```rust
// ✅ Good: Main types and functions first
pub struct ConfigLoader {
    base_path: PathBuf,
}

impl ConfigLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        let path = self.config_path();
        let content = std::fs::read_to_string(&path)
            .map_err(|source| ConfigError::FileAccess { path: path.clone(), source })?;

        serde_json::from_str(&content)
            .map_err(|source| ConfigError::InvalidJson { path, source })
    }

    fn config_path(&self) -> PathBuf {
        self.base_path.join("config.json")
    }
}

// Secondary: Error types come after main implementation
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Cannot access configuration file: {path}")]
    FileAccess {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid JSON in configuration file: {path}")]
    InvalidJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}
```

**Why**: The main purpose and capabilities of the module are immediately visible.

## 📋 Complete Ordering Guide

For a typical module, use this order:

1. **Module-level documentation** (`//!` comments)
2. **Imports** (grouped: std → external → internal)
3. **Public constants and type aliases**
4. **Public traits** (high-level abstractions)
5. **Public structs and enums** (main types)
6. **Public implementations** (for the main types)
7. **Public free functions** (module-level utilities)
8. **Private constants and type aliases**
9. **Private traits**
10. **Private structs and enums** (implementation details)
11. **Private implementations**
12. **Private helper functions**
13. **Error types** (even if public, these are secondary concerns)
14. **Test modules** (`#[cfg(test)]`)

### Complete Example

```rust
//! Configuration management for deployment environments.
//!
//! This module provides functionality to load, validate, and manage
//! configuration for different deployment environments.

// Standard library
use std::fs;
use std::path::{Path, PathBuf};

// External crates
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Internal crate
use crate::domain::Environment;

// ============================================================================
// PUBLIC API - Constants
// ============================================================================

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "config.json";

// ============================================================================
// PUBLIC API - Traits
// ============================================================================

/// Trait for loading configuration from various sources
pub trait ConfigLoader {
    fn load(&self) -> Result<Config, ConfigError>;
}

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Configuration for a deployment environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub provider: String,
    pub instance_count: usize,
}

impl Config {
    pub fn new(name: String, provider: String, instance_count: usize) -> Self {
        Self {
            name,
            provider,
            instance_count,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.instance_count == 0 {
            return Err(ConfigError::InvalidInstanceCount {
                value: self.instance_count,
            });
        }
        Ok(())
    }
}

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

/// Loads configuration from the filesystem
pub struct FileSystemConfigLoader {
    base_path: PathBuf,
}

impl FileSystemConfigLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn config_path(&self) -> PathBuf {
        self.base_path.join(DEFAULT_CONFIG_FILE)
    }
}

impl ConfigLoader for FileSystemConfigLoader {
    fn load(&self) -> Result<Config, ConfigError> {
        let path = self.config_path();
        let content = fs::read_to_string(&path)
            .map_err(|source| ConfigError::FileAccess {
                path: path.clone(),
                source,
            })?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|source| ConfigError::InvalidJson { path, source })?;

        config.validate()?;

        Ok(config)
    }
}

// ============================================================================
// PRIVATE - Helper Functions
// ============================================================================

fn default_base_path() -> PathBuf {
    PathBuf::from("./config")
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Cannot access configuration file: {path}")]
    FileAccess {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid JSON in configuration file: {path}")]
    InvalidJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid instance count: {value}, must be greater than 0")]
    InvalidInstanceCount { value: usize },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_load_valid_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(DEFAULT_CONFIG_FILE);

        let config_json = r#"{
            "name": "test",
            "provider": "lxd",
            "instance_count": 1
        }"#;

        fs::write(&config_path, config_json).unwrap();

        let loader = FileSystemConfigLoader::new(temp_dir.path().to_path_buf());
        let config = loader.load().unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.provider, "lxd");
        assert_eq!(config.instance_count, 1);
    }

    #[test]
    fn it_should_reject_zero_instance_count() {
        let config = Config::new("test".to_string(), "lxd".to_string(), 0);
        let result = config.validate();

        assert!(result.is_err());
    }
}
```

## 🚫 Anti-Patterns to Avoid

### ❌ Random Ordering

```rust
// Bad: No clear organization
fn private_helper() -> String {
    "helper".to_string()
}

pub struct MainType {
    field: String,
}

const PRIVATE_CONSTANT: &str = "value";

pub fn public_function() -> String {
    private_helper()
}

#[derive(Error)]
pub enum MyError {
    // ...
}
```

### ❌ Private Before Public

```rust
// Bad: Private implementation details first
fn internal_calculate(x: i32) -> i32 {
    x * 2
}

struct InternalState {
    value: i32,
}

// Public API buried below
pub struct Calculator {
    state: InternalState,
}

pub fn calculate(x: i32) -> i32 {
    internal_calculate(x)
}
```

### ❌ Error Types Mixed with Main Logic

```rust
// Bad: Error types interrupting the flow
pub struct Config {
    name: String,
}

#[derive(Error)]
pub enum ConfigError {
    // ...
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        // ...
    }
}

#[derive(Error)]
pub enum LoadError {
    // ...
}
```

## 📏 Guidelines Summary

### Do's ✅

- **Keep imports at the top** - Always, in organized groups
- **Public before private** - Makes the API clear
- **High-level before low-level** - Improves comprehension
- **Important before secondary** - Highlights main responsibilities
- **Group related items** - Use section comments for clarity
- **Error types at the end** - Unless they're the module's main purpose
- **Tests last** - Always in `#[cfg(test)]` modules

### Don'ts ❌

- **Don't scatter public items** - Group them together
- **Don't bury the API** - Public items should be easily found
- **Don't mix concerns** - Keep related items together
- **Don't ignore visibility** - Respect public/private boundaries
- **Don't forget documentation** - Especially for public items

## 🎯 When to Deviate

These guidelines are general principles, not absolute rules. Consider deviating when:

- **Error types are the main purpose**: If a module primarily defines error types (e.g., `domain::errors`), they should be prominent
- **Builder patterns**: When using the builder pattern, keeping the builder next to the main type may improve clarity
- **Strongly related types**: When types are tightly coupled, grouping them together may be more important than strict ordering
- **Small modules**: Very small modules (< 100 lines) may not need strict section separation

Use your judgment, but **always prioritize readability and maintainability**.

## 🔗 Related Documentation

- [Testing Conventions](./testing.md) - How to organize test code
- [Error Handling Guide](./error-handling.md) - Error type design principles
- [Development Principles](../development-principles.md) - Overall code quality standards

## 📚 Further Reading

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Official Rust API design guidelines
- [Effective Rust](https://www.lurklurk.org/effective-rust/) - Best practices for Rust code organization
- Clean Code principles applied to Rust development

By following these conventions, we ensure that modules in the Torrust Tracker Deployer project are consistent, readable, and maintainable for all contributors.
