# JSON Schema Generation for Environment Configuration

**Issue**: [#229](https://github.com/torrust/torrust-tracker-deployer/issues/229)
**Parent Epic**: N/A (Standalone task)
**Related**: [Feature Specification](../features/json-schema-generation/)

## Overview

Add a new CLI command `cargo run create schema [OUTPUT_PATH]` that generates a JSON Schema from the Rust `EnvironmentCreationConfig` type. This enables AI assistants, IDEs, and developers to benefit from autocomplete, validation, and inline documentation when creating environment configuration files.

## Goals

- [x] Generate JSON Schema from Rust configuration types using Schemars
- [x] Support output to stdout or file
- [x] Include doc comments as schema descriptions
- [x] Update template command to inform users about schema generation
- [x] Provide high-quality AI agent support for configuration file creation

## 🏗️ Architecture Requirements

**DDD Layer**: Application + Infrastructure
**Module Path**:

- CLI: `src/presentation/input/cli/commands/create/subcommands/schema/`
- Handler: `src/application/command_handlers/create/schema/`
- Generator: `src/infrastructure/schema/`
  **Pattern**: CLI Subcommand → Command Handler → Schema Generator (Infrastructure)

### Module Structure Requirements

- [x] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [x] Respect dependency flow rules (Presentation → Application → Infrastructure)
- [x] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [x] CLI parsing in Presentation layer (Clap structures)
- [x] Handler orchestration in Application layer
- [x] Schema generation in Infrastructure layer (external dependency: Schemars)
- [x] All output through `UserOutput` service (no direct `println!`)
- [x] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Using `println!` or `eprintln!` directly (must use `UserOutput`)
- ❌ Business logic in Infrastructure layer (SchemaGenerator is pure technical mechanism)
- ❌ Schemars dependency leaking into Application or Domain layers

## Specifications

See complete specifications in the [feature documentation](../features/json-schema-generation/specification.md), including:

- Component designs (Config Types, SchemaGenerator, CommandHandler, CLI)
- Architecture decisions (no Step layer, SchemaGenerator placement)
- Command structure and arguments
- Error handling requirements
- Testing strategy

### Key Design Decisions

1. **No Step Layer**: This is a single-operation command (see [specification rationale](../features/json-schema-generation/specification.md#design-overview))
2. **SchemaGenerator in Infrastructure**: External dependency wrapper, technical mechanism (see [ADR on layer placement](../contributing/ddd-layer-placement.md))
3. **All Output via UserOutput**: Enforces consistent output routing pattern

## Implementation Plan

### Phase 1: Add Schemars Derive (30 minutes)

- [x] Add `schemars = "0.8"` to `Cargo.toml`
- [x] Add `#[derive(JsonSchema)]` to `EnvironmentCreationConfig`
- [x] Add `#[derive(JsonSchema)]` to all nested config types (provider configs, SSH config, etc.)
- [x] Add doc comments to all public fields for schema descriptions
- [x] Build to verify derives work correctly

### Phase 2: Create Schema Generator (1 hour)

- [x] Create `src/infrastructure/schema/` module
- [x] Implement `SchemaGenerator` with `generate<T: JsonSchema>() -> Result<String>` method
- [x] Add error type `SchemaGenerationError` with help messages
- [x] Add unit tests for schema generation
- [x] Verify generated schema includes doc comments

### Phase 3: Create Command Handler (1 hour)

- [x] Create `src/application/command_handlers/create/schema/` module
- [x] Implement `CreateSchemaCommandHandler` with `execute(output_path: Option<PathBuf>)` method
- [x] Handler calls `SchemaGenerator::generate::<EnvironmentCreationConfig>()`
- [x] Handle stdout vs file output logic
- [x] Add error type `CreateSchemaCommandHandlerError` with help messages
- [x] Add unit tests for handler logic

### Phase 4: Add CLI Subcommand (1 hour)

- [x] Create `src/presentation/input/cli/commands/create/subcommands/schema/` module
- [x] Add `Schema` variant to `CreateSubcommand` enum
- [x] Define Clap structure with optional `output_path` argument
- [x] Wire up command dispatch in `presentation/controllers/create/mod.rs`
- [x] Add integration test for CLI parsing

### Phase 5: Update Template Command (30 minutes)

- [x] Modify template command output to mention schema generation
- [x] Add info message: "Use `cargo run create schema` to generate JSON Schema for validation"
- [x] Update template command tests

### Phase 6: Documentation (1 hour)

- [x] Add schema command to user guide documentation
- [x] Include examples of usage (stdout and file output)
- [x] Document how to use schema with IDEs (VS Code settings example)
- [x] Create schemas/ directory with README explaining usage
- [x] Add comprehensive IDE setup guide

### Phase 7: Integration Testing (1 hour)

- [x] Add E2E test that generates schema to file
- [x] Verify schema validates example configuration files
- [x] Test schema output to stdout
- [x] Verify error handling for invalid paths

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [x] `cargo run create schema` outputs schema to stdout
- [x] `cargo run create schema ./schema.json` writes schema to file
- [x] Schema includes all fields from `EnvironmentCreationConfig`
- [x] Schema includes doc comments as descriptions
- [x] Schema validates example configuration files correctly
- [x] Template command mentions schema generation availability

**Architecture Requirements**:

- [x] All output uses `UserOutput` service (no `println!` calls)
- [x] SchemaGenerator in Infrastructure layer
- [x] Handler in Application layer
- [x] CLI in Presentation layer
- [x] Error messages are clear and actionable

**Testing Requirements**:

- [x] Unit tests for SchemaGenerator
- [x] Unit tests for CreateSchemaCommandHandler
- [x] Integration test for CLI parsing
- [x] E2E test for schema generation workflow

**Documentation Requirements**:

- [x] User guide updated with schema command
- [x] Examples provided for stdout and file output
- [x] IDE integration examples included

## Related Documentation

- **Feature Specification**: [docs/features/json-schema-generation/](../features/json-schema-generation/)
- **Answered Questions**: [docs/features/json-schema-generation/questions.md](../features/json-schema-generation/questions.md)
- **DDD Layer Placement**: [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md)
- **Error Handling**: [docs/contributing/error-handling.md](../contributing/error-handling.md)
- **Module Organization**: [docs/contributing/module-organization.md](../contributing/module-organization.md)
- **Testing Conventions**: [docs/contributing/testing/unit-testing.md](../contributing/testing/unit-testing.md)
- **Schemars Documentation**: <https://graham.cool/schemars/>

## Notes

- **Priority**: High - AI agent support for configuration file creation
- **Estimated Total Time**: 6-8 hours
- **External Dependencies**: Schemars crate (well-maintained, widely used)
- **Breaking Changes**: None - this is a purely additive feature
- **Future Enhancements**: Consider validation command using the schema (out of scope for MVP)

## Implementation Summary

**Status**: ✅ **COMPLETED**

All phases successfully implemented across 6 commits:

1. `072ac9e` - Phase 1: Added schemars 1.1 dependency and JsonSchema derives
2. `3ade9db` - Phase 2: Created SchemaGenerator infrastructure component
3. `d556053` - Phase 3: Implemented CreateSchemaCommandHandler
4. `5077194` - Phase 4: Added CLI integration with proper output abstraction
5. `1fa3ca9` - Phase 5: Updated template command with schema notice
6. `53eb0ea` - Phase 6 & 7: Added JSON schema file, IDE integration, and documentation

**Key Features Delivered**:

- ✅ Clean stdout output (no progress messages when piping)
- ✅ Schema file in `schemas/environment-config.json` (committed to git)
- ✅ VS Code settings for automatic validation in `envs/*.json` files
- ✅ Comprehensive IDE setup guide at `docs/user-guide/json-schema-ide-setup.md`
- ✅ Schema directory README explaining usage and regeneration
- ✅ Full DDD architecture compliance with proper layer separation

**Usage**:

```bash
# Generate to stdout
cargo run --bin torrust-tracker-deployer -- create schema

# Generate to file
cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
```

**IDE Integration**: VS Code automatically validates files in `envs/` directory with autocomplete, inline documentation, and error checking.
