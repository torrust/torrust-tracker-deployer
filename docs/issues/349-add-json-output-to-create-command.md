# Add JSON Output to Create Command

**Issue**: #349
**Parent Epic**: #348 - Add JSON output format support
**Related**: [Roadmap Section 12.1](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support)

## Overview

Add machine-readable JSON output format (`--json` flag) to the `create` command. This enables automation workflows to programmatically extract environment creation details like paths, configuration references, and initial state.

## Goals

- [ ] Add `--json` flag to `create` command CLI interface
- [ ] Implement JSON output format containing environment metadata
- [ ] Preserve existing human-readable output as default
- [ ] Document JSON schema and usage examples
- [ ] Enable automation to track artifact locations

## Rationale

The `create` command initializes a new deployment environment and outputs information about:

- Where environment data is stored (`data/{env-name}/`)
- Where build artifacts will be generated (`build/{env-name}/`)
- Configuration file location (if using `--env-file`)
- Initial state (Created)

**Why JSON matters here:**

- **Automation workflows**: CI/CD pipelines need to know artifact paths to coordinate next steps
- **Multi-environment management**: Scripts managing multiple environments need structured data
- **Integration**: Other tools need to discover where deployer stores environment information

Currently, this information is human-readable text. JSON output makes it programmatically parsable.

## Architecture Requirements

### DDD Layer

- **Layer**: Presentation (`src/presentation/`)
- **Module Path**: `src/presentation/console/subcommands/create/`

### Patterns to Follow

- Follow the MVC pattern already established in the codebase
- Separate business logic (already in application layer) from output formatting
- Add format switching at presentation layer only
- Do NOT modify application or domain layers

### Related Architecture Documentation

- [Codebase Architecture](../docs/codebase-architecture.md)
- [Output Handling Conventions](../docs/contributing/output-handling.md)
- [DDD Layer Placement Guidelines](../docs/contributing/ddd-layer-placement.md)

### UX Research & Design Decisions

For background on the dual-channel output strategy (stdout for results, stderr for progress):

- [Console App Output Patterns](../../docs/research/UX/console-app-output-patterns.md) - Research on industry patterns (cargo, docker, npm, kubectl)
- [User Output vs Logging Separation](../../docs/research/UX/user-output-vs-logging-separation.md) - Design rationale for separating user output from logs
- [Console stdout/stderr Handling](../../docs/research/UX/console-stdout-stderr-handling.md) - Unix conventions and best practices
- [Console Output & Logging Strategy](../../docs/research/UX/console-output-logging-strategy.md) - Implementation strategy

**Key takeaway**: JSON output goes to **stdout** (results channel), progress/logs go to **stderr** (operational channel). This enables clean piping and automation.

### Current Architecture Analysis

**Existing CLI Arguments for Output Control:**

The application currently has these output-related global arguments in [`src/presentation/input/cli/args.rs`](../../src/presentation/input/cli/args.rs):

| Argument              | Purpose                            | Channel        | Values                                   |
| --------------------- | ---------------------------------- | -------------- | ---------------------------------------- |
| `--log-file-format`   | Controls log formatting for files  | File logs      | `pretty`, `json`, `compact` (default)    |
| `--log-stderr-format` | Controls log formatting for stderr | stderr (logs)  | `pretty` (default), `json`, `compact`    |
| `--log-output`        | Controls log destination           | File vs stderr | `file-only` (default), `file-and-stderr` |

**What's Missing:**

- **No CLI argument for stdout format control** - There is no `--output-format` or similar flag to control the format of user-facing output that goes to stdout (where command results and human-readable messages appear).
- **FormatterOverride exists but unused** - The [`UserOutput`](../../src/presentation/views/user_output.rs) class has a `formatter_override: Option<Box<dyn FormatterOverride>>` field and the infrastructure exists ([`JsonFormatter`](../../src/presentation/views/formatters/json.rs)), but there's no CLI mechanism to activate it.

**Implementation Decision:**

For this task, we have two architectural choices:

1. **Command-specific flag** (recommended for Phase 1): Add `--json` flag to individual commands like `create`, `provision`, etc. This keeps the change localized and follows patterns from tools like `docker`, `kubectl`, and `npm`.

2. **Global output format flag** (future consideration): Add a global `--output-format` argument (similar to `--log-stderr-format`) that applies to all commands. This would require:
   - Adding `output_format: OutputFormat` to `GlobalArgs`
   - Passing this through execution context to `UserOutput`
   - Applying `FormatterOverride` based on the global flag

