# CLI Presentation Layer

**Issue**: [#37](https://github.com/torrust/torrust-tracker-deployer/issues/37)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36) - Application Layer Command
**Related**: [Roadmap Task 1.5](../roadmap.md), [Presentation Layer Architecture](../codebase-architecture.md#presentation-layer)

## Overview

Implement the CLI presentation layer for the create command, handling Figment integration for configuration file parsing, argument processing, and user interaction. This layer serves as the delivery mechanism that converts user input into clean domain objects for the application layer.

**Key Points**:

- Add `--working-dir` flag to main CLI for production use (not just tests)
- Figment stays in presentation layer as delivery mechanism
- All errors use tiered help system with `.help()` methods

## Goals

- [ ] Create `create` subcommand in **presentation layer** (`src/presentation/console/subcommands/create/`)
  - [ ] **Figment integration for configuration file parsing** (delivery mechanism concern)
  - [ ] Argument parsing (--env-file, --generate-template [path])
  - [ ] Configuration file loading using Figment (JSON format, TOML support added in separate issue)
  - [ ] Conversion from raw file data to clean domain objects
  - [ ] Calling application layer CreateCommand with clean domain objects
  - [ ] User feedback and progress indication
  - [ ] Error message presentation with helpful context using tiered help system
- [ ] Add explicit presentation error enums
  - [ ] `CreateSubcommandError` for CLI-specific errors (file not found, parsing errors)
  - [ ] Error conversion from application layer errors to user-friendly messages
- [ ] Add command help documentation
- [ ] Integration tests for CLI interface with temporary directories
- [ ] Unit tests for argument parsing, Figment integration, and error presentation

**Estimated Time**: 3-4 hours

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation Layer (`src/presentation/console/subcommands/create/`)
**Pattern**: CLI Subcommand + Figment Integration + Error Conversion
**Dependencies**: Application layer, Domain layer, Infrastructure layer

### Module Structure

```text
src/presentation/console/subcommands/create/
├── mod.rs                    # Module exports and documentation
├── subcommand.rs             # CreateSubcommand implementation
├── args.rs                   # CLI argument definitions
├── config_loader.rs          # Figment integration for config parsing
├── errors.rs                 # Presentation-specific error types
└── tests/                    # Test module
    ├── mod.rs
    ├── integration.rs        # CLI integration tests
    └── fixtures.rs           # Test fixtures and helpers
```

## Specifications

### CLI Argument Structure

```rust
// src/presentation/console/subcommands/create/args.rs
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
        /// If not provided, creates template in current directory
        #[arg(value_name = "PATH")]
        output_path: Option<PathBuf>,
    },
}

impl CreateArgs {
    /// Get the configuration file path if creating an environment
    pub fn config_file_path(&self) -> Option<&PathBuf> {
        match &self.action {
            CreateAction::Environment { env_file } => Some(env_file),
            CreateAction::Template { .. } => None,
        }
    }

    /// Get the template output path if generating a template
    pub fn template_output_path(&self) -> Option<&PathBuf> {
        match &self.action {
            CreateAction::Environment { .. } => None,
            CreateAction::Template { output_path } => output_path.as_ref(),
        }
    }

    /// Check if this is a template generation request
    pub fn is_template_generation(&self) -> bool {
        matches!(self.action, CreateAction::Template { .. })
    }
}
```

### Configuration Loader with Figment

```rust
// src/presentation/console/subcommands/create/config_loader.rs
use figment::{Figment, providers::{Format, Json, Serialized}};
use std::path::Path;
use crate::domain::config::EnvironmentConfig;
use super::errors::CreateSubcommandError;

/// Configuration loader using Figment for file parsing
///
/// This handles the delivery mechanism concern of parsing configuration
/// files and converting them to clean domain objects. Figment integration
/// stays in the presentation layer.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self
    }

    /// Load environment configuration from file using Figment
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Ok(EnvironmentConfig)` - Successfully parsed configuration
    /// * `Err(CreateSubcommandError)` - Parsing or validation failed
    pub async fn load_from_file(&self, config_path: &Path) -> Result<EnvironmentConfig, CreateSubcommandError> {
        // Verify file exists before attempting to parse
        if !config_path.exists() {
            return Err(CreateSubcommandError::ConfigFileNotFound {
                path: config_path.to_path_buf(),
            });
        }

        // Determine file format from extension
        let file_format = self.detect_file_format(config_path)?;

        // Load configuration using Figment
        let config = match file_format {
            ConfigFormat::Json => self.load_json_config(config_path).await?,
            // Future: ConfigFormat::Toml => self.load_toml_config(config_path).await?,
        };

        // Validate the configuration using domain rules
        config.validate()
            .map_err(CreateSubcommandError::ConfigValidationFailed)?;

        Ok(config)
    }

    /// Load JSON configuration using Figment
    async fn load_json_config(&self, config_path: &Path) -> Result<EnvironmentConfig, CreateSubcommandError> {
        let figment = Figment::new()
            .merge(Serialized::defaults(EnvironmentConfig::default()))
            .merge(Json::file(config_path));

        figment.extract()
            .map_err(|source| CreateSubcommandError::ConfigParsingFailed {
                path: config_path.to_path_buf(),
                format: ConfigFormat::Json,
                source: Box::new(source),
            })
    }

    /// Detect configuration file format from file extension
    fn detect_file_format(&self, config_path: &Path) -> Result<ConfigFormat, CreateSubcommandError> {
        let extension = config_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| CreateSubcommandError::UnsupportedFileFormat {
                path: config_path.to_path_buf(),
                reason: "No file extension found".to_string(),
            })?;

        match extension.to_lowercase().as_str() {
            "json" => Ok(ConfigFormat::Json),
            _ => Err(CreateSubcommandError::UnsupportedFileFormat {
                path: config_path.to_path_buf(),
                reason: format!("Unsupported file extension: {}", extension),
            }),
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported configuration file formats
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Json,
    // Future: Toml,
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
        }
    }
}
```

### Create Subcommand Implementation

```rust
// src/presentation/console/subcommands/create/subcommand.rs
use std::sync::Arc;
use crate::application::commands::create::CreateCommand;
use crate::infrastructure::templates::{TemplateProvider, TemplateType};
use super::args::{CreateArgs, CreateAction};
use super::config_loader::ConfigLoader;
use super::errors::CreateSubcommandError;

