# Template Generation Support

**Issue**: [#41](https://github.com/torrust/torrust-tracker-deployer/issues/41)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40) - Template System Integration
**Status**: OPTIONAL ENHANCEMENT
**Related**: [Roadmap Task 1.5](../roadmap.md), [Template System Integration](./40-subissue-6-template-system-integration.md)

## Overview

Implement the `template` subcommand for the create command, allowing users to generate configuration file templates. This enhances user experience by providing a starting point for configuration creation.

## Goals

- [ ] Implement `template` subcommand in the create command structure
- [ ] Generate JSON configuration template files
- [ ] Support custom output paths for template generation
- [ ] Provide helpful placeholder content with clear replacement instructions
- [ ] Integrate with existing CLI subcommand architecture

**Estimated Time**: 1-2 hours

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (CLI interface) + Infrastructure (template generation)
**Module Path**: `src/presentation/console/subcommands/create/` + `src/infrastructure/templates/`
**Pattern**: CLI Subcommand + Template Generation

**Dependencies**: Requires Subissue 6 (Template System Integration) to be completed first.

### Module Integration

This subissue extends the CLI subcommand structure established in Subissue 3 (CLI Presentation Layer):

- **Presentation Layer**: Add `template` subcommand to existing create command structure
- **Infrastructure Layer**: Extend existing TemplateManager for configuration template generation
- **No new domain logic**: Template generation is a user convenience feature

## Specifications

### CLI Interface

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

### CLI Implementation

Extend the existing CLI subcommand structure to support template generation:

````rust
// src/presentation/console/subcommands/create/args.rs (extends existing from Subissue 3)
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

    /// Get the template output path, using default if none specified
    pub fn template_output_path(&self) -> Option<PathBuf> {
        match self {
            CreateAction::Template { output_path } => {
                Some(output_path.clone().unwrap_or_else(Self::default_template_path))
            }
            CreateAction::Environment { .. } => None,
        }
    }
}
```### Template Content

JSON configuration template with clear placeholder instructions:

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
````

### Template Generation Implementation

Extend the existing TemplateManager infrastructure:

```rust
// src/infrastructure/templates/config_template.rs
use std::path::Path;
use super::errors::TemplateError;

pub struct ConfigTemplateGenerator;

impl ConfigTemplateGenerator {
    /// Generate a configuration template file at the specified path
    pub fn generate_template(output_path: &Path) -> Result<(), TemplateError> {
        let template_content = Self::get_template_content();

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|source| TemplateError::DirectoryCreation {
                    path: parent.to_string_lossy().to_string(),
                    source,
                })?;
        }

        // Write template file
        std::fs::write(output_path, template_content)
            .map_err(|source| TemplateError::TemplateWrite {
                path: output_path.to_string_lossy().to_string(),
                source,
            })?;

        Ok(())
    }

    /// Get the embedded template content
    fn get_template_content() -> &'static str {
        r#"{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME"
  },
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
    "username": "torrust",
    "port": 22
  }
}"#
    }
}
```

### CLI Command Handler Integration

Update the existing create command handler to support template generation:

````rust
// src/presentation/console/subcommands/create/handler.rs (extends existing from Subissue 3)
use super::{CreateArgs, CreateAction};
use crate::infrastructure::templates::ConfigTemplateGenerator;

pub async fn handle_create_command(args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    match &args.action {
        CreateAction::Template { output_path } => {
            let template_path = output_path.clone()
                .unwrap_or_else(|| CreateAction::default_template_path());
            handle_template_generation(template_path).await
        }
        CreateAction::Environment { env_file } => {
            handle_environment_creation(env_file.clone()).await
        }
    }
}

async fn handle_template_generation(output_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating configuration template...");

    ConfigTemplateGenerator::generate_template(&output_path)?;

    println!("✅ Configuration template generated: {}", output_path.display());
    println!();
    println!("Next steps:");
    println!("1. Edit the template file and replace placeholder values:");
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name");
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");
    println!("2. Create the environment: torrust-tracker-deployer create environment --env-file {}", output_path.display());

    Ok(())
}

async fn handle_environment_creation(config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Existing environment creation logic from other subissues
    todo!("Environment creation logic - implemented in previous subissues")
}
```## Error Handling

Template generation errors should be integrated with existing error handling patterns:

```rust
// Extend existing TemplateError enum (from Subissue 6)
#[derive(Debug, Error)]
pub enum TemplateError {
    // ... existing variants ...

    #[error("Failed to write template to {path}")]
    TemplateWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create directory {path}")]
    DirectoryCreation {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl TemplateError {
    pub fn help(&self) -> &'static str {
        match self {
            Self::TemplateWrite { .. } => {
                "Template Write Failed - Detailed Troubleshooting:

1. Check if you have write permissions to the target directory
2. Verify the target directory exists or can be created
3. Check available disk space
4. Ensure the target path is not a directory

For more information, see the file system troubleshooting guide."
            }

            Self::DirectoryCreation { .. } => {
                "Directory Creation Failed - Detailed Troubleshooting:

1. Check if you have write permissions to the parent directory
2. Verify the path is valid and accessible
3. Check available disk space
4. Ensure no conflicting files exist at the target path

For more information, see the file system troubleshooting guide."
            }

            // ... existing help methods ...
        }
    }
}
````

## Unit Tests

```rust
// src/infrastructure/templates/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn it_should_generate_template_in_current_directory() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("environment-template.json");

        ConfigTemplateGenerator::generate_template(&template_path).unwrap();

        assert!(template_path.exists());
        let content = fs::read_to_string(&template_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(content.contains("REPLACE_WITH_SSH_PRIVATE_KEY_PATH"));
        assert!(content.contains("torrust"));
    }

    #[test]
    fn it_should_generate_template_in_custom_directory() {
        let temp_dir = TempDir::new().unwrap();
        let custom_dir = temp_dir.path().join("config");
        let template_path = custom_dir.join("my-env.json");

        ConfigTemplateGenerator::generate_template(&template_path).unwrap();

        assert!(custom_dir.exists());
        assert!(template_path.exists());
    }

    #[test]
    fn it_should_create_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("template.json");

        ConfigTemplateGenerator::generate_template(&nested_path).unwrap();

        assert!(nested_path.parent().unwrap().exists());
        assert!(nested_path.exists());
    }

    #[test]
    fn it_should_generate_valid_json_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test-template.json");

        ConfigTemplateGenerator::generate_template(&template_path).unwrap();

        let content = fs::read_to_string(&template_path).unwrap();

        // Verify it's valid JSON structure (even with placeholder values)
        let json_value: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json_value["environment"]["name"].is_string());
        assert!(json_value["ssh_credentials"]["private_key_path"].is_string());
        assert!(json_value["ssh_credentials"]["public_key_path"].is_string());
        assert!(json_value["ssh_credentials"]["username"].is_string());
        assert!(json_value["ssh_credentials"]["port"].is_number());
    }

    #[test]
    fn it_should_handle_file_write_errors() {
        // Test writing to read-only directory (Unix only)
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().unwrap();
            let readonly_dir = temp_dir.path().join("readonly");
            fs::create_dir(&readonly_dir).unwrap();

            // Make directory read-only
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444);
            fs::set_permissions(&readonly_dir, perms).unwrap();

            let template_path = readonly_dir.join("template.json");
            let result = ConfigTemplateGenerator::generate_template(&template_path);

            assert!(result.is_err());
            if let Err(TemplateError::TemplateWrite { .. }) = result {
                // Expected error
            } else {
                panic!("Expected TemplateWrite error");
            }
        }
    }
}
```

## Integration with CLI

````rust
// src/presentation/console/subcommands/create/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_handle_template_subcommand_with_default_path() {
        let action = CreateAction::Template { output_path: None };

        let template_path = action.template_output_path().unwrap();
        assert_eq!(template_path, PathBuf::from("environment-template.json"));
    }

    #[test]
    fn it_should_handle_template_subcommand_with_custom_path() {
        let custom_path = PathBuf::from("./config/my-template.json");
        let action = CreateAction::Template {
            output_path: Some(custom_path.clone())
        };

        let template_path = action.template_output_path().unwrap();
        assert_eq!(template_path, custom_path);
    }

    #[test]
    fn it_should_return_none_for_environment_action() {
        let action = CreateAction::Environment {
            env_file: PathBuf::from("config.json")
        };

        let template_path = action.template_output_path();
        assert!(template_path.is_none());
    }
}
```## Acceptance Criteria

- [ ] `torrust-tracker-deployer create template` generates `environment-template.json` in current directory
- [ ] `torrust-tracker-deployer create template ./config/env.json` generates template at specified path
- [ ] Generated template contains valid JSON structure with placeholder values
- [ ] Generated template includes all required fields from JSON schema
- [ ] Template generation creates parent directories if they don't exist
- [ ] Clear success message with next steps instructions after template generation
- [ ] Proper error handling for file write failures with actionable help messages
- [ ] Template subcommand integrates cleanly with existing create command structure
- [ ] Help system works correctly: `torrust-tracker-deployer create template --help`

## Future Enhancements

This implementation provides the foundation for future template features:

- **TOML template support**: Future enhancement for human-readable TOML templates
- **Interactive template generation**: Guided template creation with prompts
- **Custom template variables**: User-defined placeholder variables
- **Template validation**: Pre-validation of generated templates against schema

These enhancements will be implemented as separate issues after the core functionality is complete.
````
