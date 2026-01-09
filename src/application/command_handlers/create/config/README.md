# Environment Configuration DTOs

This module contains **Configuration Data Transfer Objects (DTOs)** for environment creation. These types sit at the boundary between external configuration sources (JSON files, CLI arguments) and the internal domain model.

## Purpose

These types serve multiple purposes:

1. **JSON Deserialization** - Accept user input from configuration files
2. **Validation Gateway** - Convert loose string-based input to strict domain types
3. **Schema Generation** - Derive `JsonSchema` for IDE autocomplete (see `schemas/environment-config.json`)
4. **AI Agent Reference** - Provide richer type information than JSON schema alone

## Architecture Pattern

This module implements the **Anti-Corruption Layer** pattern from Domain-Driven Design:

```text
User JSON → [Config DTOs] → validate/transform → [Domain Types]
             (String-based)                       (PathBuf, NonZeroU32, etc.)
```

### Key Differences from Domain Types

| Aspect         | Config DTOs (this module)                        | Domain Types                                     |
| -------------- | ------------------------------------------------ | ------------------------------------------------ |
| **Layer**      | Application                                      | Domain                                           |
| **Purpose**    | JSON parsing                                     | Business logic                                   |
| **Types**      | Raw primitives (`String`, `u32`)                 | Validated newtypes (`NonZeroU32`, `ProfileName`) |
| **Validation** | Deferred to `to_*_config()`                      | Enforced at construction                         |
| **Serde**      | Heavy (`Deserialize`, `Serialize`, `JsonSchema`) | Minimal                                          |

## Module Structure

```text
config/
├── environment_config.rs    # EnvironmentCreationConfig (top-level)
├── ssh_credentials_config.rs # SshCredentialsConfig
├── prometheus.rs            # PrometheusSection
├── grafana.rs               # GrafanaSection
├── errors.rs                # CreateConfigError
├── provider/
│   ├── lxd.rs               # LxdProviderSection
│   └── hetzner.rs           # HetznerProviderSection
└── tracker/
    ├── tracker_section.rs       # TrackerSection (aggregate)
    ├── tracker_core_section.rs  # TrackerCoreSection, DatabaseSection
    ├── udp_tracker_section.rs   # UdpTrackerSection
    ├── http_tracker_section.rs  # HttpTrackerSection
    ├── http_api_section.rs      # HttpApiSection
    └── health_check_api_section.rs # HealthCheckApiSection
```

## Conversion Pattern

Each DTO provides a `to_*_config()` method that validates and converts to domain types:

```rust
// Example: PrometheusSection → PrometheusConfig
impl PrometheusSection {
    pub fn to_prometheus_config(&self) -> Result<PrometheusConfig, CreateConfigError> {
        // Validates: scrape_interval must be > 0
        let interval = NonZeroU32::new(self.scrape_interval_in_secs)
            .ok_or_else(|| CreateConfigError::InvalidPrometheusConfig(...))?;
        Ok(PrometheusConfig::new(interval))
    }
}
```

## Constraints Expressed in Rust (Not in JSON Schema)

The Rust types express constraints that JSON Schema cannot fully capture:

| Constraint                 | Rust Type                                   | JSON Schema Limitation             |
| -------------------------- | ------------------------------------------- | ---------------------------------- |
| Non-zero integers          | `NonZeroU32`                                | Can only specify `minimum: 1`      |
| Mutually exclusive options | Tagged enums with `#[serde(tag = "...")]`   | `oneOf` is complex and error-prone |
| Path validation            | `PathBuf` with existence checks             | No file system awareness           |
| Format validation          | Newtype constructors (`ProfileName::new()`) | Regex patterns are limited         |
| Cross-field validation     | Custom `to_*_config()` logic                | No support                         |
| Secret handling            | `Password`, `ApiToken` wrappers             | No security semantics              |

## For AI Agents

When generating environment configuration:

1. **Reference these Rust types** for accurate constraint information
2. **Follow the structure** in `EnvironmentCreationConfig` as the root type
3. **Check validation logic** in `to_*_config()` methods for business rules
4. **Use JSON schema** (`schemas/environment-config.json`) for basic structure, but trust Rust types for constraints

## Related Documentation

- [JSON Schema](../../../../schemas/README.md) - Generated schema for IDE autocomplete
- [JSON Schema IDE Setup](../../../../docs/user-guide/json-schema-ide-setup.md) - Configure VS Code/editors
- [ADR: Configuration DTO Layer Placement](../../../../docs/decisions/configuration-dto-layer-placement.md) - Why these types are in application layer
- [DDD Layer Placement Guide](../../../../docs/contributing/ddd-layer-placement.md) - General layer placement guidance
