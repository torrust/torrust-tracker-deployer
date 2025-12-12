# JSON Schema Generation Specification

## 📋 Overview

Add a new `create schema` CLI subcommand that generates JSON Schema from the Rust configuration types used for environment creation. This enables IDE validation, auto-completion, and inline documentation for users editing environment JSON files.

### Context

Currently, users create environment configuration JSON files using the `create template` command, which generates a JSON file with placeholder values. However, users have no automated way to:

- Validate their JSON against the expected structure
- Get auto-completion when editing configuration
- See documentation for each field inline in their editor
- Detect typos or invalid values before running commands

Many modern IDEs and editors support JSON Schema for validation and tooling. By generating a schema from our Rust types, we can provide immediate feedback to users as they edit configuration files.

### Problem Statement

Users editing environment JSON files lack IDE support for validation, auto-completion, and documentation. This leads to:

1. **Configuration errors** discovered only at runtime
2. **Poor discoverability** of available options
3. **Trial and error** when filling in values
4. **No inline documentation** explaining what each field does
5. **Inconsistent formatting** across different users' files

## 🎯 Goals

### Primary Goals

- **Generate valid JSON Schema** from Rust configuration types using Schemars
- **Provide CLI command** to output schema: `create schema [PATH]`
- **Print to stdout** when no path is provided
- **Write to file** when path argument is given
- **Improve template output** to inform users about schema generation
- **Enable IDE integration** through standard JSON Schema format
- **AI agent support** - JSON Schema significantly enhances AI agents' ability to generate valid configuration files

### Secondary Goals (Nice-to-Have)

- Include Rust doc comments as descriptions in schema
- Add schema examples for common configuration patterns
- Provide IDE configuration examples (VS Code, IntelliJ)
- Auto-generate and commit schema to repository in CI

### Non-Goals

What this feature explicitly does NOT aim to do:

- Runtime validation using the schema (Rust deserialization already validates)
- Schema versioning or migration tooling
- IDE plugins or extensions
- Online schema registry or hosting
- Support for other config formats (TOML, YAML)

## 💡 Proposed Solution

### Approach

