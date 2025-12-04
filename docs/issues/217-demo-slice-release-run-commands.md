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
cargo run --bin e2e-config-and-release-tests

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

### Phase 6: Steps Layer - First Step (Prepare Compose Files) ✅ COMPLETE

**Location**: `src/application/steps/application/release.rs`

Implemented as part of the `ReleaseStep`:

- `ReleaseStep::execute()` - Uses `DockerComposeTemplateRenderer` to copy docker-compose.yml to build directory

**Deliverable**: `release` command generates files in build dir (E2E verifiable). ✅

**Manual E2E Test**: ✅ Verified

```bash
# Setup: Create and configure environment
cargo run -- create environment --env-file envs/e2e-full.json
cargo run -- provision e2e-full
cargo run -- configure e2e-full

# Release should now generate docker-compose files
cargo run -- release e2e-full

# Verify docker-compose.yml was created in build directory
cat build/e2e-full/docker-compose/docker-compose.yml
# Expected: Valid docker-compose.yml with demo-app service (nginx:alpine)

# Cleanup
cargo run -- destroy e2e-full
```

### Phase 7: Infrastructure Layer - Docker Compose Template Renderer ✅ COMPLETE

> **Status**: Implemented during Phase 6 as part of the release step integration.
> The `DockerComposeTemplateRenderer` was created following the same pattern as
> `AnsibleTemplateRenderer` and `TofuTemplateRenderer`.

**Location**: `src/infrastructure/external_tools/docker_compose/template/renderer/mod.rs`

Following the established pattern from Ansible and Tofu external tools, we created:

```text
src/infrastructure/external_tools/docker_compose/
├── mod.rs                      # Module exports and DOCKER_COMPOSE_SUBFOLDER constant
└── template/
    ├── mod.rs                  # Template module exports
    └── renderer/
        └── mod.rs              # DockerComposeTemplateRenderer implementation
```

**Created `DockerComposeTemplateRenderer`**:

- Follows the `template/renderer/` folder structure pattern
- Uses `TemplateManager` for on-demand extraction from embedded templates
- Copies static `docker-compose.yml` to build directory
- (Future: generate `.env` file)

**Deliverable**: Build directory contains correct docker-compose.yml. ✅

#### Docker Compose Source Template

**Location**: `templates/docker-compose/docker-compose.yml` (embedded in binary at compile time)

Templates are embedded using rust-embed and extracted on-demand via `TemplateManager`:

1. **Source**: `templates/docker-compose/docker-compose.yml` (embedded at compile time)
2. **Extracted to**: `{env_templates_dir}/docker-compose/docker-compose.yml` (on first use)
3. **Copied to**: `build/{env_name}/docker-compose/docker-compose.yml` (during release)

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

**Naming Convention**: Following the existing patterns (`AnsibleTemplateRenderer`, `TofuTemplateRenderer`), we named it `DockerComposeTemplateRenderer` to maintain consistency. The class handles all template management for Docker Compose files using the embedded template system.

**Manual E2E Test**: ✅ Verified

```bash
# Same as Phase 6 - verify renderer correctly copies files
cargo run -- create environment --env-file envs/e2e-full.json
cargo run -- provision e2e-full
cargo run -- configure e2e-full
cargo run -- release e2e-full

# Verify docker-compose.yml was created in build directory
cat build/e2e-full/docker-compose/docker-compose.yml
# Expected: Valid docker-compose.yml with demo-app service (nginx:alpine)

# Verify it's valid YAML
python3 -c "import yaml; yaml.safe_load(open('build/e2e-full/docker-compose/docker-compose.yml'))"
# Expected: No errors

# Cleanup
cargo run -- destroy e2e-full
```

### Phase 8: Refactor Release Handler to Follow Provision Handler Patterns ✅ COMPLETE

> **Status**: ✅ COMPLETE
>
> The `ReleaseCommandHandler` has been refactored to follow the established patterns from
> `ProvisionCommandHandler`. All components have been implemented and tested.

#### Implementation Summary

The refactoring aligned the release handler with the codebase architecture:

| Aspect                 | Before Refactor                  | After Refactor                           |
| ---------------------- | -------------------------------- | ---------------------------------------- |
| **Repository Type**    | `Arc<dyn EnvironmentRepository>` | `TypedEnvironmentRepository` ✅          |
| **Error Handling**     | Simple `Result`                  | Step tracking with `ReleaseStep` enum ✅ |
| **Failure State**      | No failure state transition      | `ReleaseFailed` with context ✅          |
| **Trace Files**        | None                             | `ReleaseTraceWriter` ✅                  |
| **File Transfer**      | Direct SSH (`sudo tee`)          | Ansible playbook ✅                      |
| **Workflow Structure** | Single `release_step.execute()`  | Multi-phase workflow ✅                  |

#### What Was Implemented

**1. `ReleaseStep` Enum for Step Tracking** ✅

```rust
// src/domain/environment/state/release_failed.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseStep {
    RenderDockerComposeTemplates,
    DeployComposeFiles,
}
```

**2. `ReleaseFailureContext`** ✅

```rust
// src/domain/environment/state/release_failed.rs
pub struct ReleaseFailureContext {
    pub failed_step: ReleaseStep,
    pub error_kind: ErrorKind,
    pub base: BaseFailureContext,
}
```

**3. `TypedEnvironmentRepository`** ✅

```rust
// src/application/command_handlers/release/handler.rs
pub struct ReleaseCommandHandler {
    clock: Arc<dyn Clock>,
    repository: TypedEnvironmentRepository,
}
```

**4. Multi-Phase Workflow** ✅

The handler now executes two distinct steps:

- `RenderDockerComposeTemplatesStep`: Renders templates to build directory
- `DeployComposeFilesStep`: Deploys files to remote via Ansible

**5. `ReleaseTraceWriter`** ✅

```rust
// src/infrastructure/trace/writer/commands/release.rs
pub struct ReleaseTraceWriter {
    traces_dir: PathBuf,
    clock: Arc<dyn Clock>,
}
```

**6. Failure State Transitions** ✅

On failure, the handler:

- Builds `ReleaseFailureContext` with step information
- Writes trace file via `ReleaseTraceWriter`
- Transitions to `ReleaseFailed` state
- Saves state via repository

#### Three-Level Architecture for File Transfer ✅

The implementation follows the codebase architecture (Command → Step → Remote Action):

```text
┌─────────────────────────────────────────────────────────────────┐
│  Level 1: ReleaseCommandHandler                                  │
│  - Orchestrates release workflow                                │
│  - Manages state transitions (Releasing → Released/ReleaseFailed)│
│  - Handles failure context and trace files                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Level 2: Steps                                                 │
│  - RenderDockerComposeTemplatesStep: Render templates to build  │
│  - DeployComposeFilesStep: Execute Ansible playbook             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Level 3: Remote Actions (Ansible Playbook)                     │
│  - deploy-compose-files.yml: Copy files to /opt/torrust/        │
│  - Uses Ansible copy module (idempotent, permissions, backup)   │
└─────────────────────────────────────────────────────────────────┘
```

#### Why Ansible for File Transfer?

| Aspect                 | Direct SSH (`sudo tee`) | Ansible Playbook            |
| ---------------------- | ----------------------- | --------------------------- |
| **Consistency**        | Different pattern       | Matches provision/configure |
| **Idempotency**        | No (overwrites)         | Yes (copy module)           |
| **Permissions**        | Manual chmod            | Built into copy module      |
| **Directory Creation** | Separate mkdir          | Automatic                   |
| **Error Handling**     | Basic success/fail      | Rich changed/ok/failed      |
| **Logging**            | Manual tracing          | Detailed Ansible output     |
| **Extensibility**      | Code changes            | Playbook changes            |
| **Ops-Friendly**       | Rust code only          | Playbook can be inspected   |

#### Ansible Playbook ✅

**Location**: `templates/ansible/deploy-compose-files.yml` (embedded at compile time)

