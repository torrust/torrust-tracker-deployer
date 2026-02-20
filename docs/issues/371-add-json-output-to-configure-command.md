# Add JSON Output to `configure` Command

**Issue**: #371
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.6: `docs/roadmap.md`
- Reference implementation (list, most recent pattern): `src/presentation/views/commands/list/`
- Reference implementation (provision): `src/presentation/views/commands/provision/`

## Overview

Add JSON output format support to the `configure` command (roadmap task 12.6). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `configure` command currently only outputs human-readable text. This task adds a machine-readable JSON alternative that automation workflows and AI agents can parse programmatically.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `ConfigureCommandController::execute()`
- [ ] Add a `ConfigureDetailsData` view DTO covering the configured environment data
- [ ] Implement `JsonView` for the configure command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `display_configure_results()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/controllers/configure/handler.rs` — add `output_format` param to `execute()`, add `display_configure_results()` method
- `src/presentation/views/commands/configure/` — new module with DTO and views (mirrors `provision/` structure with `view_data/` subdir)
  - `mod.rs`
  - `view_data/configure_details.rs` — `ConfigureDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `provision`, `create`, `show`, `run`, `list`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `provision/` (has `view_data/`) — `list/` has no `view_data/` because it uses app-layer DTOs directly; `configure` needs `view_data/` because `Environment<Configured>` is a domain type
- [ ] `ConfigureDetailsData` is a plain presentation DTO deriving only `Serialize` (not `Deserialize`) with a `From<&Environment<Configured>>` impl
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String`
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `ConfigureDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `configure` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "provider": "lxd",
  "state": "Configured",
  "instance_ip": "10.140.190.39",
  "created_at": "2026-02-20T10:00:00Z"
}
```

Fields:

| Field              | Type    | Description                                                                                                                                                   |
| ------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `environment_name` | string  | Name of the environment                                                                                                                                       |
| `instance_name`    | string  | VM instance name                                                                                                                                              |
| `provider`         | string  | Infrastructure provider (`lxd`, `hetzner`, etc.)                                                                                                              |
| `state`            | string  | Always `"Configured"` on success                                                                                                                              |
| `instance_ip`      | string? | IP address of the instance (nullable)                                                                                                                         |
| `created_at`       | string  | ISO 8601 UTC timestamp when the environment was created (same value throughout the lifecycle; there is no separate `configured_at` field in the domain model) |

### `ConfigureDetailsData` DTO

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ConfigureDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    pub provider: String,
    pub state: String,
    pub instance_ip: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
}

impl From<&Environment<Configured>> for ConfigureDetailsData {
    fn from(env: &Environment<Configured>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Configured".to_string(),
            instance_ip: env.instance_ip(),
            created_at: env.created_at(),
        }
    }
}
```

> **Notes**:
>
> - The domain model has a single `created_at()` timestamp that records when the environment was first created. There is no separate `configured_at` field — this mirrors the `ProvisionDetailsData` pattern which uses `provisioned_at: env.created_at()`.
> - Only `Serialize` is derived (not `Deserialize`) — presentation DTOs are write-only for output. See `ProvisionDetailsData` for the same convention.
> - Follow the same pattern as `ProvisionDetailsData` in `src/presentation/views/commands/provision/view_data/provision_details.rs`.

### Changes to `ConfigureCommandController::execute()`

Add `output_format: OutputFormat` parameter and a `display_configure_results()` method.

> **Pattern reference**: Use the **list command pattern** from PR #360 (most recent, canonical). This is newer than the provision pattern and should be followed for all Phase 2 commands.

```rust
pub fn execute(
    &mut self,
    environment_name: &str,
    output_format: OutputFormat,
) -> Result<Environment<Configured>, ConfigureSubcommandError> {
    let env_name = self.validate_environment_name(environment_name)?;
    let handler = self.create_command_handler()?;
    let configured = self.configure_infrastructure(&handler, &env_name)?;
    self.complete_workflow(environment_name)?;
    self.display_configure_results(&configured, output_format)?;
    Ok(configured)
}
```

The `display_configure_results()` method follows the **list pattern** — `JsonView::render()` returns `String` (with fallback error JSON via `unwrap_or_else`), so no error variant is propagated from the rendering step:

```rust
fn display_configure_results(
    &mut self,
    configured: &Environment<Configured>,
    output_format: OutputFormat,
) -> Result<(), ConfigureSubcommandError> {
    self.progress.blank_line()?;
    let details = ConfigureDetailsData::from(configured);
    let output = match output_format {
        OutputFormat::Text => TextView::render(&details),
        OutputFormat::Json => JsonView::render(&details),
    };
    self.progress.result(&output)?;
    Ok(())
}
```

### `JsonView` Return Type

`JsonView::render()` returns `String`, handling serialization errors inline with a fallback JSON error message:

```rust
pub fn render(data: &ConfigureDetailsData) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|e| {
        format!(
            r#"{{
  "error": "Failed to serialize configure details",
  "message": "{e}"
}}"#
        )
    })
}
```

> **Key difference from provision**: The provision command's `JsonView::render()` returns `Result<String, serde_json::Error>`. The list command (PR #360) improved this to `String` with inline fallback. **Use the list pattern** for all new Phase 2 commands — do NOT add an `OutputFormatting` error variant to `ConfigureSubcommandError`.

### Call Site Update

Update `src/presentation/dispatch/router.rs` where `Commands::Configure` is handled:

```rust
// Before:
Commands::Configure { environment } => {
    context.container().create_configure_controller().execute(&environment)?;
}

