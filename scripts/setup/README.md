# Setup Scripts - MIGRATION NOTICE

**⚠️ This directory has been migrated to the Rust-based dependency installer.**

## Migration Notice

The bash installation scripts (`install-opentofu.sh`, `install-ansible.sh`, `install-lxd-ci.sh`) have been **removed** and replaced with the Rust-based `dependency-installer` binary.

### New Installation Method

For dependency installation, use the `dependency-installer` binary:

```bash
# Install all dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer -- install

# Check which dependencies are installed
cargo run -p torrust-dependency-installer --bin dependency-installer -- check

# List all available dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer -- list

# Install specific dependency
cargo run -p torrust-dependency-installer --bin dependency-installer -- install --dependency opentofu

# See all options
cargo run -p torrust-dependency-installer --bin dependency-installer -- --help
```

### Benefits of the New Approach

- **Type-safe**: Rust's type system catches errors at compile time
- **Better error handling**: Clear, actionable error messages
- **Consistent logging**: Structured logging via `tracing` crate
- **Testable**: Unit and integration tests ensure reliability
- **Maintainable**: Single codebase for all dependency management
- **Cross-platform ready**: Foundation for supporting multiple platforms

### Supported Dependencies

The dependency installer supports:

- **cargo-machete** - Detects unused Rust dependencies
- **OpenTofu** - Infrastructure provisioning tool (Terraform alternative)
- **Ansible** - Configuration management and automation platform
- **LXD** - Lightweight VM manager for container-based testing

### Documentation

For complete documentation on the dependency installer, see:

- **Package README**: [packages/dependency-installer/README.md](../../packages/dependency-installer/README.md)
- **LXD Setup**: [templates/tofu/lxd/README.md](../../templates/tofu/lxd/README.md)
- **LXD Tech Stack**: [docs/tech-stack/lxd.md](../../docs/tech-stack/lxd.md)

### CI/CD Integration

GitHub Actions workflows now use the dependency installer:

- **`.github/workflows/test-e2e-provision.yml`** - E2E provision and destroy tests
- **`.github/workflows/test-e2e-config.yml`** - E2E configuration tests
- **`.github/workflows/test-lxd-provision.yml`** - LXD provisioning tests

Example workflow step:

```yaml
- name: Install dependencies
  run: |
    cargo build -p torrust-dependency-installer --bin dependency-installer
    cargo run -p torrust-dependency-installer --bin dependency-installer -- install
```

### Local Development

For local development, the dependency installer automatically handles:

- **CI Detection**: Automatically applies CI-specific configurations when needed
- **Permissions**: Proper handling of LXD socket permissions
- **Group Membership**: Sets up appropriate user groups

**Important**: For local LXD development, follow the proper group membership approach documented in the tech stack guides.

### Troubleshooting

If you encounter issues with the dependency installer:

1. **Check installation status**: `cargo run -p torrust-dependency-installer --bin dependency-installer -- check`
2. **Enable debug logging**: Add `--verbose` flag or `--log-level debug`
3. **View available dependencies**: `cargo run -p torrust-dependency-installer --bin dependency-installer -- list`
4. **Check exit codes**: Exit code 0 = success, non-zero = failure

For detailed troubleshooting, see the [dependency installer README](../../packages/dependency-installer/README.md).

### Migration Timeline

- **Created**: Issue #113 (Create Dependency Installation Package)
- **Migrated**: Issue #119 (Update CI Workflows and Remove Bash Scripts)
- **Removed bash scripts**: November 2025

This directory is preserved for documentation purposes and may be removed in future versions.
