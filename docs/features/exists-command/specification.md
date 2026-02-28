# Exists Command Specification

## 📋 Overview

This feature adds a new `exists` console command that checks whether a deployment environment exists and returns a clear boolean result. It provides a lightweight, scriptable way to probe environment existence without loading full environment state or producing verbose output.

### Context

Several existing commands (`show`, `destroy`, `provision`, `configure`, etc.) already check for environment existence internally — but only as a precondition before doing their real work. When an environment does not exist, they fail with exit code 1 and an error message on stderr. This makes it awkward to use them as a pure existence check in scripts and automation, because:

1. **Ambiguous exit codes**: Exit code 1 means "not found" _or_ "some other error" — callers cannot distinguish them.
2. **Noisy output**: `show` dumps full environment info when it succeeds, requiring callers to discard output they do not need.
3. **Misleading intent**: Using `show` as an existence check is a workaround — it does not communicate the caller's actual intent.

Meanwhile, the SDK already exposes a dedicated `Deployer::exists()` method (in both `src/presentation/sdk/deployer.rs` and `packages/sdk/src/deployer.rs`), which signals that the developers already recognized "does this environment exist?" as a distinct use case. The CLI simply has not caught up.

### Problem Statement

Users and automation scripts need a fast, unambiguous, scriptable way to answer one question: **"Does this environment exist?"** — without side effects, verbose output, or ambiguous exit codes.

## 🎯 Goals

### Primary Goals

- **Goal 1**: Provide a CLI command that checks whether a named environment exists
- **Goal 2**: Return unambiguous results (`true`/`false` on stdout, exit 0 = success, exit 1 = error) for scriptability
- **Goal 3**: Support human-readable and machine-readable (JSON) output formats
- **Goal 4**: Be fast — use `EnvironmentRepository::exists()` directly (file-existence check, no deserialization)

### Secondary Goals (Nice-to-Have)

- Integration into the SDK as a CLI-backed method (the SDK already has `exists()`)

### Non-Goals

What this feature explicitly does NOT aim to do:

- Return detailed environment information (that is the `show` command)
- Verify remote infrastructure health (that is the `test` command)
- List multiple environments (that is the `list` command)
- Check whether the environment is in a specific _state_ — only whether it exists at all
- Modify any state (this is strictly read-only)

## 💡 Proposed Solution

### Approach

Implement a new lightweight command that:

1. Validates the environment name format
2. Calls `EnvironmentRepository::exists()` — a `Path::exists()` check, no JSON deserialization
3. Prints a boolean result (`true`/`false`, or structured JSON)
4. Exits with code 0 on success (both `true` and `false` results), code 1 on error

The command is deliberately thin: no environment loading, no state extraction, no network calls.

### Command Name

**Decision**: Use `exists` (third-person singular)

**Rationale**:

- Matches the existing SDK method name: `Deployer::exists()`
- Matches the repository trait method: `EnvironmentRepository::exists()`
- Reads naturally in scripts: `if deployer exists my-env; then …`
- Self-documenting intent: the command answers "does it exist?"

### Usage Examples

```bash
# Check if an environment exists
torrust-tracker-deployer exists my-environment

# Use in shell scripts for idempotent workflows
if [ "$(torrust-tracker-deployer exists my-env)" = "true" ]; then
    echo "Environment already exists, skipping creation"
else
    torrust-tracker-deployer create environment -f config.json
fi

# JSON output for machine consumption
torrust-tracker-deployer exists my-env --format json
# true
```

### Design Overview

```text
┌─────────────────────────────────────────────────────┐
│  User / Script                                      │
└───────────────────────┬─────────────────────────────┘
                        │
                        │ torrust-tracker-deployer exists <env>
                        ↓
┌─────────────────────────────────────────────────────┐
│  Presentation Layer (CLI Controller)                │
│  - Parse CLI arguments                              │
│  - Validate environment name format                 │
│  - Dispatch to ExistsCommandHandler                 │
│  - Print boolean result to stdout                   │
└───────────────────────┬─────────────────────────────┘
                        │
                        ↓
┌─────────────────────────────────────────────────────┐
│  Application Layer (ExistsCommandHandler)           │
│  - Call repository.exists(name)                     │
│  - Return ExistsResult { name, exists: bool }       │
└───────────────────────┬─────────────────────────────┘
                        │
                        ↓
┌─────────────────────────────────────────────────────┐
│  Domain Layer                                       │
│  - EnvironmentRepository::exists() trait method     │
│  - EnvironmentName validation                       │
└─────────────────────────────────────────────────────┘
                        ↑
                        │
┌─────────────────────────────────────────────────────┐
│  Infrastructure Layer                               │
│  - FileEnvironmentRepository::exists()              │
│  - Checks Path::exists() on environment.json        │
└─────────────────────────────────────────────────────┘
```

