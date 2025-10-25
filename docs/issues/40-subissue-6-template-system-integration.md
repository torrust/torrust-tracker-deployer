# Template System Integration

**Issue**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Status**: OPTIONAL ENHANCEMENT
**Related**: [Roadmap Task 1.5](../roadmap.md), [Infrastructure Layer Architecture](../codebase-architecture.md#infrastructure-layer)

## Overview

Implement the template system for configuration file generation in the infrastructure layer. This system provides embedded JSON configuration templates that can be generated on-demand, supporting the `--generate-template` functionality with proper error handling and validation.

## Goals

- [ ] Extend existing `TemplateManager` from `src/domain/template/embedded.rs` for configuration templates
  - [ ] **Use existing template infrastructure** - no duplication needed
  - [ ] Add configuration template types to existing embedded template system
  - [ ] Leverage existing rust-embed pattern in `templates/` directory structure
  - [ ] Use existing `TemplateManagerError` for error handling
- [ ] Add configuration templates to existing template structure
  - [ ] Add to existing `templates/` directory following existing patterns
  - [ ] Use existing Tera variable syntax: `{{ variable_name }}` (not `{ { variable_name } }`)
  - [ ] Follow existing template embedding and extraction patterns
- [ ] Add `--generate-template` functionality using existing template infrastructure
- [ ] Add unit tests that integrate with existing template system
  - [ ] Test template generation using existing TemplateManager patterns
  - [ ] Test integration with existing embedded template system
  - [ ] Test template file generation following existing patterns

**Estimated Time**: 1-2 hours

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure Layer (`src/infrastructure/templates/`)
**Pattern**: Template Provider + Embedded Resources + File Operations
**Dependencies**: None (infrastructure concern)

### Module Structure

```text
src/infrastructure/templates/
├── mod.rs                    # Module exports and documentation
├── provider.rs               # Template provider implementation
├── embedded.rs               # Embedded template resources
├── errors.rs                 # Template-specific error types
└── tests/                    # Test module
    ├── mod.rs
    └── integration.rs        # Integration tests with file system
```

## Specifications

### Template Provider Implementation

````rust
// src/infrastructure/templates/provider.rs
use std::path::{Path, PathBuf};
use super::embedded::EmbeddedTemplates;
use super::errors::TemplateError;

/// Provider for configuration templates
///
/// Handles template retrieval from embedded resources and generation
/// of template files on the filesystem.
pub struct TemplateProvider {
    embedded: EmbeddedTemplates,
}

impl TemplateProvider {
    /// Create a new template provider
    pub fn new() -> Self {
        Self {
            embedded: EmbeddedTemplates::new(),
        }
    }

    /// Generate a template file at the specified path
    ///
    /// # Arguments
    /// * `template_type` - Type of template to generate (currently only JSON)
    /// * `output_path` - Path where the template file should be created
    ///
    /// # Returns
    /// * `Ok(())` - Template generated successfully
    /// * `Err(TemplateError)` - Template generation failed
    ///
    /// # Examples
    /// ```rust
    /// let provider = TemplateProvider::new();
    /// provider.generate_template(
    ///     TemplateType::Json,
    ///     Path::new("./environment-template.json")
    /// )?;
    /// ```
    pub async fn generate_template(
        &self,
        template_type: TemplateType,
        output_path: &Path,
    ) -> Result<(), TemplateError> {
        // Get template content from embedded resources
        let template_content = self.embedded.get_template(template_type)
            .ok_or_else(|| TemplateError::TemplateNotFound {
                template_type: template_type.to_string(),
            })?;

        // Validate output path
        self.validate_output_path(output_path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|source| TemplateError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }

        // Write template to file
        tokio::fs::write(output_path, template_content).await
            .map_err(|source| TemplateError::FileWriteFailed {
                path: output_path.to_path_buf(),
                source,
            })?;

        Ok(())
    }

    /// Generate template with default filename in specified directory
    ///
    /// # Arguments
    /// * `template_type` - Type of template to generate
    /// * `directory` - Directory where template should be created
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the generated template file
    /// * `Err(TemplateError)` - Template generation failed
    pub async fn generate_template_in_directory(
        &self,
        template_type: TemplateType,
        directory: &Path,
    ) -> Result<PathBuf, TemplateError> {
        let filename = template_type.default_filename();
        let output_path = directory.join(filename);

        self.generate_template(template_type, &output_path).await?;

        Ok(output_path)
    }

    /// Get template content as string without writing to file
    ///
    /// Useful for testing and programmatic access to templates.
    pub fn get_template_content(&self, template_type: TemplateType) -> Result<&str, TemplateError> {
        self.embedded.get_template(template_type)
            .ok_or_else(|| TemplateError::TemplateNotFound {
                template_type: template_type.to_string(),
            })
    }

    /// List all available template types
    pub fn available_templates(&self) -> Vec<TemplateType> {
        self.embedded.available_templates()
    }

    /// Validate that the output path is suitable for template generation
    fn validate_output_path(&self, path: &Path) -> Result<(), TemplateError> {
        // Check if path already exists and is a file
        if path.exists() && !path.is_file() {
            return Err(TemplateError::InvalidOutputPath {
                path: path.to_path_buf(),
                reason: "Path exists but is not a file".to_string(),
            });
        }

        // Validate file extension matches template type
        if let Some(extension) = path.extension() {
            if extension != "json" {
                return Err(TemplateError::InvalidOutputPath {
                    path: path.to_path_buf(),
                    reason: format!(
                        "File extension '{}' does not match JSON template type",
                        extension.to_string_lossy()
                    ),
                });
            }
        } else {
            return Err(TemplateError::InvalidOutputPath {
                path: path.to_path_buf(),
                reason: "No file extension specified".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for TemplateProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported template types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    Json,
    // Future: Toml, Yaml
}

impl TemplateType {
    /// Get the default filename for this template type
    pub fn default_filename(&self) -> &'static str {
        match self {
            Self::Json => "environment-template.json",
        }
    }

    /// Get the file extension for this template type
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = TemplateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            _ => Err(TemplateError::UnsupportedTemplateType {
                requested_type: s.to_string(),
                supported_types: vec!["json".to_string()],
            }),
        }
    }
}
````

### Embedded Templates

```rust
// src/infrastructure/templates/embedded.rs
use super::provider::TemplateType;

/// Container for embedded template resources
///
/// Templates are embedded in the binary at compile time to ensure
/// they're always available without external dependencies.
pub struct EmbeddedTemplates;

impl EmbeddedTemplates {
    /// Create a new embedded templates container
    pub fn new() -> Self {
        Self
    }

    /// Get template content for the specified type
    pub fn get_template(&self, template_type: TemplateType) -> Option<&'static str> {
        match template_type {
            TemplateType::Json => Some(JSON_TEMPLATE),
        }
    }

    /// Get list of all available template types
    pub fn available_templates(&self) -> Vec<TemplateType> {
        vec![TemplateType::Json]
    }
}

/// JSON configuration template
///
/// This template provides a complete example of the configuration format
/// with placeholder values that users can replace with their actual values.
const JSON_TEMPLATE: &str = r#"{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME"
  },
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
    "username": "torrust",
    "port": 22
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn it_should_provide_valid_json_template() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Verify the template is valid JSON
        let _: serde_json::Value = serde_json::from_str(template)
            .expect("JSON template should be valid JSON");
    }

    #[test]
    fn it_should_contain_required_placeholder_fields() {
        let embedded = EmbeddedTemplates::new();
        let template = embedded.get_template(TemplateType::Json).unwrap();

        // Verify required placeholders are present
        assert!(template.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(template.contains("REPLACE_WITH_SSH_PRIVATE_KEY_PATH"));
        assert!(template.contains("REPLACE_WITH_SSH_PUBLIC_KEY_PATH"));

        // Verify default values are present
        assert!(template.contains("\"username\": \"torrust\""));
        assert!(template.contains("\"port\": 22"));
    }

    #[test]
    fn it_should_list_available_templates() {
        let embedded = EmbeddedTemplates::new();
        let templates = embedded.available_templates();

        assert_eq!(templates.len(), 1);
        assert!(templates.contains(&TemplateType::Json));
    }
}
```

### Template Error Types

```rust
// src/infrastructure/templates/errors.rs
use thiserror::Error;
use std::path::PathBuf;

/// Errors that can occur during template operations
///
/// These errors represent infrastructure-level failures in template
/// handling and provide structured context for troubleshooting.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template not found: {template_type}")]
    TemplateNotFound { template_type: String },

    #[error("Unsupported template type: {requested_type}")]
    UnsupportedTemplateType {
        requested_type: String,
        supported_types: Vec<String>,
    },

    #[error("Invalid output path: {path} - {reason}")]
    InvalidOutputPath { path: PathBuf, reason: String },

    #[error("Failed to create directory: {path}")]
    DirectoryCreationFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write template file: {path}")]
    FileWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Template validation failed: {template_type}")]
    TemplateValidationFailed {
        template_type: String,
        #[source]
        source: serde_json::Error,
    },
}

impl TemplateError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::TemplateNotFound { .. } => {
                "Template Not Found - Detailed Troubleshooting:

1. Check if the template type is supported
2. Verify the application binary includes embedded templates
3. Try regenerating templates if they should be available
4. Report issue if template should be available but is missing

For more information, see the template documentation."
            }

            Self::UnsupportedTemplateType { .. } => {
                "Unsupported Template Type - Detailed Troubleshooting:

1. Use 'json' for JSON templates (currently supported)
2. TOML support will be added in a future release
3. Verify you are using the correct template type format

For more information, see the template format documentation."
            }

            Self::InvalidOutputPath { .. } => {
                "Invalid Output Path - Detailed Troubleshooting:

1. Ensure the path points to a file (not directory)
2. Use correct file extension (.json for JSON templates)
3. Verify parent directory exists or can be created
4. Check write permissions for the target location

For more information, see the file system documentation."
            }

            Self::DirectoryCreationFailed { .. } => {
                "Directory Creation Failed - Detailed Troubleshooting:

1. Check write permissions for the parent directory
2. Verify disk space is available: df -h
3. Ensure no file exists with the same name as the directory
4. Check path length limits on your system

For more information, see the filesystem troubleshooting guide."
            }

            Self::FileWriteFailed { .. } => {
                "Template File Write Failed - Detailed Troubleshooting:

1. Check write permissions for the target file and directory
2. Verify disk space is available: df -h
3. Ensure the file is not open in another application
4. Check if antivirus software is blocking file creation

For more information, see the file operations documentation."
            }

            Self::TemplateValidationFailed { .. } => {
                "Template Validation Failed - Detailed Troubleshooting:

1. This indicates a bug in the embedded templates
2. Report this issue with full error details
3. Use --generate-template to create a fresh template
4. Check for application updates

This is likely a software bug that needs to be reported."
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Template Infrastructure (1 hour)

- [ ] Create `src/infrastructure/templates/mod.rs` with module documentation
- [ ] Set up module structure with proper exports
- [ ] Create `TemplateType` enum with JSON support
- [ ] Add basic `TemplateProvider` struct

### Phase 2: Embedded Templates (1 hour)

- [ ] Create `EmbeddedTemplates` struct with compile-time embedded resources
- [ ] Add JSON template with proper placeholders and default values
- [ ] Implement template retrieval and validation
- [ ] Add unit tests for template content validation

### Phase 3: File Operations (1 hour)

- [ ] Implement `generate_template` method with async file operations
- [ ] Add output path validation with proper error handling
- [ ] Implement directory creation for template files
- [ ] Add support for default filenames in directories

### Phase 4: Error Handling (30 minutes)

- [ ] Create `TemplateError` enum with thiserror
- [ ] Add structured error context for all failure scenarios
- [ ] Implement tiered help system with detailed troubleshooting
- [ ] Ensure proper error chaining for I/O operations

### Phase 5: Comprehensive Testing (1-2 hours)

- [ ] Unit tests for template provider functionality
- [ ] Integration tests with temporary directories
- [ ] Tests for path validation and error scenarios
- [ ] Tests for embedded template content and format
- [ ] Error handling tests with various failure modes

## Acceptance Criteria

- [ ] `TemplateProvider` that generates JSON configuration templates
- [ ] Embedded templates stored in the binary (no external dependencies)
- [ ] Support for `--generate-template` with optional output path
- [ ] Template validation ensures valid JSON format and required fields
- [ ] Proper error handling with structured context and actionable messages
- [ ] Tiered help system for all template-related errors
- [ ] Comprehensive test coverage with temporary file operations
- [ ] Extensible architecture for future template formats (TOML, YAML)

## Testing Strategy

### Test Categories

1. **Template Content Tests**

   - Embedded templates are valid JSON
   - Required placeholder fields are present
   - Default values are correctly set
   - Template structure matches expected schema

2. **File Operation Tests**

   - Template generation in current directory
   - Template generation with custom path
   - Directory creation for nested paths
   - Overwrite protection and validation

3. **Path Validation Tests**

   - Valid file paths are accepted
   - Invalid paths are rejected with clear errors
   - File extension validation
   - Directory vs file path handling

4. **Error Handling Tests**
   - Permission denied scenarios
   - Disk space exhaustion simulation
   - Invalid template type handling
   - Path validation failures

### Integration Testing

- Use `tempfile::TempDir` for isolated filesystem testing
- Test complete template generation workflow
- Verify generated templates can be parsed by configuration system
- Test error recovery and cleanup scenarios

## Related Documentation

- [Infrastructure Layer Architecture](../codebase-architecture.md#infrastructure-layer)
- [Template System Design](../codebase-architecture.md#template-system)
- [Error Handling Guidelines](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)

## Notes

- Templates are embedded at compile time for reliability and portability
- The system is designed for easy extension to support TOML and YAML formats
- Error handling provides complete troubleshooting guidance for users
- Integration tests ensure templates work with the actual configuration system
