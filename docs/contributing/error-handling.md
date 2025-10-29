# Error Handling Guide

This guide establishes principles and best practices for error handling in the Torrust Tracker Deployer application, aligning with our [development principles](../development-principles.md) of observability, traceability, and actionability.

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

### Unwrap and Expect Usage

The use of `unwrap()` is generally **discouraged** in this project. While it may make sense in certain contexts, we prefer alternatives that align with our principles of observability, traceability, and user-friendliness.

#### General Rule

**Prefer `expect()` over `unwrap()`** - Even in cases where panicking is acceptable, use `expect()` to provide meaningful context that aids debugging and aligns with our observability principles.

#### When Unwrap/Expect is Acceptable

##### Tests

In test code, panicking on unexpected failures is acceptable and even desired. However, **prefer `expect()` with descriptive messages** to make test failures easier to understand.

```rust
// ✅ Good: Tests with expect() providing context
#[test]
fn it_should_parse_valid_config() {
    let config_str = r#"{"name": "test", "port": 8080}"#;
    let config: Config = serde_json::from_str(config_str)
        .expect("Failed to parse valid test configuration - this indicates a parsing bug");

    assert_eq!(config.name, "test");
}

// ✅ Also acceptable in tests with clear context
#[test]
fn it_should_create_temp_directory() {
    let temp_dir = TempDir::new()
        .expect("Failed to create temporary directory for test - check filesystem permissions");
    // ... rest of test
}

// ❌ Avoid: Unwrap without context
#[test]
fn it_should_parse_valid_config() {
    let config_str = r#"{"name": "test", "port": 8080}"#;
    let config: Config = serde_json::from_str(config_str).unwrap();  // What failed? Why?

    assert_eq!(config.name, "test");
}
```

##### Infallible Operations

Use `expect()` (not `unwrap()`) for operations that are logically infallible but return `Result` or `Option` due to API design.

```rust
// ✅ Good: expect() with clear reasoning
let port: u16 = env::var("PORT")
    .expect("PORT environment variable must be set during application initialization")
    .parse()
    .expect("PORT must be a valid u16 number - validated during configuration loading");

// ✅ Good: Mutex operations that should never fail
let data = self.state.lock()
    .expect("State mutex poisoned - indicates a panic occurred while holding the lock");

// ❌ Avoid: unwrap() without explanation
let port: u16 = env::var("PORT").unwrap().parse().unwrap();
```

#### When to Use Proper Error Handling Instead

In production code and public APIs, prefer proper error handling over `unwrap()` or `expect()`:

```rust
// ✅ Production code: Return proper errors
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|source| ConfigError::FileAccess {
            path: path.to_path_buf(),
            source
        })?;

    serde_json::from_str(&content)
        .map_err(|source| ConfigError::InvalidJson {
            path: path.to_path_buf(),
            source
        })
}

// ❌ Don't do this in production code
pub fn load_config(path: &Path) -> Config {
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}
```

#### Context Requirements for Expect

When using `expect()`, the message should explain:

1. **What** was expected to succeed
2. **Why** it should succeed (if not obvious)
3. **What** the failure indicates (if relevant)

```rust
// ✅ Good: Complete context
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("System time is before UNIX epoch - this indicates a serious system clock issue");

// ✅ Good: Clear explanation
let config = CONFIG.get()
    .expect("Configuration must be initialized before starting application - call init_config() first");

// ❌ Insufficient context
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time error");

// ❌ No context at all
let config = CONFIG.get().unwrap();
```

#### Summary

- **Default**: Use proper error handling with `Result` and specific error types
- **Tests**: Use `expect()` with descriptive messages explaining what failed
- **Infallible operations**: Use `expect()` with clear reasoning about why failure is impossible
- **Never**: Use `unwrap()` without a very good reason (prefer `expect()` even in those cases)

This approach ensures that even panic messages provide valuable debugging context, maintaining our commitment to observability and traceability throughout the codebase.

### Tiered Help System for Actionable Errors

For errors that require detailed troubleshooting guidance without cluttering the error message, use the **tiered help system** pattern. This approach balances brevity with actionability.

