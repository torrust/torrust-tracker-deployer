# Add JSON Output to `register` Command

**Issue**: #396
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/roadmap.md` section 12
- Roadmap section 12.13: `docs/roadmap.md`
- Reference implementation (purge, most recent pattern): `src/presentation/cli/views/commands/purge/`
- Reference implementation (validate): `src/presentation/cli/views/commands/validate/`

## Overview

Add JSON output format support to the `register` command (roadmap task 12.13). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `register` command links an existing (pre-provisioned) instance to a deployer environment by recording its IP address and optional SSH port in the environment state. It transitions the environment from `Created` to `Provisioned`. The application-layer `RegisterCommandHandler::execute()` returns `Environment<Provisioned>` on success.

**Key characteristic**: Unlike `purge` (which returns `()`), the `register` command returns a rich result — `Environment<Provisioned>` — giving access to `environment_name`, `instance_ip()`, and `ssh_port()`. The JSON output captures the registered instance details for automation and AI agent workflows.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `RegisterCommandController::execute()`
- [ ] Add a `RegisterDetailsData` view DTO
- [ ] Implement `JsonView` for the register command — `render()` returns `String` using the inline `unwrap_or_else` fallback pattern
- [ ] Implement `TextView` to present the same data in human-readable format (preserving current `complete_workflow` output)
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/cli/controllers/register/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()`
- `src/presentation/cli/views/commands/register/` — new module with DTO and views
  - `mod.rs`
  - `view_data/register_details.rs` — `RegisterDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `purge`, `validate`, `destroy`, etc.)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `purge/` and `validate/` (has `view_data/`)
- [ ] `RegisterDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a named constructor `from_environment` — built from `&Environment<Provisioned>`
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else`
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String` (preserving existing output)
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `RegisterDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `register` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "instance_ip": "192.168.1.100",
  "ssh_port": 22,
  "registered": true
}
```

Fields:

| Field              | Type    | Description                                            |
| ------------------ | ------- | ------------------------------------------------------ |
| `environment_name` | string  | Name of the environment the instance was registered to |
| `instance_ip`      | string  | IP address of the registered instance                  |
| `ssh_port`         | integer | SSH port used for the instance (default 22)            |
| `registered`       | boolean | Always `true` when the command exits successfully      |

**Note on `registered`**: This field is always `true` when the command exits successfully. Register failures exit with a non-zero status code and produce no JSON output. The field is included for self-documenting JSON and consistency with programmatic parsers — same pattern as `is_valid` in `validate` and `purged` in `purge`.

### `RegisterDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegisterDetailsData {
    pub environment_name: String,
    pub instance_ip: String,
    pub ssh_port: u16,
    pub registered: bool,
}

impl RegisterDetailsData {
    pub fn from_environment(env: &Environment<Provisioned>) -> Self {
        Self {
            environment_name: env.name().to_string(),
            instance_ip: env
                .instance_ip()
                .map_or_else(String::new, |ip| ip.to_string()),
            ssh_port: env.ssh_port(),
            registered: true,
        }
    }
}
```

### Text Output Format (preserved)

When `--output-format text` (or default), the `register` command outputs the existing human-readable format:

```text
✅ Instance registered successfully with environment 'my-env'
```

The `TextView` reproduces this by returning `"Instance registered successfully with environment 'my-env'"` (without the `✅` prefix — that is added by `ProgressReporter::complete()`).

### Controller Changes

**Current signature**:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    instance_ip_str: &str,
    ssh_port: Option<u16>,
) -> Result<Environment<Provisioned>, RegisterSubcommandError>
```

