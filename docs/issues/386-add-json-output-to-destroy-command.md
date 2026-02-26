# Add JSON Output to `destroy` Command

**Issue**: #386
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.9: `docs/roadmap.md`
- Reference implementation (release, most recent state-transition pattern): `src/presentation/cli/views/commands/release/`
- Reference implementation (test, most recent pattern): `src/presentation/cli/views/commands/test/`

## Overview

Add JSON output format support to the `destroy` command (roadmap task 12.9). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `destroy` command currently only outputs human-readable text progress messages and a purge hint. This task adds a machine-readable JSON alternative that automation workflows, CI/CD pipelines, and AI agents can parse programmatically.

**Key characteristic**: Unlike most commands, `destroy` accepts environments in **any state** (it can destroy from `Created`, `Provisioned`, `Running`, etc.). As a result, the `instance_ip` field in the output DTO may be `None` if the environment was never provisioned. The JSON schema must handle this nullable field, consistent with the `release` command pattern.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `DestroyCommandController::execute()`
- [ ] Add a `DestroyDetailsData` view DTO covering the destroyed environment data
- [ ] Implement `JsonView` for the destroy command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/cli/controllers/destroy/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()` method
- `src/presentation/cli/views/commands/destroy/` — new module with DTO and views (mirrors `release/` structure with `view_data/` subdir)
  - `mod.rs`
  - `view_data/destroy_details.rs` — `DestroyDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `provision`, `create`, `show`, `run`, `list`, `configure`, `release`, `test`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `release/` (has `view_data/`) — `destroy` needs `view_data/` because `Environment<Destroyed>` is a domain type that must not leak into views
- [ ] `DestroyDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a `From<&Environment<Destroyed>>` impl — `PartialEq` is needed for unit test assertions
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String`
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `DestroyDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `destroy` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "provider": "lxd",
  "state": "Destroyed",
  "instance_ip": "10.140.190.39",
  "created_at": "2026-02-23T10:00:00Z"
}
```

When the environment was never provisioned (e.g., destroyed from `Created` state), `instance_ip` is `null`:

```json
{
  "environment_name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "provider": "lxd",
  "state": "Destroyed",
  "instance_ip": null,
  "created_at": "2026-02-23T10:00:00Z"
}
```

Fields:

| Field              | Type              | Description                                                     |
| ------------------ | ----------------- | --------------------------------------------------------------- |
| `environment_name` | string            | Name of the destroyed environment                               |
| `instance_name`    | string            | Name of the VM instance                                         |
| `provider`         | string            | Infrastructure provider (lowercase: `"lxd"`, `"hetzner"`, etc.) |
| `state`            | string            | Always `"Destroyed"` for this command                           |
| `instance_ip`      | string \| null    | IP address of the instance, or `null` if never provisioned      |
| `created_at`       | string (ISO 8601) | Timestamp when the environment was originally created           |

### `DestroyDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DestroyDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    pub provider: String,
    pub state: String,
    pub instance_ip: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
}

impl From<&Environment<Destroyed>> for DestroyDetailsData {
    fn from(env: &Environment<Destroyed>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Destroyed".to_string(),
            instance_ip: env.instance_ip(),
            created_at: env.created_at(),
        }
    }
}
```

### View Implementations

#### JsonView

```rust
pub struct JsonView;

impl JsonView {
    pub fn render(data: &DestroyDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            format!(r#"{{"error": "Failed to serialize destroy results: {e}"}}"#)
        })
    }
}
```

#### TextView

```rust
pub struct TextView;

impl TextView {
    pub fn render(data: &DestroyDetailsData) -> String {
        let ip_str = data
            .instance_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        format!(
            r"Destroy Results:
  Environment:   {}
  Instance:      {}
  Provider:      {}
  State:         {}
  Instance IP:   {}
  Created At:    {}",
            data.environment_name,
            data.instance_name,
            data.provider,
            data.state,
            ip_str,
            data.created_at.format("%Y-%m-%dT%H:%M:%SZ"),
        )
    }
}
```

### Controller Integration

Update `DestroyCommandController::execute()` to accept and propagate `output_format`:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    output_format: OutputFormat,
) -> Result<(), DestroySubcommandError> {
    let env_name = self.validate_environment_name(environment_name)?;
    let handler = self.create_command_handler()?;
    let destroyed = self.tear_down_infrastructure(&handler, &env_name)?;
    self.complete_workflow(environment_name, &destroyed, output_format)?;
    Ok(())
}
```