See [Decision Record: Actionable Error Messages](../decisions/actionable-error-messages.md) for the rationale behind this pattern.

#### Pattern Overview

1. **Base error message**: Concise with essential context
2. **Brief tip**: One-liner actionable hint in the error message
3. **`.help()` method**: Detailed troubleshooting available on-demand
4. **Rustdoc**: Developer-oriented documentation

#### Implementation Example

````rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileLockError {
    /// Failed to acquire lock within timeout period
    ///
    /// This typically means another process is holding the lock.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Failed to acquire lock for '{path}' within {timeout:?} (held by process {holder_pid})
Tip: Use 'ps -p {holder_pid}' to check if process is running")]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: ProcessId,
        timeout: Duration,
    },

    /// Failed to create lock file
    ///
    /// This usually indicates permission issues or file system problems.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Failed to create lock file at '{path}': {source}
Tip: Check directory permissions and disk space")]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl FileLockError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// if let Err(e) = FileLock::acquire(&path, timeout) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// ```
    pub fn help(&self) -> &'static str {
        match self {
            Self::AcquisitionTimeout { .. } => {
                "Lock Acquisition Timeout - Detailed Troubleshooting:

1. Check if the holder process is still running:
   Unix/Linux/macOS: ps -p <pid>
   Windows: tasklist /FI \"PID eq <pid>\"

2. If the process is running and should release the lock:
   - Wait for the process to complete its operation
   - Or increase the timeout duration in your configuration

3. If the process is stuck or hung:
   - Try graceful termination: kill <pid>  (Unix) or taskkill /PID <pid> (Windows)
   - Force terminate if needed: kill -9 <pid>  (Unix) or taskkill /F /PID <pid> (Windows)

4. If the process doesn't exist (stale lock):
   - This should be handled automatically by the lock system
   - If you see this error repeatedly, it indicates a bug
   - Please report the issue with full details

For more information, see the documentation on file locking."
            }

            Self::CreateFailed { .. } => {
                "Lock Creation Failed - Detailed Troubleshooting:

1. Check directory permissions and ensure write access
2. Verify parent directory exists
3. Check available disk space: df -h  (Unix) or wmic logicaldisk (Windows)
4. Check for file system issues

If the problem persists, report it with system details."
            }
        }
    }
}
````

#### When to Use This Pattern

Use the tiered help system when:

- ✅ Errors require detailed troubleshooting steps
- ✅ Platform-specific guidance is needed (Unix vs Windows commands)
- ✅ Multiple resolution approaches exist
- ✅ Brief error messages would be insufficient
- ✅ Verbose error messages would be overwhelming

Don't use this pattern when:

- ❌ The error is self-explanatory
- ❌ Resolution is a single, obvious step
- ❌ The error is purely internal (developers only)

#### Application Integration

```rust
// Basic usage: just show the error
match FileLock::acquire(&path, timeout) {
    Ok(lock) => { /* use lock */ }
    Err(e) => {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

// Advanced usage: show help based on verbosity
match FileLock::acquire(&path, timeout) {
    Ok(lock) => { /* use lock */ }
    Err(e) => {
        eprintln!("Error: {e}");

        if verbose {
            eprintln!("\n{}", e.help());
        } else {
            eprintln!("\nRun with --verbose for detailed troubleshooting");
        }

        std::process::exit(1);
    }
}
```

#### Benefits

- ✅ Balances brevity with actionability
- ✅ No external infrastructure required
- ✅ Help always available at runtime
- ✅ Easy to maintain (help lives with error definition)
- ✅ Platform-aware guidance included
- ✅ Users control verbosity level

## 📐 Error Structure Template

When defining new error types, use this template to ensure consistency:

