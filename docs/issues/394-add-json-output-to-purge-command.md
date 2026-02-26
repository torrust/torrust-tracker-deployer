# Add JSON Output to `purge` Command

**Issue**: #394
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/roadmap.md` section 12
- Roadmap section 12.12: `docs/roadmap.md`
- Reference implementation (render, most recent pattern): `src/presentation/cli/views/commands/render/`
- Reference implementation (validate, most recent pattern): `src/presentation/cli/views/commands/validate/`

## Overview

Add JSON output format support to the `purge` command (roadmap task 12.12). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `purge` command deletes all local data for an environment: the `data/<env>/` directory, the `build/<env>/` directory, and the environment registry entry. It takes an environment name and an optional `--force` flag to skip the confirmation prompt.

The application-layer `PurgeCommandHandler::execute()` returns `()` on success — there is no result struct. The presentation-layer DTO is built from the environment name alone.

**Key characteristic**: Unlike commands with rich result structs (provision, run, render), purge only confirms that the named environment was purged. The JSON output is intentionally minimal: `environment_name` and `purged: true`.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `PurgeCommandController::execute()`
- [ ] Add a `PurgeDetailsData` view DTO
- [ ] Implement `JsonView` for the purge command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback)
- [ ] Implement `TextView` to present the same data in human-readable format (preserving current `complete_workflow` output)
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/cli/controllers/purge/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()`
- `src/presentation/cli/views/commands/purge/` — new module with DTO and views
  - `mod.rs`
  - `view_data/purge_details.rs` — `PurgeDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `render`, `validate`, `destroy`, etc.)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `render/` and `validate/` (has `view_data/`)
- [ ] `PurgeDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a named constructor `from_environment_name` — built from just the environment name string since purge returns no result struct
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else`
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String` (preserving existing output)
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `PurgeDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `purge` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "purged": true
}
```

Fields:

| Field              | Type    | Description                                       |
| ------------------ | ------- | ------------------------------------------------- |
| `environment_name` | string  | Name of the environment that was purged           |
| `purged`           | boolean | Always `true` when the command exits successfully |

**Note on `purged`**: This field is always `true` when the command exits successfully. Purge failures exit with a non-zero status code and output error text; they do not produce JSON (or produce no output). The field is included for self-documenting JSON and consistency with programmatic parsers that check it explicitly — same pattern as `is_valid` in `validate`.

**Note on confirmation prompt**: When `--output-format json` is used without `--force`, the confirmation prompt is still displayed on stderr. The JSON result is written to stdout only after successful confirmation and purge execution. Users running in automation contexts should always pass `--force` alongside `--output-format json`.

### `PurgeDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PurgeDetailsData {
    pub environment_name: String,
    pub purged: bool,
}

impl PurgeDetailsData {
    pub fn from_environment_name(environment_name: &str) -> Self {
        Self {
            environment_name: environment_name.to_string(),
            purged: true,
        }
    }
}
```

### Text Output Format (preserved)

When `--output-format text` (or default), the `purge` command outputs the existing human-readable format:

```text
✅ Environment 'my-env' purged successfully
```

The `TextView` reproduces this by returning `"Environment 'my-env' purged successfully"` (without the `✅` prefix — that is added by `ProgressReporter::complete()`).

### Controller Changes

**Current signature**:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    force: bool,
) -> Result<(), PurgeSubcommandError>
```

**New signature**:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    force: bool,
    output_format: OutputFormat,
) -> Result<(), PurgeSubcommandError>
```

Update `complete_workflow()` to accept and dispatch on `output_format`:

```rust
fn complete_workflow(
    &mut self,
    environment_name: &str,
    output_format: OutputFormat,
) -> Result<(), PurgeSubcommandError> {
    let data = PurgeDetailsData::from_environment_name(environment_name);

    match output_format {
        OutputFormat::Text => {
            self.progress.complete(&TextView::render(&data))?;
        }
        OutputFormat::Json => {
            self.progress.result(&JsonView::render(&data))?;
        }
    }

    Ok(())
}
```

### Router Changes

```rust
Commands::Purge { environment, force } => {
    let output_format = context.output_format();
    context
        .container()
        .create_purge_controller()
        .execute(&environment, force, output_format)
        .await?;
    Ok(())
}
```

## Implementation Checklist

### Phase 1 — View Module

- [ ] Create `src/presentation/cli/views/commands/purge/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/purge/view_data/purge_details.rs` with `PurgeDetailsData`
- [ ] Create `src/presentation/cli/views/commands/purge/views/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/purge/views/json_view.rs` with `JsonView`
- [ ] Create `src/presentation/cli/views/commands/purge/views/text_view.rs` with `TextView`
- [ ] Register `purge` in `src/presentation/cli/views/commands/mod.rs`

### Phase 2 — Controller Changes

- [ ] Add `output_format: OutputFormat` parameter to `PurgeCommandController::execute()`
- [ ] Update `complete_workflow()` to accept and dispatch on `output_format`
- [ ] Import and use `PurgeDetailsData`, `JsonView`, `TextView`, `OutputFormat`

### Phase 3 — Router Changes

- [ ] Add `let output_format = context.output_format();` in `Commands::Purge` arm
- [ ] Pass `output_format` as new argument to `execute()`

### Phase 4 — Tests

- [ ] Unit tests for `JsonView::render()` — assert serialized JSON matches expected
- [ ] Unit tests for `TextView::render()` — assert formatted string matches expected
- [ ] Verify existing tests still pass (`cargo test`)

### Phase 5 — Manual Output Verification

Before opening the PR, run the real command and confirm both outputs match expectations.

**Text output** (requires an existing environment):

```bash
cargo run -- purge my-env --force 2>/dev/null
```

Expected:

```text
✅ Environment 'my-env' purged successfully
```

**JSON output**:

```bash
cargo run -- purge my-env --force --output-format json 2>/dev/null
```

Expected:

```json
{
  "environment_name": "my-env",
  "purged": true
}
```

- [ ] Text output matches documented format
- [ ] JSON output is valid JSON and matches expected fields
- [ ] `echo $?` returns `0` for both commands

### Phase 6 — Lint & Documentation

- [ ] `cargo run --bin linter all` passes (stable + nightly)
- [ ] `cargo machete` passes (no unused dependencies)
- [ ] Update `docs/user-guide/commands/purge.md` — add JSON output section

## Testing Strategy

### Unit Tests

Follow the pattern in `src/presentation/cli/views/commands/render/views/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> PurgeDetailsData {
        PurgeDetailsData {
            environment_name: "my-env".to_string(),
            purged: true,
        }
    }

    #[test]
    fn it_renders_json() {
        let data = sample_data();
        let rendered = JsonView::render(&data);
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["environment_name"], "my-env");
        assert_eq!(parsed["purged"], true);
    }

    #[test]
    fn it_renders_text() {
        let data = sample_data();
        let rendered = TextView::render(&data);
        assert!(rendered.contains("my-env"));
        assert!(rendered.contains("purged successfully"));
    }
}
```

## Related Resources

- Render command views (most recent): `src/presentation/cli/views/commands/render/`
- Validate command views: `src/presentation/cli/views/commands/validate/`
- Epic roadmap: `docs/roadmap.md` section 12
- Router: `src/presentation/cli/dispatch/router.rs`
- User docs: `docs/user-guide/commands/purge.md`
- Output handling guide: `docs/contributing/output-handling.md`
- Error handling guide: `docs/contributing/error-handling.md`
