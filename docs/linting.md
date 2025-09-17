# Linting

This project uses automated linting to maintain code quality and consistency across different file types.

## Available Linters

- **Markdown**: Uses `markdownlint-cli` with configuration in `.markdownlint.json`
- **YAML**: Uses `yamllint` with configuration in `.yamllint-ci.yml`
- **Shell Scripts**: Uses `ShellCheck` for bash/shell script analysis
- **Rust Code**: Uses `clippy` for Rust code analysis
- **Rust Formatting**: Uses `rustfmt` to check code formatting

## Usage

### Quick Commands (Recommended)

```bash
# Run all linters
cargo run --bin linter all

# Run specific linters
cargo run --bin linter markdown    # Markdown only
cargo run --bin linter yaml        # YAML only
cargo run --bin linter toml        # TOML only
cargo run --bin linter clippy      # Rust code analysis only
cargo run --bin linter rustfmt     # Rust formatting check only
cargo run --bin linter shellcheck  # Shell scripts only

# Show help
cargo run --bin linter --help
```

## Installation

The Rust linter binary will automatically install the required tools if they're not already present:

- **markdownlint-cli**: Installed via npm
- **yamllint**: Installed via system package manager (apt, dnf, pacman) or pip3
- **Taplo CLI**: Installed via cargo for TOML linting and formatting
- **ShellCheck**: Installed via system package manager (apt, dnf, pacman, brew)
- **Rust clippy & rustfmt**: Installed as part of the Rust toolchain

## CI/CD Integration

The same Rust binary is used in GitHub Actions, ensuring consistency between local development
and CI environments. The workflow runs on every push and pull request.

## Configuration

- **Markdown**: `.markdownlint.json` - Controls line length, heading styles, etc.
- **YAML**: `.yamllint-ci.yml` - Controls line length, indentation, etc.
- **TOML**: `.taplo.toml` - Controls formatting, indentation, array handling, etc.

## Rust Code Style Guidelines

### Error Enums

Error enum variants should be separated by blank lines for better readability.

❌ **Incorrect** - No line breaks between variants:

```rust
/// Errors that can occur when creating a `CloudInitContext`
#[derive(Error, Debug, Clone)]
pub enum CloudInitContextError {
    #[error("SSH public key is required but not provided")]
    MissingSshPublicKey,
    #[error("Failed to read SSH public key from file: {0}")]
    SshPublicKeyReadError(String),
}
```

✅ **Correct** - Line breaks between variants:

```rust
/// Errors that can occur when creating a `CloudInitContext`
#[derive(Error, Debug, Clone)]
pub enum CloudInitContextError {
    #[error("SSH public key is required but not provided")]
    MissingSshPublicKey,

    #[error("Failed to read SSH public key from file: {0}")]
    SshPublicKeyReadError(String),
}
```

### Documentation Comments

Documentation sections like `# Errors`, `# Panics`, `# Examples`, etc. should have a blank line after the heading.

❌ **Incorrect** - No blank line after heading:

```rust
/// # Errors
/// Returns an error if the file cannot be read
pub fn name() { /* ... */ }
```

✅ **Correct** - Blank line after heading:

```rust
/// # Errors
///
/// Returns an error if the file cannot be read
pub fn name() { /* ... */ }
```

## Benefits

✅ **Consistent formatting** across all team members  
✅ **Automatic tool installation** for easy setup  
✅ **Same binary** used locally and in CI  
✅ **Structured logging** with timestamps and targets  
✅ **Cross-platform support** for different package managers  
✅ **Type-safe implementation** with Rust error handling
