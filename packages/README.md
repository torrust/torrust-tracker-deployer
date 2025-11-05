# Torrust Tracker Deployer - Packages

This directory contains reusable Rust workspace packages that support the Torrust Tracker Deployer project. These packages are designed to be modular, maintainable, and potentially reusable across other Torrust projects.

## 📦 Available Packages

### [`dependency-installer/`](./dependency-installer/)

**Purpose**: Dependency detection and installation utilities for development environments

**Key Features**:

- Detects if required development tools are installed (OpenTofu, Ansible, LXD, cargo-machete)
- Installs missing dependencies automatically
- Provides CLI for manual and automated use
- Designed for CI/CD pipelines and automated workflows
- Uses structured logging (tracing) for observability
- Exit-code based success/failure indication for automation

**Use Cases**:

- Setting up development environments for humans and AI agents
- Pre-flight checks in E2E test suites
- CI/CD pipeline dependency validation
- Automated development environment provisioning

**Documentation**: See [packages/dependency-installer/README.md](./dependency-installer/README.md)

### [`linting/`](./linting/)

**Purpose**: Unified linting framework for Rust projects

**Key Features**:

- Supports multiple linters: markdown, YAML, TOML, Rust (clippy + rustfmt), shellcheck
- Pre-built CLI components for easy binary creation
- Extensible architecture for adding new linters
- Uses existing configuration files (`.taplo.toml`, `.yamllint.yml`, etc.)

**Use Cases**:

- Enforcing code quality standards
- Pre-commit validation
- CI/CD linting pipelines
- Standardizing linting across multiple projects

**Documentation**: See [packages/linting/README.md](./linting/README.md)

## 🏗️ Package Architecture

All packages in this directory:

- Are part of a Cargo workspace (defined in root `Cargo.toml`)
- Can be used independently or as library crates
- Follow the project's development principles (observability, testability, user-friendliness)
- Provide both CLI binaries and programmatic APIs
- Use structured logging via the `tracing` crate
- Follow consistent error handling patterns

## 🚀 Using Packages

### As Library Crates

```rust
// Add to your Cargo.toml
[dependencies]
torrust-linting = { path = "packages/linting" }
torrust-dependency-installer = { path = "packages/dependency-installer" }
```

### As CLI Binaries

```bash
# Run the linter
cargo run --bin linter all

# Run the dependency installer
cargo run --bin dependency-installer check
cargo run --bin dependency-installer install
```

## 🎯 Package Design Principles

### Automation-First Design

Packages prioritize **automation and CI/CD workflows**:

- **Exit codes** indicate success/failure (0 = success, non-zero = failure)
- **Structured logging** provides rich context without parsing output
- **Flags for verbosity** (`--verbose`, `--log-level`) control output detail
- **Minimal output** by default, detailed only when needed

### Type Safety

- Use strongly-typed enums and structs
- Leverage Rust's type system for compile-time guarantees
- Avoid stringly-typed APIs

### Error Handling

- Clear, actionable error messages
- Preserve error context with source chains
- Use thiserror for structured error types

### Extensibility

- Easy to add new functionality (linters, dependencies)
- Plugin-like architecture where appropriate
- Trait-based abstractions for flexibility

## 📋 Adding New Packages

When creating new packages:

1. **Create package directory** under `packages/`
2. **Add to workspace** in root `Cargo.toml`:

   ```toml
   [workspace]
   members = [
       "packages/your-new-package",
       # ...
   ]
   ```

3. **Create package README** documenting purpose and usage
4. **Update this file** to include the new package in the list above
5. **Follow conventions**:
   - Use `tracing` for logging
   - Provide both CLI and library interfaces
   - Follow project development principles
   - Add comprehensive tests

## 🔗 Related Documentation

- [Development Principles](../docs/development-principles.md) - Core principles guiding all packages
- [Error Handling Guide](../docs/contributing/error-handling.md) - Error handling patterns
- [Testing Conventions](../docs/contributing/testing/) - Testing standards
- [E2E Testing Guide](../docs/e2e-testing.md) - How packages integrate with E2E tests

## 💡 Future Packages

Potential future packages that could be added:

- **Configuration Management**: Reusable config loading and validation
- **Template Engine**: Tera template rendering utilities
- **SSH Client**: SSH operations and connectivity checking
- **Infrastructure Clients**: OpenTofu, Ansible, LXD client abstractions
- **Test Utilities**: Common test helpers and fixtures

These packages would further modularize the codebase and improve reusability across the Torrust ecosystem.
