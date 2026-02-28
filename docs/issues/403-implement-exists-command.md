# Implement `exists` CLI Command

**Issue**: #403
**Parent Epic**: None
**Related**:

- Feature specification: `docs/features/exists-command/specification.md`
- Feature questions: `docs/features/exists-command/questions.md`
- SDK `Deployer::exists()`: `packages/sdk/src/deployer.rs`
- Console commands overview: `docs/console-commands.md`

## Overview

Implement a new `exists` CLI command that checks whether a deployment environment exists and returns a boolean result. The command outputs `true` or `false` to stdout and always exits 0 on success (exit 1 only for errors), following the project's standard exit code convention.

This fills a gap in the CLI: the SDK already has `Deployer::exists()`, but there is no CLI equivalent. Existing commands (`show`, `destroy`, etc.) check existence internally as a precondition, but their exit code 1 is ambiguous (not found vs. other error) and they produce verbose output unsuitable for scripting.

## Goals

- [ ] Provide a CLI command that checks whether a named environment exists
- [ ] Output `true`/`false` to stdout (exit 0 = success, exit 1 = error)
- [ ] Support human-readable and JSON output formats (both output bare `true`/`false`)
- [ ] Use `EnvironmentRepository::exists()` directly (file-existence check, no deserialization)
- [ ] Update SDK `Deployer::exists()` to use the new `ExistsCommandHandler`

## 🏗️ Architecture Requirements

**DDD Layers**: Application + Presentation

**Module Paths**:

- `src/application/command_handlers/exists/mod.rs` — Module definition
- `src/application/command_handlers/exists/handler.rs` — `ExistsCommandHandler`
- `src/application/command_handlers/exists/errors.rs` — `ExistsCommandHandlerError`
- `src/presentation/cli/controllers/exists/mod.rs` — Module definition
- `src/presentation/cli/controllers/exists/handler.rs` — `ExistsCommandController`
- `src/presentation/cli/controllers/exists/errors.rs` — `ExistsSubcommandError`
- `src/presentation/cli/input/cli/commands.rs` — `Commands::Exists` variant
- `src/presentation/cli/dispatch/router.rs` — Router dispatch entry
- `src/bootstrap/container.rs` — DI factory method

**Patterns**: Command Handler + CLI Controller + Router Dispatch (same as all other commands)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Error types implement `Traceable` trait with `help()` method
- [ ] "Not found" is a valid result (`exists = false`), NOT an error — no `EnvironmentNotFound` error variant
- [ ] Skip `ProgressReporter` — sub-millisecond operation, use direct `UserOutput` calls

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Domain layer depending on infrastructure
- ❌ Monolithic modules with multiple responsibilities
- ❌ Using exit codes to communicate boolean results (use stdout instead)

## Specifications

### Exit Code Semantics

| Scenario                   | Exit Code | Stdout  | Stderr          |
| -------------------------- | --------- | ------- | --------------- |
| Environment exists         | **0**     | `true`  | —               |
| Environment does not exist | **0**     | `false` | —               |
| Invalid environment name   | **1**     | —       | Error with help |
| Repository/IO error        | **1**     | —       | Error with help |

### Output Format

Both human-readable and JSON formats output bare `true` or `false` — these are valid JSON values.

### Usage Examples

```bash
# Basic check
torrust-tracker-deployer exists my-environment

# Bash scripting
if [ "$(torrust-tracker-deployer exists my-env)" = "true" ]; then
    echo "Environment already exists, skipping creation"
else
    torrust-tracker-deployer create environment -f config.json
fi

# JSON output
torrust-tracker-deployer exists my-env --format json
# true
```

### Router Dispatch

```rust
Commands::Exists { environment } => {
    context
        .container()
        .create_exists_controller()
        .execute(&environment, context.output_format())?;
    Ok(())
}
```

No special exit code handling needed — the controller prints the result and returns `Ok(())`.

### SDK Update

Update `Deployer::exists()` to use the new handler instead of wrapping `show()`:

```rust
pub fn exists(&self, env_name: &EnvironmentName) -> Result<bool, ExistsCommandHandlerError> {
    let handler = ExistsCommandHandler::new(self.repository.clone());
    Ok(handler.execute(env_name)?.exists)
}
```

This is a breaking change to the SDK return type (acceptable — no stable release yet).

## Implementation Plan

### Phase 1: Application Layer

- [ ] Create `ExistsCommandHandler` with `execute()` method
- [ ] Create `ExistsCommandHandlerError` with `Traceable` implementation
- [ ] Create `ExistsResult` DTO (`name: String`, `exists: bool`)
- [ ] Add `pub mod exists;` to `src/application/command_handlers/mod.rs`
- [ ] Add unit tests (exists, not exists, repository error)

### Phase 2: Presentation Layer

- [ ] Add `Exists` variant to `Commands` enum in `commands.rs`
- [ ] Create `ExistsCommandController` with output formatting
- [ ] Create `ExistsSubcommandError` with error mapping
- [ ] Add router dispatch entry in `router.rs`
- [ ] Add `create_exists_controller()` to DI container
- [ ] Add `pub mod exists;` to `src/presentation/cli/controllers/mod.rs`

### Phase 3: SDK Integration

- [ ] Update `Deployer::exists()` to use `ExistsCommandHandler`
- [ ] Update return type from `ShowCommandHandlerError` to `ExistsCommandHandlerError`

### Phase 4: Testing and Documentation

- [ ] Add E2E tests (exists, not exists, invalid name, JSON output)
- [ ] Update `docs/console-commands.md`
- [ ] Update `docs/user-guide/commands/` with `exists` command page
- [ ] Update `docs/features/active-features.md` status

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `torrust-tracker-deployer exists <env>` outputs `true` and exits 0 when environment exists
- [ ] `torrust-tracker-deployer exists <env>` outputs `false` and exits 0 when environment does not exist
- [ ] `--format json` produces `true` or `false` (valid JSON values)
- [ ] Invalid environment names produce a clear error (exit 1) with help text
- [ ] Error types implement `Traceable` trait with `help()` method
- [ ] Code follows DDD layer conventions (command handler pattern)
- [ ] SDK `Deployer::exists()` updated to use `ExistsCommandHandler`
- [ ] Unit tests cover handler logic (exists, not exists, repository error)
- [ ] E2E tests validate full command execution and exit codes
- [ ] Documentation updated (`console-commands.md`, user guide)

## Related Documentation

- [Feature Specification](../features/exists-command/specification.md)
- [Feature Questions](../features/exists-command/questions.md)
- [Console Commands Overview](../console-commands.md)
- [Codebase Architecture](../codebase-architecture.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Error Handling Conventions](../contributing/error-handling.md)

## Notes

- The `EnvironmentRepository::exists()` trait method and `FileEnvironmentRepository::exists()` implementation already exist — no domain or infrastructure changes needed
- The command is deliberately thin: no environment loading, no state extraction, no network calls
- Performance: sub-millisecond (`Path::exists()` system call only)
- Edge cases (corrupt files, permissions, symlinks, race conditions) are documented in the feature specification