**Rationale for command-specific approach:**

- **Incremental adoption**: Not all commands produce structured output suitable for JSON
- **Simpler implementation**: No need to modify global argument handling or execution context
- **Clear opt-in**: Users explicitly request JSON where it makes sense
- **Industry pattern**: Common in CLI tools (`docker inspect --format=json`, `kubectl get pods -o json`)

**Note**: If multiple commands adopt JSON output (tasks 12.1-12.5), we may want to refactor to a global flag in a future iteration to reduce duplication.

### Architecture Gap: Missing View Layer

**Current State** - The `create` command violates MVC separation:

**❌ Create command (mixed architecture)**:

```rust
// File: src/presentation/controllers/create/subcommands/environment/handler.rs
// Lines 282-305

fn display_creation_results(&mut self, environment: &Environment<Created>) -> Result<...> {
    self.progress.complete(&format!("Environment '{}' created successfully", ...));
    self.progress.steps("Environment Details:", &[
        &format!("Environment name: {}", ...),
        &format!("Instance name: {}", ...),
        &format!("Data directory: {}", ...),
        &format!("Build directory: {}", ...),
    ])?;
}
```

**Problem**: Output formatting logic embedded directly in controller.

**✅ Provision command (correct architecture)**:

```rust
// File: src/presentation/controllers/provision/handler.rs
// Lines 269-271

// Uses dedicated view
self.progress.result(&ConnectionDetailsView::render(
    &ConnectionDetailsData::from(&provisioned)
))?;
```

**Architecture comparison**:

| Command     | View Module                                  | Controller Output Logic | Architecture Status |
| ----------- | -------------------------------------------- | ----------------------- | ------------------- |
| `provision` | `src/presentation/views/commands/provision/` | Calls `View::render()`  | ✅ Clean MVC        |
| `list`      | `src/presentation/views/commands/list/`      | Calls `View::render()`  | ✅ Clean MVC        |
| `show`      | `src/presentation/views/commands/show/`      | Calls `View::render()`  | ✅ Clean MVC        |
| `create`    | ❌ Missing                                   | Direct formatting       | ❌ Mixed concerns   |

**Consequence**: Adding JSON output to `create` command without refactoring would:

- Further entrench mixed concerns
- Make format switching harder (conditional logic in controller)
- Create technical debt for future commands
- Violate established MVC pattern

**Required Refactoring** (Prerequisite for JSON support):

1. **Extract view data structure**:

   ```rust
   // New file: src/presentation/views/commands/create/environment_details.rs

   pub struct EnvironmentDetailsData {
       pub environment_name: String,
       pub instance_name: String,
       pub data_dir: PathBuf,
       pub build_dir: PathBuf,
   }

   impl From<&Environment<Created>> for EnvironmentDetailsData { ... }
   ```

2. **Create view with format switching**:

   ```rust
   pub struct EnvironmentDetailsView;

   impl EnvironmentDetailsView {
       pub fn render_human_readable(data: &EnvironmentDetailsData) -> String { ... }
       pub fn render_json(data: &EnvironmentDetailsData) -> String { ... }
   }
   ```

3. **Update controller to use view**:

   ```rust
   fn display_creation_results(&mut self, environment: &Environment<Created>) -> Result<...> {
       let data = EnvironmentDetailsData::from(environment);
       let output = EnvironmentDetailsView::render_human_readable(&data);
       self.progress.result(&output)?;
   }
   ```

**Benefits of refactoring first**:

- ✅ Consistent with `provision`, `list`, `show` commands
- ✅ Format switching becomes straightforward (call different view method)
- ✅ Controller stays focused on orchestration
- ✅ Views become independently testable
- ✅ Easier to add more formats in future (XML, YAML, CSV, etc.)

**Implementation Strategy**:

This refactoring should be done in **two separate commits**:

1. **Commit 1**: Extract view (preserving behavior)
   - Create view module structure
   - Move formatting logic to view
   - Update controller to call view
   - Verify output unchanged (run golden test)

2. **Commit 2**: Add JSON format support
   - Add `--json` CLI flag
   - Add `render_json()` method to view
   - Add format switching in controller
   - Update tests and documentation

## Specifications

### CLI Interface

