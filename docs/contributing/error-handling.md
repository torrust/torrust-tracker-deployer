# Error Handling Guide

This guide establishes principles and best practices for error handling in the Torrust Tracker Deploy application, aligning with our [development principles](../development-principles.md) of observability, traceability, and actionability.

## 🎯 Core Principles

### 1. Clarity - No Ambiguity

Errors must be clear and unambiguous. Users should immediately understand what went wrong without needing to guess or interpret vague messages.

#### ✅ Good Examples

```rust
// Clear, specific error
pub enum ConfigError {
    FileNotFound { path: PathBuf },
    InvalidFormat { line: usize, reason: String },
    MissingRequiredField { field: String },
}
```

#### ❌ Bad Examples

```rust
// Vague, unclear error
return Err("Something went wrong".into());
return Err("Invalid input".into());
return Err("Error".into());
```

### 2. Context and Traceability

Errors should include sufficient context to make them easy to diagnose and fix. This aligns with our **Observability** and **Traceability** principles.

#### Context Requirements

- **What**: What operation was being performed?
- **Where**: Which component, file, or resource was involved?
- **When**: Under what conditions did this occur?
- **Why**: What caused the error?

#### ✅ Good Examples

```rust
pub enum ProvisioningError {
    InstanceAlreadyExists {
        instance_name: String,
        provider: String
    },
    InvalidConfiguration {
        config_path: PathBuf,
        validation_errors: Vec<String>
    },
    NetworkTimeout {
        operation: String,
        timeout_duration: Duration,
        endpoint: String
    },
}

impl Display for ProvisioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InstanceAlreadyExists { instance_name, provider } => {
                write!(f, "Instance '{}' already exists in {} provider. Use a different name or remove the existing instance.", instance_name, provider)
            },
            Self::InvalidConfiguration { config_path, validation_errors } => {
                write!(f, "Configuration file '{}' is invalid:\n{}",
                    config_path.display(),
                    validation_errors.join("\n"))
            },
            Self::NetworkTimeout { operation, timeout_duration, endpoint } => {
                write!(f, "Network timeout during '{}' operation to '{}' after {:?}. Check network connectivity and endpoint availability.",
                    operation, endpoint, timeout_duration)
            },
        }
    }
}
```

### 3. Actionability

Errors should be actionable, telling users how to fix them when possible. This aligns with our **Actionability** principle.

#### Requirements

- **Clear Instructions**: Provide specific steps to resolve the issue
- **Command Examples**: Include exact commands when applicable
- **Alternative Solutions**: Offer multiple approaches when possible
- **Next Steps**: Guide users on what to do next

#### ✅ Good Examples

```rust
pub enum DeploymentError {
    SshKeyNotFound {
        expected_path: PathBuf,
        alternative_paths: Vec<PathBuf>
    },
    InsufficientPermissions {
        required_permissions: String,
        current_user: String
    },
}

impl Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SshKeyNotFound { expected_path, alternative_paths } => {
                write!(f, "SSH key not found at '{}'. \n\nTo fix this:\n1. Generate a new SSH key: ssh-keygen -t rsa -b 4096 -f '{}'\n2. Or specify an existing key path using --ssh-key-path\n3. Alternative locations checked: {}",
                    expected_path.display(),
                    expected_path.display(),
                    alternative_paths.iter().map(|p| format!("'{}'", p.display())).collect::<Vec<_>>().join(", "))
            },
            Self::InsufficientPermissions { required_permissions, current_user } => {
                write!(f, "User '{}' lacks required permissions: {}\n\nTo fix this:\n1. Add your user to the required group: sudo usermod -aG lxd {}\n2. Log out and log back in to apply group changes\n3. Or run with sudo (not recommended for regular use)",
                    current_user, required_permissions, current_user)
            },
        }
    }
}
```

## 🛠️ Implementation Guidelines

### Prefer Explicit Enum Errors with Thiserror

Use explicit, strongly-typed enum errors with the `thiserror` crate instead of generic string-based errors for better pattern matching, automatic `Display` implementation, and error handling.

#### Using Thiserror for Error Definitions

