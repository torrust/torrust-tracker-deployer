# Template System Integration (v2 - Struct-Based Generation)

**Issue**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40)  
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command  
**Status**: REVISED APPROACH  
**Related**: [Roadmap Task 1.5](../roadmap.md), [Original Issue v1](./40-subissue-6-template-system-integration.md)  
**Supersedes**: [v1 - Embedded Template Approach](./40-subissue-6-template-system-integration.md)

## 🔄 Revision History

**v2 (Current)**: Struct-based JSON generation - no template files needed  
**v1 (Deprecated)**: Embedded template approach (led to confusion, see analysis below)

## Overview

Implement configuration template generation for the `create template` command using **struct-based JSON generation**. This approach generates the template directly from the `EnvironmentCreationConfig` Rust struct, eliminating the need for separate template files and ensuring type safety.

## Goals

- [ ] Add `template()` method to `EnvironmentCreationConfig` that returns a struct with placeholder values
- [ ] Add `generate_template_file()` method that serializes the template struct to JSON
- [ ] Implement error handling with actionable messages
- [ ] Add comprehensive unit tests for template generation
- [ ] Integrate with CLI `create template` subcommand (Issue #41)

**Estimated Time**: 1-2 hours

## 🎯 Why Struct-Based Generation?

### The Problem with Template Files

The original approach (v1) suggested using the existing `TemplateManager` system or creating embedded template files. This led to confusion because:

1. **Duplication**: Template file structure would duplicate the `EnvironmentCreationConfig` struct definition
2. **Manual Sync**: Changes to the struct require manual updates to template files
3. **Not Type-Safe**: Template files are strings that can become out of sync with code
4. **Wrong System**: The existing `TemplateManager` is for **infrastructure templates** (Ansible, OpenTofu), not **user configuration templates**

### The Solution: Generate from Struct

Since the template structure **exactly matches** `EnvironmentCreationConfig`, we should generate it programmatically:

```rust
// The struct definition IS the template structure
impl EnvironmentCreationConfig {
    pub fn template() -> Self {
        Self {
            environment: EnvironmentSection {
                name: "REPLACE_WITH_ENVIRONMENT_NAME".to_string(),
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: "REPLACE_WITH_SSH_PRIVATE_KEY_PATH".to_string(),
                public_key_path: "REPLACE_WITH_SSH_PUBLIC_KEY_PATH".to_string(),
                username: "torrust".to_string(), // sensible default
                port: 22, // sensible default
            },
        }
    }
}
```

**Benefits:**

- ✅ **Type-Safe**: Compiler guarantees structure is valid
- ✅ **Zero Duplication**: No separate template file to maintain
- ✅ **Auto-Synced**: Adding/removing fields automatically updates template
- ✅ **Guaranteed Valid JSON**: serde serialization handles formatting
- ✅ **Clear Defaults**: Optional fields get defaults, required get placeholders

## 🏗️ Architecture Requirements

**DDD Layer**: Domain Layer (`src/domain/config/`)  
**Pattern**: Struct Factory Method + JSON Serialization  
**Dependencies**: Only `serde` and `serde_json` (already in use)

### Module Integration

```text
src/domain/config/
├── mod.rs
├── environment_config.rs    # ← Add template() method here
├── ssh_credentials_config.rs
└── errors.rs                # May need TemplateGenerationError
```

## 📋 Understanding Existing Template Systems

### Context: Two Different Template Systems

The project has **two types of templates** that serve different purposes:

#### 1. Infrastructure Templates (Existing in `templates/`)

**Purpose**: Configuration files used by deployment tools (Ansible, OpenTofu, etc.)  
**Location**: `templates/` folder, embedded with `rust-embed`  
**System**: `TemplateManager` in `src/domain/template/embedded.rs`

**Two categories:**

**A) Static Templates** (No variables - used as-is):

- `templates/ansible/ansible.cfg`
- `templates/ansible/install-docker.yml`
- `templates/tofu/lxd/main.tf`
- Many others

**How they work:**

1. Embedded at compile time with `#[derive(RustEmbed)]`
2. `TemplateManager::get_template_path()` extracts to filesystem
3. External tools use them directly