/// CLI subcommand for creating environments and generating templates
///
/// This handles the presentation layer concerns of CLI interaction,
/// configuration file parsing with Figment, and user feedback.
pub struct CreateSubcommand {
    create_command: CreateCommand,
    template_provider: TemplateProvider,
    config_loader: ConfigLoader,
}

impl CreateSubcommand {
    /// Create a new create subcommand with dependencies
    pub fn new(
        create_command: CreateCommand,
        template_provider: TemplateProvider,
    ) -> Self {
        Self {
            create_command,
            template_provider,
            config_loader: ConfigLoader::new(),
        }
    }

    /// Execute the create subcommand
    ///
    /// # Arguments
    /// * `args` - Parsed CLI arguments
    ///
    /// # Returns
    /// * `Ok(())` - Command executed successfully
    /// * `Err(CreateSubcommandError)` - Command execution failed
    pub async fn execute(&self, args: CreateArgs) -> Result<(), CreateSubcommandError> {
        match args.action {
            CreateAction::Environment { env_file } => {
                self.create_environment(&env_file).await
            }
            CreateAction::Template { output_path } => {
                self.generate_template(output_path.as_deref()).await
            }
        }
    }

    /// Create environment from configuration file
    async fn create_environment(&self, config_path: &std::path::Path) -> Result<(), CreateSubcommandError> {
        println!("Loading configuration from: {}", config_path.display());

        // Load and parse configuration using Figment
        let config = self.config_loader.load_from_file(config_path).await?;

        println!("Creating environment: {}", config.environment_name());

        // Pass clean domain object to application layer
        let environment = self.create_command.execute(config).await
            .map_err(CreateSubcommandError::CommandExecutionFailed)?;

        println!("✅ Environment '{}' created successfully!", environment.name().as_str());
        println!("   Data directory: {}", environment.data_dir().display());
        println!("   Traces directory: {}", environment.data_dir().join("traces").display());

        println!("\nNext steps:");
        println!("  1. Review the environment configuration");
        println!("  2. Provision infrastructure: torrust-tracker-deployer provision {}", environment.name().as_str());

        Ok(())
    }

