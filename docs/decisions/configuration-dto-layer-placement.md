# Decision: Configuration DTO Layer Placement

## Status

Accepted

## Date

2026-01-09

## Context

The project has configuration types for environment creation that sit at the boundary between external configuration sources (JSON files, CLI arguments) and the internal domain model. These types:

1. **Deserialize from JSON** - Accept user input with raw primitives (`String`, `u32`)
2. **Validate and convert** - Transform to strongly-typed domain objects via `to_*_config()` methods
3. **Generate JSON Schema** - Derive `JsonSchema` for IDE autocomplete

The question arose: where should these types live in the DDD architecture?

**Options considered:**

1. **Domain layer** (`src/domain/`) - Where business entities live
2. **Application layer** (`src/application/`) - Where use cases and commands live
3. **Separate workspace package** - For reuse by other applications

Additionally, AI agents need to reference these types to generate accurate configurations, as Rust types express constraints that JSON Schema cannot fully capture (e.g., `NonZeroU32`, tagged enums, custom validation logic).

## Decision

**Keep configuration DTOs in `src/application/command_handlers/create/config/`** (application layer).

**Do not extract to a separate package** at this time.

### Rationale for Application Layer

1. **DTOs are not domain concepts** - They represent input formats, not business entities. The domain layer should contain semantic business concepts like `Environment`, `TrackerConfig`, not parsing concerns.

2. **Unidirectional dependency** - Config DTOs import domain types to convert to (`DTO → Domain`). This is correct: application layer depends on domain, not vice versa.

3. **Serialization is application concern** - Heavy `serde` usage (`Deserialize`, `Serialize`, `JsonSchema`) belongs at application boundaries, not in the domain core.

4. **Command-specific** - These types are specific to the `create` command. Other commands may have different configuration needs.

5. **Follows Anti-Corruption Layer pattern** - The config types protect the domain from external JSON format changes.

### Rationale Against Separate Package (Now)

1. **No immediate consumers** - No other Rust applications currently need these types
2. **Versioning overhead** - Separate packages require versioning discipline
3. **Premature abstraction** - YAGNI (You Aren't Gonna Need It)
4. **Types are well-organized** - Already isolated in a submodule, easy to extract later

## Consequences

### Positive

- **Clear architectural boundaries** - DTOs vs domain types are distinct
- **AI agents can reference types** - Folder path documented in `AGENTS.md` rule 19
- **No versioning complexity** - Single crate, simple dependency management
- **Future-ready** - Types are organized for potential extraction

### Negative

- **Cannot reuse from external Rust apps** - Must copy types or depend on entire crate
- **AI agents need to read Rust code** - JSON schema alone is insufficient for full constraints

### Risks

- If multiple applications need these types, extraction becomes necessary
- Changes to domain types may require updates to DTOs (but this is expected)

## Alternatives Considered

### 1. Domain Layer Placement

**Rejected because:**

- Domain types should represent business concepts, not input formats
- Would pollute domain with serialization concerns
- Domain already has clean types (`TrackerConfig`, `GrafanaConfig`)

### 2. Separate Workspace Package

**Deferred because:**

- No current consumers
- Adds versioning complexity
- Can be done later if needed (types are already isolated)

### 3. Infrastructure Layer

**Rejected because:**

- Config parsing is not external system integration
- Would mix concerns with SSH, Ansible, OpenTofu adapters

## Related Decisions

- [Secrecy Crate for Sensitive Data](./secrecy-crate-for-sensitive-data.md) - How secrets are handled in DTOs vs domain
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md) - General guidance for layer decisions

## References

- [Anti-Corruption Layer pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/anti-corruption-layer) - Microsoft patterns documentation
- [Config README](../../src/application/command_handlers/create/config/README.md) - Detailed documentation for AI agents
- [JSON Schema](../../schemas/README.md) - Generated schema for IDE autocomplete
