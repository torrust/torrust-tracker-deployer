# Add JSON Output to `validate` Command

**Issue**: #390
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.10: `docs/roadmap.md`
- Reference implementation (destroy, most recent pattern): `src/presentation/cli/views/commands/destroy/`
- Reference implementation (test, most recent pattern): `src/presentation/cli/views/commands/test/`

## Overview

Add JSON output format support to the `validate` command (roadmap task 12.10). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `validate` command currently only outputs human-readable text progress messages and a validation summary. This task adds a machine-readable JSON alternative that automation workflows, CI/CD pipelines, and AI agents can parse programmatically.

**Key characteristic**: The `validate` command takes an `--env-file <PATH>` argument and performs validation without creating any deployment state. Its output reflects configuration properties extracted from the file — environment name, provider, and enabled feature flags (Prometheus, Grafana, HTTPS, backup). There is no domain `Environment` state object involved — the DTO is built directly from `ValidationResult`.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `ValidateCommandController::execute()`
- [ ] Add a `ValidateDetailsData` view DTO covering the validated configuration data
- [ ] Implement `JsonView` for the validate command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format (preserving current `complete_workflow` output)
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/cli/controllers/validate/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()` method
- `src/presentation/cli/views/commands/validate/` — new module with DTO and views
  - `mod.rs`
  - `view_data/validate_details.rs` — `ValidateDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `provision`, `create`, `show`, `run`, `list`, `configure`, `release`, `test`, `destroy`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `destroy/` (has `view_data/`) — `validate` needs `view_data/` to decouple the `ValidationResult` application type from view formatting
- [ ] `ValidateDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a `From<(&Path, &ValidationResult)>` impl — `PartialEq` is needed for unit test assertions
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String` (preserving existing output)
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `ValidateDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `validate` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "config_file": "/home/user/envs/my-env.json",
  "provider": "lxd",
  "is_valid": true,
  "has_prometheus": true,
  "has_grafana": true,
  "has_https": false,
  "has_backup": false
}
```

Fields:

| Field              | Type    | Description                                                      |
| ------------------ | ------- | ---------------------------------------------------------------- |
| `environment_name` | string  | Name of the validated environment                                |
| `config_file`      | string  | Absolute (or as-given) path to the validated configuration file  |
| `provider`         | string  | Infrastructure provider (lowercase: `"lxd"`, `"hetzner"`, etc.)  |
| `is_valid`         | boolean | Always `true` when the command succeeds (errors produce no JSON) |
| `has_prometheus`   | boolean | Whether Prometheus monitoring is configured                      |
| `has_grafana`      | boolean | Whether Grafana dashboard is configured                          |
| `has_https`        | boolean | Whether HTTPS is configured                                      |
| `has_backup`       | boolean | Whether backups are configured                                   |

**Note on `is_valid`**: This field is always `true` when the command exits successfully. Validation failures exit with a non-zero status code and output error text; they do not produce JSON. The field is included for self-documenting JSON and consistency with programmatic parsers that check it explicitly.

### `ValidateDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidateDetailsData {
    pub environment_name: String,
    pub config_file: String,
    pub provider: String,
    pub is_valid: bool,
    pub has_prometheus: bool,
    pub has_grafana: bool,
    pub has_https: bool,
    pub has_backup: bool,
}

impl ValidateDetailsData {
    pub fn from_result(env_file: &Path, result: &ValidationResult) -> Self {
        Self {
            environment_name: result.environment_name.clone(),
            config_file: env_file.display().to_string(),
            provider: result.provider.clone(),
            is_valid: true,
            has_prometheus: result.has_prometheus,
            has_grafana: result.has_grafana,
            has_https: result.has_https,
            has_backup: result.has_backup,
        }
    }
}
```

**Note**: Unlike domain-backed DTOs (which use `From<&Environment<State>>`), `ValidateDetailsData` combines two inputs (`&Path` + `&ValidationResult`). Use a named constructor `from_result` rather than `From` to keep the API clear.

### Text Output Format (preserved)

When `--output-format text` (or default), the `validate` command outputs the existing human-readable format.

Verified against real command output (`cargo run -- validate --env-file envs/lxd-local-example.json`):

```text
⏳ [1/3] Loading configuration file...
⏳   ✓ Configuration file loaded (took 0ms)
⏳ [2/3] Validating JSON schema...
⏳   ✓ Schema validation passed (took 0ms)
⏳ [3/3] Validating configuration fields...
⏳   ✓ Field validation passed (took 0ms)

✅ Configuration file 'envs/lxd-local-example.json' is valid

