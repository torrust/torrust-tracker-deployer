# Docker Testing Infrastructure

This directory contains Docker configurations for testing the dependency-installer CLI.

## Images

### ubuntu-24.04.Dockerfile

Base Ubuntu 24.04 image for testing the CLI binary in a clean environment.

**Purpose**: Verify that the `dependency-installer check` command correctly detects missing tools.

**Usage in tests**:

```rust
let image = GenericImage::new("ubuntu", "24.04")
    .with_wait_for(WaitFor::message_on_stdout("Ready"));
```

## Testing Strategy

1. Build the binary: `cargo build --bin dependency-installer`
2. Copy binary into container using testcontainers
3. Run `check` command in container
4. Verify it correctly reports missing tools
5. (Phase 4) Install tools and verify installation

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
