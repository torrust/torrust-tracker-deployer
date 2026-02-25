---
name: environment-config-architecture
description: Developer reference for the environment configuration DTO architecture, Rust type system, validation logic, and JSON schema. Use when modifying config DTOs, adding new config sections, debugging validation errors in Rust code, or understanding the TryFrom conversion pattern. Triggers on "config DTO", "environment config types", "config validation logic", "TryFrom implementation", or "config architecture".
metadata:
  author: torrust
  version: "1.0"
---

# Environment Config Architecture

This skill is a developer reference for the internal architecture of environment configurations — the DTO types, validation patterns, and domain conversion logic.

## Configuration Structure Reference

**Authoritative sources** (in order of precedence):

1. **Rust types**: `src/application/command_handlers/create/config/` - Express richer constraints than JSON schema
2. **JSON schema**: `schemas/environment-config.json` - Basic structure for IDE autocomplete
3. **Examples**: `docs/ai-training/dataset/environment-configs/` - 15 real-world configurations

Read `src/application/command_handlers/create/config/README.md` for the complete DTO architecture guide.

### DTO Architecture

The configuration system uses a layered conversion pipeline:

```text
JSON file → serde deserialization → DTO structs → TryFrom → Domain types
```

Each layer adds progressively stricter validation:

| Layer        | Location                                    | Validates               |
| ------------ | ------------------------------------------- | ----------------------- |
| JSON Schema  | `schemas/environment-config.json`           | Basic structure, types  |
| Serde        | DTO structs with `#[serde(...)]` attributes | Field presence, formats |
| `TryFrom`    | `impl TryFrom<Dto> for DomainType`          | Business rules          |
| Domain types | `src/domain/`                               | Invariants              |

### Key DTO Types

Located in `src/application/command_handlers/create/config/`:

- `EnvironmentConfigDto` - Top-level configuration DTO
- `EnvironmentDto` - Name, instance name, description
- `SshCredentialsDto` - SSH key paths, username, port
- `ProviderDto` - LXD or Hetzner (enum-tagged)
- `TrackerDto` - Core settings, UDP/HTTP trackers, API
- `PrometheusDto` - Monitoring settings
- `GrafanaDto` - Dashboard settings
- `BackupDto` - Backup schedule and retention
- `HttpsDto` - TLS settings

### Cross-Field Constraints

Implemented in domain-level validation (not expressible in JSON schema):

- **Grafana requires Prometheus**: Cannot enable Grafana without Prometheus
- **HTTPS required for TLS**: If any service configures TLS, the HTTPS section must be present
- **MySQL requires credentials**: When `driver: "mysql"`, host/port/username/password are required
- **Unique bind addresses**: No two services can bind to the same address:port

## Validation and Error Handling

### Validation Flow

```text
1. JSON parsing (serde_json) → SyntaxError
2. DTO deserialization (serde) → MissingField, InvalidType
3. TryFrom conversion → DomainValidationError
4. Domain invariant checks → BusinessRuleViolation
```

### Common Validation Errors and Root Causes

| Error Message                            | Rust Type / Constraint | Fix                                      |
| ---------------------------------------- | ---------------------- | ---------------------------------------- |
| "scrape_interval must be greater than 0" | `NonZeroU32`           | Use a positive integer                   |
| "Invalid profile name format"            | Regex `[a-z0-9-]+`     | Use lowercase alphanumeric + hyphens     |
| "SSH private key does not exist"         | `Path::exists()` check | Use absolute path to existing file       |
| "Grafana requires Prometheus"            | Cross-field validation | Add Prometheus section or remove Grafana |
| "TLS configured but no admin_email"      | Cross-field validation | Add HTTPS section with admin_email       |

### Fix Strategy

When debugging validation errors:

1. Read the `TryFrom` implementation for the failing DTO type
2. Check the domain type's constructor or `new()` method for invariants
3. Look at the error type's variants for all possible failure modes
4. Cross-reference with `schemas/environment-config.json` for structural constraints

## Adding New Configuration Sections

When adding a new top-level configuration section:

1. **Create DTO struct** in `src/application/command_handlers/create/config/`
2. **Add serde attributes** for JSON deserialization
3. **Create domain type** in `src/domain/` with appropriate invariants
4. **Implement `TryFrom<Dto> for DomainType`** with validation logic
5. **Add to `EnvironmentConfigDto`** as an `Option<NewSectionDto>` field
6. **Update JSON schema** in `schemas/environment-config.json`
7. **Add AI training examples** in `docs/ai-training/dataset/environment-configs/`
8. **Update the user-facing skill** in `usage/operations/create-environment-config/`

## Related Documentation

- **Config DTO Architecture**: `src/application/command_handlers/create/config/README.md`
- **JSON Schema**: `schemas/README.md` - Schema generation and IDE setup
- **ADR - Configuration Layer**: `docs/decisions/configuration-dto-layer-placement.md` - Why DTOs are in application layer
- **ADR - TryFrom Pattern**: `docs/decisions/tryfrom-for-dto-to-domain-conversion.md` - DTO to domain conversion pattern
- **DDD Layer Placement**: `docs/contributing/ddd-layer-placement.md`

## See Also

- For **creating environment configs** (end-user guide): see the `create-environment-config` skill in `usage/operations/`