Update `complete_workflow()` to use Strategy Pattern:

```rust
fn complete_workflow(
    &mut self,
    name: &str,
    destroyed: &Environment<Destroyed>,
    output_format: OutputFormat,
) -> Result<(), DestroySubcommandError> {
    let data = DestroyDetailsData::from(destroyed);

    let output = match output_format {
        OutputFormat::Text => TextView::render(&data),
        OutputFormat::Json => JsonView::render(&data),
    };

    self.progress.result(&output)?;

    // Purge hint only in text mode (does not belong in structured JSON output)
    if output_format == OutputFormat::Text {
        self.progress.blank_line()?;
        self.progress.output().lock().borrow_mut().result(&format!(
            "💡 Local data preserved for debugging. To completely remove and reuse the name:\n   torrust-tracker-deployer purge {name} --force"
        ));
    }

    Ok(())
}
```

**Note**: The purge hint is informational and should only be shown in text mode. JSON consumers do not need human-readable hints.

## Implementation Checklist

### Step 1: Create View Module Structure

- [ ] Create `src/presentation/cli/views/commands/destroy/` directory
- [ ] Create `mod.rs` with module declarations
- [ ] Create `view_data/destroy_details.rs` (inline `pub mod` in `mod.rs`, no separate `view_data/mod.rs`)
- [ ] Create `views/json_view.rs` and `views/text_view.rs` (inline `pub mod` in `mod.rs`, no separate `views/mod.rs`)

### Step 2: Implement DTO

- [ ] Implement `DestroyDetailsData` struct with all required fields
- [ ] Add `#[derive(Debug, Clone, PartialEq, Serialize)]`
- [ ] Implement `From<&Environment<Destroyed>>` conversion

### Step 3: Implement Views

- [ ] Implement `JsonView::render()` with inline error handling
- [ ] Implement `TextView::render()` with formatted text output
- [ ] Follow existing patterns from `release` command

### Step 4: Update Controller

- [ ] Add `output_format: OutputFormat` parameter to `execute()`
- [ ] Update `complete_workflow()` to accept `&Environment<Destroyed>` and `output_format`
- [ ] Implement Strategy Pattern for view selection
- [ ] Move purge hint to text-only branch
- [ ] Replace hardcoded success message with view output via `self.progress.result()`

### Step 5: Update Router

- [ ] Update `src/presentation/cli/dispatch/router.rs` to pass `output_format` from context
- [ ] Ensure `--output-format` flag is propagated to controller

### Step 6: Add Tests

- [ ] Unit tests for `DestroyDetailsData::from()` (provisioned env with IP, and never-provisioned env with null IP)
- [ ] Unit tests for `JsonView::render()` (valid JSON structure, IP present, IP null)
- [ ] Unit tests for `TextView::render()` (contains expected strings, IP present, `N/A` for null IP)

### Step 7: Documentation

- [x] Update command documentation in `docs/user-guide/commands/destroy.md` if it exists
- [ ] Add JSON output examples

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_destroyed_environment_with_ip_to_dto() {
        // Test DTO conversion from environment that was provisioned (has IP)
    }

    #[test]
    fn it_should_convert_destroyed_environment_without_ip_to_dto() {
        // Test DTO conversion from environment destroyed before provisioning (IP is None)
    }

    #[test]
    fn it_should_render_valid_json_with_ip() {
        // Test JSON view produces valid JSON with instance_ip as string
    }

    #[test]
    fn it_should_render_valid_json_with_null_ip() {
        // Test JSON view produces valid JSON with instance_ip as null
    }

    #[test]
    fn it_should_render_text_with_all_fields() {
        // Test text view contains all expected information
    }

    #[test]
    fn it_should_render_text_with_na_for_missing_ip() {
        // Test text view shows "N/A" when instance_ip is None
    }
}
```

### Integration Tests

- [ ] Test `destroy` command with `--output-format json` (covered by E2E pre-commit tests)
- [ ] Test `destroy` command with `--output-format text` (default, covered by E2E pre-commit tests)
- [ ] Verify JSON can be parsed by external tools

## Definition of Done

- [ ] All implementation steps completed
- [ ] Unit tests pass
- [ ] Documentation updated
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] PR approved and merged

## Related

- Parent: #348 (EPIC: Add JSON output format support)
- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.9: `docs/roadmap.md`
- Reference implementation (release): `src/presentation/cli/views/commands/release/`
- Reference implementation (test): `src/presentation/cli/views/commands/test/`
