# Add JSON Output to `render` Command

**Issue**: #392
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/roadmap.md` section 12
- Roadmap section 12.11: `docs/roadmap.md`
- Reference implementation (validate, most recent pattern): `src/presentation/cli/views/commands/validate/`
- Reference implementation (destroy, most recent pattern): `src/presentation/cli/views/commands/destroy/`

## Overview

Add JSON output format support to the `render` command (roadmap task 12.11). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `render` command generates deployment artifacts (docker-compose files, Ansible playbooks, tracker config, etc.) without executing any infrastructure operations. It takes either `--env-name` or `--env-file` (mutually exclusive), plus `--instance-ip` and `--output-dir`. Its output reflects the result of rendering: the environment name, the configuration source, the target IP used, and the output directory where artifacts were placed.

The application-layer `RenderResult` type provides: `environment_name`, `target_ip`, `output_dir`, and `config_source`.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `RenderCommandController::execute()`
- [ ] Add a `RenderDetailsData` view DTO covering the rendered artifact data
- [ ] Implement `JsonView` for the render command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format (preserving current `show_success` output)
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in the controller using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/cli/controllers/render/handler.rs` — add `output_format` param to `execute()`, replace `show_success()` with view dispatch
- `src/presentation/cli/views/commands/render/` — new module with DTO and views
  - `mod.rs`
  - `view_data/render_details.rs` — `RenderDetailsData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `validate`, `destroy`, `test`, `configure`, `release`, `run`, `list`, `show`, `provision`, `create`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `validate/` and `destroy/` (has `view_data/`) — `render` needs `view_data/` to decouple the `RenderResult` application type from view formatting
- [ ] `RenderDetailsData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a named constructor `from_result` — `PartialEq` is needed for unit test assertions
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String` (preserving existing `show_success` output)
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `RenderDetailsData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `render` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "config_source": "Config file: /home/user/envs/my-env.json",
  "target_ip": "192.168.1.100",
  "output_dir": "/tmp/build/my-env"
}
```

Fields:

| Field              | Type   | Description                                                       |
| ------------------ | ------ | ----------------------------------------------------------------- |
| `environment_name` | string | Name of the environment whose artifacts were generated            |
| `config_source`    | string | Description of the configuration source (env name or config file) |
| `target_ip`        | string | IP address used in artifact generation                            |
| `output_dir`       | string | Absolute path to the directory containing generated artifacts     |

### `RenderDetailsData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RenderDetailsData {
    pub environment_name: String,
    pub config_source: String,
    pub target_ip: String,
    pub output_dir: String,
}

impl RenderDetailsData {
    pub fn from_result(result: &RenderResult) -> Self {
        Self {
            environment_name: result.environment_name.clone(),
            config_source: result.config_source.clone(),
            target_ip: result.target_ip.to_string(),
            output_dir: result.output_dir.display().to_string(),
        }
    }
}
```

**Note**: `RenderResult` is defined in `src/application/command_handlers/render/handler.rs` and has fields `environment_name: String`, `target_ip: IpAddr`, `output_dir: PathBuf`, `config_source: String`.

### Text Output Format (preserved)

When `--output-format text` (or default), the `render` command outputs the existing human-readable format.

Current output (from `show_success()` in the controller — note this goes through `UserOutput::success()`):

```text
✅ Deployment artifacts generated successfully!

  Source: Config file: envs/lxd-local-example.json
  Target IP: 1.2.3.4
  Output: /path/to/output

Next steps:
  - Review artifacts in the output directory
  - Use 'provision' command to deploy infrastructure
  - Or use artifacts manually with your deployment tools
