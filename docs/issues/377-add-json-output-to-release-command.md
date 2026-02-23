# Add JSON Output to `release` Command

**Issue**: #377
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.7: `docs/roadmap.md`
- Reference implementation (configure, most recent pattern): `src/presentation/views/commands/configure/`
- Reference implementation (list): `src/presentation/views/commands/list/`

## Overview

Add JSON output format support to the `release` command (roadmap task 12.7). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `release` command currently only outputs human-readable text progress messages. This task adds a machine-readable JSON alternative that automation workflows and AI agents can parse programmatically.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `ReleaseCommandController::execute()`
- [ ] Add a `ReleaseDetailsData` view DTO covering the released environment data
- [ ] Implement `JsonView` for the release command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/controllers/release/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()` method
- `src/presentation/views/commands/release/` — new module with DTO and views (mirrors `configure/` structure with `view_data/` subdir)
  - `mod.rs`
  - `view_data/release_details.rs` — `ReleaseDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `provision`, `create`, `show`, `run`, `list`, `configure`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `configure/` (has `view_data/`) — `release` needs `view_data/` because `Environment<Released>` is a domain type
- [ ] `ReleaseDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a `From<&Environment<Released>>` impl — `PartialEq` is needed for unit test assertions
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String`
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `ReleaseDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `release` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "instance_name": "torrust-tracker-vm-my-env",
  "provider": "lxd",
  "state": "Released",
  "instance_ip": "10.140.190.39",
  "created_at": "2026-02-20T10:00:00Z"
}
```

Fields:

| Field              | Type    | Description                                                                          |
| ------------------ | ------- | ------------------------------------------------------------------------------------ |
| `environment_name` | string  | Name of the environment                                                              |
| `instance_name`    | string  | VM instance name                                                                     |
| `provider`         | string  | Infrastructure provider (`lxd`, `hetzner`, etc.)                                     |
| `state`            | string  | Always `"Released"` on success                                                       |
| `instance_ip`      | string? | IP address of the instance (nullable; Rust type: `Option<IpAddr>`)                   |
| `created_at`       | string  | ISO 8601 UTC timestamp when the environment was created (Rust type: `DateTime<Utc>`) |

### `ReleaseDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReleaseDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    pub provider: String,
    pub state: String,
    pub instance_ip: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
}

impl From<&Environment<Released>> for ReleaseDetailsData {
    fn from(env: &Environment<Released>) -> Self {
        Self {
            environment_name: env.name().as_str().to_string(),
            instance_name: env.instance_name().as_str().to_string(),
            provider: env.provider_config().provider_name().to_string(),
            state: "Released".to_string(),
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
    pub fn render(data: &ReleaseDetailsData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            format!(r#"{{"error": "Failed to serialize release details: {e}"}}'#)
        })
    }
}
```

#### TextView

```rust
pub struct TextView;

impl TextView {
    pub fn render(data: &ReleaseDetailsData) -> String {
        let instance_ip = data
            .instance_ip
            .map_or_else(|| "Not available".to_string(), |ip| ip.to_string());

        format!(
            r"Environment Details:
  Name:              {}
  Instance:          {}
  Provider:          {}
  State:             {}
  Instance IP:       {}
  Created:           {}",
            data.environment_name,
            data.instance_name,
            data.provider,
            data.state,
            instance_ip,
            data.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}
```

### Controller Integration

Update `ReleaseCommandController`:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    output_format: OutputFormat,
) -> Result<(), ReleaseSubcommandError> {
    let env_name = self.validate_environment_name(environment_name)?;

    let released_env = self.release_application(&env_name).await?;

    self.complete_workflow(&released_env, output_format)?;

    Ok(())
}

fn complete_workflow(
    &mut self,
    released_env: &Environment<Released>,
    output_format: OutputFormat,
) -> Result<(), ReleaseSubcommandError> {
    // Convert to DTO
    let details = ReleaseDetailsData::from(released_env);

    // Render using Strategy Pattern
    let output = match output_format {
        OutputFormat::Text => TextView::render(&details),
        OutputFormat::Json => JsonView::render(&details),
    };

    self.progress.result(&output)?;

    Ok(())
}
```

## Implementation Checklist

### Step 1: Create View Module Structure

- [ ] Create `src/presentation/views/commands/release/` directory
- [ ] Create `mod.rs` with module declarations
- [ ] Create `view_data/mod.rs` and `view_data/release_details.rs`
- [ ] Create `views/mod.rs`, `views/text_view.rs`, and `views/json_view.rs`

### Step 2: Implement DTO

- [ ] Implement `ReleaseDetailsData` struct with all required fields
- [ ] Add `#[derive(Debug, Clone, Serialize)]`
- [ ] Implement `From<&Environment<Released>>` conversion

### Step 3: Implement Views

- [ ] Implement `JsonView::render()` with inline error handling
- [ ] Implement `TextView::render()` with formatted text output
- [ ] Follow existing patterns from `configure` command

### Step 4: Update Controller

- [ ] Add `output_format: OutputFormat` parameter to `execute()`
- [ ] Change `release_application()` return type from `Result<(), ReleaseSubcommandError>` to `Result<Environment<Released>, ReleaseSubcommandError>` — the released env is already captured as `_released_env` but currently discarded
- [ ] Update `complete_workflow()` to accept `released_env` and `output_format`
- [ ] Implement Strategy Pattern for view selection
- [ ] Remove hardcoded success message, replace with view output

### Step 5: Update Router

- [ ] Update `src/presentation/dispatch/router.rs` to pass `output_format` from context
- [ ] Ensure `--output-format` flag is propagated to controller

### Step 6: Add Tests

- [ ] Unit tests for `ReleaseDetailsData::from()`
- [ ] Unit tests for `JsonView::render()` (valid JSON structure)
- [ ] Unit tests for `TextView::render()` (contains expected strings)
- [ ] Integration test for complete workflow with both formats

### Step 7: Documentation

- [ ] Update command documentation in `docs/user-guide/commands/release.md`
- [ ] Add JSON output examples
- [ ] Update CLI help text if needed

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_released_env_to_dto() {
        // Test DTO conversion
    }

    #[test]
    fn it_should_render_valid_json() {
        // Test JSON view produces valid JSON
    }

    #[test]
    fn it_should_render_text_with_all_fields() {
        // Test text view contains all expected information
    }
}
```

### Integration Tests

- [ ] Test `release` command with `--output-format json`
- [ ] Test `release` command with `--output-format text` (default)
- [ ] Verify JSON can be parsed by external tools

## Definition of Done

- [ ] All implementation steps completed
- [ ] Unit tests pass with >80% coverage
- [ ] Integration tests pass
- [ ] Documentation updated
- [ ] Pre-commit checks pass (linters, formatters)
- [ ] PR approved and merged

## Related

- Parent: #348 (EPIC: Add JSON output format support)
- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.7: `docs/roadmap.md`
- Reference implementation (configure): `src/presentation/views/commands/configure/`
- Reference implementation (list): `src/presentation/views/commands/list/`
