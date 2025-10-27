# Template Generation Support (CLI Integration)

**Issue**: [#41](https://github.com/torrust/torrust-tracker-deployer/issues/41)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40) - Template System Integration ✅ **COMPLETED in PR #48**
**Status**: READY TO IMPLEMENT
**Related**: [Roadmap Task 1.5](../roadmap.md)

## Overview

Implement the `template` subcommand for the create command, allowing users to generate configuration file templates via the CLI. This is a **thin CLI layer** that calls the existing template generation functionality implemented in PR #48.

**Key Point**: The template generation logic is **already complete** in `src/domain/config/environment_config.rs` (PR #48). This issue is only about adding the CLI subcommand to expose that functionality to users.

## Goals

- [ ] Add `template` subcommand to the create command CLI structure
- [ ] Call existing `EnvironmentCreationConfig::generate_template_file()` method
- [ ] Support custom output paths for template generation
- [ ] Provide helpful success messages with next steps

**Estimated Time**: 1-2 hours (simple CLI integration)

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation Layer only (thin CLI wrapper)
**Module Path**: `src/presentation/console/subcommands/create/`
**Pattern**: CLI Subcommand → Domain Method Call

**What's Already Done** ✅:

- Template generation logic: `EnvironmentCreationConfig::generate_template_file()` in `src/domain/config/environment_config.rs`
- Error handling: `CreateConfigError` with template-specific variants
- Tests: Comprehensive test coverage in domain layer

**What Needs to Be Done**:

- CLI subcommand argument parsing
- Handler to call the existing domain method
- User-friendly success messages

## CLI Interface Specification

```bash
# Generate template configuration file (JSON format)
torrust-tracker-deployer create template
# Creates: ./environment-template.json in current working directory

# Generate template in specific directory
torrust-tracker-deployer create template ./config/environment.json

# Generate template with custom filename
torrust-tracker-deployer create template ./my-environment.json

# Show help for template subcommand
torrust-tracker-deployer create template --help
```

## Implementation Guide

### Step 1: Extend CLI Arguments

Update `src/presentation/console/subcommands/create/args.rs`:

```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Arguments for the create subcommand
#[derive(Debug, Args)]
pub struct CreateArgs {
    #[command(subcommand)]
    pub action: CreateAction,
}

/// Actions available for the create command
#[derive(Debug, Subcommand)]
pub enum CreateAction {
    /// Create environment from configuration file
    Environment {
        /// Path to the environment configuration file
        #[arg(long, short = 'f', value_name = "FILE")]
        env_file: PathBuf,
    },

    /// Generate template configuration file
    Template {
        /// Output path for the template file (optional)
        /// If not provided, creates environment-template.json in current directory
        #[arg(value_name = "PATH")]
        output_path: Option<PathBuf>,
    },
}

impl CreateAction {
    /// Get the default template output path
    pub fn default_template_path() -> PathBuf {
        PathBuf::from("environment-template.json")
    }
}
```

### Step 2: Implement Handler

Update `src/presentation/console/subcommands/create/handler.rs`:

```rust
use super::{CreateArgs, CreateAction};
use crate::domain::config::EnvironmentCreationConfig;
use std::path::PathBuf;

pub async fn handle_create_command(args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        CreateAction::Template { output_path } => {
            let template_path = output_path
                .unwrap_or_else(|| CreateAction::default_template_path());
            handle_template_generation(template_path).await
        }
        CreateAction::Environment { env_file } => {
            handle_environment_creation(env_file).await
        }
    }
}

async fn handle_template_generation(output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating configuration template...");

    // Call existing domain method - template generation implemented in PR #48
    EnvironmentCreationConfig::generate_template_file(&output_path).await?;

    println!("✅ Configuration template generated: {}", output_path.display());
    println!();
    println!("Next steps:");
    println!("1. Edit the template file and replace placeholder values:");
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')");
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");
    println!("2. Review default values:");
    println!("   - username: 'torrust' (can be changed if needed)");
    println!("   - port: 22 (standard SSH port)");
    println!("3. Create the environment:");
    println!("   torrust-tracker-deployer create environment --env-file {}", output_path.display());

    Ok(())
}

async fn handle_environment_creation(config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Existing environment creation logic from other subissues
    todo!("Environment creation logic - implemented in previous subissues")
}
```

### Step 3: Update Module Exports

Ensure `src/presentation/console/subcommands/create/mod.rs` exports the necessary types:

```rust
mod args;
mod handler;

pub use args::{CreateArgs, CreateAction};
pub use handler::handle_create_command;
```

## Generated Template Format

The template generated by `EnvironmentCreationConfig::generate_template_file()` looks like this:

```json
{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME"
  },
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
    "username": "torrust",
    "port": 22
  }
}
```

**Note**: This format is guaranteed to be valid and up-to-date because it's generated directly from the `EnvironmentCreationConfig` struct using serde serialization.

## Error Handling

Errors are automatically handled by the existing `CreateConfigError` enum (implemented in PR #48):

- `TemplateSerializationFailed`: JSON serialization errors (should never happen)
- `TemplateDirectoryCreationFailed`: Parent directory cannot be created
- `TemplateFileWriteFailed`: File cannot be written

All errors include `.help()` methods with actionable troubleshooting guidance. The CLI handler just needs to propagate these errors using `?` operator.

## Testing

### Unit Tests for CLI Arguments

Add to `src/presentation/console/subcommands/create/args.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_use_default_template_path_when_none_provided() {
        let default_path = CreateAction::default_template_path();
        assert_eq!(default_path, PathBuf::from("environment-template.json"));
    }

    #[test]
    fn it_should_parse_template_action_with_custom_path() {
        // Test with clap parsing if needed
        // This tests the CLI argument structure
    }
}
```

### Integration Test for Handler

Add to `src/presentation/console/subcommands/create/handler.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn it_should_generate_template_with_default_path() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = CreateArgs {
            action: CreateAction::Template {
                output_path: None,
            },
        };

        let result = handle_create_command(args).await;
        assert!(result.is_ok());

        // Verify file exists
        let template_path = temp_dir.path().join("environment-template.json");
        assert!(template_path.exists());
    }

    #[tokio::test]
    async fn it_should_generate_template_with_custom_path() {
        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("config").join("my-env.json");

        let args = CreateArgs {
            action: CreateAction::Template {
                output_path: Some(custom_path.clone()),
            },
        };

        let result = handle_create_command(args).await;
        assert!(result.is_ok());

        // Verify file exists at custom path
        assert!(custom_path.exists());
        // Verify parent directory was created
        assert!(custom_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn it_should_generate_valid_json_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test.json");

        let args = CreateArgs {
            action: CreateAction::Template {
                output_path: Some(template_path.clone()),
            },
        };

        handle_create_command(args).await.unwrap();

        // Read and parse the generated template
        let content = std::fs::read_to_string(&template_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify structure
        assert!(parsed["environment"]["name"].is_string());
        assert!(parsed["ssh_credentials"]["private_key_path"].is_string());
        assert_eq!(parsed["ssh_credentials"]["username"], "torrust");
        assert_eq!(parsed["ssh_credentials"]["port"], 22);
    }
}
```

**Note**: The domain-layer tests in `src/domain/config/environment_config.rs` already cover the template generation logic comprehensively. These CLI tests only verify the CLI integration works correctly.

## Acceptance Criteria

- [ ] `torrust-tracker-deployer create template` generates `environment-template.json` in current directory
- [ ] `torrust-tracker-deployer create template ./config/env.json` generates template at specified path
- [ ] Generated template contains valid JSON structure with placeholder values
- [ ] Template generation creates parent directories if they don't exist
- [ ] Clear success message with next steps instructions after template generation
- [ ] Proper error handling with actionable help messages (inherited from domain layer)
- [ ] Template subcommand integrates cleanly with existing create command structure
- [ ] Help system works correctly: `torrust-tracker-deployer create template --help`
- [ ] Unit tests verify CLI argument parsing
- [ ] Integration tests verify end-to-end template generation via CLI

## Success Criteria

When complete, users should be able to:

1. **Generate a template**: `torrust-tracker-deployer create template`
2. **See clear output**:

   ```text
   Generating configuration template...
   ✅ Configuration template generated: environment-template.json

   Next steps:
   1. Edit the template file and replace placeholder values:
      - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')
      - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key
      - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key
   2. Review default values:
      - username: 'torrust' (can be changed if needed)
      - port: 22 (standard SSH port)
   3. Create the environment:
      torrust-tracker-deployer create environment --env-file environment-template.json
   ```

3. **Edit the template** with their actual values
4. **Create the environment** using the edited configuration

## Implementation Notes

### What NOT to Do

❌ **Don't create new template generation logic** - it's already done in the domain layer
❌ **Don't create infrastructure code** - `src/infrastructure/templates/` is not needed
❌ **Don't duplicate error types** - use existing `CreateConfigError` from domain layer
❌ **Don't implement synchronous file operations** - the domain method is already async

### What TO Do

✅ **Call existing domain method**: `EnvironmentCreationConfig::generate_template_file(&path).await`
✅ **Keep it simple**: This is just a thin CLI wrapper
✅ **Focus on user experience**: Good error messages and clear next steps
✅ **Write integration tests**: Test the CLI flow, not the template generation (already tested)

### Key API Reference

```rust
// Domain layer - already implemented in PR #48
impl EnvironmentCreationConfig {
    /// Creates a template instance with placeholder values
    pub fn template() -> Self;

    /// Generates a configuration template file at the specified path
    /// - Creates parent directories if needed
    /// - Generates pretty-printed JSON
    /// - Returns CreateConfigError on failure
    pub async fn generate_template_file(path: &std::path::Path) -> Result<(), CreateConfigError>;
}

// Error type - already implemented in PR #48
pub enum CreateConfigError {
    TemplateSerializationFailed { source: serde_json::Error },
    TemplateDirectoryCreationFailed { path: PathBuf, source: std::io::Error },
    TemplateFileWriteFailed { path: PathBuf, source: std::io::Error },
    // ... other variants
}
```

## Future Enhancements

This implementation provides the foundation for future template features:

- **TOML template support**: Add `--format` flag to choose JSON or TOML
- **Interactive template generation**: Guided template creation with prompts
- **Template examples**: Pre-filled templates for common scenarios
- **Template validation**: Pre-validation before writing to disk

These enhancements will be implemented as separate issues after the core functionality is complete.