```yaml
---
# Deploy Docker Compose Files Playbook
# Copies the local docker-compose build folder to the remote host

- name: Deploy Docker Compose Files
  hosts: all
  gather_facts: false
  become: true

  vars:
    remote_deploy_dir: /opt/torrust
    local_compose_dir: "{{ compose_files_source_dir }}"

  tasks:
    - name: 📦 Starting Docker Compose files deployment
      ansible.builtin.debug:
        msg: "🚀 Deploying Docker Compose files to {{ inventory_hostname }}:{{ remote_deploy_dir }}"

    - name: Ensure remote deployment directory exists
      ansible.builtin.file:
        path: "{{ remote_deploy_dir }}"
        state: directory
        mode: "0755"
        owner: root
        group: root

    - name: Copy Docker Compose files to remote host
      ansible.builtin.copy:
        src: "{{ local_compose_dir }}/"
        dest: "{{ remote_deploy_dir }}/"
        mode: "0644"
        directory_mode: "0755"
        owner: root
        group: root

    - name: Verify docker-compose.yml exists on remote
      ansible.builtin.stat:
        path: "{{ remote_deploy_dir }}/docker-compose.yml"
      register: compose_file_check

    - name: Fail if docker-compose.yml was not deployed
      ansible.builtin.fail:
        msg: "docker-compose.yml was not found at {{ remote_deploy_dir }}/docker-compose.yml after deployment"
      when: not compose_file_check.stat.exists

    - name: Display deployment summary
      ansible.builtin.debug:
        msg: |
          ✅ Docker Compose files deployed successfully!
          📁 Destination: {{ remote_deploy_dir }}
          📄 Files deployed: {{ deployed_files.files | length }}
```

#### `DeployComposeFilesStep` ✅

**Location**: `src/application/steps/application/deploy_compose_files.rs`

```rust
/// Step that deploys Docker Compose files to a remote host via Ansible
///
/// This step uses the `deploy-compose-files.yml` playbook to:
/// - Create the deployment directory on the remote host
/// - Copy docker-compose.yml with proper permissions
/// - Verify successful deployment
pub struct DeployComposeFilesStep {
    ansible_client: Arc<AnsibleClient>,
    compose_build_dir: PathBuf,
}
```

#### Files Created/Modified ✅

**New Files Created**:

- `src/domain/environment/state/release_failed.rs` ✅ - `ReleaseFailed` state, `ReleaseStep` enum, and `ReleaseFailureContext`
- `src/infrastructure/trace/writer/commands/release.rs` ✅ - `ReleaseTraceWriter`
- `src/application/steps/application/deploy_compose_files.rs` ✅ - `DeployComposeFilesStep`
- `src/application/steps/rendering/docker_compose_templates.rs` ✅ - `RenderDockerComposeTemplatesStep`
- `templates/ansible/deploy-compose-files.yml` ✅ - Ansible playbook

**Modified Files**:

- `src/application/command_handlers/release/handler.rs` ✅ - Refactored to match provision patterns
- `src/application/command_handlers/release/errors.rs` ✅ - Added step-aware errors
- `src/domain/environment/state/mod.rs` ✅ - Export new state types
- `src/infrastructure/external_tools/ansible/template/renderer/mod.rs` ✅ - Registered new playbook

**Removed Files** (deprecated code cleanup):

- `src/application/steps/application/release.rs` ❌ - Removed deprecated `ReleaseStep` class
- `src/adapters/ssh/client.rs` - Removed `write_remote_file` method (no longer needed)

#### Deliverables ✅

All deliverables completed:

1. ✅ `ReleaseCommandHandler` follows the same patterns as `ProvisionCommandHandler`
2. ✅ File transfer uses Ansible (consistent with configure command)
3. ✅ Failure states are properly tracked with `ReleaseFailed` and trace files
4. ✅ Step tracking enables precise error reporting
5. ✅ Deprecated code removed (`write_remote_file`, old `ReleaseStep` class)

**Manual E2E Test**: ✅ Verified