**B) Tera Templates** (With `{{ variables }}` - need rendering):

- `templates/ansible/inventory.yml.tera`
- `templates/tofu/lxd/cloud-init.yml.tera`
- `templates/tofu/lxd/variables.tfvars.tera`

**How they work:**

1. Embedded at compile time
2. Extracted to filesystem
3. Separate renderer (e.g., `InventoryTemplateRenderer`) loads `.tera` file
4. Tera engine substitutes variables
5. Produces final file (e.g., `inventory.yml`)

**Key Point**: Both static and Tera templates use the **same `TemplateManager` infrastructure**. The difference is whether they need Tera rendering after extraction.

#### 2. Configuration Template (This Issue - NEW)

**Purpose**: Help users bootstrap their first `environment.json` config file  
**Audience**: End users creating their deployment configuration  
**Use Case**: One-time generation, then user edits manually  
**Not for Deployment**: Used BEFORE deployment to create user config

**This is fundamentally different** - it's a file users **edit themselves**, not something the application renders during deployment.

### Why Not Use Existing TemplateManager?

The existing `TemplateManager` system is designed for:

- Files that are part of the deployment infrastructure
- Files used by external tools (Ansible, OpenTofu)
- Files that may need variable substitution at deployment time

The configuration template is:

- A user-facing helper to bootstrap config files
- Not part of the deployment infrastructure
- Generated once and then manually edited by users
- Structure directly maps to a Rust struct

**Using embedded templates would:**

- ❌ Create unnecessary duplication
- ❌ Require maintaining template files separate from struct
- ❌ Be overkill for a simple JSON structure
- ❌ Mix deployment concerns with user configuration concerns

## Specifications

### Implementation

#### Add Template Methods to `EnvironmentCreationConfig`

````rust
// src/domain/config/environment_config.rs

use std::path::Path;
use tokio::fs;

impl EnvironmentCreationConfig {
    /// Generate a template configuration with placeholder values
    ///
    /// This creates a new configuration with:
    /// - Placeholder strings for required fields (REPLACE_WITH_*)
    /// - Sensible defaults for optional fields
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::config::EnvironmentCreationConfig;
    ///
    /// let template = EnvironmentCreationConfig::template();
    /// assert_eq!(template.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
    /// assert_eq!(template.ssh_credentials.username, "torrust"); // default
    /// assert_eq!(template.ssh_credentials.port, 22); // default
    /// ```
    #[must_use]
    pub fn template() -> Self {
        Self {
            environment: EnvironmentSection {
                name: "REPLACE_WITH_ENVIRONMENT_NAME".to_string(),
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: "REPLACE_WITH_SSH_PRIVATE_KEY_PATH".to_string(),
                public_key_path: "REPLACE_WITH_SSH_PUBLIC_KEY_PATH".to_string(),
                username: "torrust".to_string(),
                port: 22,
            },
        }
    }

    /// Generate template JSON file at the specified path
    ///
    /// Creates a pretty-printed JSON configuration template file with
    /// placeholder values that users can replace with their actual values.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the template file should be created
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directory cannot be created
    /// - File cannot be written due to permissions or I/O errors
    /// - JSON serialization fails (should never happen with valid struct)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::domain::config::EnvironmentCreationConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// EnvironmentCreationConfig::generate_template_file(
    ///     Path::new("./environment-template.json")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_template_file(path: &Path) -> Result<(), TemplateGenerationError> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|source| TemplateGenerationError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    source,
                })?;
        }

        // Generate template struct
        let template = Self::template();

        // Serialize to pretty JSON
        let json = serde_json::to_string_pretty(&template)
            .map_err(|source| TemplateGenerationError::SerializationFailed { source })?;

        // Write to file
        fs::write(path, json)
            .await
            .map_err(|source| TemplateGenerationError::FileWriteFailed {
                path: path.to_path_buf(),
                source,
            })?;

        Ok(())
    }

    /// Generate template JSON file with default filename in specified directory
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory where template should be created
    ///
    /// # Returns
    ///
    /// Path to the generated template file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::domain::config::EnvironmentCreationConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let template_path = EnvironmentCreationConfig::generate_template_in_directory(
    ///     Path::new("./configs")
    /// ).await?;
    /// println!("Template created at: {}", template_path.display());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_template_in_directory(
        directory: &Path,
    ) -> Result<PathBuf, TemplateGenerationError> {
        const DEFAULT_TEMPLATE_FILENAME: &str = "environment-template.json";
        let template_path = directory.join(DEFAULT_TEMPLATE_FILENAME);
        Self::generate_template_file(&template_path).await?;
        Ok(template_path)
    }
}
````

