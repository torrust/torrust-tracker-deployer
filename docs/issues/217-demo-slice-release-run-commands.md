# Demo Slice - Release and Run Commands Scaffolding

**Issue**: [#217](https://github.com/torrust/torrust-tracker-deployer/issues/217)
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)

## Overview

This task implements the foundational scaffolding for the `release` and `run` commands using a minimal docker-compose deployment with nginx. The goal is to validate the full pipeline (release → run → verify) before adding complexity with the actual Torrust Tracker services.

## Goals

- [ ] Create `ReleaseCommandHandler` (App layer) with state transitions
- [ ] Create `RunCommandHandler` (App layer) with state transitions
- [ ] Create `release` CLI subcommand (Presentation layer)
- [ ] Create `run` CLI subcommand (Presentation layer)
- [ ] Add docker-compose file infrastructure
- [ ] Deploy and run demo-app (nginx) container on provisioned VM
- [ ] Verify container is running and healthy
- [ ] Rename `e2e_config_tests.rs` → `e2e_config_and_release_tests.rs` and extend
- [ ] Update `e2e_tests_full.rs` to include release and run commands

## Architecture Overview

### State Transitions

The environment lifecycle extends with new states:

```text
Configured → Releasing → Released → Starting → Running
                ↓                       ↓          ↓
         ReleaseFailed              StartFailed  RunFailed
```

- **Release**: `Configured` → `Releasing` → `Released` (or `ReleaseFailed`)
- **Run**: `Released` → `Starting` → `Running` (or `StartFailed` / `RunFailed`)

> **Note**: The `Starting` state allows us to wait for containers to become healthy
> (for services with health checks) before transitioning to `Running`.

### Three-Level Architecture

Following the existing pattern:

1. **Command Handlers (Level 1)**: `ReleaseCommandHandler`, `RunCommandHandler`
2. **Steps (Level 2)**: `PrepareComposeFilesStep`, `TransferFilesStep`, `StartServicesStep`
3. **Remote Actions (Level 3)**: SSH file transfer, docker compose commands

## Implementation Plan

> **Implementation Order: Outside-In**
>
> We follow an **outside-in** approach (Presentation → Application → Infrastructure) rather than inside-out. This is critical for an infrastructure project where:
>
> - Unit tests for internal layers are difficult or impossible
> - E2E tests are our primary safety net
> - We want working (even if no-op) commands after each step
>
> Each phase produces a **runnable command** that can be E2E tested, even if it does nothing yet. This aligns with our Agile/Lean philosophy: working software at every increment.

### Phase 1: Presentation Layer - CLI Commands (No-Op)

**Goal**: Add `release` and `run` CLI commands that do nothing yet but are runnable.

**Location**: `src/presentation/input/cli/commands.rs`

Add new CLI commands:

```rust
/// Release application files to a configured environment
Release {
    /// Name of the environment to release to
    environment: String,
},

/// Run the application stack on a released environment
Run {
    /// Name of the environment to run
    environment: String,
},
```

**Expected CLI Help Output**:

```text
$ cargo run -- --help
...
Commands:
  create      Create a new environment or resource
  provision   Provision infrastructure for an environment
  configure   Configure a provisioned environment
  release     Release application files to a configured environment
  run         Run the application stack on a released environment
  destroy     Destroy an environment and its resources
  help        Print this message or the help of the given subcommand(s)

$ cargo run -- release --help
Release application files to a configured environment

This command prepares and transfers application files (docker-compose.yml, .env, etc.)
to a configured VM. The environment must be in the 'Configured' state.

After successful release:
- Docker compose files are copied to /opt/torrust/ on the VM
- Environment transitions to 'Released' state
- You can then run 'run <environment>' to start the services

Usage: torrust-tracker-deployer release <ENVIRONMENT>

Arguments:
  <ENVIRONMENT>  Name of the environment to release to

Options:
  -h, --help  Print help

Examples:
  torrust-tracker-deployer release my-env
  torrust-tracker-deployer release production

$ cargo run -- run --help
Run the application stack on a released environment

This command starts the docker compose services on a released VM.
The environment must be in the 'Released' state.

After successful run:
- Docker containers are started via 'docker compose up -d'
- Environment transitions to 'Running' state
- Services are accessible on the VM

Usage: torrust-tracker-deployer run <ENVIRONMENT>

Arguments:
  <ENVIRONMENT>  Name of the environment to run

Options:
  -h, --help  Print help

Examples:
  torrust-tracker-deployer run my-env
  torrust-tracker-deployer run production
```

