# Implement Environment Show Command

**Issue**: #241
**Parent Epic**: N/A (Part of Roadmap Section 5: "Add extra console app commands")
**Related**:

- Roadmap #1 - Task 5.1
- Feature Specification: [docs/features/environment-status-command/](../features/environment-status-command/)

## Overview

Implement a new console command `show` that displays environment information with state-aware details. The command provides a read-only view of stored environment data without remote verification, making it fast and reliable for users to inspect their deployment state.

This command displays different information based on the environment's current state (Created, Provisioned, Configured, Released, Running) and provides next-step guidance to help users understand what actions to take.

## Goals

- [x] Provide console command to display environment information
- [x] Show state-aware details (different information per state)
- [x] Use human-friendly output formatting with UserOutput
- [x] Handle errors gracefully with actionable messages
- [x] Maintain fast execution (< 100ms for typical environments)
- [x] Ensure comprehensive test coverage (unit + E2E)

## 🏗️ Architecture Requirements

**DDD Layers**: Application + Presentation

**Module Paths**:

- `src/application/commands/show/` (command handler, DTOs, errors, formatter)
- `src/presentation/input/cli/subcommands/show.rs` (CLI interface)

**Patterns**:

- Application Layer: Command Handler pattern
- Presentation Layer: CLI Subcommand pattern
- Data Transfer: DTO pattern (EnvironmentInfo, InfrastructureInfo, ServicesInfo)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Application layer handles business logic (loading, extraction, formatting)
- [ ] Presentation layer handles CLI input/output only
- [ ] No direct infrastructure dependencies (use existing EnvironmentLoader)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Follow import conventions (imports first, prefer imports over full paths)

### Architectural Constraints

