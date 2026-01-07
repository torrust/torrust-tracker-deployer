# Implement Environment List Command

**Issue**: #260
**Parent Epic**: N/A (Part of Roadmap Section 5: "Add extra console app commands")
**Related**:

- Roadmap #1 - Task 5.3
- Related: #241 - Show command (similar read-only command pattern)

## Overview

Implement a new console command `list` that displays all environments in the deployment workspace. The command provides a quick overview of all environments with their names and current states, helping users navigate between multiple deployments.

This command scans the data directory and lists all environments with basic information (name, state, provider), making it easy to see all available environments at a glance.

## Goals

- [ ] Provide console command to list all environments
- [ ] Display basic info for each environment (name, state, provider)
- [ ] Handle empty workspace (no environments) gracefully
- [ ] Use human-friendly output formatting with UserOutput
- [ ] Handle errors gracefully with actionable messages
- [ ] Maintain fast execution (< 200ms for typical workspaces)
- [ ] Ensure comprehensive test coverage (unit + E2E)

## 🏗️ Architecture Requirements

**DDD Layers**: Application + Presentation

**Module Paths**:

- `src/application/command_handlers/list/` (command handler, DTOs, errors)
- `src/presentation/controllers/list/` (CLI controller)
- `src/presentation/input/cli/subcommands/list.rs` (CLI interface)

**Patterns**:

- Application Layer: Command Handler pattern
- Presentation Layer: CLI Subcommand + Controller pattern
- Data Transfer: DTO pattern (EnvironmentSummary)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Application layer handles business logic (scanning, loading, extraction)
- [ ] Presentation layer handles CLI input/output only
- [ ] Use existing EnvironmentLoader/RepositoryFactory infrastructure
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Follow import conventions (imports first, prefer imports over full paths)

### Architectural Constraints

