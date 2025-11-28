# Register Existing Instances Command

**Issue**: [#203](https://github.com/torrust/torrust-tracker-deployer/issues/203)
**Parent Epic**: #1 - Roadmap
**Related**: [docs/features/import-existing-instances/specification.md](../features/import-existing-instances/specification.md)

## Overview

Add a new `torrust-tracker-deployer register` command that serves as an **alternative to the `provision` command** for environments that already have existing infrastructure. Instead of provisioning new infrastructure, `register` allows users to provide the IP address of an existing instance.

This solves two critical needs:

1. **For End Users**: Deploy on existing infrastructure (spare servers, unsupported cloud providers, custom setups)
2. **For E2E Testing**: Replace the hacky `run_provision_simulation.rs` workaround with a proper command for container-based tests

### Key Insight

The `create environment` command creates the environment _concept_ (SSH credentials, name, configuration) - not the actual infrastructure. The infrastructure is either:

- **Provisioned** via the `provision` command (creates new VMs)
- **Registered** via the `register` command (uses existing infrastructure)

Both paths lead to the same `Provisioned` state, and the only runtime output from provisioning stored is the instance IP (see `src/domain/environment/runtime_outputs.rs`). Therefore, `register` only needs the instance IP as input.

### Workflow Comparison

```text
Standard Workflow:
  create environment → provision → configure → release → run
       |                  |
    [Created]        [Provisioned]
                          ↓
                     (continues...)

Register Workflow (Alternative to provision):
  create environment → register → configure → release → run
       |                  |
    [Created]        [Provisioned]
                          ↓
                     (continues...)
```

## Goals

- [x] Enable users to register existing VMs, physical servers, or containers as an alternative to provisioning
- [x] Transition environments from `Created` to `Provisioned` state by providing instance IP
- [x] Replace `run_provision_simulation.rs` hack in E2E tests with proper `register` command
- [x] Validate SSH connectivity (minimal validation for v1)
- [x] Mark registered environments with metadata for safety (prevent accidental destroy)

## 🏗️ Architecture Requirements

**DDD Layer**: Application (command handler), Domain (state transition), Presentation (CLI), Infrastructure (SSH validation)
**Module Path**: `src/application/commands/register/`, `src/presentation/input/cli/`
**Pattern**: Command Handler with Three-Level Pattern (Command orchestrates Steps)

### Module Structure Requirements

- [x] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [x] Respect dependency flow rules (dependencies flow toward domain)
- [x] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [x] Follow Three-Level Pattern: Commands → Steps → Actions

### Architectural Constraints

- [x] No business logic in presentation layer (CLI only parses arguments and delegates to application layer)
- [x] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [x] Errors must be clear, actionable, and include fix instructions
- [x] Testing strategy aligns with layer responsibilities
- [x] `register` operates on environments in `Created` state (requires prior `create environment`)

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Domain layer depending on infrastructure
- ❌ Monolithic modules with multiple responsibilities
- ❌ Using anyhow for errors where explicit enum errors are better
- ❌ Duplicating SSH credential handling already done in `create environment`

## Specifications

### Prerequisites

Before running `register`, users must first create the environment:

```bash
# Step 1: Create the environment (provides SSH credentials, name, etc.)
torrust-tracker-deployer create environment --env-file config.json

# Step 2: Register existing instance (provides only the instance IP)
torrust-tracker-deployer register <environment-name> --instance-ip <IP_ADDRESS>
```

### Command Syntax

```bash
torrust-tracker-deployer register <environment-name> --instance-ip <IP_ADDRESS>
```

### Arguments

- `<environment-name>` - Name of an existing environment in `Created` state
- `--instance-ip <IP_ADDRESS>` - IP address of the existing instance (IPv4 or IPv6)

### Example Usage

```bash
# First, create the environment with SSH credentials
torrust-tracker-deployer create environment --env-file prod-config.json

# Then, register the existing production server
torrust-tracker-deployer register production-tracker --instance-ip 192.168.1.100

# Continue with normal workflow
torrust-tracker-deployer configure production-tracker
torrust-tracker-deployer release production-tracker
torrust-tracker-deployer run production-tracker
```

### Instance Requirements (Documented in Help Text)

The existing instance must meet these requirements (same as provisioned instances):

**REQUIRED**:

- Ubuntu 24.04 LTS
- SSH connectivity with credentials from `create environment`
- Public SSH key installed for access
- Public IP address reachable from deployer
- Username with sudo access
- Cloud-init completion mark (see `templates/tofu/lxd/cloud-init.yml.tera`)

**MUST NOT HAVE**:

- Incompatible dependencies (old Docker, old systemd, etc.)
- Custom configurations preventing deployer operation

### Validation Strategy (v1 - Minimal)

- **Only SSH connectivity validation** - connect using credentials from environment and verify authentication works
- Environment transitions to `Provisioned` even if validation fails (with warning to user)
- Subsequent commands (`configure`, `release`, `run`) will fail with clear errors if requirements not met
- Future v2 may add advanced validation (OS version, architecture, disk space, memory)

### State Transition

```rust
// register command transitions environment from Created to Provisioned
Environment<Created> → Environment<Provisioned>
```

The `register` command:

1. Loads existing environment in `Created` state
2. Validates SSH connectivity using credentials already in the environment
3. Sets `runtime_outputs.instance_ip` to the provided IP address
4. Adds `"registered": "true"` metadata
5. Transitions to `Provisioned` state

### Metadata and Safety

- Registered environments marked with `"registered": "true"` metadata
- Destroy command will require explicit confirmation for registered instances (prevents accidental data loss)
- Environment must exist and be in `Created` state

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("Environment '{name}' not found")]
    EnvironmentNotFound {
        name: EnvironmentName,
    },

    #[error("Environment '{name}' is not in Created state (current: {current_state})")]
    InvalidState {
        name: EnvironmentName,
        current_state: String,
    },

    #[error("Failed to connect to instance at {address}")]
    ConnectivityFailed {
        address: IpAddr,
        #[source]
        source: SshError,
    },

    #[error("Invalid IP address: {value}")]
    InvalidIpAddress {
        value: String,
    },

    #[error("Failed to save environment")]
    RepositorySave {
        #[source]
        source: RepositoryError,
    },
}
```

## Implementation Plan

### Phase 1: Domain Layer (0.5 day)

- [x] Add state transition method `Environment<Created>::register(instance_ip)` returning `Environment<Provisioned>`
- [x] Set `runtime_outputs.instance_ip` during transition
- [x] Support metadata parameter for "registered" flag
- [x] Add unit tests for state transition

### Phase 2: Application Layer - Command Handler (1 day)

- [x] Create `src/application/commands/register/mod.rs` module structure
- [x] Create `src/application/commands/register/handler.rs` with `RegisterCommandHandler`
- [x] Create `src/application/commands/register/error.rs` with `RegisterError` enum
- [x] Load existing environment and verify `Created` state
- [x] Implement SSH connectivity validation using environment's SSH credentials
- [x] Handle validation failures gracefully (transition anyway, warn user)
- [x] Add metadata tracking for registered instances
- [x] Add unit tests for command handler logic

### Phase 3: Presentation Layer - CLI (0.5 day)

- [x] Add `Register` variant to CLI commands enum in `src/presentation/input/cli/commands.rs`
- [x] Add argument parsing (environment name, instance-ip)
- [x] Add detailed help text documenting prerequisites and instance requirements
- [x] Wire up command handler execution in `src/presentation/input/cli/mod.rs`
- [x] Add user output formatting for success/warning/error cases
- [x] Add CLI integration tests

### Phase 4: E2E Test Migration (2 days)

- [x] Refactor `src/bin/e2e_config_tests.rs` to be a true black-box test (like `e2e_provision_and_destroy_tests.rs`)
- [x] Use `register` command instead of `run_provision_simulation` in the refactored E2E test
- [x] Update `src/testing/e2e/tasks/container/mod.rs` to support register workflow
- [x] Remove `run_provision_simulation.rs` after migration complete
- [x] Verify all E2E tests pass on GitHub Actions
- [x] Manual test: Register LXD VM successfully (cross-environment technique, see `docs/e2e-testing.md`)
- [x] Manual test: Register Docker container successfully

**Note**: `tests/e2e_create_command.rs` and `tests/e2e_destroy_command.rs` do NOT need updates - they test create/destroy commands without provisioning. Only `src/bin/e2e_config_tests.rs` uses `run_provision_simulation` and needs migration.

### Phase 5: Documentation (1 day)

- [x] Create `docs/user-guide/commands/register.md` with examples and troubleshooting
- [x] Update `docs/console-commands.md` with register command
- [x] Update any other documentation listing available commands
- [x] Create ADR `docs/decisions/register-existing-instances.md`
- [x] Update `docs/features/import-existing-instances/README.md` status to complete

**Total Estimated Duration**: 5 days

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Criteria**:

- [x] Command requires environment to exist in `Created` state
- [x] Command only requires `--instance-ip` parameter (SSH credentials come from environment)
- [x] Environment transitions to `Provisioned` state with `runtime_outputs.instance_ip` set
- [x] Registered environments marked with `provision_method: Registered` metadata
- [x] SSH connectivity validated using environment's SSH credentials
- [x] Environment transitions even if validation fails (with warning to user)
- [x] Clear error messages for environment not found
- [x] Clear error messages for wrong environment state
- [x] Clear error messages for connectivity failures
- [x] Manual test: Successfully register LXD VM (cross-environment technique)
- [x] Manual test: Successfully register Docker container

**Testing Criteria**:

- [x] Unit tests cover `RegisterCommandHandler` logic
- [x] Unit tests verify state transition from `Created` to `Provisioned`
- [x] Unit tests verify metadata tracking
- [x] Integration tests verify SSH connectivity validation
- [x] Integration tests verify environment repository integration
- [x] `src/bin/e2e_config_tests.rs` refactored to black-box test using register command
- [x] All E2E tests pass on GitHub Actions
- [x] `run_provision_simulation.rs` removed from codebase

**Documentation Criteria**:

- [x] User-facing documentation in `docs/user-guide/commands/register.md`
- [x] ADR in `docs/decisions/register-existing-instances.md`
- [x] `docs/console-commands.md` updated
- [x] Prerequisites (create environment first) documented clearly
- [x] Instance requirements documented in command help text
- [x] Feature marked as complete in `docs/features/import-existing-instances/README.md`

## Related Documentation

- [Feature Specification](../features/import-existing-instances/specification.md) - Full feature specification with Q&A
- [Feature Questions & Answers](../features/import-existing-instances/questions.md) - All clarifying questions answered
- [Decisions Summary](../features/import-existing-instances/decisions-summary.md) - Key decisions made
- [Runtime Outputs](../../src/domain/environment/runtime_outputs.rs) - Shows instance_ip is the only provisioning output
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Where to place code
- [Error Handling](../contributing/error-handling.md) - Error handling conventions
- [Codebase Architecture](../codebase-architecture.md) - Three-level pattern documentation
- [Development Principles](../development-principles.md) - Observability, testability, user friendliness
- [User Guide: Register Command](../user-guide/commands/register.md) - User documentation
- [ADR: Register Existing Instances](../decisions/register-existing-instances.md) - Design decisions

## Notes

### Priority

**HIGH** - This feature must be implemented before the Hetzner provider to simplify E2E tests, making subsequent test changes easier.

### Key Design Decisions

1. **Register as alternative to Provision**: Rather than creating a new environment, `register` transitions an existing `Created` environment to `Provisioned` state - the same as `provision` but using existing infrastructure
2. **Minimal input**: Only `--instance-ip` required because SSH credentials and other configuration already exist from `create environment`
3. **Command Name**: `register` chosen based on industry precedent (GitHub/GitLab runners, Consul, Vault)
4. **State Management**: Transitions `Created` → `Provisioned` (same target state as `provision`)
5. **Validation**: Minimal for v1 (SSH connectivity only), advanced validation deferred to v2
6. **Safety**: Registered instances marked with metadata, destroy command will require confirmation

### Benefits of This Approach

1. **No duplication** - SSH credentials, environment name, etc. are set once in `create environment`
2. **Simple command** - Only needs the instance IP (the runtime output from provisioning)
3. **Consistent model** - `register` is just another way to populate `RuntimeOutputs`
4. **Clean separation** - "Creating the concept" vs "materializing infrastructure"
5. **Extensible** - If `RuntimeOutputs` grows, `register` parameters grow accordingly

### Future Enhancements (Out of Scope)

- Advanced validation (OS version, architecture, disk space, memory) - v2
- Cloud provider registration (Hetzner instance ID) - after Hetzner provider
- Re-register command to remove metadata without destroying instance
- ProvisionFailed state for validation failures