### Output Format

#### Human-Readable (default)

When the environment **exists**:

```text
true
```

When the environment **does not exist**:

```text
false
```

#### JSON Format (`--format json`)

`true` and `false` are valid JSON values, so the output is simply:

```json
true
```

```json
false
```

### Exit Code Semantics

| Scenario                   | Exit Code | Stdout (default) | Stderr          |
| -------------------------- | --------- | ---------------- | --------------- |
| Environment exists         | **0**     | `true`           | —               |
| Environment does not exist | **0**     | `false`          | —               |
| Invalid environment name   | **1**     | —                | Error with help |
| Repository/IO error        | **1**     | —                | Error with help |

**Design decision**: This command uses the same exit code convention as all other commands in the project (exit 0 = success, exit 1 = error). The boolean result is communicated via stdout (`true`/`false`), not via exit codes. This avoids conflating "the answer is false" with "the command failed" — a known limitation of the POSIX boolean exit code pattern (where `if/else` cannot distinguish "false" from "error").

### Key Design Decisions

1. **Exit code 0 for both `true` and `false` results**
   - The command always exits 0 on success, regardless of whether the environment exists
   - The boolean result is communicated via stdout (`true`/`false`), not via exit codes
   - This follows Rust's `Result<bool, Error>` philosophy: success carries data, failure carries an error
   - It forces callers to explicitly handle all three scenarios (exists, does not exist, error) rather than conflating "false" with "error"
   - Actual errors (invalid name, IO failure) use exit code 1, consistent with all other commands
   - Usage in bash: `if [ "$(deployer exists my-env)" = "true" ]; then ...`
   - **Rationale**: The POSIX boolean exit code pattern (0=true, 1=false, 2=error) has a fundamental flaw — `if/else` constructs conflate "the answer is false" with "the command failed". Modern shells (Nushell, PowerShell, Fish) have moved past this 1970s limitation by separating the data channel from the error channel. Our approach does the same: stdout carries the data, exit code carries the error status.

2. **No environment loading — file existence check only**
   - Uses `EnvironmentRepository::exists()` which just checks `Path::exists()`
   - Does NOT deserialize the environment JSON
   - If the `environment.json` file exists but is corrupt, the command reports `exists = true` (the environment _exists_, even if its state is broken)
   - Use `show` to validate that the environment can be loaded and read

3. **Result printed to stdout via `UserOutput`**
   - The result is the primary output of the command, not a side effect
   - Output is minimal: bare `true` or `false` (same for human-readable and JSON formats)
   - Progress reporting (`ProgressReporter` / step tracking) is unnecessary for a sub-millisecond operation

> **Note**: A `--quiet` mode (suppress stdout, communicate only via exit code) is intentionally deferred. It will be implemented as a transversal feature for all commands in the future, rather than as a per-command flag.

### Alternatives Considered

#### Option 1: Add `--quiet` flag to `show` instead

- **Pros**: No new command, reuses existing code
- **Cons**: `show` loads and deserializes the full environment (slower); conflates two different concerns (info display vs. existence check); exit code 1 from `show` is ambiguous (not found vs. other error)
- **Decision**: Not chosen — separate command is cleaner and faster

#### Option 2: Add an `--exists` flag to `list`

- **Pros**: Reuses `list` infrastructure
- **Cons**: `list` scans all environments; asking it to check one specific name is semantically wrong; awkward usage (`list --exists my-env`)
- **Decision**: Not chosen — wrong abstraction

#### Option 3: No new command — recommend using `show` in scripts

- **Pros**: Zero implementation effort
- **Cons**: Ambiguous exit codes, verbose output, misleading intent, slower (loads full state)
- **Decision**: Not chosen — poor developer experience

## 🔧 Implementation Details

### Architecture Changes

No architectural changes required. This follows the existing command pattern exactly:

- New application-layer command handler (`ExistsCommandHandler`)
- New presentation-layer CLI controller (`ExistsCommandController`)
- New `Commands::Exists` variant in the CLI enum
- New DI container factory method
- New router dispatch entry

### Component Design

#### Component 1: ExistsCommandHandler (Application Layer)

**Purpose**: Check environment existence via repository

**Location**: `src/application/command_handlers/exists/`

**Interface**:

```rust
use std::sync::Arc;
use crate::domain::environment::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;

pub struct ExistsResult {
    pub name: String,
    pub exists: bool,
}

pub struct ExistsCommandHandler {
    repository: Arc<dyn EnvironmentRepository>,
}

impl ExistsCommandHandler {
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    #[instrument(
        name = "exists_command",
        skip_all,
        fields(
            command_type = "exists",
            environment = %env_name
        )
    )]
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<ExistsResult, ExistsCommandHandlerError> {
        let exists = self.repository.exists(env_name)?;
        Ok(ExistsResult {
            name: env_name.to_string(),
            exists,
        })
    }
}
```

**Dependencies**: `EnvironmentRepository` (via DI)

#### Component 2: ExistsCommandHandlerError (Application Layer)

**Purpose**: Error type for the exists command

**Location**: `src/application/command_handlers/exists/errors.rs`

**Interface**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExistsCommandHandlerError {
    #[error("Failed to check environment existence: {0}")]
    RepositoryError(#[from] RepositoryError),
}
```

Note: There is deliberately **no** `EnvironmentNotFound` variant — "not found" is a valid result (`exists = false`), not an error.

#### Component 3: ExistsCommandController (Presentation Layer)

**Purpose**: CLI controller that validates input, calls the handler, and formats output

**Location**: `src/presentation/cli/controllers/exists/`

**Interface**:

```rust
pub struct ExistsCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    user_output: Arc<UserOutput>,
}

impl ExistsCommandController {
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        user_output: Arc<UserOutput>,
    ) -> Self { ... }

    pub fn execute(
        &self,
        environment: &str,
        output_format: OutputFormat,
    ) -> Result<(), ExistsSubcommandError> {
        // 1. Validate environment name
        let env_name = EnvironmentName::new(environment)?;

        // 2. Execute handler
        let result = ExistsCommandHandler::new(self.repository.clone())
            .execute(&env_name)?;

        // 3. Display result (prints "true" or "false" to stdout)
        self.display_result(&result, output_format)?;

        Ok(())
    }
}
```

#### Component 4: CLI Subcommand

**Purpose**: Clap argument definition

**Location**: Added to `src/presentation/cli/input/cli/commands.rs`

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing variants ...

    /// Check if a deployment environment exists
    Exists {
        /// Name of the environment to check
        #[arg(value_name = "ENVIRONMENT")]
        environment: String,
    },
}
```

#### Component 5: Router Dispatch

**Location**: `src/presentation/cli/dispatch/router.rs`

```rust
Commands::Exists { environment } => {
    context
        .container()
        .create_exists_controller()
        .execute(&environment, context.output_format())?;
    Ok(())
}
```

**Note**: Unlike the POSIX boolean pattern, the router does not need special exit code handling. The controller prints the boolean result to stdout and returns `Ok(())`. Errors propagate normally and cause exit code 1, consistent with all other commands.

### Data Model

No new persistent data structures. The command only checks file existence:

```text
data/<environment-name>/environment.json  →  Path::exists() → bool
```

### API Changes

New public API in application layer:

```rust
// src/application/command_handlers/exists/mod.rs
pub use errors::ExistsCommandHandlerError;
pub use handler::{ExistsCommandHandler, ExistsResult};
```

New console subcommand added to `Commands` enum.

### Configuration

No new configuration required.

## 📊 Impact Analysis

### Files to Create

| File Path                                            | Purpose           | Effort |
| ---------------------------------------------------- | ----------------- | ------ |
| `src/application/command_handlers/exists/mod.rs`     | Module definition | Low    |
| `src/application/command_handlers/exists/handler.rs` | Command handler   | Low    |
| `src/application/command_handlers/exists/errors.rs`  | Error types       | Low    |
| `src/application/command_handlers/exists/tests/`     | Unit tests        | Low    |
| `src/presentation/cli/controllers/exists/mod.rs`     | Module definition | Low    |
| `src/presentation/cli/controllers/exists/handler.rs` | CLI controller    | Low    |
| `src/presentation/cli/controllers/exists/errors.rs`  | CLI error types   | Low    |