Environment Details:
• Name: lxd-local-example
• Provider: lxd
• Prometheus: Enabled
• Grafana: Enabled
• HTTPS: Disabled
• Backups: Disabled
```

> **Note**: Timing values (`took 0ms`) vary per run — the `TextView` should reproduce this format but tests should not assert on exact timing values.

### Controller Changes

Current signature:

```rust
pub fn execute(&mut self, env_file: &Path) -> Result<(), ValidateSubcommandError>
```

New signature:

```rust
pub fn execute(&mut self, env_file: &Path, output_format: OutputFormat) -> Result<(), ValidateSubcommandError>
```

The `complete_workflow()` method receives `output_format` and dispatches to `TextView` or `JsonView`:

```rust
fn complete_workflow(
    &mut self,
    env_file: &Path,
    result: &ValidationResult,
    output_format: OutputFormat,
) -> Result<(), ValidateSubcommandError> {
    let data = ValidateDetailsData::from_result(env_file, result);
    let rendered = if matches!(output_format, OutputFormat::Json) {
        JsonView::render(&data)
    } else {
        TextView::render(&data)
    };
    self.progress.complete(&rendered)?;
    Ok(())
}
```

### Router Changes

```rust
Commands::Validate { env_file } => {
    let output_format = context.output_format();
    context
        .container()
        .create_validate_controller()
        .execute(&env_file, output_format)?;
    Ok(())
}
```

## Implementation Checklist

### Phase 1 — View Module

- [ ] Create `src/presentation/cli/views/commands/validate/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/validate/view_data/validate_details.rs` with `ValidateDetailsData`
- [ ] Create `src/presentation/cli/views/commands/validate/views/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/validate/views/json_view.rs` with `JsonView`
- [ ] Create `src/presentation/cli/views/commands/validate/views/text_view.rs` with `TextView`
- [ ] Register `validate` in `src/presentation/cli/views/commands/mod.rs`

### Phase 2 — Controller Changes

- [ ] Add `output_format: OutputFormat` parameter to `ValidateCommandController::execute()`
- [ ] Refactor `complete_workflow()` to accept and dispatch on `output_format`
- [ ] Import and use `ValidateDetailsData`, `JsonView`, `TextView`

### Phase 3 — Router Changes

- [ ] Add `let output_format = context.output_format();` in `Commands::Validate` arm
- [ ] Pass `output_format` to `execute()`

### Phase 4 — Tests

- [ ] Unit tests for `JsonView::render()` — assert serialized JSON matches expected
- [ ] Unit tests for `TextView::render()` — assert formatted string matches expected
- [ ] Verify existing integration tests still pass (`cargo test`)

### Phase 5 — Manual Output Verification

Before opening the PR, run the real command against a known env file and confirm both outputs match expectations:

**Text output** (must match the documented format above exactly, aside from timing):

```bash
cargo run -- validate --env-file envs/lxd-local-example.json 2>/dev/null
```

Expected (based on current output captured 2026-02-26):

```text
⏳ [1/3] Loading configuration file...
⏳   ✓ Configuration file loaded (took 0ms)
⏳ [2/3] Validating JSON schema...
⏳   ✓ Schema validation passed (took 0ms)
⏳ [3/3] Validating configuration fields...
⏳   ✓ Field validation passed (took 0ms)

✅ Configuration file 'envs/lxd-local-example.json' is valid

Environment Details:
• Name: lxd-local-example
• Provider: lxd
• Prometheus: Enabled
• Grafana: Enabled
• HTTPS: Disabled
• Backups: Disabled
```

**JSON output** (stdout only — redirect stderr to /dev/null to get clean JSON):

```bash
cargo run -- validate --env-file envs/lxd-local-example.json --output-format json 2>/dev/null
```

Expected:

```json
{
  "environment_name": "lxd-local-example",
  "config_file": "envs/lxd-local-example.json",
  "provider": "lxd",
  "is_valid": true,
  "has_prometheus": true,
  "has_grafana": true,
  "has_https": false,
  "has_backup": false
}
```

- [ ] Text output matches documented format (excluding timing values)
- [ ] JSON output is valid JSON and matches expected fields
- [ ] `echo $?` returns `0` for both commands

### Phase 6 — Lint & Documentation

- [ ] `cargo run --bin linter all` passes (stable + nightly)
- [ ] `cargo machete` passes (no unused dependencies)
- [ ] Update `docs/user-guide/commands/validate.md` — add JSON output section

## Testing Strategy

### Unit Tests

Follow the pattern in `src/presentation/cli/views/commands/destroy/views/` (or `test/views/`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_data() -> ValidateDetailsData {
        ValidateDetailsData {
            environment_name: "my-env".to_string(),
            config_file: "/home/user/envs/my-env.json".to_string(),
            provider: "lxd".to_string(),
            is_valid: true,
            has_prometheus: true,
            has_grafana: false,
            has_https: false,
            has_backup: true,
        }
    }

    #[test]
    fn it_renders_json() {
        let data = sample_data();
        let rendered = JsonView::render(&data);
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["environment_name"], "my-env");
        assert_eq!(parsed["provider"], "lxd");
        assert_eq!(parsed["is_valid"], true);
        assert_eq!(parsed["has_prometheus"], true);
        assert_eq!(parsed["has_grafana"], false);
    }

    #[test]
    fn it_renders_text() {
        let data = sample_data();
        let rendered = TextView::render(&data);
        assert!(rendered.contains("my-env"));
        assert!(rendered.contains("lxd"));
        assert!(rendered.contains("Enabled"));
    }
}
```

## Related Resources

- Destroy command views (most recent): `src/presentation/cli/views/commands/destroy/`
- Test command views (most recent): `src/presentation/cli/views/commands/test/`
- Epic spec: `docs/issues/348-epic-add-json-output-format-support.md`
- Router: `src/presentation/cli/dispatch/router.rs`
- User docs: `docs/user-guide/commands/validate.md`
- Output handling guide: `docs/contributing/output-handling.md`
- Error handling guide: `docs/contributing/error-handling.md`