**Deliverable**: `cargo run -- release my-env` runs without error (prints "not implemented yet").

**Manual E2E Test**:

```bash
# Both commands should be recognized and print "not implemented yet"
cargo run -- release my-env
cargo run -- run my-env

# Help should show new commands (see "Expected CLI Help Output" above)
cargo run -- --help
cargo run -- release --help
cargo run -- run --help
```

### Phase 2: E2E Test Refactoring (Safety Net)

**Goal**: Rename E2E tests and add initial test cases for the new commands. This establishes our safety net before proceeding with internal implementation.

**Changes**:

1. **Rename `e2e_config_tests.rs` → `e2e_config_and_release_tests.rs`**

```bash
git mv src/bin/e2e_config_tests.rs src/bin/e2e_config_and_release_tests.rs
```

1. **Update `Cargo.toml`**:

```toml
[[bin]]
name = "e2e-config-and-release-tests"
path = "src/bin/e2e_config_and_release_tests.rs"
```

1. **Update `scripts/pre-commit.sh`**:

```bash
# Change from:
cargo run --bin e2e-config-tests

# To:
cargo run --bin e2e-config-and-release-tests
```

1. **Add initial test cases** for CLI command existence:

```rust
fn test_release_command_help() {
    // cargo run -- release --help should succeed
    let output = Command::new("cargo")
        .args(["run", "--", "release", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Release application files"));
}

fn test_run_command_help() {
    // cargo run -- run --help should succeed
    let output = Command::new("cargo")
        .args(["run", "--", "run", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run the application stack"));
}
```

1. **Update `e2e_tests_full.rs`** with placeholder test:

```rust
// TODO: Extend with release and run once implemented
fn test_full_pipeline_placeholder() {
    // For now, just verify the commands exist
    // Will be extended in Phase 10
}
```

**Deliverable**: E2E tests renamed, new test cases added, pre-commit hook updated.

**Manual Verification**:

```bash
# Run the renamed E2E tests
cargo run --bin e2e-config-and-release-tests

# Verify pre-commit still works
./scripts/pre-commit.sh
```

### Phase 3: Presentation Layer - Controllers (Wired but Empty)

**Location**: `src/presentation/controllers/`

Create controllers:

- `release/` - `ReleaseCommandController` that calls handler (which does nothing)
- `run/` - `RunCommandController` that calls handler (which does nothing)

**Deliverable**: CLI commands are wired to controllers, still no-op but architecture is in place.

**Manual E2E Test**:

```bash
# Commands should still work, now going through controller layer
cargo run -- release my-env
cargo run -- run my-env

# Verify no regressions in existing commands
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- destroy my-env
```

### Phase 4: Application Layer - Command Handlers (Skeleton)

**Location**: `src/application/command_handlers/release/` and `run/`

Create files:

- `mod.rs` - Module definition and re-exports
- `handler.rs` - `ReleaseCommandHandler` / `RunCommandHandler` (empty impl)
- `errors.rs` - Error types with `.help()` support