### Files to Modify

| File Path                                    | Changes Required                 | Effort  |
| -------------------------------------------- | -------------------------------- | ------- |
| `src/application/command_handlers/mod.rs`    | Add `pub mod exists;`            | Trivial |
| `src/presentation/cli/controllers/mod.rs`    | Add `pub mod exists;`            | Trivial |
| `src/presentation/cli/input/cli/commands.rs` | Add `Exists` variant             | Low     |
| `src/presentation/cli/dispatch/router.rs`    | Add routing for `Exists`         | Low     |
| `src/bootstrap/container.rs`                 | Add `create_exists_controller()` | Low     |
| `docs/console-commands.md`                   | Document new command             | Low     |
| `docs/user-guide/commands.md`                | Add command reference            | Low     |
| `docs/features/active-features.md`           | Add feature entry                | Trivial |

### Breaking Changes

None. This is a new standalone command with no impact on existing functionality.

### Performance Impact

Positive. The `exists` command is faster than any existing way to check environment existence:

- `EnvironmentRepository::exists()` is a `Path::exists()` system call — sub-millisecond
- No JSON deserialization, no environment loading, no network calls
- Negligible resource usage

### Security Considerations

None. The command is read-only and performs only a file-existence check.

## 🗓️ Implementation Plan

### Phase 1: Application Layer

- [ ] Create `ExistsCommandHandler` with `execute()` method
- [ ] Create `ExistsCommandHandlerError` with `Traceable` implementation
- [ ] Create `ExistsResult` DTO
- [ ] Add unit tests

**Estimated Duration**: 1 day

### Phase 2: Presentation Layer

- [ ] Add `Exists` variant to `Commands` enum
- [ ] Create `ExistsCommandController` with output formatting
- [ ] Create `ExistsSubcommandError` with error mapping
- [ ] Add router dispatch entry
- [ ] Add DI container factory method

**Estimated Duration**: 1 day

### Phase 3: Exit Code Handling

- [ ] Verify exit code 0 for successful queries (both `true` and `false` results)
- [ ] Verify exit code 1 for actual errors (invalid name, IO failure)
- [ ] Test exit codes in E2E tests

**Estimated Duration**: 0.5 days

### Phase 4: Testing and Documentation

- [ ] Add E2E tests (exists, does not exist, invalid name)
- [ ] Update console commands documentation
- [ ] Update user guide commands page
- [ ] Create command-specific documentation page

**Estimated Duration**: 1 day

**Total Estimated Duration**: 3-4 days

## ✅ Definition of Done

### Functional Requirements

- [ ] `torrust-tracker-deployer exists <env>` returns exit 0 and outputs `true` when environment exists
- [ ] `torrust-tracker-deployer exists <env>` returns exit 0 and outputs `false` when environment does not exist
- [ ] `--format json` produces `true` or `false` (valid JSON values)
- [ ] Invalid environment names produce a clear error (exit 1) with help

### Technical Requirements

- [ ] Code follows project conventions (DDD layers, command pattern)
- [ ] All linters pass (clippy, rustfmt, etc.)
- [ ] No compiler warnings
- [ ] Error types implement `Traceable` trait
- [ ] Error types provide `help()` method with troubleshooting guidance

### Testing Requirements

- [ ] Unit tests cover handler logic (exists, does not exist, repository error)
- [ ] E2E tests validate full command execution
- [ ] E2E tests verify exit codes (0 for both true/false, 1 for errors)
- [ ] Edge cases tested (see Edge Cases section below)

### Documentation Requirements

- [ ] `docs/console-commands.md` updated
- [ ] `docs/user-guide/commands.md` updated
- [ ] `docs/user-guide/commands/exists.md` created
- [ ] `docs/features/active-features.md` updated

## 🧪 Testing Strategy

### Unit Tests

```rust
#[test]
fn it_should_return_exists_true_when_environment_exists() {
    let (handler, _temp_dir) = create_test_handler_with_environment("my-env");
    let name = EnvironmentName::new("my-env").unwrap();

    let result = handler.execute(&name).unwrap();

    assert!(result.exists);
    assert_eq!(result.name, "my-env");
}

#[test]
fn it_should_return_exists_false_when_environment_does_not_exist() {
    let (handler, _temp_dir) = create_test_handler();
    let name = EnvironmentName::new("nonexistent").unwrap();

    let result = handler.execute(&name).unwrap();

    assert!(!result.exists);
}
```