```bash
# Human-readable output (default, unchanged)
torrust-tracker-deployer create environment --env-file envs/my-env.json

# JSON output (new)
torrust-tracker-deployer create environment --env-file envs/my-env.json --json
```

### Interaction with Existing `--log-output` Flag

The `--json` flag controls **user-facing output format**, while `--log-output` controls **logging destination**. These are independent concerns that work together:

| Flag           | Purpose                                            | Output Channel |
| -------------- | -------------------------------------------------- | -------------- |
| `--json`       | User output format (JSON vs human-readable)        | stdout         |
| `--log-output` | Logging destination (file-only vs file-and-stderr) | stderr or file |

**Key points:**

- **Logs** (tracing data with progress indicators like `⏳`, `✓`, `❌`) go to stderr or file based on `--log-output`
- **User output** (success message and environment details) goes to stdout
- When `--json` is used, the JSON goes to stdout, logs continue to stderr/file
- These flags do not conflict - they can be used together

**Examples:**

```bash
# Production: JSON output to stdout, logs to file only
torrust-tracker-deployer create environment --env-file envs/my-env.json --json --log-output file-only

# Development: JSON output to stdout, logs to both file and stderr
torrust-tracker-deployer create environment --env-file envs/my-env.json --json --log-output file-and-stderr

# Default: Human-readable output, logs to file only
torrust-tracker-deployer create environment --env-file envs/my-env.json
```

**Rationale:** Separating user output (stdout) from logs (stderr) is a Unix best practice that enables:

- Clean piping: `create --json | jq .data_dir` extracts only the JSON, no log noise
- Proper redirection: `create --json > output.json 2> logs.txt` separates concerns
- Tool integration: JSON parsers don't see log messages

### JSON Output Schema

```json
{
  "environment_name": "my-env",
  "state": "Created",
  "data_dir": "data/my-env",
  "build_dir": "build/my-env",
  "config_file": "envs/my-env.json",
  "state_file": "data/my-env/environment.json",
  "created_at": "2026-02-13T13:00:00Z"
}
```

### Field Descriptions

| Field              | Type   | Description                                        |
| ------------------ | ------ | -------------------------------------------------- |
| `environment_name` | string | Name of the created environment                    |
| `state`            | string | Current state (always "Created" for this command)  |
| `data_dir`         | string | Path to environment data directory                 |
| `build_dir`        | string | Path where build artifacts will be generated       |
| `config_file`      | string | Path to configuration file (if using `--env-file`) |
| `state_file`       | string | Path to environment state JSON file                |
| `created_at`       | string | ISO 8601 timestamp of creation                     |

### Human-Readable Output (Unchanged)

The default output should remain exactly as it is now. This is the **golden test** - the JSON output implementation must not break this existing behavior.

**Test Configuration**: `envs/golden-test-json-create.json`

