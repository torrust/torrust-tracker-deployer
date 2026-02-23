# Add JSON Output to `test` Command

**Issue**: #380
**Parent Epic**: #348 - EPIC: Add JSON output format support
**Related**:

- Epic specification: `docs/issues/348-epic-add-json-output-format-support.md`
- Roadmap section 12.8: `docs/roadmap.md`
- Reference implementation (release, most recent pattern): `src/presentation/views/commands/release/`
- Reference implementation (configure): `src/presentation/views/commands/configure/`

## Overview

Add JSON output format support to the `test` command (roadmap task 12.8). This is part of Phase 2 of the JSON output epic (#348), which aims to implement JSON output for all remaining commands so that JSON can eventually become the **default output format**.

The `test` command currently only outputs human-readable text progress messages and DNS warnings. This task adds a machine-readable JSON alternative that automation workflows, CI/CD pipelines, and AI agents can parse programmatically.

**Key difference from other commands**: The `test` command does not transition environment state. It loads the current environment (any state), validates running services, and returns a `TestResult` containing pass/fail status and advisory DNS warnings. The JSON output schema must reflect this test-result nature rather than the environment-state pattern used by other commands.

## Goals

- [ ] Add `output_format: OutputFormat` parameter to `TestCommandController::execute()`
- [ ] Add a `TestResultData` view DTO covering the test outcome and DNS warnings
- [ ] Implement `JsonView` for the test command — `render()` returns `String` using the list-command pattern (inline `unwrap_or_else` fallback, no `OutputFormatting` error variant)
- [ ] Implement `TextView` to present the same data in human-readable format
- [ ] Handle `OutputFormat::Json` and `OutputFormat::Text` branches in `complete_workflow()` using Strategy Pattern
- [ ] Update router to pass `output_format` from context to controller
- [ ] Add unit tests for `JsonView` and `TextView`

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation

**Module Paths**:

- `src/presentation/controllers/test/handler.rs` — add `output_format` param to `execute()`, update `complete_workflow()` method
- `src/presentation/views/commands/test/` — new module with DTO and views (mirrors `release/` structure with `view_data/` subdir)
  - `mod.rs`
  - `view_data/test_result_data.rs` — `TestResultData` DTO
  - `views/text_view.rs` — `TextView`
  - `views/json_view.rs` — `JsonView`

**Pattern**: Strategy Pattern for rendering (same as `provision`, `create`, `show`, `run`, `list`, `configure`, `release`)

### Module Structure Requirements

- [ ] Follow the existing view module structure established in `release/` (has `view_data/`) — `test` needs `view_data/` because `TestResult` is an application layer type that must not leak into views
- [ ] `TestResultData` is a plain presentation DTO deriving `Serialize`, `PartialEq` (not `Deserialize`) with a `From` impl converting from the application layer `TestResult` and environment metadata
- [ ] `JsonView::render()` returns `String` — serialization errors handled inline via `unwrap_or_else` (list pattern, not provision pattern)
- [ ] `TextView::render()` formats the same data as human-readable text and also returns `String`
- [ ] Follow module organization conventions (`docs/contributing/module-organization.md`)

### Architectural Constraints

- [ ] No business logic in the presentation layer — only rendering
- [ ] Error handling follows project conventions (`docs/contributing/error-handling.md`)
- [ ] Output must go through `UserOutput` methods — never `println!` or `eprintln!` directly (`docs/contributing/output-handling.md`)
- [ ] The `TestResultData` DTO must derive `serde::Serialize` (output-only — no `Deserialize` needed)

### Anti-Patterns to Avoid

- ❌ Mixing rendering concerns in the controller
- ❌ Adding business logic to view structs
- ❌ Using `println!`/`eprintln!` instead of `UserOutput`

## Specifications

### JSON Output Format

When `--output-format json` is passed, the `test` command outputs a single JSON object to stdout:

```json
{
  "environment_name": "my-env",
  "instance_ip": "10.140.190.39",
  "result": "pass",
  "dns_warnings": []
}
```

With DNS warnings:

```json
{
  "environment_name": "my-env",
  "instance_ip": "10.140.190.39",
  "result": "pass",
  "dns_warnings": [
    {
      "domain": "tracker.local",
      "expected_ip": "10.140.190.39",
      "issue": "tracker.local does not resolve (expected: 10.140.190.39): name resolution failed"
    },
    {
      "domain": "api.tracker.local",
      "expected_ip": "10.140.190.39",
      "issue": "api.tracker.local resolves to [192.168.1.1] but expected 10.140.190.39"
    }
  ]
}
```

Fields:

| Field              | Type   | Description                                                        |
| ------------------ | ------ | ------------------------------------------------------------------ |
| `environment_name` | string | Name of the environment tested                                     |
| `instance_ip`      | string | IP address of the tested instance                                  |
| `result`           | string | Overall test result: `"pass"` (test command only succeeds on pass) |
| `dns_warnings`     | array  | List of advisory DNS warnings (may be empty)                       |

DNS warning fields:

| Field         | Type   | Description                                 |
| ------------- | ------ | ------------------------------------------- |
| `domain`      | string | The domain that was checked                 |
| `expected_ip` | string | The expected IP address (instance IP)       |
| `issue`       | string | Human-readable description of the DNS issue |

**Note**: The `result` field is always `"pass"` because the test command returns an error (not JSON) when infrastructure validation fails. DNS warnings are advisory and do not affect the `result` value.

### `TestResultData` DTO

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestResultData {
    pub environment_name: String,
    pub instance_ip: String,
    pub result: String,
    pub dns_warnings: Vec<DnsWarningData>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DnsWarningData {
    pub domain: String,
    pub expected_ip: String,
    pub issue: String,
}
```

The conversion needs both the environment name, instance IP, and the `TestResult`:

```rust
impl TestResultData {
    pub fn new(
        environment_name: &str,
        instance_ip: IpAddr,
        test_result: &TestResult,
    ) -> Self {
        Self {
            environment_name: environment_name.to_string(),
            instance_ip: instance_ip.to_string(),
            result: "pass".to_string(),
            dns_warnings: test_result
                .dns_warnings
                .iter()
                .map(|w| DnsWarningData {
                    domain: w.domain.to_string(),
                    expected_ip: w.expected_ip.to_string(),
                    issue: w.to_string(),
                })
                .collect(),
        }
    }
}
```

### View Implementations

#### JsonView

```rust
pub struct JsonView;

impl JsonView {
    pub fn render(data: &TestResultData) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|e| {
            format!(r#"{{"error": "Failed to serialize test results: {e}"}}"#)
        })
    }
}
```

#### TextView

```rust
pub struct TextView;

impl TextView {
    pub fn render(data: &TestResultData) -> String {
        let mut output = format!(
            r"Test Results:
  Environment:       {}
  Instance IP:       {}
  Result:            {}",
            data.environment_name,
            data.instance_ip,
            data.result,
        );

        if data.dns_warnings.is_empty() {
            output.push_str("\n  DNS Warnings:      None");
        } else {
            output.push_str(&format!("\n  DNS Warnings:      {}", data.dns_warnings.len()));
            for warning in &data.dns_warnings {
                output.push_str(&format!(
                    "\n    - {} (expected: {}): {}",
                    warning.domain, warning.expected_ip, warning.issue
                ));
            }
        }

        output
    }
}
```

### Controller Integration

Update `TestCommandController`:

The key change is that `execute()` must now:

1. Accept `output_format`
2. Capture the `TestResult` from `fixture_infrastructure()` (currently consumed inline)
3. Capture the `instance_ip` (currently only used inside the handler)
4. Pass all data to `complete_workflow()` for rendering

```rust
pub async fn execute(
    &mut self,
    environment_name: &str,
    output_format: OutputFormat,
) -> Result<(), TestSubcommandError> {
    // 1. Validate environment name
    let env_name = self.validate_environment_name(environment_name)?;

    // 2. Create command handler
    let handler = self.create_command_handler()?;

    // 3. Execute validation workflow via application layer
    let test_result = self.fixture_infrastructure(&handler, &env_name).await?;

    // 4. Complete workflow with structured output
    self.complete_workflow(environment_name, &test_result, output_format)?;

    Ok(())
}
```

**Important**: The `fixture_infrastructure()` method currently renders DNS warnings inline via `self.progress`. With JSON output, the DNS warnings should be part of the structured output instead. The method needs to return the `TestResult` so that `complete_workflow()` can include DNS warnings in the DTO.

For text mode, DNS warnings can still be rendered as part of the `TextView` output, or kept as progress warnings — the implementation should decide which approach provides the best UX consistency.

```rust
fn complete_workflow(
    &mut self,
    environment_name: &str,
    test_result: &TestResult,
    output_format: OutputFormat,
) -> Result<(), TestSubcommandError> {
    // Build DTO from test result
    // Note: instance_ip will need to be captured earlier in the workflow
    let data = TestResultData::new(environment_name, instance_ip, test_result);

    // Render using Strategy Pattern
    let output = match output_format {
        OutputFormat::Text => TextView::render(&data),
        OutputFormat::Json => JsonView::render(&data),
    };

    self.progress.result(&output)?;

    Ok(())
}
```

**Design consideration**: The `instance_ip` is currently resolved inside `TestCommandHandler::execute()`. To include it in the presentation DTO, either:

1. Return it as part of `TestResult` (preferred — add `instance_ip: IpAddr` field to `TestResult`), or
2. Load the environment again in the controller (wasteful), or
3. Pass it through from the handler execution context

Option 1 is cleanest — extend `TestResult` with `instance_ip` since it's part of the test output data.

## Implementation Checklist

### Step 1: Create View Module Structure

- [ ] Create `src/presentation/views/commands/test/` directory
- [ ] Create `mod.rs` with module declarations
- [ ] Create `view_data/mod.rs` and `view_data/test_result_data.rs`
- [ ] Create `views/mod.rs`, `views/text_view.rs`, and `views/json_view.rs`

### Step 2: Extend TestResult (Application Layer)

- [ ] Add `instance_ip: IpAddr` field to `TestResult` in `src/application/command_handlers/test/result.rs`
- [ ] Update `TestResult::success()` and `TestResult::with_dns_warnings()` constructors to accept `instance_ip`
- [ ] Update handler to pass `instance_ip` when constructing `TestResult`
- [ ] Update existing tests for `TestResult`

### Step 3: Implement DTO

- [ ] Implement `TestResultData` and `DnsWarningData` structs with all required fields
- [ ] Add `#[derive(Debug, Clone, PartialEq, Serialize)]`
- [ ] Implement `TestResultData::new()` constructor

### Step 4: Implement Views

- [ ] Implement `JsonView::render()` with inline error handling
- [ ] Implement `TextView::render()` with formatted text output
- [ ] Follow existing patterns from `release` command

### Step 5: Update Controller

- [ ] Add `output_format: OutputFormat` parameter to `execute()`
- [ ] Update `fixture_infrastructure()` to return `TestResult` (currently it renders DNS warnings inline and discards the result — the method needs to return it)
- [ ] Update `complete_workflow()` to accept `TestResult` and `output_format`
- [ ] Implement Strategy Pattern for view selection
- [ ] Remove hardcoded success message, replace with view output
- [ ] Decide how to handle DNS warning rendering in text mode (via `TextView` vs. progress reporter) for consistency

### Step 6: Update Router

- [ ] Update `src/presentation/dispatch/router.rs` to pass `output_format` from context
- [ ] Ensure `--output-format` flag is propagated to controller

### Step 7: Add Tests

- [ ] Unit tests for `TestResultData::new()` (with and without DNS warnings)
- [ ] Unit tests for `DnsWarningData` construction
- [ ] Unit tests for `JsonView::render()` (valid JSON structure, empty warnings, with warnings)
- [ ] Unit tests for `TextView::render()` (contains expected strings, with and without warnings)

### Step 8: Documentation

- [ ] Update command documentation in `docs/user-guide/commands/test.md`
- [ ] Add JSON output examples
- [ ] Update CLI help text if needed
- [ ] Correct existing inaccuracies in user docs (see below)

**User docs corrections needed** (`docs/user-guide/commands/test.md`):

The current user documentation describes checks that do not match the actual implementation. These should be corrected as part of this task:

1. **"Test Categories" section is inaccurate**: The docs list separate categories (Connectivity Tests, Docker Tests, Docker Compose Tests, Permission Tests) with individual checks (Docker Daemon, Docker CLI, Docker Info, Non-root Access, Docker Group, Socket Access, etc.). The actual implementation performs **external health checks** (Tracker API + HTTP Tracker endpoints) plus **advisory DNS resolution checks** — it does not individually test Docker, SSH, or permissions.

2. **"What Happens" section is misleading**: Lists 7 steps including "Checks VM connectivity", "Tests Docker installation", "Tests Docker Compose", "Verifies user permissions". The actual code has 3 steps: ValidateEnvironment → CreateCommandHandler → TestInfrastructure (external health checks + DNS).

3. **State validation claim**: Docs say "Tests verify the environment is in 'Configured' state". The actual code loads `AnyEnvironmentState` — it accepts the environment in **any state**.

4. **Exit codes**: Docs claim specific exit codes (0, 1, 2, 3) that likely don't match the actual `TestSubcommandError` variants.

5. **"Test Execution Flow"**: Lists steps (Docker Checks, Compose Checks, Permission Checks) that don't exist in the implementation.

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_test_result_data_with_no_warnings() {
        // Test DTO creation with empty DNS warnings
    }

    #[test]
    fn it_should_create_test_result_data_with_dns_warnings() {
        // Test DTO creation with DNS warnings
    }

    #[test]
    fn it_should_render_valid_json_with_no_warnings() {
        // Test JSON view produces valid JSON with empty dns_warnings array
    }

    #[test]
    fn it_should_render_valid_json_with_dns_warnings() {
        // Test JSON view produces valid JSON with populated dns_warnings
    }

    #[test]
    fn it_should_render_text_with_all_fields() {
        // Test text view contains all expected information
    }

    #[test]
    fn it_should_render_text_with_dns_warnings() {
        // Test text view renders DNS warnings correctly
    }
}
```

### Integration Tests

- [ ] Test `test` command with `--output-format json`
- [ ] Test `test` command with `--output-format text` (default)
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
- Roadmap section 12.8: `docs/roadmap.md`
- Reference implementation (release): `src/presentation/views/commands/release/`
- Reference implementation (configure): `src/presentation/views/commands/configure/`