    /// Generate template configuration file
    async fn generate_template(&self, output_path: Option<&std::path::Path>) -> Result<(), CreateSubcommandError> {
        let template_path = match output_path {
            Some(path) => {
                println!("Generating template at: {}", path.display());
                self.template_provider.generate_template(TemplateType::Json, path).await
                    .map_err(CreateSubcommandError::TemplateGenerationFailed)?;
                path.to_path_buf()
            }
            None => {
                let current_dir = std::env::current_dir()
                    .map_err(CreateSubcommandError::CurrentDirectoryNotAccessible)?;
                println!("Generating template in current directory");
                self.template_provider.generate_template_in_directory(TemplateType::Json, &current_dir).await
                    .map_err(CreateSubcommandError::TemplateGenerationFailed)?
            }
        };

        println!("✅ Template generated successfully: {}", template_path.display());
        println!("\nNext steps:");
        println!("  1. Edit the template file and replace placeholder values");
        println!("  2. Create environment: torrust-tracker-deployer create environment --env-file {}", template_path.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::application::commands::create::tests::CreateCommandTestBuilder;

    #[tokio::test]
    async fn it_should_create_environment_from_valid_config() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_base_directory(temp_dir.path())
            .build();

        let template_provider = TemplateProvider::new();
        let subcommand = CreateSubcommand::new(command, template_provider);

        let config_path = create_test_config_file(&temp_dir);
        let args = CreateArgs {
            action: CreateAction::Environment { env_file: config_path },
        };

        // Act
        let result = subcommand.execute(args).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_should_generate_template_in_current_directory() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_base_directory(temp_dir.path())
            .build();

        let template_provider = TemplateProvider::new();
        let subcommand = CreateSubcommand::new(command, template_provider);

        let args = CreateArgs {
            action: CreateAction::Template { output_path: None },
        };

        // Act - change to temp directory for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let result = subcommand.execute(args).await;
        std::env::set_current_dir(original_dir).unwrap();

        // Assert
        assert!(result.is_ok());
        assert!(temp_dir.path().join("environment-template.json").exists());
    }

    fn create_test_config_file(temp_dir: &TempDir) -> std::path::PathBuf {
        let config_content = r#"{
            "environment": {"name": "test-env"},
            "ssh_credentials": {
                "private_key_path": "/tmp/test_key",
                "public_key_path": "/tmp/test_key.pub"
            }
        }"#;

        let config_path = temp_dir.path().join("test-config.json");
        fs::write(&config_path, config_content).unwrap();

        // Create dummy SSH key files
        fs::write("/tmp/test_key", "dummy_private_key").unwrap();
        fs::write("/tmp/test_key.pub", "dummy_public_key").unwrap();

        config_path
    }
}
```

### Presentation Layer Error Types

```rust
// src/presentation/console/subcommands/create/errors.rs
use thiserror::Error;
use std::path::PathBuf;
use crate::domain::config::ConfigValidationError;
use crate::application::commands::create::CreateCommandError;
use crate::infrastructure::templates::TemplateError;
use super::config_loader::ConfigFormat;

/// Errors specific to the create subcommand presentation layer
///
/// These errors handle CLI-specific concerns like file parsing,
/// argument validation, and user interaction failures.
#[derive(Debug, Error)]
pub enum CreateSubcommandError {
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound { path: PathBuf },

    #[error("Unsupported file format: {path} - {reason}")]
    UnsupportedFileFormat { path: PathBuf, reason: String },