```json
{
  "environment": {
    "name": "golden-test-json-create",
    "description": "Golden test for JSON output - create command",
    "instance_name": null
  },
  "ssh_credentials": {
    "private_key_path": "/home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa",
    "public_key_path": "/home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa.pub",
    "username": "deployer",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "default"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "127.0.0.1:1313"
    }
  },
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin"
  },
  "https": null,
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

**Expected Output** (captured on 2026-02-13):

```text
⏳ [1/3] Loading configuration...
⏳     → Loading configuration from 'envs/golden-test-json-create.json'...
⏳   ✓ Configuration loaded: golden-test-json-create (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Creating environment...
⏳     → Creating environment 'golden-test-json-create'...
⏳     → Validating configuration and creating environment...
⏳   ✓ Environment created: golden-test-json-create (took 11ms)
✅ Environment 'golden-test-json-create' created successfully

Environment Details:
1. Environment name: golden-test-json-create
2. Instance name: torrust-tracker-vm-golden-test-json-create
3. Data directory: ./data/golden-test-json-create
4. Build directory: ./build/golden-test-json-create
```

**Note**: The golden test configuration is stored in the repository at `envs/golden-test-json-create.json` and can be used during development to verify backward compatibility.

## Implementation Plan

### Phase 0: Refactor - Extract View Layer (Prerequisite)

**Purpose**: Separate output formatting from controller logic to enable clean format switching.

- [ ] Create view module structure: `src/presentation/views/commands/create/`
- [ ] Create `EnvironmentDetailsData` struct (presentation DTO)
- [ ] Implement `From<&Environment<Created>>` for data conversion
- [ ] Create `EnvironmentDetailsView` with `render_human_readable()` method
- [ ] Update controller to use view instead of direct formatting
- [ ] Run golden test to verify output unchanged
- [ ] Commit refactoring (preserving behavior)

**Files to create:**

- `src/presentation/views/commands/create/mod.rs`
- `src/presentation/views/commands/create/environment_details.rs`

**Files to modify:**

- `src/presentation/controllers/create/subcommands/environment/handler.rs` (lines 282-305)
- `src/presentation/views/commands/mod.rs` (add `create` module)

**Expected changes in controller**:

```diff
- fn display_creation_results(&mut self, environment: &Environment<Created>) -> Result<...> {
-     self.progress.complete(&format!("Environment '{}' created successfully", ...));
-     self.progress.steps("Environment Details:", &[
-         &format!("Environment name: {}", ...),
-         &format!("Instance name: {}", ...),
-         &format!("Data directory: {}", ...),
-         &format!("Build directory: {}", ...),
-     ])?;
- }
+ fn display_creation_results(&mut self, environment: &Environment<Created>) -> Result<...> {
+     let data = EnvironmentDetailsData::from(environment);
+     let output = EnvironmentDetailsView::render_human_readable(&data);
+     self.progress.result(&output)?;
+ }
```

**Verification**:

```bash
# Run golden test to ensure output unchanged
torrust-tracker-deployer create environment --env-file envs/golden-test-json-create.json

# Compare with expected output (documented in this spec)
# All 4 lines should match exactly
```

### Phase 1: Add CLI Flag

- [ ] Add `--json` flag to `create` subcommand argument parser
- [ ] Pass format flag through to presentation layer
- [ ] No business logic changes

**Files to modify:**

- `src/presentation/console/subcommands/create/mod.rs` or wherever CLI args are defined

### Phase 2: Add JSON Output Method to View

- [ ] Add `render_json()` method to `EnvironmentDetailsView`
- [ ] Implement JSON serialization using `serde_json`
- [ ] Use existing `EnvironmentDetailsData` struct (add `Serialize` derive)
- [ ] Include timestamp field (`created_at`) in JSON output

**Files to modify:**

- `src/presentation/views/commands/create/environment_details.rs`

**New code:**

```rust
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize)]  // Add Serialize
pub struct EnvironmentDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    pub data_dir: String,        // Use String for JSON serialization
    pub build_dir: String,        // Use String for JSON serialization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,
    pub state: String,
    pub state_file: String,
    pub created_at: DateTime<Utc>,
}

impl EnvironmentDetailsView {
    pub fn render_human_readable(data: &EnvironmentDetailsData) -> String {
        // Existing implementation
    }

