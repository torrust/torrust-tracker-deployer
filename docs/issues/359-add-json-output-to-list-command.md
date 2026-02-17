# Add JSON Output to List Command

**Issue**: [#359](https://github.com/torrust/torrust-tracker-deployer/issues/359)
**Parent Epic**: [#348](https://github.com/torrust/torrust-tracker-deployer/issues/348) - Add JSON output format support
**Related**: [Roadmap Section 12.5](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support), [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed, [Issue #352 - Add JSON output to provision command](https://github.com/torrust/torrust-tracker-deployer/issues/352) ✅ Completed, [Issue #355 - Add JSON output to show command](https://github.com/torrust/torrust-tracker-deployer/issues/355) ✅ Completed, [Issue #357 - Add JSON output to run command](https://github.com/torrust/torrust-tracker-deployer/issues/357) ✅ Completed

**Implementation Status**: ⏳ **NOT STARTED**

## Overview

Add machine-readable JSON output format (`--output-format json`) to the `list` command. This enables automation workflows and AI agents to programmatically extract environment information (full names, states, providers, timestamps) without parsing human-readable text tables that may truncate long names.

## Goals

- [ ] Implement JSON output format for list command
- [ ] Preserve existing human-readable table output as default
- [ ] Enable automation to extract full environment names without truncation
- [ ] Follow the architecture pattern established in #349, #352, #355, and #357

## Rationale

**Why JSON Output for List?**

The table format truncates long environment names to fit the terminal width, making it difficult to:

- Parse environment names programmatically
- Disambiguate environments with long names
- Extract complete environment information for automation workflows

JSON output provides:

- Full environment names without truncation
- Structured data for easy parsing with `jq` or programming languages
- Complete state and timestamp information for each environment
- Failed environment details for error handling

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (`src/presentation/`)
**Module Path**: `src/presentation/views/commands/list/`
**Pattern**: Strategy Pattern with TextView and JsonView (established in previous implementations)

### Current Module Structure

```rust
src/presentation/views/commands/list/
├── views/
│   ├── mod.rs        (Re-exports TextView)
│   └── text_view.rs  (TextView - human-readable table output)
└── mod.rs            (Re-exports views module)
```

### Required Module Structure

```rust
src/presentation/views/commands/list/
├── views/
│   ├── mod.rs        (Re-exports TextView and JsonView)
│   ├── text_view.rs  (TextView - human-readable table output)
│   └── json_view.rs  (NEW - JsonView implementation)
└── mod.rs            (Re-exports views module)
```

**Key Changes Required:**

1. Create new `views/json_view.rs` with `JsonView` struct
2. Update `views/mod.rs` to re-export both `TextView` and `JsonView`
3. Update `ListCommandController` to accept `output_format` parameter
4. Update router to pass `output_format` from `ExecutionContext`
5. Wire output_format through ExecutionContext → Router → Controller

**Note**: The list command uses the `EnvironmentList` DTO from the application layer (`src/application/command_handlers/list/info/`). Both TextView and JsonView consume the same `EnvironmentList` DTO.

### Architectural Constraints

- [ ] No business logic in presentation layer (views only format existing data)
- [ ] No changes to application or domain layers
- [ ] Follow output handling conventions ([docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] Use existing `OutputFormat` enum and `--output-format` flag

### Anti-Patterns to Avoid

- ❌ Embedding output formatting logic in controller
- ❌ Mixing business logic with view formatting
- ❌ Changing the application command handler interface
- ❌ Creating redundant view_data DTOs (use existing `EnvironmentList`)

## Specifications

### JSON Output Schema

The JSON output should follow the structure of the `EnvironmentList` DTO from the application layer (`src/application/command_handlers/list/info/`).

#### Example 1: Empty Workspace

```json
{
  "environments": [],
  "total_count": 0,
  "failed_environments": [],
  "data_directory": "/path/to/project/data"
}
```

#### Example 2: Single Environment

```json
{
  "environments": [
    {
      "name": "my-production-env",
      "state": "Running",
      "provider": "Hetzner",
      "created_at": "2026-02-15T14:30:00Z"
    }
  ],
  "total_count": 1,
  "failed_environments": [],
  "data_directory": "/path/to/project/data"
}
```

#### Example 3: Multiple Environments

```json
{
  "environments": [
    {
      "name": "staging-environment",
      "state": "Running",
      "provider": "LXD",
      "created_at": "2026-02-10T09:15:00Z"
    },
    {
      "name": "production-high-availability-tracker",
      "state": "Provisioned",
      "provider": "Hetzner",
      "created_at": "2026-02-14T16:45:00Z"
    },
    {
      "name": "development-test-instance",
      "state": "Created",
      "provider": "LXD",
      "created_at": "2026-02-16T11:20:00Z"
    }
  ],
  "total_count": 3,
  "failed_environments": [],
  "data_directory": "/path/to/project/data"
}
```

#### Example 4: With Partial Failures

```json
{
  "environments": [
    {
      "name": "production-env",
      "state": "Running",
      "provider": "Hetzner",
      "created_at": "2026-02-15T14:30:00Z"
    }
  ],
  "total_count": 1,
  "failed_environments": [
    ["corrupted-env", "Failed to deserialize environment.json: unexpected EOF"],
    ["invalid-state", "Unknown state in environment file"]
  ],
  "data_directory": "/path/to/project/data"
}
```

> **Note on Schema Flexibility**: The JSON schema shown above is **not mandatory**. The actual JSON output should mirror the structure of the Rust `EnvironmentList` DTO. If the natural Rust serialization (via `#[derive(Serialize)]`) produces a slightly different format that is easier to maintain or more idiomatic, **prefer the Rust-native structure**. The goal is simplicity and consistency with the codebase, not rigid adherence to a predefined schema. The examples above serve as a guide for the expected information.

### Field Descriptions

#### Top-Level Fields

| Field                 | Type                         | Description                                     |
| --------------------- | ---------------------------- | ----------------------------------------------- |
| `environments`        | array[EnvironmentSummary]    | Successfully loaded environment summaries       |
| `total_count`         | number                       | Total count of successfully loaded environments |
| `failed_environments` | array[tuple(string, string)] | Environments that failed to load (name, error)  |
| `data_directory`      | string                       | Path to the data directory that was scanned     |

#### EnvironmentSummary Fields

| Field        | Type   | Description                                      |
| ------------ | ------ | ------------------------------------------------ |
| `name`       | string | Environment name (full name, no truncation)      |
| `state`      | string | Human-readable state (Created, Provisioned, etc) |
| `provider`   | string | Infrastructure provider (LXD, Hetzner)           |
| `created_at` | string | ISO 8601 timestamp of environment creation       |

### Command Behavior

#### Command Syntax

```bash
# Text output (default)
torrust-tracker-deployer list

# JSON output
torrust-tracker-deployer list --output-format json
# Or short form:
torrust-tracker-deployer list -o json
```

#### Output Routing

- **Text format**: Renders table with progress messages to stdout
- **JSON format**: Renders JSON object to stdout (no progress messages in JSON)
- **Errors**: Always to stderr (both formats)

## Implementation Plan

### Step 1: Create JsonView

**File**: `src/presentation/views/commands/list/views/json_view.rs`

```rust
//! JSON View for List Command Output
//!
//! This module provides JSON-based rendering for the list command output.

use serde::Serialize;

use crate::application::command_handlers::list::info::EnvironmentList;

/// View for rendering list command output as JSON
pub struct JsonView;

impl JsonView {
    /// Render list command output as JSON
    ///
    /// Serializes the environment list to pretty-printed JSON format.
    ///
    /// # Arguments
    ///
    /// * `list` - Environment list containing summaries and metadata
    ///
    /// # Returns
    ///
    /// A JSON string containing the serialized environment list.
    #[must_use]
    pub fn render(list: &EnvironmentList) -> String {
        serde_json::to_string_pretty(list)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize: {e}"}}"#))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::command_handlers::list::info::EnvironmentSummary;

    #[test]
    fn it_should_render_empty_list_as_json() {
        let list = EnvironmentList::new(vec![], vec![], "/path/to/data".to_string());
        let output = JsonView::render(&list);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_count"], 0);
        assert!(parsed["environments"].is_array());
        assert!(parsed["failed_environments"].is_array());
    }

    #[test]
    fn it_should_render_single_environment() {
        let summary = EnvironmentSummary::new(
            "my-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-02-16T10:00:00Z".to_string(),
        );
        let list = EnvironmentList::new(vec![summary], vec![], "/path/to/data".to_string());
        let output = JsonView::render(&list);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_count"], 1);
        assert_eq!(parsed["environments"][0]["name"], "my-env");
    }

    #[test]
    fn it_should_include_failures() {
        let failures = vec![
            ("corrupted-env".to_string(), "Parse error".to_string()),
        ];
        let list = EnvironmentList::new(vec![], failures, "/path/to/data".to_string());
        let output = JsonView::render(&list);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["failed_environments"].as_array().unwrap().len(), 1);
    }
}
```

**Key Requirements:**

- Use `serde_json::to_string_pretty()` for readable output
- Handle serialization errors gracefully (return error JSON)
- Add comprehensive unit tests (empty list, single environment, multiple environments, with failures)

### Step 2: Add Serialize Derive to DTOs

Ensure the DTO types have `#[derive(Serialize)]`:

**Files to Update:**

- `src/application/command_handlers/list/info.rs`
  - `EnvironmentList` - should have `#[derive(Serialize)]`
  - `EnvironmentSummary` - should have `#[derive(Serialize)]`

**Example:**

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentSummary {
    pub name: String,
    pub state: String,
    pub provider: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentList {
    pub environments: Vec<EnvironmentSummary>,
    pub total_count: usize,
    pub failed_environments: Vec<(String, String)>,
    pub data_directory: String,
}
```

### Step 3: Update Module Structure

**File**: `src/presentation/views/commands/list/views/mod.rs`

```rust
//! Views for List Command
//!
//! This module contains different view implementations for rendering
//! list command output using the Strategy Pattern.

mod json_view;
mod text_view;

pub use json_view::JsonView;
pub use text_view::TextView;
```

### Step 4: Update Controller

**File**: `src/presentation/controllers/list/handler.rs`

Update the controller to accept `output_format` and choose the appropriate view:

```rust
// Add import
use crate::presentation::input::cli::OutputFormat;
use crate::presentation::views::commands::list::{JsonView, TextView};

impl ListCommandController {
    /// Execute the list command workflow
    ///
    /// # Arguments
    ///
    /// * `output_format` - Output format (Text or Json)
    ///
    /// # Errors
    ///
    /// Returns `ListSubcommandError` if any step fails
    pub fn execute(&mut self, output_format: OutputFormat) -> Result<(), ListSubcommandError> {
        let env_list = self.scan_environments()?;
        self.display_results(&env_list, output_format)?;
        Ok(())
    }

    fn display_results(
        &mut self,
        env_list: &EnvironmentList,
        output_format: OutputFormat,
    ) -> Result<(), ListSubcommandError> {
        self.progress
            .start_step(ListStep::DisplayResults.description())?;

        // Render using appropriate view based on output format (Strategy Pattern)
        let output = match output_format {
            OutputFormat::Text => TextView::render(env_list),
            OutputFormat::Json => JsonView::render(env_list),
        };

        // Pipeline: EnvironmentList → render → output to stdout
        self.progress.result(&output)?;

        self.progress.complete_step(Some("Results displayed"))?;

        Ok(())
    }
}
```

### Step 5: Update Router

**File**: `src/presentation/dispatch/router.rs`

Update the router to pass `output_format` to the controller:

```rust
Commands::List => {
    let output_format = context.output_format();
    context
        .container()
        .create_list_controller()
        .execute(output_format)?;
    Ok(())
}
```

### Step 6: Update Tests

**Files to Update:**

- Unit tests in `src/presentation/controllers/list/handler.rs`
- Update all test calls to `execute()` to pass `OutputFormat::Text`

**Example:**

```rust
controller.execute(OutputFormat::Text)?;
```

### Step 7: Update Documentation

**File**: `docs/user-guide/commands/list.md` (create if it doesn't exist)

Add comprehensive JSON output documentation:

- Command syntax with `--output-format json` flag
- JSON output structure with examples
- Automation use cases with `jq` examples
- Field descriptions table

**Example sections to include:**

```markdown
## JSON Output

The `list` command supports JSON output for automation workflows using the `--output-format json` or `-o json` flag.

### Command Syntax

\`\`\`bash
torrust-tracker-deployer list --output-format json

# Or use the short form:

torrust-tracker-deployer list -o json
\`\`\`

### Automation Use Cases

#### Extract All Environment Names

\`\`\`bash

# Get array of environment names

torrust-tracker-deployer list -o json | jq -r '.environments[].name'
\`\`\`

#### Filter Environments by State

\`\`\`bash

# Get only running environments

torrust-tracker-deployer list -o json | jq -r '.environments[] | select(.state == "Running") | .name'
\`\`\`

#### Count Environments by Provider

\`\`\`bash

# Count LXD vs Hetzner environments

torrust-tracker-deployer list -o json | jq '.environments | group_by(.provider) | map({provider: .[0].provider, count: length})'
\`\`\`

#### Check for Failed Environments

\`\`\`bash

# Exit with error if any environment failed to load

HAS_FAILURES=$(torrust-tracker-deployer list -o json | jq '.failed_environments | length > 0')
if [ "$HAS_FAILURES" = "true" ]; then
echo "Warning: Some environments failed to load"
torrust-tracker-deployer list -o json | jq -r '.failed_environments[] | "\(.[0]): \(.[1])"'
fi
\`\`\`
```

## Testing Requirements

### Unit Tests

- [ ] JsonView renders empty list correctly
- [ ] JsonView renders single environment
- [ ] JsonView renders multiple environments
- [ ] JsonView includes failed environments
- [ ] JsonView produces valid JSON (parseable)
- [ ] Controller passes output_format to display_results
- [ ] All existing controller tests updated to pass OutputFormat::Text

### Integration Tests

- [ ] E2E test: `list -o json` produces valid JSON
- [ ] E2E test: JSON contains all environment names without truncation
- [ ] E2E test: JSON includes failed_environments when corruption exists
- [ ] E2E test: Empty workspace returns valid JSON with empty arrays

### Verification Checklist

- [ ] All unit tests pass (`cargo test`)
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] JSON output is valid (can be parsed by `jq`)
- [ ] Text output unchanged (backward compatible)
- [ ] Documentation includes automation examples

## Success Criteria

- [ ] ✅ JSON output implemented following Strategy Pattern
- [ ] ✅ Full environment names provided without truncation
- [ ] ✅ All tests passing (unit + integration)
- [ ] ✅ All linters passing
- [ ] ✅ Documentation updated with JSON examples and automation use cases
- [ ] ✅ Backward compatible (text output unchanged)

## Related Documentation

- [Contributing: Error Handling](../contributing/error-handling.md)
- [Contributing: Output Handling](../contributing/output-handling.md)
- [Contributing: DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Contributing: Module Organization](../contributing/module-organization.md)

## References

- **Pattern Reference**: [Issue #349](https://github.com/torrust/torrust-tracker-deployer/issues/349) - Create command JSON output (original implementation)
- **Similar Implementation**: [Issue #352](https://github.com/torrust/torrust-tracker-deployer/issues/352) - Provision command JSON output
- **Similar Implementation**: [Issue #355](https://github.com/torrust/torrust-tracker-deployer/issues/355) - Show command JSON output
- **Similar Implementation**: [Issue #357](https://github.com/torrust/torrust-tracker-deployer/issues/357) - Run command JSON output
- **Parent Epic**: [Issue #348](https://github.com/torrust/torrust-tracker-deployer/issues/348) - Add JSON output format support
- **Roadmap**: [Section 12.5](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support)
