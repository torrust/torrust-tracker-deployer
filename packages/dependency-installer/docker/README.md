# Docker Testing Infrastructure

This directory contains Docker configurations for testing the dependency-installer CLI.

## Images

### ubuntu-24.04.Dockerfile

Pre-configured Ubuntu 24.04 image with all operating system prerequisites installed.

**Purpose**: This Dockerfile represents the **operating system dependencies (pre-conditions)** required before using the dependency installers. It includes:

- System packages: `ca-certificates`, `sudo`, `curl`, `build-essential`
- Rust toolchain: `nightly-2025-10-15` installed via rustup
- PATH configuration for cargo binaries

The tests verify that given these OS pre-conditions, the installers can successfully install their target dependencies (cargo-machete, OpenTofu, Ansible, LXD).

**Usage in tests**:

```rust
let image = GenericImage::new("ubuntu", "24.04")
    .with_wait_for(WaitFor::message_on_stdout("Ready"));
```

## Testing Strategy

The Docker-based integration tests verify that the installers work correctly **given the declared OS pre-conditions**:

1. **Pre-built Image**: Use `ubuntu-24.04.Dockerfile` with all OS dependencies pre-installed
2. **Build Binary**: `cargo build --bin dependency-installer`
3. **Copy Into Container**: Use testcontainers to copy binary into pre-configured container
4. **Test Installers**: Run `install` commands and verify successful installation
5. **Verify Detection**: Use `check` command to confirm installations are detected

This approach ensures:

- Fast test execution (OS dependencies installed once during image build)
- Clear documentation of required OS pre-conditions (declared in Dockerfile)
- Confidence that installers work in production environments matching the Dockerfile

## Building Images

```bash
cd packages/dependency-installer
docker build -f docker/ubuntu-24.04.Dockerfile -t dependency-installer-test:ubuntu-24.04 .
```

## Running Tests

```bash
# Run all Docker-based integration tests
cd packages/dependency-installer
cargo test --test docker_check_command

# Run a specific test
cargo test --test docker_check_command test_check_all_reports_missing_dependencies
```

## Container Architecture

The tests use testcontainers to:

- Automatically start and stop Docker containers
- Copy the compiled binary into containers
- Execute commands and capture output
- Verify exit codes and output messages

This ensures tests run in isolated, reproducible environments.

## Related

- `tests/docker_check_command.rs` - Integration tests using this infrastructure
- `tests/containers/` - Container helper utilities