    pub fn render_json(data: &EnvironmentDetailsData) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(data)
    }
}
```

### Phase 3: Implement Format Switching

- [ ] Add conditional logic based on `--json` flag in controller
- [ ] Call `EnvironmentDetailsView::render_json()` when flag set
- [ ] Call `EnvironmentDetailsView::render_human_readable()` otherwise (default)
- [ ] Handle JSON serialization errors appropriately

**Pattern:**

```rust
fn display_creation_results(
    &mut self,
    environment: &Environment<Created>,
    format: OutputFormat,  // Passed from CLI args
) -> Result<(), CreateEnvironmentCommandError> {
    let data = EnvironmentDetailsData::from(environment);

    match format {
        OutputFormat::Json => {
            let json_output = EnvironmentDetailsView::render_json(&data)
                .map_err(|e| CreateEnvironmentCommandError::JsonSerializationFailed { source: e })?;
            self.progress.result(&json_output)?;
        }
        OutputFormat::HumanReadable => {
            let output = EnvironmentDetailsView::render_human_readable(&data);
            self.progress.result(&output)?;
        }
    }

    Ok(())
}
```

**Files to modify:**

- `src/presentation/controllers/create/subcommands/environment/handler.rs`
- `src/presentation/controllers/create/errors.rs` (add JSON serialization error variant)

### Phase 4: Documentation

- [ ] Update user guide with JSON output examples
- [ ] Document JSON schema
- [ ] Add usage examples for automation

**Files to create/modify:**

- `docs/user-guide/commands/create.md` (update existing)
- Add JSON output section with schema and examples

### Phase 5: Testing

- [ ] Manual testing: verify JSON is valid with `--json` flag
- [ ] Manual testing: verify default output unchanged without flag
- [ ] Manual testing: pipe to `jq` to verify parsability
- [ ] Consider adding integration test (optional for v1)

## Acceptance Criteria

### Architecture

- [ ] View layer extracted for create command (Phase 0 complete)
- [ ] Controller delegates output formatting to view
- [ ] View module structure matches `provision`, `list`, `show` commands
- [ ] Golden test passes after refactoring (output unchanged)
- [ ] Commit 1: Refactoring (behavior preserved)
- [ ] Commit 2: JSON support (new feature)

### Functionality

- [ ] `--json` flag is accepted by create command
- [ ] With `--json` flag, command outputs valid JSON to stdout
- [ ] JSON contains all specified fields with correct values
- [ ] JSON is parsable by standard tools (`jq`, `serde_json`, etc.)
- [ ] Without `--json` flag, output is unchanged (human-readable format)
- [ ] Errors are still output to stderr (not to stdout)

### Code Quality

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All linters pass (clippy, rustfmt)
- [ ] No unused dependencies added
- [ ] Code follows existing patterns in presentation layer
- [ ] No changes to application or domain layers

### Documentation

- [ ] User guide updated with JSON output section
- [ ] JSON schema documented with field descriptions
- [ ] At least one usage example provided
- [ ] Automation use case documented

### User Experience

- [ ] Default behavior (no flag) is identical to before
- [ ] JSON output is pretty-printed for readability
- [ ] Timestamps use ISO 8601 format
- [ ] Paths use forward slashes (cross-platform)

## Testing

### Phase 0: Refactoring Verification

**Test case**: Verify output unchanged after view extraction

```bash
# After Phase 0 refactoring
torrust-tracker-deployer create environment --env-file envs/golden-test-json-create.json

# Expected output should match exactly the golden test output documented above
# All 4 detail lines should be present and formatted identically
```

### Manual Test Cases (JSON Feature)

1. **Basic JSON output**:

   ```bash
   torrust-tracker-deployer create --env-file envs/test.json --json
   ```

   - Should output valid JSON
   - Should contain all required fields

2. **Default output unchanged**:

   ```bash
   torrust-tracker-deployer create --env-file envs/test.json
   ```

   - Should output human-readable text (no JSON)
   - Output should match pre-change behavior

3. **JSON parsability**:

   ```bash
   torrust-tracker-deployer create --env-file envs/test.json --json | jq .
   ```

   - `jq` should successfully parse the output
   - No errors

4. **Extract specific field**:

   ```bash
   DATA_DIR=$(torrust-tracker-deployer create environment --env-file envs/test.json --json | jq -r .data_dir)
   echo "Data directory: $DATA_DIR"
   ```

   - Should successfully extract field value
   - Demonstrates automation use case

5. **JSON with file-only logging** (production scenario):

   ```bash
   torrust-tracker-deployer create environment --env-file envs/test.json --json --log-output file-only
   ```

   - JSON should go to stdout only
   - No log messages on stderr (only in log file)
   - Clean output for piping

6. **JSON with file-and-stderr logging** (development scenario):

   ```bash
   torrust-tracker-deployer create environment --env-file envs/test.json --json --log-output file-and-stderr
   ```

   - JSON should go to stdout
   - Logs should appear on stderr
   - JSON should not be mixed with logs

7. **Output channel separation**:

   ```bash
   torrust-tracker-deployer create environment --env-file envs/test.json --json --log-output file-and-stderr > output.json 2> logs.txt
   ```

   - `output.json` should contain only the JSON (no log messages)
   - `logs.txt` should contain only log messages (no JSON)
   - Verifies proper stdout/stderr separation

## Related Documentation

- [Epic #348 - Add JSON output format support](../issues/348-epic-add-json-output-format-support.md)
- [Roadmap Section 12](../roadmap.md#12-add-json-output-format-support)
- [Output Handling Conventions](../contributing/output-handling.md)
- [User Guide - Create Command](../user-guide/commands/create.md)

## Notes

- The `serde_json` dependency is already in `Cargo.toml` (used for state serialization)
- Follow the existing pattern from other commands in the presentation layer
- Keep business logic in application layer - only format output in presentation layer
- Consider this as a template for implementing JSON output in other commands (12.2-12.5)