#### Error Types

```rust
// src/domain/config/errors.rs

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateGenerationError {
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

    #[error("Failed to serialize template to JSON")]
    SerializationFailed {
        #[source]
        source: serde_json::Error,
    },
}

impl TemplateGenerationError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::DirectoryCreationFailed { .. } => {
                "Failed to create directory for template file.

Troubleshooting steps:
1. Check that you have write permissions for the target directory
2. Verify that the parent directory exists and is accessible
3. Check available disk space: df -h (Unix) or wmic logicaldisk (Windows)
4. Ensure the path is valid and doesn't contain invalid characters

If the problem persists, try specifying a different output directory."
            }
            Self::FileWriteFailed { .. } => {
                "Failed to write template file to disk.

Troubleshooting steps:
1. Check write permissions for the target file and directory
2. Verify available disk space
3. Ensure no other process has the file open or locked
4. Check that the filename is valid for your filesystem

If the problem persists, try a different filename or directory."
            }
            Self::SerializationFailed { .. } => {
                "Failed to serialize configuration template to JSON.

This is an internal error that should not occur. The template structure
should always be valid for JSON serialization.

If you see this error, please report it as a bug with:
- Full error message
- Version of the application
- Operating system and version"
            }
        }
    }
}
```

### Generated Template Output

When users run the command, they'll get:

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

Users can then:

1. Replace `REPLACE_WITH_*` placeholders with actual values
2. Modify defaults (`username`, `port`) if needed
3. Save and use with `torrust-tracker-deployer create environment -f environment.json`

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod template_generation_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_template_with_placeholders() {
        let template = EnvironmentCreationConfig::template();

        assert_eq!(template.environment.name, "REPLACE_WITH_ENVIRONMENT_NAME");
        assert_eq!(
            template.ssh_credentials.private_key_path,
            "REPLACE_WITH_SSH_PRIVATE_KEY_PATH"
        );
        assert_eq!(
            template.ssh_credentials.public_key_path,
            "REPLACE_WITH_SSH_PUBLIC_KEY_PATH"
        );
    }

    #[test]
    fn it_should_create_template_with_sensible_defaults() {
        let template = EnvironmentCreationConfig::template();

        assert_eq!(template.ssh_credentials.username, "torrust");
        assert_eq!(template.ssh_credentials.port, 22);
    }

    #[test]
    fn it_should_serialize_template_to_valid_json() {
        let template = EnvironmentCreationConfig::template();
        let json = serde_json::to_string_pretty(&template).unwrap();

        // Should be valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Should contain expected placeholders
        assert!(json.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
        assert!(json.contains("REPLACE_WITH_SSH_PRIVATE_KEY_PATH"));
    }

    #[tokio::test]
    async fn it_should_generate_template_file() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("test-template.json");

        EnvironmentCreationConfig::generate_template_file(&template_path)
            .await
            .unwrap();

        assert!(template_path.exists());

        let content = tokio::fs::read_to_string(&template_path).await.unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[tokio::test]
    async fn it_should_create_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("configs/templates/test.json");

        EnvironmentCreationConfig::generate_template_file(&nested_path)
            .await
            .unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn it_should_generate_in_directory_with_default_name() {
        let temp_dir = TempDir::new().unwrap();

        let template_path = EnvironmentCreationConfig::generate_template_in_directory(temp_dir.path())
            .await
            .unwrap();

        assert!(template_path.exists());
        assert_eq!(
            template_path.file_name().unwrap(),
            "environment-template.json"
        );
    }

    #[tokio::test]
    async fn it_should_fail_with_invalid_directory() {
        let invalid_path = Path::new("/root/impossible/path/template.json");

        let result = EnvironmentCreationConfig::generate_template_file(invalid_path).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TemplateGenerationError::DirectoryCreationFailed { .. }
        ));
    }
}
```

## Integration with CLI (Issue #41)

The CLI `create template` subcommand will use these methods:

```rust
// From Issue #41 - CLI integration
match create_args.action {
    CreateAction::Template { output_path } => {
        let path = output_path.unwrap_or_else(|| PathBuf::from("environment-template.json"));

        EnvironmentCreationConfig::generate_template_file(&path).await?;

        println!("✓ Template created: {}", path.display());
        println!("\nNext steps:");
        println!("1. Edit the template file and replace REPLACE_WITH_* placeholders");
        println!("2. Run: torrust-tracker-deployer create environment -f {}", path.display());
    }
    // ... other actions
}
```

## Benefits of This Approach

### Comparison with Alternatives

| Aspect       | Const String         | Embedded Template    | Struct-Based (This)      |
| ------------ | -------------------- | -------------------- | ------------------------ |
| Type safety  | ❌ String literals   | ❌ Template file     | ✅ Rust types            |
| Maintenance  | ⚠️ Manual sync       | ⚠️ Manual sync       | ✅ Auto-synced           |
| Valid JSON   | ⚠️ Must verify       | ⚠️ Must verify       | ✅ Guaranteed            |
| Refactoring  | ❌ Manual updates    | ❌ Manual updates    | ✅ Compiler enforced     |
| Dependencies | ✅ None extra        | ❌ rust-embed + Tera | ✅ Only serde (existing) |
| Duplication  | ❌ Duplicates struct | ❌ Duplicates struct | ✅ IS the struct         |

### Key Advantages

1. **Type Safety**: Compiler guarantees the template structure is valid
2. **Zero Duplication**: No separate template representation to maintain
3. **Refactoring-Safe**: Adding/removing fields automatically updates template
4. **Guaranteed Valid JSON**: serde ensures proper serialization
5. **Clear Intent**: Code explicitly shows what's a placeholder vs default
6. **Simple**: Just methods on existing domain type, no new infrastructure

## Implementation Checklist

- [ ] Add `template()` static method to `EnvironmentCreationConfig`
- [ ] Add `generate_template_file()` async method
- [ ] Add `generate_template_in_directory()` convenience method
- [ ] Define `TemplateGenerationError` with actionable help messages
- [ ] Write comprehensive unit tests (8+ test cases)
- [ ] Update module documentation
- [ ] Test CLI integration (Issue #41)

## Related Documentation

- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Development Principles](../development-principles.md)
- [Original Issue v1 (Deprecated)](./40-subissue-6-template-system-integration.md)
- [Issue #41 - Template Generation Support](./41-subissue-7-template-generation-support.md)

## Analysis: Why the Approach Changed

### What Went Wrong Initially

The original issue (v1) suggested "extending the existing TemplateManager" which led to confusion:

1. **Misunderstood Template Purpose**: The issue conflated two different types of templates:

   - Infrastructure templates (Ansible, OpenTofu) - for deployment
   - Configuration templates (this issue) - for user setup

2. **Wrong System Choice**: The `TemplateManager` is designed for infrastructure templates that are:

   - Part of the application's deployment logic
   - Used by external tools
   - Sometimes need variable substitution

3. **Missed Opportunity**: Since the template structure exactly matches `EnvironmentCreationConfig`, we should generate it programmatically rather than maintain a duplicate representation.

### Lessons Learned

1. **Template Systems Have Different Purposes**: Infrastructure templates vs user configuration templates are fundamentally different concerns
2. **Avoid Duplication**: When a template structure exactly matches a data structure, generate it programmatically
3. **Use Type System**: Rust's type system can guarantee correctness better than template files
4. **Clear Documentation**: Issue descriptions should clearly explain which existing systems apply and which don't

### For Future Issues

When dealing with templates:

- Clearly distinguish between infrastructure templates and user-facing templates
- Consider struct-based generation before creating template files
- Document why existing template systems do or don't apply
- Provide code examples showing the intended approach