```bash
# Setup: Full pipeline to configured state
cargo run -- create environment --env-file envs/e2e-full.json
cargo run -- provision e2e-full
cargo run -- configure e2e-full

# Release should transfer files via Ansible
cargo run -- release e2e-full

# Get VM IP
VM_IP=$(cd build/e2e-full/tofu && tofu output -raw instance_ip)

# Verify files were transferred
ssh -i fixtures/testing_rsa torrust@$VM_IP "cat /opt/torrust/docker-compose.yml"
# Expected: Contents of docker-compose.yml with nginx:alpine service ✅

# Cleanup
cargo run -- destroy e2e-full
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
cargo run --bin e2e-config-and-release-tests

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
- `src/application/command_handlers/release/handler.rs` - Refactor to match provision patterns
- `src/application/command_handlers/release/errors.rs` - Add step-aware errors
- `src/application/command_handlers/run/mod.rs`
- `src/application/command_handlers/run/handler.rs`
- `src/application/command_handlers/run/errors.rs`

### Steps Layer

- `src/application/steps/application/deploy_compose_files.rs` ✅ (`DeployComposeFilesStep` - deploys via Ansible)
- `src/application/steps/rendering/docker_compose_templates.rs` ✅ (`RenderDockerComposeTemplatesStep` - renders templates)
- `src/application/steps/application/start_services.rs` (future - start docker compose services)
- `src/application/steps/application/verify_services.rs` (future - verify services are running)

### Infrastructure Layer

- `src/infrastructure/external_tools/docker_compose/mod.rs` ✅ (module exports and `DOCKER_COMPOSE_SUBFOLDER` constant)
- `src/infrastructure/external_tools/docker_compose/template/mod.rs` ✅ (template module exports)
- `src/infrastructure/external_tools/docker_compose/template/renderer/mod.rs` ✅ (`DockerComposeTemplateRenderer` implementation)
- `src/infrastructure/trace/writer/commands/release.rs` ✅ (`ReleaseTraceWriter` for failure trace files)
- `templates/docker-compose/docker-compose.yml` ✅ (embedded source template with nginx:alpine demo service)
- `templates/ansible/deploy-compose-files.yml` ✅ (Ansible playbook for file transfer)
- `docker/ssh-server/Dockerfile` ✅ (updated with passwordless sudo for testing)

### Presentation Layer

- `src/presentation/controllers/release/mod.rs` ✅
- `src/presentation/controllers/release/errors.rs` ✅
- `src/presentation/controllers/run/mod.rs` ✅
- `src/presentation/controllers/run/errors.rs` ✅

### Domain Layer

- `src/domain/environment/state/release_failed.rs` ✅ (`ReleaseFailed` state, `ReleaseStep` enum, and `ReleaseFailureContext`)
- `src/domain/environment/state/releasing.rs` ✅ (state transitions)
- `src/domain/environment/state/released.rs` ✅ (released state)
- `src/domain/environment/state/running.rs` ✅ (running state)
- `src/domain/environment/state/run_failed.rs` ✅ (run failed state)

### E2E Tests (rename and update)

- `src/bin/e2e_config_tests.rs` → `src/bin/e2e_config_and_release_tests.rs` ✅ (renamed)
- `src/bin/e2e_tests_full.rs` (update to include release and run)
- Update `Cargo.toml` binary definition ✅
- Update `scripts/pre-commit.sh` to use new test name ✅

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - Three-level architecture (Command → Step → Remote Action)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Module Organization](../contributing/module-organization.md)

## Reference Implementation

- **`ProvisionCommandHandler`** - Primary pattern to follow for handler implementation:
  - `TypedEnvironmentRepository` for typed save methods
  - `StepResult<T, E, S>` for step tracking
  - `ProvisionFailureContext` and `ProvisionTraceWriter` for failure handling
  - Multi-phase workflow with dedicated methods
- **`ConfigureCommandHandler`** - Pattern for Ansible playbook execution
- **`WaitForCloudInitStep`** - Example of step wrapping Ansible playbook
- [torrust-demo compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml) - Reference docker-compose for future slices
- Existing `ProvisionCommandHandler` - Pattern for async operations

## Notes

- This is the scaffolding slice - minimal complexity, validates the pipeline
- Uses nginx as a long-running demo service (instead of hello-world which exits immediately)
- No environment configuration options yet - hardcoded demo-app service
- Future slices will add tracker, MySQL, Prometheus, Grafana services
- File transfer could be enhanced with rsync in future iterations
- May need to add `Starting` and `StartFailed` domain states if they don't exist yet