**New signature**:

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    instance_ip_str: &str,
    ssh_port: Option<u16>,
    output_format: OutputFormat,
) -> Result<Environment<Provisioned>, RegisterSubcommandError>
```

Update `complete_workflow()` to accept the provisioned environment and dispatch on `output_format`:

```rust
fn complete_workflow(
    &mut self,
    environment_name: &str,
    provisioned: &Environment<Provisioned>,
    output_format: OutputFormat,
) -> Result<(), RegisterSubcommandError> {
    let data = RegisterDetailsData::from_environment(provisioned);
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

Note that `complete_workflow()` currently takes only `&str` for the name. It must be updated to also receive `&Environment<Provisioned>` to build the DTO.

### Router Changes

```rust
Commands::Register {
    environment,
    instance_ip,
    ssh_port,
} => {
    let output_format = context.output_format();
    context
        .container()
        .create_register_controller()
        .execute(&environment, &instance_ip, ssh_port, output_format)
        .await?;
    Ok(())
}
```

## Implementation Checklist

### Phase 1 — View Module

- [ ] Create `src/presentation/cli/views/commands/register/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/register/view_data/register_details.rs` with `RegisterDetailsData`
- [ ] Create `src/presentation/cli/views/commands/register/views/json_view.rs` with `JsonView`
- [ ] Create `src/presentation/cli/views/commands/register/views/text_view.rs` with `TextView`
- [ ] Register `register` in `src/presentation/cli/views/commands/mod.rs`

### Phase 2 — Controller Changes

- [ ] Add `output_format: OutputFormat` parameter to `RegisterCommandController::execute()`
- [ ] Update `complete_workflow()` to accept `&Environment<Provisioned>` and `output_format`
- [ ] Pass `provisioned` to `complete_workflow()` in `execute()`
- [ ] Import and use `RegisterDetailsData`, `JsonView`, `TextView`, `OutputFormat`

### Phase 3 — Router Changes

- [ ] Add `let output_format = context.output_format();` in `Commands::Register` arm
- [ ] Pass `output_format` as new argument to `execute()`

### Phase 4 — Tests

- [ ] Unit tests for `JsonView::render()` — assert serialized JSON matches expected
- [ ] Unit tests for `TextView::render()` — assert formatted string matches expected
- [ ] Unit tests for `RegisterDetailsData::from_environment()` — assert all fields mapped correctly
- [ ] Verify existing tests still pass (`cargo test`)

### Phase 5 — Manual Output Verification

Before opening the PR, run the real command and confirm both outputs match expectations.

**Text output** (requires an environment in `Created` state and a reachable instance):

```bash
cargo run -- register my-env 192.168.1.100 2>/dev/null
```

Expected:

```text
✅ Instance registered successfully with environment 'my-env'
```

**JSON output**:

```bash
cargo run -- register my-env 192.168.1.100 --output-format json 2>/dev/null
```

Expected:

```json
{
  "environment_name": "my-env",
  "instance_ip": "192.168.1.100",
  "ssh_port": 22,
  "registered": true
}
```

- [ ] Text output matches documented format
- [ ] JSON output is valid JSON and matches expected fields
- [ ] `echo $?` returns `0` for both commands

### Phase 6 — Lint & Documentation

- [ ] `cargo run --bin linter all` passes (stable + nightly)
- [ ] `cargo machete` passes (no unused dependencies)
- [ ] Update `docs/user-guide/commands/register.md` — add JSON output section

## Testing Strategy

### Unit Tests

Follow the pattern in `src/presentation/cli/views/commands/purge/views/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> RegisterDetailsData {
        RegisterDetailsData {
            environment_name: "my-env".to_string(),
            instance_ip: "192.168.1.100".to_string(),
            ssh_port: 22,
            registered: true,
        }
    }

    #[test]
    fn it_renders_json() {
        let data = sample_data();
        let rendered = JsonView::render(&data);
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["environment_name"], "my-env");
        assert_eq!(parsed["instance_ip"], "192.168.1.100");
        assert_eq!(parsed["ssh_port"], 22);
        assert_eq!(parsed["registered"], true);
    }

    #[test]
    fn it_renders_text() {
        let data = sample_data();
        let rendered = TextView::render(&data);
        assert!(rendered.contains("my-env"));
        assert!(rendered.contains("registered successfully"));
    }
}
```

## Related Resources

- Purge command views (most recent): `src/presentation/cli/views/commands/purge/`
- Validate command views: `src/presentation/cli/views/commands/validate/`
- Epic roadmap: `docs/roadmap.md` section 12
- Router: `src/presentation/cli/dispatch/router.rs`
- User docs: `docs/user-guide/commands/register.md`
- Output handling guide: `docs/contributing/output-handling.md`
- Error handling guide: `docs/contributing/error-handling.md`
