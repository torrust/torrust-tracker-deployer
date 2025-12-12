# JSON Schema Generation for Environment Configuration

**Issue**: [#229](https://github.com/torrust/torrust-tracker-deployer/issues/229)
**Parent Epic**: N/A (Standalone task)
**Related**: [Feature Specification](../features/json-schema-generation/)

## Overview

Add a new CLI command `cargo run create schema [OUTPUT_PATH]` that generates a JSON Schema from the Rust `EnvironmentCreationConfig` type. This enables AI assistants, IDEs, and developers to benefit from autocomplete, validation, and inline documentation when creating environment configuration files.

## Goals

- [ ] Generate JSON Schema from Rust configuration types using Schemars
- [ ] Support output to stdout or file
- [ ] Include doc comments as schema descriptions
- [ ] Update template command to inform users about schema generation
- [ ] Provide high-quality AI agent support for configuration file creation

## 🏗️ Architecture Requirements

**DDD Layer**: Application + Infrastructure
**Module Path**:

- CLI: `src/presentation/input/cli/commands/create/subcommands/schema/`
- Handler: `src/application/command_handlers/create/schema/`
- Generator: `src/infrastructure/schema/`
  **Pattern**: CLI Subcommand → Command Handler → Schema Generator (Infrastructure)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (Presentation → Application → Infrastructure)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] CLI parsing in Presentation layer (Clap structures)
- [ ] Handler orchestration in Application layer
- [ ] Schema generation in Infrastructure layer (external dependency: Schemars)
- [ ] All output through `UserOutput` service (no direct `println!`)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

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

- [ ] Add `schemars = "0.8"` to `Cargo.toml`
- [ ] Add `#[derive(JsonSchema)]` to `EnvironmentCreationConfig`
- [ ] Add `#[derive(JsonSchema)]` to all nested config types (provider configs, SSH config, etc.)
- [ ] Add doc comments to all public fields for schema descriptions
- [ ] Build to verify derives work correctly

### Phase 2: Create Schema Generator (1 hour)

- [ ] Create `src/infrastructure/schema/` module
- [ ] Implement `SchemaGenerator` with `generate<T: JsonSchema>() -> Result<String>` method
- [ ] Add error type `SchemaGenerationError` with help messages
- [ ] Add unit tests for schema generation
- [ ] Verify generated schema includes doc comments

### Phase 3: Create Command Handler (1 hour)

- [ ] Create `src/application/command_handlers/create/schema/` module
- [ ] Implement `CreateSchemaCommandHandler` with `execute(output_path: Option<PathBuf>)` method
- [ ] Handler calls `SchemaGenerator::generate::<EnvironmentCreationConfig>()`
- [ ] Handle stdout vs file output logic
- [ ] Add error type `CreateSchemaCommandHandlerError` with help messages
- [ ] Add unit tests for handler logic

### Phase 4: Add CLI Subcommand (1 hour)

- [ ] Create `src/presentation/input/cli/commands/create/subcommands/schema/` module
- [ ] Add `Schema` variant to `CreateSubcommand` enum
- [ ] Define Clap structure with optional `output_path` argument
- [ ] Wire up command dispatch in `presentation/controllers/create/mod.rs`
- [ ] Add integration test for CLI parsing

### Phase 5: Update Template Command (30 minutes)

- [ ] Modify template command output to mention schema generation
- [ ] Add info message: "Use `cargo run create schema` to generate JSON Schema for validation"
- [ ] Update template command tests

### Phase 6: Documentation (1 hour)

- [ ] Add schema command to `docs/user-guide/commands/create.md`
- [ ] Include examples of usage (stdout and file output)
- [ ] Document how to use schema with IDEs (VS Code settings example)
- [ ] Update main README with schema generation feature

### Phase 7: Integration Testing (1 hour)

- [ ] Add E2E test that generates schema to file
- [ ] Verify schema validates example configuration files
- [ ] Test schema output to stdout
- [ ] Verify error handling for invalid paths

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] `cargo run create schema` outputs schema to stdout
- [ ] `cargo run create schema ./schema.json` writes schema to file
- [ ] Schema includes all fields from `EnvironmentCreationConfig`
- [ ] Schema includes doc comments as descriptions
- [ ] Schema validates example configuration files correctly
- [ ] Template command mentions schema generation availability

**Architecture Requirements**:

- [ ] All output uses `UserOutput` service (no `println!` calls)
- [ ] SchemaGenerator in Infrastructure layer
- [ ] Handler in Application layer
- [ ] CLI in Presentation layer
- [ ] Error messages are clear and actionable

**Testing Requirements**:

- [ ] Unit tests for SchemaGenerator
- [ ] Unit tests for CreateSchemaCommandHandler
- [ ] Integration test for CLI parsing
- [ ] E2E test for schema generation workflow

**Documentation Requirements**:

- [ ] User guide updated with schema command
- [ ] Examples provided for stdout and file output
- [ ] IDE integration examples included

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