- [ ] **CRITICAL**: All output through UserOutput - **NEVER use `println!`, `eprintln!`, or direct stdio** (see [docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] No business logic in presentation layer (only CLI argument parsing and dispatch)
- [ ] Error handling with explicit enum types (not anyhow) - see [docs/contributing/error-handling.md](../contributing/error-handling.md)
- [ ] Read-only operation - no state modifications
- [ ] No remote network calls (scans local data directory only)
- [ ] Testing strategy: unit tests for each component + E2E tests

### Anti-Patterns to Avoid

- ❌ **Direct `println!`/`eprintln!` usage** (must use UserOutput)
- ❌ Business logic in CLI subcommand or controller
- ❌ Using anyhow instead of explicit error enums
- ❌ Hardcoding data directory path (use InternalConfig)
- ❌ Loading full environment data when only summaries are needed

## Specifications

### Command Interface

**Command Name**: `list`

**Usage**:

```bash
torrust-tracker-deployer list [OPTIONS]
```

**Options** (future enhancement):

- `--format <FORMAT>` - Output format: `table` (default), `json`, `simple`
- `--state <STATE>` - Filter by state: `created`, `provisioned`, `configured`, `released`, `running`

**Initial implementation**: No options (always table format, no filtering)

**Sorting**: Environments are displayed in filesystem order (no specific sorting). Users typically manage a small number of environments, so explicit sorting is not required.

**Example**:

```bash
torrust-tracker-deployer list
```

### Output Format

#### Standard List (with environments)

```text
Environments (4 found):

Name                 State          Provider    Created
────────────────────────────────────────────────────────────────────────────
my-production        Running        Hetzner     2026-01-05T10:30:00Z
dev-environment      Provisioned    LXD         2026-01-06T14:15:30Z
test-tracker         Created        LXD         2026-01-07T09:00:12Z
old-staging          Destroyed      LXD         2025-12-20T11:45:00Z
```

**Formatting**:

- **Dates**: ISO 8601 format (`2026-01-05T10:30:00Z`)
- **States**: Includes all states, including `Destroyed`
- **Sorting**: Filesystem order (no specific sorting)
- **Wrapping**: No truncation or wrapping (terminal handles display)

#### Empty Workspace (no environments)

```text
No environments found in: /path/to/workspace/data

To create a new environment:
  torrust-tracker-deployer create environment --env-file <config.json>

For more information, see docs/user-guide/commands.md
```

**Behavior**:

- Display data directory path for verification
- Exit code: 0 (success - empty list is a valid state)

#### Corrupted Data Handling

When some environments have corrupted state files, display valid environments and list errors at the end:

```text
Environments (2 found):

Name                 State          Provider    Created
────────────────────────────────────────────────────────────────────────────
my-production        Running        Hetzner     2026-01-05T10:30:00Z
dev-environment      Provisioned    LXD         2026-01-06T14:15:30Z

Warning: Failed to load the following environments:
  - test-env: Invalid JSON in environment.json
  - old-env: Permission denied reading environment.json

For troubleshooting, see docs/user-guide/commands.md
```

**Behavior**:

- Show all successfully loaded environments
- List errors at the end (non-fatal)
- Exit code: 0 (partial success - valid environments were displayed)

#### Error Cases

**Data Directory Not Found**:

```text
❌ List command failed: Data directory not found
Tip: Run from the deployer workspace directory or specify --working-dir

For detailed troubleshooting:
Data Directory Not Found - Detailed Troubleshooting:

1. Verify current directory:
   - Run: pwd
   - Expected: Your deployer workspace directory

2. Check if data directory exists:
   - Run: ls -la data/
   - Should contain environment subdirectories

3. Create environment first:
   - Run: torrust-tracker-deployer create environment --env-file <config.json>
```

**Permission Errors**:

```text
❌ List command failed: Permission denied reading environment 'my-env'
Tip: Check file permissions for data/my-env/

For detailed troubleshooting:
Permission Error - Detailed Troubleshooting:

1. Check directory permissions:
   - Run: ls -ld data/my-env/
   - Should have read permission (r--)

2. Check file permissions:
   - Run: ls -l data/my-env/environment.json
   - Should have read permission (r--)

3. Fix permissions if needed:
   - Run: chmod +r data/my-env/environment.json
   - Run: chmod +rx data/my-env/
```

### Data Transfer Objects

**EnvironmentSummary** (lightweight DTO for list display):

```rust
pub struct EnvironmentSummary {
    pub name: String,
    pub state: String,  // Simple string representation
    pub provider: String,
    pub created_at: Option<String>,  // ISO 8601 format: "2026-01-05T10:30:00Z"
}
```

**EnvironmentList** (collection of summaries):

```rust
pub struct EnvironmentList {
    pub environments: Vec<EnvironmentSummary>,
    pub total_count: usize,
}
```

### Error Handling

**ListCommandError** enum with explicit variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ListCommandError {
    #[error("Permission denied accessing directory: {0}")]
    PermissionDenied(PathBuf),

    #[error("Failed to scan environments directory: {0}")]
    ScanError(String),

    #[error("Failed to load some environments (partial success)")]
    PartialLoadError {
        // Non-fatal: some environments couldn't be loaded
        // Command succeeds but reports issues at the end
        failed_environments: Vec<(String, String)>,  // (name, error_message)
    },
}
```

**Error Handling Strategy**:

- **Empty directory**: Not an error - show friendly message, exit code 0
- **Fatal errors**: Permission denied, scan failure - exit code 1
- **Partial failure**: Show valid environments + warnings, exit code 0
- All errors must include actionable messages guiding users on how to resolve the issue

## Implementation Plan

### Phase 1: Application Layer Foundation (2-3 hours)

- [ ] Create `ListCommandHandler` in `src/application/command_handlers/list/mod.rs`
- [ ] Create `ListCommandError` enum in `src/application/command_handlers/list/errors.rs`
- [ ] Create DTOs in `src/application/command_handlers/list/info.rs`:
  - `EnvironmentSummary` - lightweight environment info with ISO 8601 timestamps
  - `EnvironmentList` - collection wrapper with partial failure tracking
- [ ] Implement directory scanning (filesystem order, no specific sorting)
- [ ] Implement lightweight environment loading:
  - Extract only name, state, provider, created_at (ISO 8601 format)
  - Include all states (including `Destroyed`)
  - Collect errors for corrupted environments (non-fatal)
- [ ] Handle empty workspace gracefully (exit code 0)
- [ ] Comprehensive unit tests for handler logic and error scenarios
- [ ] Manual verification: Create 2-3 test environments in different states

**Result**: Handler can scan data directory and extract environment summaries, handling partial failures gracefully.

### Phase 2: Presentation Layer - CLI Integration (1-2 hours)

- [ ] Create CLI subcommand in `src/presentation/input/cli/subcommands/list.rs`
- [ ] Add `List` variant to `Commands` enum
- [ ] Create `ListController` in `src/presentation/controllers/list/mod.rs`
- [ ] Create `ListSubcommandError` in `src/presentation/controllers/list/errors.rs`
- [ ] Add command dispatch in router
- [ ] Connect controller to handler
- [ ] Verify command appears in `--help` output
- [ ] Unit tests for controller and CLI integration

**Result**: Command is runnable from CLI with basic output.

### Phase 3: Output Formatting (2-3 hours)

- [ ] Implement table formatter for environment list
- [ ] Add column headers and separators
- [ ] Format timestamps in human-readable form
- [ ] Handle empty workspace message
- [ ] Add visual improvements (spacing, alignment)
- [ ] Use UserOutput for all output (no println!/eprintln!)
- [ ] Unit tests for formatting logic
- [ ] Manual terminal testing with various workspace scenarios

**Result**: Human-friendly, well-formatted table output.

### Phase 4: Error Handling & Edge Cases (2-3 hours)

- [ ] Handle data directory not found
- [ ] Handle permission errors
- [ ] Handle corrupted environment files (skip with warning)
- [ ] Handle partial load failures (list what can be loaded)
- [ ] Add error context and actionable messages
- [ ] Implement Help trait for all error types
- [ ] Unit tests for all error scenarios
- [ ] Manual testing of error cases

**Result**: Graceful error handling with clear user guidance.

### Phase 5: Testing & Documentation (2-3 hours)

- [ ] Create E2E test in `tests/e2e/commands/list_command.rs`:
  - Test empty workspace
  - Test workspace with multiple environments
  - Test error scenarios
- [ ] Add integration with existing E2E workflow tests (list after create/provision/etc)
- [ ] Verify test coverage meets requirements
- [ ] Write user documentation in `docs/user-guide/commands/list.md`
- [ ] Update `docs/console-commands.md` with list command
- [ ] Update `docs/user-guide/commands/README.md` with command reference
- [ ] Update `docs/roadmap.md` to mark task 5.3 as complete
- [ ] Add inline code documentation (doc comments)

**Result**: Complete test coverage and user documentation.

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
  - All unit tests pass
  - Documentation builds successfully
  - E2E tests pass

**Functional Requirements**:

- [ ] Command lists all environments in data directory (filesystem order)
- [ ] Display shows name, state, provider, and creation timestamp (ISO 8601) for each environment
- [ ] Includes all states including `Destroyed`
- [ ] Empty workspace shows helpful message with data directory path and next steps (exit code 0)
- [ ] Corrupted environments: displays valid ones, lists errors at end, exit code 0
- [ ] Command handles permission errors gracefully
- [ ] Output is human-friendly and easy to read (table format)
- [ ] No truncation or wrapping (terminal handles display)
- [ ] Command execution is fast (< 200ms for typical workspaces)

**Technical Requirements**:

- [ ] **All output via UserOutput** (no println!/eprintln!)
- [ ] Follows DDD layer placement (Application + Presentation)
- [ ] Error types use explicit enums (not anyhow)
- [ ] Follows module organization conventions (public first, imports at top)
- [ ] Follows import conventions (prefer imports over full paths)
- [ ] Uses existing infrastructure (RepositoryFactory, EnvironmentLoader)
- [ ] Lightweight data loading (doesn't load full environment configs)
- [ ] Performance acceptable (< 200ms)

**Testing Requirements**:

- [ ] Unit tests cover ListCommandHandler logic
- [ ] Unit tests cover directory scanning and environment loading
- [ ] Unit tests cover output formatting (table generation)
- [ ] Unit tests cover all error scenarios
- [ ] Unit tests use behavior-driven naming (`it_should_*_when_*`)
- [ ] E2E test validates empty workspace
- [ ] E2E test validates workspace with multiple environments
- [ ] E2E test validates error handling
- [ ] Test coverage includes permission errors and corrupted files

**Documentation Requirements**:

- [ ] List command section added to `docs/console-commands.md`
- [ ] Command reference added to `docs/user-guide/commands/README.md`
- [ ] Detailed guide created in `docs/user-guide/commands/list.md`
- [ ] Roadmap updated (task 5.3 marked complete)
- [ ] Inline code documentation (doc comments for public APIs)
- [ ] Command help text complete (shows in `--help`)

**Code Quality**:

- [ ] No code duplication (DRY principle)
- [ ] Clear separation of concerns (scanning vs loading vs formatting vs presentation)
- [ ] Meaningful variable and function names
- [ ] Proper error context with actionable messages
- [ ] Follows behavior-driven test naming (`it_should_*_when_*`, never `test_*`)

## Related Documentation

- **Similar Command**: [Issue #241 - Show Command](./241-implement-environment-show-command.md) (read-only command pattern)
- **Codebase Architecture**: [docs/codebase-architecture.md](../codebase-architecture.md)
- **DDD Layer Placement**: [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md)
- **Output Handling**: [docs/contributing/output-handling.md](../contributing/output-handling.md)
- **Error Handling**: [docs/contributing/error-handling.md](../contributing/error-handling.md)
- **Module Organization**: [docs/contributing/module-organization.md](../contributing/module-organization.md)
- **Unit Testing Conventions**: [docs/contributing/testing/unit-testing.md](../contributing/testing/unit-testing.md)

## Notes

### Key Design Decisions

- **Command Name**: `list` (standard CLI convention for listing items)
- **No Remote Verification**: Lists local environments only - doesn't check if infrastructure still exists
- **Lightweight Loading**: Loads only summary data (name, state, provider, created_at), not full configs
- **Table Format**: Default output is human-readable table (future: add `--format json` option)
- **Skip Corrupted**: If an environment file is corrupted, skip it with a warning rather than failing the entire command
- **Clear Command Separation**:
  - `list` - Quick overview of all environments (fast, local scan)
  - `show` - Detailed information for one environment
  - `test` - Verify infrastructure (cloud-init, Docker, Docker Compose)
  - `status` (future) - Check service health (connectivity, health endpoints)

### Development Approach

- **Phased Implementation**: Build incrementally with testing at each phase
- **E2E Test Evolution**: Create comprehensive E2E tests + integrate with existing workflow tests
- **Unit Tests**: Write at every phase for code added/modified
- **Follow Show Command Pattern**: Use similar architecture and conventions as the show command (#241)

### Future Enhancements

- **Output Formats**: Add `--format json|table|simple` flag
- **Filtering**: Add `--state <STATE>` to filter by state
- **Sorting**: Add `--sort-by <FIELD>` to sort by name/state/created
- **Detailed View**: Add `--detailed` flag to show more information (like compressed `show` output)
- **Provider-Specific Icons**: Add visual indicators for different providers

### Estimated Duration

**Total**: 9-14 hours (1-2 days) for complete implementation

**Target Completion**: January 2026