```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum YourCommandError {
    // ===== File/Configuration Errors =====

    /// Brief description of when this error occurs
    ///
    /// More detailed explanation if needed.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Clear error message with context: {path}
Tip: Brief actionable hint - command example if applicable")]
    ConfigFileNotFound {
        /// Path to the missing file
        path: PathBuf,
    },

    /// Brief description
    #[error("Error message with multiple context values: '{path}' as {format}: {source}
Tip: Validate format with: command --check {path}")]
    ParsingFailed {
        /// Path to the file
        path: PathBuf,
        /// Expected format
        format: String,
        /// Original parsing error
        #[source]
        source: SomeError,
    },

    // ===== Operation Errors =====

    /// Brief description
    ///
    /// Explanation of when this occurs.
    #[error("Operation '{operation}' failed for '{name}': {source}
Tip: Check logs with: --verbose or --log-output file-and-stderr")]
    OperationFailed {
        /// Name of the resource
        name: String,
        /// Operation being performed
        operation: String,
        /// Underlying error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // Add more error variants as needed, grouped by category
}

impl YourCommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// if let Err(e) = some_operation() {
    ///     eprintln!("Error: {e}");
    ///     if verbose {
    ///         eprintln!("\nTroubleshooting:\n{}", e.help());
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => {
                "Configuration File Not Found - Detailed Troubleshooting:

1. Check file path is correct
   - Verify path spelling: ls -la <path>
   - Use absolute or relative paths correctly

2. Verify file permissions
   - Check read permissions: ls -l <path>
   - Fix if needed: chmod 644 <path>

3. Common solutions
   - Create the file if missing
   - Check current directory: pwd
   - Provide correct path in arguments

For more information, see the documentation."
            }

            Self::ParsingFailed { .. } => {
                "Parsing Failed - Detailed Troubleshooting:

1. Validate syntax
   - Use appropriate validator tool
   - Check for common syntax errors

2. Verify format matches expectation
   - Check file extension
   - Validate structure

For more information, see format documentation."
            }

            Self::OperationFailed { .. } => {
                "Operation Failed - Detailed Troubleshooting:

1. Check system state
   - Verify resources are available
   - Check permissions

2. Review logs for details
   - Run with --verbose
   - Check log output

For persistent issues, contact support."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_help_for_all_variants() {
        // Create instances of all error variants
        let errors: Vec<YourCommandError> = vec![
            // ... create test instances
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn it_should_display_context_in_errors() {
        // Test that context fields appear in error messages
    }
}
```

### Template Guidelines

1. **Group related errors** with section comments (e.g., `// ===== File Errors =====`)
2. **Always include context fields** (paths, names, IDs) relevant to the error
3. **Use `#[source]`** for all wrapped errors to preserve error chains
4. **Add brief tips** in error messages using `\nTip:` format
5. **Document each variant** with rustdoc comments explaining when it occurs
6. **Implement `.help()`** with detailed troubleshooting for each variant
7. **Write comprehensive tests** covering all variants and help text

## 📋 Error Review Checklist

When reviewing error handling code, verify:

- [ ] **Clarity**: Is the error message clear and unambiguous?
- [ ] **Context**: Does the error include sufficient context (what, where, when, why)?
- [ ] **Actionability**: Does the error tell users how to fix it?
- [ ] **Tiered Help**: If detailed guidance is needed, does the error use the `.help()` pattern?
- [ ] **Brief Tips**: Does the error include a concise tip in the message?
- [ ] **Type Safety**: Are domain-specific errors using enums instead of strings?
- [ ] **Thiserror Usage**: Are enum errors using `thiserror` with proper `#[error]` attributes?
- [ ] **Source Preservation**: Are source errors preserved with `#[source]` for traceability?
- [ ] **Pattern Matching**: Can callers handle different error cases appropriately?
- [ ] **Unwrap/Expect**: Is `unwrap()` avoided in favor of `expect()` with descriptive messages?
- [ ] **Consistency**: Does the error follow project conventions?
- [ ] **Error Grouping**: Are related errors grouped with section comments?

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
5. **Prefer `expect()` over `unwrap()`**: Provide meaningful context even when panicking
6. **Test error paths**: Write tests for error scenarios
7. **Document error types**: Document when and why specific errors occur

By following these guidelines, we ensure that errors in the Torrust Tracker Deployer application are not just informative, but truly helpful in guiding users toward solutions.