The `thiserror` crate provides powerful macros for defining structured errors:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateManagerError {
    #[error("Failed to create templates directory: {path}")]
    DirectoryCreation {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Template file not found in embedded resources: {relative_path}")]
    TemplateNotFound { relative_path: String },

    #[error("Invalid UTF-8 in embedded template: {relative_path}")]
    InvalidUtf8 {
        relative_path: String,
        #[source]
        source: std::str::Utf8Error,
    },

    #[error("Failed to write template file: {path}")]
    TemplateWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
```

#### Key Benefits of Thiserror

- **Automatic `Display`**: The `#[error("...")]` attribute generates the `Display` implementation
- **Source Error Chaining**: The `#[source]` attribute maintains error chains for traceability
- **Structured Data**: Variant fields provide context and enable pattern matching
- **Better Debugging**: Automatic `Error` trait implementation with proper source chaining

#### When to Use Enum Errors

- **Recoverable errors**: When callers can take specific actions based on error type
- **Domain-specific errors**: When errors are specific to your application domain
- **Pattern matching**: When you need different handling for different error cases
- **API boundaries**: When errors cross module or crate boundaries

```rust
// ✅ Preferred: Explicit enum with context using thiserror
#[derive(Debug, Error)]
pub enum ConfigValidationError {
    #[error("Missing required field '{field}' in section '{section}'")]
    MissingField { field: String, section: String },

    #[error("Field '{field}' has invalid value '{value}', expected: {expected}")]
    InvalidValue { field: String, value: String, expected: String },

    #[error("Cannot access configuration file: {path}")]
    FileAccessError {
        path: PathBuf,
        #[source]
        source: std::io::Error
    },
}

// Allows for precise error handling
match config_result {
    Err(ConfigValidationError::MissingField { field, section }) => {
        println!("Please add '{}' to the '{}' section", field, section);
    },
    Err(ConfigValidationError::InvalidValue { field, value, expected }) => {
        println!("Field '{}' has invalid value '{}', expected: {}", field, value, expected);
    },
    // ... handle other cases
}
```

### Source Error Preservation for Traceability

Always include source errors when wrapping underlying errors. This maintains the full error chain and enables complete traceability.

#### ✅ Good: Preserve source errors

```rust
#[derive(Debug, Error)]
pub enum DeploymentError {
    #[error("Failed to read SSH key from {path}")]
    SshKeyRead {
        path: PathBuf,
        #[source]  // Preserves the original I/O error
        source: std::io::Error,
    },

    #[error("Network operation '{operation}' failed")]
    NetworkError {
        operation: String,
        #[source]  // Preserves the original network error
        source: reqwest::Error,
    },
}

// Usage with source preservation
fn read_ssh_key(path: &Path) -> Result<String, DeploymentError> {
    std::fs::read_to_string(path)
        .map_err(|source| DeploymentError::SshKeyRead {
            path: path.to_path_buf(),
            source,  // Original error is preserved
        })
}
```

#### ❌ Bad: Losing source information

```rust
// Don't do this - loses original error information
fn read_ssh_key(path: &Path) -> Result<String, DeploymentError> {
    std::fs::read_to_string(path)
        .map_err(|e| DeploymentError::SshKeyRead {
            path: path.to_path_buf(),
            // Missing source - loses traceability!
        })
}
```

### When to Use Anyhow

Use `anyhow` only when the caller cannot do anything meaningful to handle different error types, even with pattern matching.

#### Appropriate Use Cases

- **Utility functions**: Internal helpers where specific error handling isn't needed
- **One-way operations**: When all errors should bubble up unchanged
- **Rapid prototyping**: Early development phases (but migrate to enums later)
- **External library integration**: When wrapping third-party errors temporarily

```rust
// ✅ Acceptable: Internal utility where caller can't handle specifics
fn read_and_parse_internal_cache() -> anyhow::Result<CacheData> {
    let content = std::fs::read_to_string("cache.json")?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

// ✅ Public API should use enums
pub fn load_user_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::FileAccess { path: path.to_path_buf(), source: e })?;

    serde_json::from_str(&content)
        .map_err(|e| ConfigError::InvalidJson { path: path.to_path_buf(), source: e })
}
```

### Error Conversion Patterns

```rust
// Convert anyhow errors to domain errors at boundaries
impl From<anyhow::Error> for DeploymentError {
    fn from(err: anyhow::Error) -> Self {
        DeploymentError::InternalError {
            message: err.to_string(),
            context: format!("{:?}", err.chain().collect::<Vec<_>>())
        }
    }
}
```

## 📋 Error Review Checklist

When reviewing error handling code, verify:

- [ ] **Clarity**: Is the error message clear and unambiguous?
- [ ] **Context**: Does the error include sufficient context (what, where, when, why)?
- [ ] **Actionability**: Does the error tell users how to fix it?
- [ ] **Type Safety**: Are domain-specific errors using enums instead of strings?
- [ ] **Thiserror Usage**: Are enum errors using `thiserror` with proper `#[error]` attributes?
- [ ] **Source Preservation**: Are source errors preserved with `#[source]` for traceability?
- [ ] **Pattern Matching**: Can callers handle different error cases appropriately?
- [ ] **Consistency**: Does the error follow project conventions?

## 🔗 Related Documentation

- [Development Principles](../development-principles.md) - Core principles including observability and actionability
- [Contributing Guidelines](./README.md) - General contribution guidelines
- [Testing Conventions](./testing.md) - Testing error scenarios

## 📚 Examples in Codebase

Look for error handling examples in:

- `src/domain/` - Domain-specific error enums
- `src/infrastructure/` - Infrastructure and I/O error handling
- `src/application/commands/` - Command-level error aggregation

## 🚀 Best Practices Summary

1. **Design errors first**: Consider error cases during API design
2. **Use enums by default**: Only use `anyhow` when justified
3. **Include context**: Always provide enough information for diagnosis
4. **Make errors actionable**: Tell users how to fix the problem
5. **Test error paths**: Write tests for error scenarios
6. **Document error types**: Document when and why specific errors occur

By following these guidelines, we ensure that errors in the Torrust Tracker Deploy application are not just informative, but truly helpful in guiding users toward solutions.