    #[error("Failed to parse {format} configuration file: {path}")]
    ConfigParsingFailed {
        path: PathBuf,
        format: ConfigFormat,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Configuration validation failed")]
    ConfigValidationFailed(#[source] ConfigValidationError),

    #[error("Command execution failed")]
    CommandExecutionFailed(#[source] CreateCommandError),

    #[error("Template generation failed")]
    TemplateGenerationFailed(#[source] TemplateError),

    #[error("Cannot access current directory")]
    CurrentDirectoryNotAccessible(#[source] std::io::Error),
}

impl CreateSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => {
                "Configuration File Not Found - Detailed Troubleshooting:

1. Check if the file path is correct and the file exists
2. Verify file permissions allow reading
3. Use absolute path or correct relative path
4. Generate a template first: torrust-tracker-deployer create template

For more information, see the configuration file documentation."
            }

            Self::UnsupportedFileFormat { .. } => {
                "Unsupported File Format - Detailed Troubleshooting:

1. Use .json extension for JSON configuration files
2. TOML support (.toml) will be added in a future release
3. Check the file extension matches the content format
4. Generate a template: torrust-tracker-deployer create template

For more information, see the supported file formats documentation."
            }

            Self::ConfigParsingFailed { .. } => {
                "Configuration Parsing Failed - Detailed Troubleshooting:

1. Check JSON syntax and format
2. Verify all required fields are present
3. Ensure no trailing commas or syntax errors
4. Use a JSON validator to check the file
5. Generate a fresh template: torrust-tracker-deployer create template

For more information, see the configuration format documentation."
            }

            Self::ConfigValidationFailed(_) => {
                "Configuration Validation Failed - Detailed Troubleshooting:

1. Check that SSH key files exist and are accessible
2. Verify environment name follows naming conventions
3. Ensure all paths are correct and files are readable
4. Review the validation error details above

For more information, see the configuration validation documentation."
            }

            Self::CommandExecutionFailed(_) => {
                "Command Execution Failed - Detailed Troubleshooting:

1. Check the application logs for detailed error information
2. Verify you have write permissions to the data directory
3. Ensure sufficient disk space is available
4. Review the command error details above

For more information, see the command execution documentation."
            }

            Self::TemplateGenerationFailed(_) => {
                "Template Generation Failed - Detailed Troubleshooting:

1. Check write permissions for the output directory
2. Verify sufficient disk space is available
3. Ensure no file exists with the same name
4. Try generating in a different directory

For more information, see the template generation documentation."
            }

            Self::CurrentDirectoryNotAccessible(_) => {
                "Current Directory Not Accessible - Detailed Troubleshooting:

1. Check if the current directory exists
2. Verify read permissions for the current directory
3. Try specifying an explicit output path
4. Change to a different directory and retry

For more information, see the file system documentation."
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: CLI Structure (1 hour)

- [ ] Create `src/presentation/console/subcommands/create/mod.rs` with module documentation
- [ ] Set up module structure with proper exports
- [ ] Create `CreateArgs` and `CreateAction` with clap derive
- [ ] Add argument validation and accessor methods

### Phase 2: Figment Integration (1 hour)

- [ ] Create `ConfigLoader` with Figment integration
- [ ] Implement JSON file parsing with proper error handling
- [ ] Add file format detection based on extension
- [ ] Integrate domain validation after parsing

### Phase 3: Subcommand Implementation (1 hour)

- [ ] Create `CreateSubcommand` with dependency injection
- [ ] Implement environment creation workflow
- [ ] Implement template generation workflow
- [ ] Add user feedback and progress indication

### Phase 4: Error Handling (1 hour)

- [ ] Create `CreateSubcommandError` enum with thiserror
- [ ] Add CLI-specific error variants with proper context
- [ ] Implement tiered help system with detailed troubleshooting
- [ ] Ensure proper error conversion from other layers

### Phase 5: Comprehensive Testing (1-2 hours)

- [ ] Unit tests for argument parsing and validation
- [ ] Integration tests for complete CLI workflows
- [ ] Tests for Figment configuration loading
- [ ] Tests for error handling and user feedback
- [ ] Tests for template generation with various paths

## Acceptance Criteria

- [ ] `torrust-tracker-deployer create environment --env-file config.json` creates environment
- [ ] `torrust-tracker-deployer create template` generates template in current directory
- [ ] `torrust-tracker-deployer create template ./config/env.json` generates template at path
- [ ] Figment integration properly parses JSON configuration files
- [ ] Configuration validation provides clear error messages
- [ ] User feedback shows progress and next steps
- [ ] Error messages follow project guidelines with actionable help
- [ ] CLI help documentation is comprehensive and useful

## Testing Strategy

### Test Categories

1. **CLI Argument Tests**

   - Valid argument combinations are accepted
   - Invalid arguments show appropriate errors
   - Help text is comprehensive and accurate
   - Argument accessor methods work correctly

2. **Figment Integration Tests**

   - JSON configuration files are parsed correctly
   - File format detection works for supported extensions
   - Parsing errors provide clear feedback
   - Domain validation is triggered after parsing

3. **Workflow Tests**

   - Environment creation end-to-end workflow
   - Template generation with various output paths
   - User feedback and progress indication
   - Error recovery and cleanup scenarios

4. **Error Handling Tests**
   - File not found scenarios
   - Permission denied errors
   - Invalid configuration formats
   - Command execution failures

### Integration Testing

- Use `tempfile::TempDir` for isolated filesystem testing
- Mock application layer dependencies for pure CLI testing
- Test complete workflows with real configuration files
- Verify error messages and help text quality

## Related Documentation

- [Presentation Layer Architecture](../codebase-architecture.md#presentation-layer)
- [CLI Subcommand Structure](../codebase-architecture.md#presentation-layer)
- [Error Handling Guidelines](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Figment Documentation](https://docs.rs/figment/)

## Notes

- Figment integration stays in the presentation layer as a delivery mechanism
- The subcommand is completely delivery-specific and handles user interaction
- Error handling provides complete troubleshooting guidance with next steps
- The implementation supports future extension to TOML and other formats