### E2E Tests

```rust
#[test]
fn it_should_exit_0_when_environment_exists() {
    // Create environment, then run `exists` command
    // Assert exit code 0
    // Assert stdout contains "true"
}

#[test]
fn it_should_exit_0_when_environment_does_not_exist() {
    // Run `exists` command on non-existent environment
    // Assert exit code 0
    // Assert stdout contains "false"
}

#[test]
fn it_should_produce_json_output_when_format_is_json() {
    // Run `exists --format json` command
    // Parse stdout as JSON
    // Assert structure matches expected schema
}
```

### Edge Cases

See dedicated section below.

## 🔍 Edge Cases Analysis

### 1. Corrupt Environment File

**Scenario**: `data/my-env/environment.json` exists but contains invalid JSON.

**Expected behavior**: `exists` returns `true` (exit 0). The file exists, so the environment exists. Whether the environment is _healthy_ is not the concern of `exists` — use `show` for that.

**Rationale**: `EnvironmentRepository::exists()` uses `Path::exists()`, not `load()`. It does not read or validate file contents.

### 2. Empty Environment Directory

**Scenario**: `data/my-env/` directory exists but `environment.json` is missing.

**Expected behavior**: `exists` returns `false` (exit 0, stdout = `false`). The `EnvironmentRepository::exists()` checks for the `environment.json` file specifically, not just the directory.

### 3. Environment Name with Special Characters

**Scenario**: User passes an environment name with characters not allowed by `EnvironmentName::new()`.

**Expected behavior**: Exit code 1 with a clear error message explaining valid name format. The handler is never called.

### 4. Data Directory Does Not Exist

**Scenario**: The `data/` directory does not exist at all (fresh workspace).

**Expected behavior**: `exists` returns `false` (exit 0, stdout = `false`). `EnvironmentRepository::exists()` will return `false` because the path does not exist.

### 5. Permission Denied on Data Directory

**Scenario**: User does not have read access to `data/` or `data/my-env/`.

**Expected behavior**: This depends on `EnvironmentRepository::exists()` implementation. If `Path::exists()` returns `false` when permissions are denied (which it does on many systems), the command would report `false`. If the implementation propagates the IO error, exit code 1 with an error message. This needs investigation.

### 6. Symbolic Link to Environment

**Scenario**: `data/my-env/environment.json` is a symbolic link (possibly broken).

**Expected behavior**: `Path::exists()` follows symlinks. If the target exists, returns `true`. If the symlink is broken, returns `false`.

### 7. Race Condition

**Scenario**: Environment is being created or destroyed concurrently while `exists` runs.

**Expected behavior**: Non-deterministic result (either `true` or `false`), which is acceptable. The `exists` command is a point-in-time snapshot. No locking is needed for a read-only file-existence check.

### 8. Very Long Environment Name

**Scenario**: User passes a very long string as environment name.

**Expected behavior**: `EnvironmentName::new()` validation should reject names that exceed reasonable length. If it does, exit code 2 with an error. If it does not, the filesystem will handle it (path too long error).

### 9. Running from Wrong Working Directory

**Scenario**: User runs `exists` from a directory that is not a deployer workspace (no `data/` directory).

**Expected behavior**: `exists` returns `false`. There is no `data/my-env/environment.json`, so the environment does not exist from this workspace's perspective.

## 📚 Related Documentation

- [Console Commands Overview](../../console-commands.md)
- [Command Architecture](../../codebase-architecture.md)
- [User Guide - Commands](../../user-guide/commands.md)
- [Development Principles](../../development-principles.md)
- [Show Command Feature](../environment-status-command/specification.md) — related command for displaying environment info
- [SDK Deployer::exists()](../sdk/README.md) — existing SDK method

## 🔗 References

- Rust's `Result<T, E>` pattern: separates success (data) from failure (error), avoiding the POSIX trap of overloading exit codes with semantic meaning
- POSIX `test` command uses 0 = true, 1 = false, 2 = error — but this conflates "the answer is false" with "the command failed" in `if/else` constructs
- Modern shells (Nushell, PowerShell, Fish) separate the data channel (stdout) from the error channel (exit code), which is the pattern this command follows

---

**Created**: February 27, 2026
**Last Updated**: February 27, 2026
**Status**: Planning