**Handler Workflow** (skeleton - logs but doesn't execute):

1. Load environment (validates state)
2. Log "would transition to Releasing/Running"
3. Return success

**Deliverable**: Full vertical slice wired, validates environment state, logs intent.

**Manual E2E Test**:

```bash
# Create and configure an environment first
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config

# Release should validate state (must be Configured)
cargo run -- release e2e-config
# Expected: Logs intent, validates state is Configured

# Run on non-Released environment should fail with helpful error
cargo run -- run e2e-config
# Expected: Error - environment must be in Released state

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 5: Application Layer - State Transitions

Add actual state transitions to handlers:

- `ReleaseCommandHandler`: `Configured` → `Releasing` → `Released`
- `RunCommandHandler`: `Released` → `Starting` → `Running`

**Note**: Most state types already exist in `src/domain/environment/state/`. We may need to add the `Starting` and `StartFailed` states if they don't exist yet.

**Deliverable**: Commands change environment state correctly (E2E testable).

**Manual E2E Test**:

```bash
# Setup: Create and configure environment
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config

# Verify state is Configured
cat build/e2e-config/state.json
# Expected: "state": "Configured"

# Release should transition state
cargo run -- release e2e-config

# Verify state changed to Released
cat build/e2e-config/state.json
# Expected: "state": "Released"

# Run should transition state
cargo run -- run e2e-config

# Verify state changed to Running
cat build/e2e-config/state.json
# Expected: "state": "Running"

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 6: Steps Layer - First Step (Prepare Compose Files)

**Location**: `src/application/steps/application/`

Create first step:

- `prepare_compose_files.rs` - Copy static docker-compose.yml to build directory

**Deliverable**: `release` command generates files in build dir (E2E verifiable).

**Manual E2E Test**:

```bash
# Setup: Create and configure environment
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config

# Release should now generate docker-compose files
cargo run -- release e2e-config

# Verify docker-compose.yml was created in build directory
cat build/e2e-config/docker-compose/docker-compose.yml
# Expected: Valid docker-compose.yml with demo-app service

# Verify file contents match source
diff data/e2e-config/templates/docker-compose/docker-compose.yml build/e2e-config/docker-compose/docker-compose.yml
# Expected: No differences (or expected differences)

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 7: Infrastructure Layer - Docker Compose File Manager

**Location**: `src/infrastructure/external_tools/docker_compose/`

Create `DockerComposeFileManager`:

- Copy static `docker-compose.yml` to build directory
- (Future: generate `.env` file)

**Deliverable**: Build directory contains correct docker-compose.yml.

#### Docker Compose Source Files

**Location**: `data/{environment}/templates/docker-compose/` (following existing pattern for environment-specific templates)

For this demo slice, we use a **simple long-running service** that better emulates real application behavior (unlike `hello-world` which exits immediately):

```yaml
# docker-compose.yml (static file)
services:
  demo-app:
    image: nginx:alpine
    ports:
      - "8080:80"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 5s
```

> **Why nginx instead of hello-world?**
>
> The `hello-world` Docker image prints a message and exits immediately. This doesn't
> accurately test our deployment pipeline because:
>
> - We can't verify the service stays running
> - We can't test health checks
> - It doesn't emulate real services like the Torrust Tracker
>
> Using `nginx:alpine` gives us a lightweight, long-running service with a health check
> that better validates the full pipeline.

#### Design Decision: Static vs Dynamic Templates

Docker Compose supports environment variable substitution natively:

- **`.env` file** - Docker Compose auto-loads from same directory
- **`${VAR:-default}` syntax** - Variable substitution in compose files
- **`--env-file` flag** - Pass env file at runtime

This means we likely don't need Tera templates for compose files. Instead:

1. **Static `docker-compose.yml`** - Copied as-is to VM
2. **Generated `.env` file** - Created from environment config (if needed)
3. **Environment variables** - Passed via `docker compose --env-file` or shell

**MVP Approach**: For this demo slice, no `.env` file is needed. For future slices (tracker, MySQL), we'll use the `.env` file approach to reproduce what already works in the live [torrust-demo](https://github.com/torrust/torrust-demo). This is the simplest path forward.

**Future Consideration**: For enhanced security, we could switch to runtime variable injection via `docker compose up --env-file` or shell environment variables, avoiding secrets stored in files on the VM.

**Naming Convention**: Following the existing patterns (`AnsibleTemplateRenderer`, `TofuTemplateRenderer`), we'll start with `DockerComposeFileManager` for now. These renderer classes manage **all templates** for their respective tools, not single files. If we later need Tera template resolution for docker-compose files (e.g., dynamic service definitions), we can rename to `DockerComposeTemplateRenderer`.

**Manual E2E Test**:

```bash
# Same as Phase 5 - verify file manager correctly copies files
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config
cargo run -- release e2e-config

# Verify docker-compose.yml structure and content
cat build/e2e-config/docker-compose/docker-compose.yml

# Verify it's valid YAML
python3 -c "import yaml; yaml.safe_load(open('build/e2e-config/docker-compose/docker-compose.yml'))"
# Expected: No errors

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 8: Steps Layer - Transfer Files to VM

**Location**: `src/application/steps/application/`

Create step:

- `transfer_files.rs` - Transfer release files to VM via SSH

**Location**: `src/adapters/ssh/` or `src/infrastructure/remote_actions/`

Add file transfer capability:

- SCP-based file transfer using SSH client
- Create target directories on VM
- Handle permissions

**Target directory on VM**: `/opt/torrust/` (configurable)

**Deliverable**: `release` command copies files to VM (E2E verifiable via SSH).

**Manual E2E Test**:

```bash
# Setup: Full pipeline to configured state
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config

# Release should now transfer files to VM
cargo run -- release e2e-config

# Get VM IP from tofu output
VM_IP=$(cd build/e2e-config/tofu && tofu output -raw instance_ip)

# SSH into VM and verify files were transferred
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/docker-compose.yml"
# Expected: Contents of docker-compose.yml

# Verify directory structure on VM
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "ls -la /opt/torrust/"
# Expected: docker-compose.yml present

# Verify file permissions
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "stat /opt/torrust/docker-compose.yml"
# Expected: Appropriate permissions for docker compose

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 9: Steps Layer - Start Services

**Location**: `src/application/steps/application/`

Create steps:

- `start_services.rs` - Execute `docker compose up -d` on VM
- `verify_services.rs` - Check that containers are running

**Deliverable**: `run` command starts containers on VM (E2E verifiable).

**Manual E2E Test**:

```bash
# Setup: Full pipeline to released state
cargo run -- create environment --env-file envs/e2e-config.json
cargo run -- provision e2e-config
cargo run -- configure e2e-config
cargo run -- release e2e-config

# Run should start docker compose services
cargo run -- run e2e-config

# Get VM IP
VM_IP=$(cd build/e2e-config/tofu && tofu output -raw instance_ip)

# Verify containers are running on VM
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust && docker compose ps"
# Expected: demo-app service listed as "running" (healthy)

# Verify service is accessible
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "curl -s http://localhost:8080"
# Expected: nginx welcome page HTML

# Check container health status
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust && docker compose ps --format json | jq '.Health'"
# Expected: "healthy"

# Verify docker compose can be stopped/started
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust && docker compose down"
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust && docker compose up -d"
# Expected: No errors, service comes back up

# Cleanup
cargo run -- destroy e2e-config
```

### Phase 10: E2E Test Coverage

- Extend E2E tests to cover full release and run workflow
- Test full pipeline: create → provision → configure → release → run
- Verify demo-app container runs successfully and is healthy

**Deliverable**: Complete E2E test suite for new commands.

**Manual E2E Test - Full Pipeline**:

```bash
# Complete end-to-end test of entire pipeline
echo "=== Phase 9: Full Pipeline E2E Test ==="

# Step 1: Create environment
echo "Creating environment..."
cargo run -- create environment --env-file envs/e2e-config.json

# Step 2: Provision VM
echo "Provisioning VM..."
cargo run -- provision e2e-config

# Step 3: Configure VM
echo "Configuring VM..."
cargo run -- configure e2e-config

# Step 4: Release application
echo "Releasing application..."
cargo run -- release e2e-config

# Step 5: Run application
echo "Running application..."
cargo run -- run e2e-config

# Step 6: Verify everything works
echo "Verifying deployment..."
VM_IP=$(cd build/e2e-config/tofu && tofu output -raw instance_ip)

# Check state file shows Running
echo "Checking state..."
cat build/e2e-config/state.json | grep -q '"state": "Running"' && echo "✓ State is Running" || echo "✗ State check failed"

# Check files on VM
echo "Checking files on VM..."
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "test -f /opt/torrust/docker-compose.yml" && echo "✓ docker-compose.yml exists" || echo "✗ File check failed"

# Check containers are running
echo "Checking containers..."
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust && docker compose ps"

# Check service is healthy and accessible
echo "Checking service health..."
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "curl -sf http://localhost:8080 > /dev/null" && echo "✓ Service is accessible" || echo "✗ Service check failed"

# Step 7: Cleanup
echo "Cleaning up..."
cargo run -- destroy e2e-config

echo "=== E2E Test Complete ==="
```

**Automated E2E Test Integration**:

E2E tests serve as the safety net throughout all implementation phases. We update them incrementally as we implement each phase.

### E2E Test Strategy

We have three E2E test binaries with different purposes:

| Binary                            | Environment    | Purpose                                                         |
| --------------------------------- | -------------- | --------------------------------------------------------------- |
| `e2e_provision_and_destroy_tests` | GitHub runners | Tests provisioning commands (requires LXD but works in CI)      |
| `e2e_config_and_release_tests`    | GitHub runners | Tests config, release, and run commands (uses Docker container) |
| `e2e_tests_full`                  | Local only     | Full pipeline test (requires VM network connectivity)           |

### Changes Required

**1. Rename `e2e_config_tests.rs` → `e2e_config_and_release_tests.rs`**

The existing config tests use a Docker container that simulates a pre-configured VM. We extend this to also test `release` and `run` commands using the same container.

```bash
# Rename the file
git mv src/bin/e2e_config_tests.rs src/bin/e2e_config_and_release_tests.rs
```

Update `Cargo.toml`:

```toml
[[bin]]
name = "e2e-config-and-release-tests"
path = "src/bin/e2e_config_and_release_tests.rs"
```

**2. Update `e2e_config_and_release_tests.rs`**

Add tests for `release` and `run` commands:

```rust
// Phase 1: Test CLI commands exist (no-op)
fn test_release_command_exists() {
    // cargo run -- release --help should work
}

fn test_run_command_exists() {
    // cargo run -- run --help should work
}

// Phase 3+: Test with configured container
fn test_release_command_on_configured_environment() {
    // Setup: Use existing Docker container with Configured state
    // Action: cargo run -- release e2e-config
    // Verify: State transitions, files generated
}

fn test_run_command_on_released_environment() {
    // Setup: Environment in Released state
    // Action: cargo run -- run e2e-config
    // Verify: State transitions, containers started
}
```

**3. Update `e2e_tests_full.rs`**

Extend the full pipeline test to include release and run:

```rust
fn test_full_pipeline_with_release_and_run() {
    // create → provision → configure → release → run → destroy

    // ... existing setup ...

    // Release
    run_command("release", &env_name);
    assert_state(&env_name, "Released");

    // Run
    run_command("run", &env_name);
    assert_state(&env_name, "Running");

    // Verify service is accessible
    verify_service_health(&vm_ip, 8080);

    // ... cleanup ...
}
```

### Incremental Test Updates

As we implement each phase, we update tests accordingly:

| Phase     | Test Updates                                                 |
| --------- | ------------------------------------------------------------ |
| Phase 1   | Add `test_release_command_exists`, `test_run_command_exists` |
| Phase 2   | Verify tests run with renamed file, pre-commit works         |
| Phase 3   | Verify controllers are wired (commands don't error)          |
| Phase 4   | Add state validation tests                                   |
| Phase 5   | Verify state transitions in `state.json`                     |
| Phase 6-7 | Verify files generated in build directory                    |
| Phase 8   | Verify files transferred to container/VM                     |
| Phase 9   | Verify containers running, service accessible                |
| Phase 10  | Full integration with all verifications                      |

### Pre-commit Hook Update

Update `scripts/pre-commit.sh` to run the renamed test:

```bash
# Change from:
cargo run --bin e2e-config-tests

# To:
cargo run --bin e2e-config-and-release-tests
```

## Technical Specifications

### Docker Compose Deployment Path

- **VM deployment directory**: `/opt/torrust/`
- **Docker compose file**: `/opt/torrust/docker-compose.yml`

### File Transfer Method

Use SSH with `scp` command via `SshClient`:

```rust
// Pseudo-code for file transfer step
ssh_client.execute(&format!(
    "mkdir -p /opt/torrust && cat > /opt/torrust/docker-compose.yml << 'EOF'\n{}\nEOF",
    compose_content
))
```

### Docker Compose Commands

- **Start**: `cd /opt/torrust && docker compose up -d`
- **Status**: `cd /opt/torrust && docker compose ps`
- **Stop**: `cd /opt/torrust && docker compose down`

### Error Handling

All errors must:

- Implement `.help()` with actionable guidance
- Include trace context for debugging
- Transition environment to appropriate failed state

## Acceptance Criteria

### Functional Requirements

- [ ] `release` command copies docker-compose.yml to configured VM
- [ ] `run` command starts docker-compose services on VM
- [ ] Demo-app (nginx) container runs successfully and is healthy
- [ ] Environment state transitions correctly through the lifecycle
- [ ] Failed operations transition to appropriate failed states

### Quality Requirements

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Unit tests for code that doesn't require infrastructure (pure logic, error types, etc.)
- [ ] E2E tests validate full pipeline (primary testing strategy for infrastructure code)
- [ ] Error messages are clear with actionable `.help()`
- [ ] Documentation is updated

> **Note on Testing Strategy**: For infrastructure projects, E2E tests are the primary safety net.
> Unit tests are written where practical (pure functions, error handling, state transitions)
> but not for code that requires complex scaffolding or real infrastructure.

### Architecture Requirements

- [ ] Follows DDD layer separation
- [ ] Follows three-level architecture (Commands → Steps → Actions)
- [ ] Uses existing patterns from configure/provision commands
- [ ] Static file handling follows existing patterns

## Files to Create

### Application Layer

- `src/application/command_handlers/release/mod.rs`
- `src/application/command_handlers/release/handler.rs`
- `src/application/command_handlers/release/errors.rs`
- `src/application/command_handlers/run/mod.rs`
- `src/application/command_handlers/run/handler.rs`
- `src/application/command_handlers/run/errors.rs`

### Steps Layer

- `src/application/steps/application/prepare_compose_files.rs`
- `src/application/steps/application/transfer_files.rs`
- `src/application/steps/application/start_services.rs`
- `src/application/steps/application/verify_services.rs`

### Infrastructure Layer

- `src/infrastructure/external_tools/docker_compose/` (file manager, not template renderer)
- Static docker compose files in `data/{environment}/templates/docker-compose/`
- (Future: `.env` generation if needed)

### Presentation Layer

- `src/presentation/controllers/release/mod.rs`
- `src/presentation/controllers/release/handler.rs`
- `src/presentation/controllers/release/errors.rs`
- `src/presentation/controllers/run/mod.rs`
- `src/presentation/controllers/run/handler.rs`
- `src/presentation/controllers/run/errors.rs`

### Domain Layer (if needed)

- `src/domain/environment/state/starting.rs` (if `Starting` state doesn't exist)
- `src/domain/environment/state/start_failed.rs` (if `StartFailed` state doesn't exist)

### E2E Tests (rename and update)

- `src/bin/e2e_config_tests.rs` → `src/bin/e2e_config_and_release_tests.rs` (rename)
- `src/bin/e2e_tests_full.rs` (update to include release and run)
- Update `Cargo.toml` binary definition
- Update `scripts/pre-commit.sh` to use new test name

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Module Organization](../contributing/module-organization.md)

## Reference Implementation

- [torrust-demo compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml) - Reference docker-compose for future slices
- Existing `ConfigureCommandHandler` - Pattern to follow for handler implementation
- Existing `ProvisionCommandHandler` - Pattern for async operations

## Notes

- This is the scaffolding slice - minimal complexity, validates the pipeline
- Uses nginx as a long-running demo service (instead of hello-world which exits immediately)
- No environment configuration options yet - hardcoded demo-app service
- Future slices will add tracker, MySQL, Prometheus, Grafana services
- File transfer could be enhanced with rsync in future iterations
- May need to add `Starting` and `StartFailed` domain states if they don't exist yet