```

The `TextView::render()` must reproduce this format.

### Controller Changes

**Current signature**:

```rust
pub async fn execute(
    &mut self,
    env_name: Option<&str>,
    env_file: Option<&Path>,
    ip: &str,
    output_dir: &Path,
    force: bool,
    working_dir: &Path,
) -> Result<(), RenderCommandError>
```

**New signature**:

```rust
pub async fn execute(
    &mut self,
    env_name: Option<&str>,
    env_file: Option<&Path>,
    ip: &str,
    output_dir: &Path,
    force: bool,
    working_dir: &Path,
    output_format: OutputFormat,
) -> Result<(), RenderCommandError>
```

Replace the `show_success()` call at the end of `execute()` with a view dispatch:

```rust
let data = RenderDetailsData::from_result(&result);
let rendered = if matches!(output_format, OutputFormat::Json) {
    JsonView::render(&data)
} else {
    TextView::render(&data)
};
self.progress.complete(&rendered)?;
```

The `show_success()` helper method can be removed once the view dispatch is in place.

### Router Changes

```rust
Commands::Render {
    env_name,
    env_file,
    instance_ip,
    output_dir,
    force,
} => {
    let output_format = context.output_format();
    context
        .container()
        .create_render_controller()
        .execute(
            env_name.as_deref(),
            env_file.as_deref(),
            &instance_ip,
            output_dir.as_path(),
            force,
            context.working_dir(),
            output_format,
        )
        .await?;
    Ok(())
}
```

## Implementation Checklist

### Phase 1 — View Module

- [ ] Create `src/presentation/cli/views/commands/render/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/render/view_data/render_details.rs` with `RenderDetailsData`
- [ ] Create `src/presentation/cli/views/commands/render/views/mod.rs`
- [ ] Create `src/presentation/cli/views/commands/render/views/json_view.rs` with `JsonView`
- [ ] Create `src/presentation/cli/views/commands/render/views/text_view.rs` with `TextView`
- [ ] Register `render` in `src/presentation/cli/views/commands/mod.rs`

### Phase 2 — Controller Changes

- [ ] Add `output_format: OutputFormat` parameter to `RenderCommandController::execute()`
- [ ] Replace `show_success()` call with view dispatch block
- [ ] Remove the `show_success()` method (or adapt it to `TextView`)
- [ ] Import and use `RenderDetailsData`, `JsonView`, `TextView`, `OutputFormat`

### Phase 3 — Router Changes

- [ ] Add `let output_format = context.output_format();` in `Commands::Render` arm
- [ ] Pass `output_format` as last argument to `execute()`

### Phase 4 — Tests

- [ ] Unit tests for `JsonView::render()` — assert serialized JSON matches expected
- [ ] Unit tests for `TextView::render()` — assert formatted string matches expected
- [ ] Verify existing integration tests still pass (`cargo test`)

### Phase 5 — Manual Output Verification

Before opening the PR, run the real command against a known env file and confirm both outputs match expectations:

**Text output**:

```bash
cargo run -- render --env-file envs/lxd-local-example.json --instance-ip 1.2.3.4 --output-dir /tmp/render-test 2>/dev/null
```

**JSON output** (stdout only — redirect stderr to /dev/null to get clean JSON):

```bash
cargo run -- render --env-file envs/lxd-local-example.json --instance-ip 1.2.3.4 --output-dir /tmp/render-test --output-format json 2>/dev/null
```

Expected:

```json
{
  "environment_name": "lxd-local-example",
  "config_source": "Config file: envs/lxd-local-example.json",
  "target_ip": "1.2.3.4",
  "output_dir": "/tmp/render-test"
}
```

- [ ] Text output matches documented format
- [ ] JSON output is valid JSON and matches expected fields
- [ ] `echo $?` returns `0` for both commands

### Phase 6 — Lint & Documentation

- [ ] `cargo run --bin linter all` passes (stable + nightly)
- [ ] `cargo machete` passes (no unused dependencies)
- [ ] Update `docs/user-guide/commands/render.md` — add JSON output section

## Testing Strategy

### Unit Tests

Follow the pattern in `src/presentation/cli/views/commands/validate/views/`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> RenderDetailsData {
        RenderDetailsData {
            environment_name: "my-env".to_string(),
            config_source: "Config file: /home/user/envs/my-env.json".to_string(),
            target_ip: "192.168.1.100".to_string(),
            output_dir: "/tmp/build/my-env".to_string(),
        }
    }

    #[test]
    fn it_renders_json() {
        let data = sample_data();
        let rendered = JsonView::render(&data);
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["environment_name"], "my-env");
        assert_eq!(parsed["target_ip"], "192.168.1.100");
        assert_eq!(parsed["output_dir"], "/tmp/build/my-env");
    }

    #[test]
    fn it_renders_text() {
        let data = sample_data();
        let rendered = TextView::render(&data);
        assert!(rendered.contains("my-env"));
        assert!(rendered.contains("192.168.1.100"));
        assert!(rendered.contains("/tmp/build/my-env"));
    }
}
```

## Related Resources

- Validate command views (most recent): `src/presentation/cli/views/commands/validate/`
- Destroy command views: `src/presentation/cli/views/commands/destroy/`
- Epic spec roadmap: `docs/roadmap.md` section 12
- Router: `src/presentation/cli/dispatch/router.rs`
- User docs: `docs/user-guide/commands/render.md`
- Output handling guide: `docs/contributing/output-handling.md`
- Error handling guide: `docs/contributing/error-handling.md`