// After:
Commands::Configure { environment } => {
    let output_format = context.output_format();
    context.container().create_configure_controller().execute(&environment, output_format)?;
}
```

## Implementation Plan

### Phase 1: View Module (1–2 hours)

- [ ] Create `src/presentation/views/commands/configure/` module
- [ ] Implement `ConfigureDetailsData` DTO with `From<&Environment<Configured>>`
- [ ] Implement `JsonView::render()` using `serde_json::to_string_pretty`
- [ ] Implement `TextView::render()` with human-readable format
- [ ] Register module in `src/presentation/views/commands/mod.rs`
- [ ] Add unit tests for both views

### Phase 2: Controller Update (30 min)

- [ ] Add `output_format: OutputFormat` parameter to `ConfigureCommandController::execute()`
- [ ] Add `display_configure_results()` method to the controller (Strategy Pattern with `Text`/`Json` match, both arms produce `String`)
- [ ] Update router to extract `output_format` from context and pass it to `execute()` (`src/presentation/dispatch/router.rs`)
- [ ] No new error variant needed — JSON serialization failures are handled inline in `JsonView::render()` via `unwrap_or_else`

### Phase 3: Verification (30 min)

- [ ] Run `cargo test` to verify all tests pass
- [ ] Manually verify `configure my-env --output-format json` outputs valid JSON
- [ ] Manually verify `configure my-env` (default text) still works correctly
- [ ] Run `./scripts/pre-commit.sh`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `configure my-env --output-format json` outputs valid, parseable JSON to stdout
- [ ] JSON output contains at minimum: `environment_name`, `state`, `created_at`, `instance_ip`
- [ ] `configure my-env` (default, no flag) continues to output human-readable text unchanged
- [ ] `JsonView` and `TextView` have unit tests following naming convention (`it_should_...`)
- [ ] No `println!`/`eprintln!` added — all output goes through `UserOutput`
- [ ] Error handling for JSON serialization failure is explicit (not `unwrap`)

## Related Documentation

- [docs/codebase-architecture.md](../codebase-architecture.md) — DDD layer overview
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) — which code belongs where
- [docs/contributing/output-handling.md](../contributing/output-handling.md) — output conventions
- [docs/contributing/error-handling.md](../contributing/error-handling.md) — error handling conventions
- [docs/contributing/testing/unit-testing/naming-conventions.md](../contributing/testing/unit-testing/naming-conventions.md) — test naming

## Notes

- This is the first command in Phase 2 — the implementation establishes the pattern for the remaining 7 commands (12.7–12.13). Use the **list command pattern** (not the older provision pattern): `JsonView::render()` returns `String`, no `OutputFormatting` error variant.
- The `configure` command's `execute()` currently takes no `output_format`. The call site in `src/presentation/dispatch/router.rs` (the `Commands::Configure` arm) must be updated to extract `output_format` via `context.output_format()` and forward it to `execute()`.
- Once all Phase 2 commands are done, issue #348 task 12.14 will switch `#[default]` from `Text` to `Json`.