Use the [Schemars](https://graham.cool/schemars/) crate to derive JSON Schema from the existing Rust configuration types. Schemars provides a `JsonSchema` derive macro that works with Serde types, making it straightforward to generate schemas without duplicating type definitions.

**Why Schemars?**

- Works seamlessly with existing Serde derives
- Mature crate with wide adoption
- Supports custom schema attributes for fine-tuning
- Includes descriptions from doc comments
- Handles complex Rust types (enums, generics, options)

### Design Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    User runs command                        │
│           cargo run create schema [optional-path]           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Presentation Layer (CLI Parser)                │
│  - Parse `create schema` subcommand                         │
│  - Extract optional output path                             │
│  - Dispatch to CreateSchemaCommand                          │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│            Application Layer (Command Handler)              │
│  - CreateSchemaCommandHandler                               │
│  - Calls SchemaGenerator directly (no Step needed)          │
│  - Handles output routing (stdout vs file)                  │
│  - Manages success/error presentation via UserOutput        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Infrastructure Layer                           │
│  - SchemaGenerator (technical implementation)               │
│  - Uses schemars crate to generate JSON Schema              │
│  - Calls EnvironmentCreationConfig::json_schema()           │
│  - Returns schema as String (JSON format)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Config Types (with JsonSchema derive)          │
│  - EnvironmentCreationConfig                                │
│  - All nested types derive JsonSchema                       │
│  - Schema includes doc comments as descriptions             │
└─────────────────────────────────────────────────────────────┘
```

**Note**: No Step layer needed - command has only one operation. Handler directly calls infrastructure service.

### Key Design Decisions

1. **Use Schemars derive macro**: Minimizes code duplication and maintenance burden
2. **Output flexibility**: Support both stdout (for piping) and file output
3. **Schema location**: Generate from `EnvironmentCreationConfig` (top-level type)
4. **Update template output**: Inform users about schema availability after template creation
5. **No Step layer**: Command has single operation - handler directly calls `SchemaGenerator`
6. **Infrastructure placement**: `SchemaGenerator` is infrastructure (external dependency, technical mechanism)

### Alternatives Considered

#### Option 1: Manual Schema Definition

- **Pros**: Full control over schema structure, no new dependency
- **Cons**: High maintenance burden, prone to drift from Rust types, duplicates effort
- **Decision**: Rejected - too much manual work and error-prone

#### Option 2: Build-time Schema Generation

- **Pros**: Schema always in sync, can be committed to repo
- **Cons**: More complex build setup, harder to debug
- **Decision**: Deferred - start with runtime generation, consider build-time later

#### Option 3: External Schema Tool

- **Pros**: No Rust code changes needed
- **Cons**: Doesn't integrate with existing types, requires separate tooling
- **Decision**: Rejected - Schemars provides better integration

## 🔧 Implementation Details

### Architecture Changes

**No major architectural changes** - this feature adds new functionality without modifying existing components.

### Component Design

#### Component 1: Config Types with JsonSchema Derive

**Purpose**: Add JSON Schema generation capability to existing config types

**Changes**:

```rust
// src/application/command_handlers/create/config/environment_config.rs

use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentCreationConfig {
    /// Environment-specific settings
    pub environment: EnvironmentSection,

    /// SSH credentials configuration
    pub ssh_credentials: SshCredentialsConfig,

    /// Provider-specific configuration (LXD, Hetzner, etc.)
    pub provider: ProviderSection,

    /// Tracker deployment configuration
    pub tracker: TrackerSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentSection {
    /// Name of the environment to create
    /// Must follow environment naming rules
    pub name: String,

    /// Optional instance name override
    pub instance_name: Option<String>,
}
```

**Dependencies**: Schemars crate

#### Component 2: Schema Generator (Infrastructure)

**Purpose**: Generate JSON Schema from config types

**Interface**:

```rust
// src/infrastructure/schema/generator.rs

use schemars::schema_for;
use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

pub struct SchemaGenerator;

impl SchemaGenerator {
    /// Generate JSON Schema for environment configuration
    pub fn generate() -> Result<String, SchemaGenerationError> {
        let schema = schema_for!(EnvironmentCreationConfig);
        serde_json::to_string_pretty(&schema)
            .map_err(|e| SchemaGenerationError::SerializationFailed(e))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaGenerationError {
    #[error("Failed to serialize schema: {0}")]
    SerializationFailed(#[from] serde_json::Error),
}
```

**Dependencies**: Schemars, serde_json

**Design Note**: Placed in infrastructure layer because:

- Uses external crate (`schemars`) - could be swapped for alternatives
- Technical implementation detail, not business logic
- Format-specific (JSON Schema) - could add other formats (TOML schema, OpenAPI, etc.)

#### Component 3: Create Schema Command Handler (Application)

**Purpose**: Handle the `create schema` command - directly calls infrastructure service

**Interface**:

```rust
// src/application/command_handlers/create_schema/handler.rs

use std::path::PathBuf;
use crate::infrastructure::schema::SchemaGenerator;
use crate::presentation::views::UserOutput;

use super::errors::CreateSchemaError;

pub struct CreateSchemaCommandHandler;

impl CreateSchemaCommandHandler {
    pub fn handle(output_path: Option<PathBuf>, output: &mut UserOutput) -> Result<(), CreateSchemaError> {
        output.step(1, 1, "Generating JSON Schema...");

        match output_path {
            None => {
                // Generate and output to stdout
                let schema = SchemaGenerator::generate()
                    .map_err(CreateSchemaError::from)?;
                output.data(&schema);  // Output raw JSON schema to stdout
                output.success("JSON Schema generated");
            }
            Some(path) => {
                // Generate and write to file
                let schema = SchemaGenerator::generate()
                    .map_err(CreateSchemaError::from)?;
                std::fs::write(&path, schema)
                    .map_err(|e| CreateSchemaError::FileWriteFailed {
                        path: path.clone(),
                        source: e,
                    })?;
                output.success_with_detail(
                    "JSON Schema generated",
                    &format!("Schema written to: {}", path.display())
                );
            }
        }

        Ok(())
    }
}
```

**Dependencies**: Infrastructure SchemaGenerator, presentation views

**Design Notes**:

- **No Step layer** - single operation command calls infrastructure directly
- Handler accepts `&mut UserOutput` parameter for consistent output routing
- Uses `output.data()` to write schema to stdout (machine-readable JSON output)
- All output goes through `UserOutput` service - no direct `println!` usage
- Respects user's configured output formatter and verbosity settings

#### Component 4: CLI Integration (Presentation)

**Purpose**: Add `create schema` subcommand to CLI parser

**Interface**:

```rust
// src/presentation/cli.rs

#[derive(Subcommand)]
pub enum CreateSubcommand {
    /// Create a deployment environment configuration
    Environment {
        #[arg(long, value_name = "FILE")]
        env_file: PathBuf,
    },

    /// Generate a configuration template
    Template {
        #[arg(long, value_name = "PROVIDER")]
        provider: String,

        #[arg(value_name = "OUTPUT_PATH")]
        output_path: PathBuf,
    },

    /// Generate JSON Schema for environment configuration
    Schema {
        /// Optional output file path. If not provided, prints to stdout.
        #[arg(value_name = "OUTPUT_PATH")]
        output_path: Option<PathBuf>,
    },
}
```

**Dependencies**: Clap

### Data Model

No new data models required - schema is generated from existing config types.

### API Changes

**New CLI Command**:

```bash
# Print schema to stdout
cargo run create schema

# Write schema to file
cargo run create schema ./envs/environment-schema.json
```

**Updated Output for `create template`**:

```text
✅ Configuration template ready: ./envs/example.json

💡 Tip: Generate JSON Schema for IDE validation:
   torrust-tracker-deployer create schema ./envs/environment-schema.json
```

### Configuration

No new configuration options needed.

## 📊 Impact Analysis

### Files to Create

| File Path                                                   | Purpose                        | Effort |
| ----------------------------------------------------------- | ------------------------------ | ------ |
| `src/infrastructure/schema/mod.rs`                          | Schema module                  | Low    |
| `src/infrastructure/schema/generator.rs`                    | Schema generation logic        | Low    |
| `src/infrastructure/schema/errors.rs`                       | Schema-specific errors         | Low    |
| `src/application/command_handlers/create_schema/mod.rs`     | Command handler module         | Low    |
| `src/application/command_handlers/create_schema/handler.rs` | Command handler implementation | Medium |
| `src/application/command_handlers/create_schema/errors.rs`  | Command-specific errors        | Low    |
| `examples/environment-schema.json`                          | Example schema output          | Low    |
| `.vscode/settings.json.example`                             | VS Code integration example    | Low    |

### Files to Modify

| File Path                                                     | Changes Required                | Effort |
| ------------------------------------------------------------- | ------------------------------- | ------ |
| `Cargo.toml`                                                  | Add schemars dependency         | Low    |
| `src/application/command_handlers/create/config/*.rs`         | Add `JsonSchema` derives        | Low    |
| `src/presentation/cli.rs`                                     | Add `Schema` subcommand         | Low    |
| `src/presentation/dispatch/mod.rs`                            | Handle `Schema` subcommand      | Low    |
| `src/application/command_handlers/create_template_handler.rs` | Add schema tip to output        | Low    |
| `src/infrastructure/mod.rs`                                   | Export schema module            | Low    |
| `src/application/command_handlers/mod.rs`                     | Export create_schema module     | Low    |
| `src/presentation/views/user_output.rs`                       | (Optional) Add `raw()` method\* | Low    |
| `docs/user-guide/commands/create.md`                          | Document schema subcommand\*\*  | Medium |
| `docs/console-commands.md`                                    | Add schema command reference    | Low    |
| `README.md`                                                   | Mention schema generation       | Low    |
| `tests/e2e/create_command.rs`                                 | Add E2E tests for schema        | Medium |

\* **Optional Enhancement**: If `data()` method applies formatting that interferes with raw JSON output, add a `raw()` method to `UserOutput` that outputs unmodified strings to stdout. This ensures schema output remains valid JSON regardless of formatter settings.

\*\* **Documentation Location**: All `create` subcommand documentation goes in `docs/user-guide/commands/create.md` - don't create separate files for subcommands.

### Breaking Changes

**None** - This is a purely additive feature.

### Performance Impact

**Neutral to Positive**:

- Schema generation should complete in reasonable time (no specific requirement, just shouldn't hang or crash)
- No impact on other commands
- Improves user productivity (fewer config errors)

### Security Considerations

**Low Risk**:

- Schema generation is read-only operation
- No sensitive data in schema
- File writes use standard permissions
- No network access required

## 🗓️ Implementation Plan

### Phase 1: Foundation

- [ ] Add `schemars` dependency to `Cargo.toml`
- [ ] Add `JsonSchema` derive to `EnvironmentCreationConfig`
- [ ] Add `JsonSchema` derive to all nested config types
- [ ] Verify schema compiles without errors

**Estimated Time**: 1-2 hours

### Phase 2: Infrastructure Layer

- [ ] Create `src/infrastructure/schema/` module
- [ ] Implement `SchemaGenerator` with `generate()` method
- [ ] Create `SchemaGenerationError` with `.help()` method
- [ ] Write unit tests for schema generation

**Estimated Time**: 2-3 hours

### Phase 3: Application Layer

- [ ] Create `CreateSchemaCommandHandler` in `src/application/command_handlers/create_schema/`
- [ ] Handler directly calls `SchemaGenerator::generate()` (no Step layer needed)
- [ ] Implement stdout and file output logic in handler
- [ ] Create `CreateSchemaError` with `.help()` method
- [ ] Write unit tests for command handler

**Estimated Time**: 1-2 hours

**Note**: No Step layer - this command has only one operation

### Phase 4: Presentation Layer

- [ ] Add `Schema` subcommand to CLI parser in `src/presentation/cli.rs`
- [ ] Update dispatch logic to handle schema command
- [ ] Pass `UserOutput` instance to command handler (follow existing patterns)
- [ ] Use `output.data()` for schema output to stdout (never use `println!` directly)
- [ ] (Optional) Add `raw()` method to `UserOutput` if `data()` applies unwanted formatting
- [ ] Update `create_template_handler.rs` output with schema tip
- [ ] Write integration tests for CLI command

**Estimated Time**: 2-3 hours

**Important**: All output must go through `UserOutput` service to respect user's formatter and verbosity settings. Never use `println!`, `eprintln!`, or direct stdout/stderr writes in command handlers.

### Phase 5: Documentation & Examples

- [ ] Generate example schema file: `examples/environment-schema.json`
- [ ] Create VS Code settings example: `.vscode/settings.json.example`
- [ ] Update `docs/user-guide/commands/create.md`
- [ ] Update `docs/console-commands.md`
- [ ] Update README with schema generation mention
- [ ] Add troubleshooting section for IDE integration

**Estimated Time**: 2-3 hours

### Phase 6: Testing & Validation

- [ ] Run full test suite
- [ ] Test schema command with stdout output
- [ ] Test schema command with file output
- [ ] Validate generated schema against example JSON files
- [ ] Test IDE integration (VS Code)
- [ ] Run linters and fix issues

**Estimated Time**: 1-2 hours

### Phase 7: Finalization

- [ ] Code review
- [ ] Address feedback
- [ ] Update feature documentation status
- [ ] Commit with conventional commit message
- [ ] Create pull request

**Estimated Time**: 1-2 hours

## ✅ Definition of Done

The feature is complete when:

- [ ] All code changes implemented and tested
- [ ] `cargo run create schema` prints schema to stdout
- [ ] `cargo run create schema path/to/file.json` writes schema to file
- [ ] `create template` output mentions schema generation
- [ ] Generated schema validates example JSON files
- [ ] Unit tests pass for all new components
- [ ] Integration tests pass for CLI command
- [ ] Documentation updated (user guide, console commands, README)
- [ ] Example schema committed to repository
- [ ] VS Code settings example provided
- [ ] All linters pass
- [ ] No unused dependencies
- [ ] Feature marked as complete in `docs/features/README.md`

## 🧪 Testing Strategy

### Unit Tests

```rust
// tests for schema generator (infrastructure)
#[test]
fn it_should_generate_valid_json_schema_when_called() {
    let result = SchemaGenerator::generate();
    assert!(result.is_ok());

    let schema_str = result.unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
    assert!(schema["properties"].is_object());
    assert!(schema["properties"]["environment"].is_object());
    assert!(schema["properties"]["ssh_credentials"].is_object());
}

// tests for command handler (application)
#[test]
fn it_should_generate_schema_to_stdout_when_no_path_provided() {
    let mut output = UserOutput::new(VerbosityLevel::Normal);
    let result = CreateSchemaCommandHandler::handle(None, &mut output);
    assert!(result.is_ok());
}

#[test]
fn it_should_write_schema_to_file_when_path_provided() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("schema.json");
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    let result = CreateSchemaCommandHandler::handle(Some(output_path.clone()), &mut output);
    assert!(result.is_ok());
    assert!(output_path.exists());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("environment"));
    assert!(content.contains("ssh_credentials"));
}
```

### Integration Tests

```rust
// tests for CLI command
#[test]
fn it_should_output_schema_to_stdout_when_no_path_provided() {
    let output = Command::new("cargo")
        .args(&["run", "create", "schema"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("$schema"));
    assert!(stdout.contains("environment"));
}

#[test]
fn it_should_write_schema_to_file_when_path_provided() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("schema.json");

    let output = Command::new("cargo")
        .args(&["run", "create", "schema", output_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output_path.exists());
}
```

### Manual Testing Checklist

- [ ] Run `cargo run create schema` and verify output
- [ ] Run `cargo run create schema ./test-schema.json` and verify file created
- [ ] Configure VS Code to use schema for validation
- [ ] Edit environment JSON file and verify IDE shows errors for invalid values
- [ ] Verify IDE provides auto-completion for known fields
- [ ] Verify IDE shows descriptions from doc comments
- [ ] Test schema validates against all example JSON files in `envs/`

## 🔍 Validation Criteria

### Schema Quality

- Schema includes all fields from `EnvironmentCreationConfig`
- Descriptions are present (from Rust doc comments)
- Enum values are correctly represented
- Required vs optional fields are correctly marked
- Schema follows JSON Schema Draft 7 spec

### Code Quality

- All code follows project coding standards
- DDD layer placement is correct
- Error handling follows project principles
- All errors have `.help()` methods
- Code is well-documented with doc comments
- **All output uses `UserOutput` service** - no direct `println!` or `eprintln!` usage
- Handler accepts `&mut UserOutput` parameter following existing command handler patterns

### User Experience

- CLI command is intuitive and follows existing patterns
- Output messages are clear and actionable
- Error messages provide specific guidance
- Documentation is comprehensive and easy to follow

## 📚 Documentation Updates

### User Guide

Update `docs/user-guide/commands/create.md`:

````markdown
### Generate JSON Schema

Generate a JSON Schema for environment configuration files:

```bash
# Print schema to stdout
torrust-tracker-deployer create schema

# Write schema to file
torrust-tracker-deployer create schema ./envs/environment-schema.json
```
````

#### IDE Integration

To enable IDE validation and auto-completion:

1. Generate the schema file
2. Configure your IDE to associate JSON files with the schema

**VS Code Example:**

```json
{
  "json.schemas": [
    {
      "fileMatch": ["envs/*.json"],
      "url": "./envs/environment-schema.json"
    }
  ]
}
```

### Console Commands Reference

Update `docs/console-commands.md`:

````markdown
#### create schema

Generate JSON Schema for environment configuration.

**Usage:**

```bash
torrust-tracker-deployer create schema [OUTPUT_PATH]
```
````

**Arguments:**

- `OUTPUT_PATH` - Optional file path to write schema. If omitted, prints to stdout.

**Examples:**

```bash
# Print to stdout
torrust-tracker-deployer create schema

# Write to file
torrust-tracker-deployer create schema ./envs/environment-schema.json
```

## 🎓 Lessons Learned

(To be filled after implementation)

## 🔗 References

- [Schemars Documentation](https://graham.cool/schemars/)
- [JSON Schema Specification](https://json-schema.org/)
- [VS Code JSON Schema Support](https://code.visualstudio.com/docs/languages/json#_json-schemas-and-settings)

---

**Status**: Draft specification, awaiting implementation
**Created**: December 12, 2025
**Last Updated**: December 12, 2025