- [ ] **CRITICAL**: All output through UserOutput - **NEVER use `println!`, `eprintln!`, or direct stdio** (see [docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] No business logic in presentation layer (only CLI argument parsing and dispatch)
- [ ] Error handling with explicit enum types (not anyhow) - see [docs/contributing/error-handling.md](../contributing/error-handling.md)
- [ ] Read-only operation - no state modifications
- [ ] No remote network calls (displays stored data only)
- [ ] Testing strategy: unit tests at each phase + E2E test evolution

### Anti-Patterns to Avoid

- ❌ **Direct `println!`/`eprintln!` usage** (must use UserOutput)
- ❌ Business logic in CLI subcommand
- ❌ Using anyhow instead of explicit error enums
- ❌ Remote verification/health checks (out of scope - use `test` command)
- ❌ Batching test completions (mark complete immediately after each phase)

## Specifications

### Command Interface

**Command Name**: `show` (not `status` - reserves that for future service health checks)

**Usage**:

```bash
torrust-tracker-deployer show <environment-name>
```

**Example**:

```bash
torrust-tracker-deployer show my-environment
```

### Output Format by State

#### Created State

```text
Environment: my-environment
State: Created
Provider: LXD

The environment configuration is ready. Run 'provision' to create infrastructure.
```

#### Provisioned State

```text
Environment: my-environment
State: Provisioned
Provider: LXD

Infrastructure:
  Instance IP: 10.140.190.14
  SSH Port: 22
  SSH User: ubuntu
  SSH Key: /home/user/.ssh/torrust_deployer_key

Connection:
  ssh -i /home/user/.ssh/torrust_deployer_key ubuntu@10.140.190.14

Next step: Run 'configure' to set up the system.
```

#### Running State

```text
Environment: my-environment
State: Running
Provider: Hetzner Cloud

Infrastructure:
  Instance IP: 157.10.23.45
  SSH Port: 22
  SSH User: root

Tracker Services:
  UDP Trackers:
    - udp://157.10.23.45:6868/announce
    - udp://157.10.23.45:6969/announce
  HTTP Tracker:
    - http://157.10.23.45:7070/announce
    - Health: http://157.10.23.45:7070/health_check
  API Endpoint:
    - http://157.10.23.45:1212/api
    - Health: http://157.10.23.45:1212/api/health_check

Status: ✓ All services running
```

### Data Transfer Objects

**EnvironmentInfo**:

```rust
pub struct EnvironmentInfo {
    pub name: String,
    pub state: EnvironmentState,
    pub provider: String,
    pub created_at: Option<String>,
    pub infrastructure: Option<InfrastructureInfo>,
    pub services: Option<ServicesInfo>,
    pub next_step: String,
}
```

**InfrastructureInfo**:

```rust
pub struct InfrastructureInfo {
    pub instance_ip: IpAddr,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: String,
}
```

**ServicesInfo**:

```rust
pub struct ServicesInfo {
    pub udp_trackers: Vec<url::Url>,
    pub http_tracker: Option<url::Url>,
    pub http_tracker_health: Option<url::Url>,
    pub api_endpoint: Option<url::Url>,
    pub api_health: Option<url::Url>,
}
```

### Error Handling

**ShowCommandError** enum with explicit variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ShowCommandError {
    #[error("Environment '{0}' not found")]
    EnvironmentNotFound(EnvironmentName),

    #[error("Failed to load environment: {0}")]
    LoadError(#[from] EnvironmentLoadError),

    #[error("Invalid environment state: {0}")]
    InvalidState(String),
}
```

All errors must include actionable messages guiding users on how to resolve the issue.

## Implementation Plan

### Phase 1: Presentation Layer - CLI Skeleton (1-2 hours) ✅

- [x] Create CLI subcommand in `src/presentation/input/cli/subcommands/show.rs`
- [x] Add `Show` variant to `Commands` enum in `src/presentation/input/cli/commands.rs`
- [x] Add command dispatch in `src/presentation/dispatch/mod.rs`
- [x] Add placeholder handler returning "Not implemented" message via UserOutput
- [x] Verify command appears in `--help` output
- [x] Create initial E2E test in `tests/e2e/commands/show_command.rs` (creates environment, runs show)
- [x] Manual CLI testing - command is runnable

**Result**: Command runnable from CLI with placeholder message. E2E test validates basic execution.

### Phase 2: Application Layer Foundation (2-3 hours) ✅

- [x] Create `ShowCommandHandler` in `src/application/commands/show/mod.rs`
- [x] Create `ShowCommandError` enum in `src/application/commands/show/error.rs`
- [x] Create `EnvironmentInfo` DTO in `src/application/commands/show/info.rs`
- [x] Implement environment loading with error handling (environment not found)
- [x] Extract basic info (name, state, provider) from environment
- [x] Display basic information via UserOutput (no println!/eprintln!)
- [x] Comprehensive unit tests for handler logic and error scenarios
- [x] Update E2E test to validate basic info display

**Result**: Command displays environment name, state, and provider.

### Phase 3: State-Aware Information Extraction (3-4 hours) ✅

- [x] Implement state-specific extraction using existing environment data
- [x] Add infrastructure details for Provisioned state (IP, SSH port, SSH user, SSH key path)
- [x] Handle missing runtime_outputs gracefully with clear errors
- [x] Extract service configuration for Running state from tracker config
- [x] Handle all environment states (Created, Configured, Released, Running, etc.)
- [x] Handle invalid/corrupted state data
- [x] Comprehensive unit tests for all states and edge cases
- [x] Update E2E test to validate state-specific details

**Result**: Command shows state-aware information for all environment states.

**Note**: Uses only data already in environment JSON - no new fields yet.

### Phase 4: Output Formatting (2-3 hours) ✅

- [x] Implement output formatter using UserOutput
- [x] Add state-aware formatting for each environment state
- [x] Include next-step guidance based on current state
- [x] Add visual improvements (colors, spacing, structured output)
- [x] Unit tests for formatting logic
- [x] Update E2E test to validate output formatting
- [x] Manual terminal testing

**Result**: Human-friendly, well-formatted output with next-step guidance.

### Phase 5: Testing Strategy Analysis and Documentation (2-3 hours) ✅

- [x] Analyze E2E testing strategies for different states:
  - **Strategy 1**: Call show in existing workflow tests after each state transition
  - **Strategy 2**: Mock internal state in dedicated E2E test
  - **Strategy 3**: Test different states via unit tests only (message formatting)
  - **Decision**: Use combination (Strategy 3 for formatting + Strategy 1 for E2E)
- [x] Implement chosen strategy
- [x] Add E2E tests for error scenarios (missing environment, corrupted data)
- [x] Verify test coverage meets requirements
- [x] Write user documentation in `docs/user-guide/commands/show.md`
- [x] Update `docs/console-commands.md` with show command
- [x] Update `docs/user-guide/commands.md` with command reference
- [x] Update `docs/roadmap.md` to mark task 5.1 as complete

**Result**: Complete test coverage and user documentation.

### Phase 6: Add Creation Timestamp (1-2 hours) ✅

- [x] Add `created_at` field to Environment domain model
- [x] Update environment JSON serialization to include timestamp
- [x] Update `create` command to populate creation timestamp
- [x] Update `show` command to display creation timestamp
- [x] Unit tests for timestamp persistence and display
- [x] Update E2E test to validate timestamp display

**Result**: Show command displays when environment was created.

**Note**: Extends domain model with new field. No backward compatibility needed (early development).

### Phase 7: Add Service URLs to RuntimeOutputs (2-3 hours) ✅

- [x] Add `service_endpoints` field to RuntimeOutputs domain model (`src/domain/environment/runtime_outputs.rs`)
- [x] Define `ServiceEndpoints` struct with `url::Url` fields (not String)
- [x] Update `run` command to populate service URLs after successful startup
- [x] Update `show` command to read from RuntimeOutputs (with fallback to construction)
- [x] Unit tests for ServiceEndpoints persistence and display
- [x] Update E2E test to validate service URLs display

**Result**: Service URLs stored in RuntimeOutputs and displayed in show command.

**Note**: Makes service URLs first-class deployment state, available to all commands.

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
  - All unit tests pass
  - Documentation builds successfully
  - E2E tests pass

**Functional Requirements**:

- [ ] Command displays environment name and state for all state types
- [ ] Command shows IP, SSH port, SSH user, SSH key path for Provisioned+ states
- [ ] Command includes ready-to-use SSH connection command
- [ ] Command shows service URLs for Running state
- [ ] Command provides next-step guidance based on state
- [ ] Command handles missing environment with clear error + suggestion to use `list` command
- [ ] Command handles invalid/corrupted data gracefully
- [ ] Output is human-friendly and easy to read
- [ ] Command execution is fast (< 100ms typical)

**Technical Requirements**:

- [ ] **All output via UserOutput** (no println!/eprintln!)
- [ ] Follows DDD layer placement (Application + Presentation)
- [ ] Error types use explicit enums (not anyhow)
- [ ] Follows module organization conventions (public first, imports at top)
- [ ] Follows import conventions (prefer imports over full paths)
- [ ] Performance acceptable (< 100ms)

**Testing Requirements**:

- [ ] Unit tests cover ShowCommandHandler logic
- [ ] Unit tests cover info extraction for all states
- [ ] Unit tests cover output formatting
- [ ] Unit tests cover all error scenarios and edge cases
- [ ] Unit tests use behavior-driven naming (`it_should_*_when_*`)
- [ ] E2E test validates Created environment
- [ ] E2E test validates Provisioned environment
- [ ] E2E test validates Running environment
- [ ] E2E test validates error handling for missing environment
- [ ] Test coverage includes missing data and invalid states

**Documentation Requirements**:

- [ ] Feature docs complete in `docs/features/environment-status-command/`
- [ ] Show command section added to `docs/console-commands.md`
- [ ] Command reference added to `docs/user-guide/commands.md`
- [ ] Detailed guide created in `docs/user-guide/commands/show.md`
- [ ] Roadmap updated (task 5.1 marked complete)
- [ ] Inline code documentation (doc comments for public APIs)
- [ ] Command help text complete (shows in `--help`)

**Code Quality**:

- [ ] No code duplication (DRY principle)
- [ ] Clear separation of concerns (formatting vs logic vs presentation)
- [ ] Meaningful variable and function names
- [ ] Proper error context with actionable messages
- [ ] Follows behavior-driven test naming (`it_should_*_when_*`, never `test_*`)

## Related Documentation

- **Feature Specification**: [docs/features/environment-status-command/specification.md](../features/environment-status-command/specification.md)
- **Questions & Answers**: [docs/features/environment-status-command/questions.md](../features/environment-status-command/questions.md)
- **Feature Overview**: [docs/features/environment-status-command/README.md](../features/environment-status-command/README.md)
- **Codebase Architecture**: [docs/codebase-architecture.md](../codebase-architecture.md)
- **Output Handling**: [docs/contributing/output-handling.md](../contributing/output-handling.md)
- **Error Handling**: [docs/contributing/error-handling.md](../contributing/error-handling.md)
- **Module Organization**: [docs/contributing/module-organization.md](../contributing/module-organization.md)
- **Unit Testing Conventions**: [docs/contributing/testing/unit-testing.md](../contributing/testing/unit-testing.md)

## Notes

### Key Design Decisions

- **Command Name**: `show` (not `status`) - reserves `status` for future service health checks
- **No Remote Verification**: Displays stored state only - infrastructure verification is in `test` command
- **Clear Command Separation**:
  - `show` - Display stored data (fast, no network)
  - `test` - Verify infrastructure (cloud-init, Docker, Docker Compose)
  - `status` (future) - Check service health (connectivity, health endpoints)
- **Destroyed State Handling**: Show destroyed environments. Since internal state persists after destruction, users who don't want destroyed environments in `list` or `show` should manually remove the internal state. This assumes users won't have many environments (deployment tool use case).
- **Service URLs Storage**: Service endpoints are added to `RuntimeOutputs` after the `run` command succeeds, following the same pattern as `instance_ip` and `provision_method` which are added after `provision`.
- **Next Step Messages**: Hardcoded strings in the formatter (simple approach for now).

### Development Approach

- **Top-Down Development**: Implement presentation layer first for immediate runnability
- **E2E Test Evolution**: Create in Phase 1, update at each phase for continuous validation
- **Unit Tests**: Write at every phase for code added/modified
- **Incremental Progress**: Each phase produces working, testable command with more features

### Future Enhancements

- **JSON Output Format**: Can be added later with `--format json` flag
- **Provider-Specific Details**: LXD container ID/profile, Hetzner server ID/datacenter/type

### Estimated Duration

**Total**: 14-22 hours (2-3 days) for complete implementation

**Target Completion**: January 2026
