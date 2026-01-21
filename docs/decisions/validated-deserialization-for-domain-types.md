# Decision: Validated Deserialization for Domain Types

## Status

Accepted

## Date

2026-01-21

## Context

Domain types in a DDD architecture should maintain invariants at all times. The Rust ecosystem commonly uses `#[derive(Deserialize)]` for serde integration, which directly populates struct fields during deserialization. This creates a problem: **invalid domain state can be created through deserialization**, bypassing the validated constructor.

Consider a domain type with business rules:

```rust
pub struct HttpApiConfig {
    bind_address: SocketAddr,  // port != 0
    domain: Option<DomainName>,
    use_tls_proxy: bool,       // if true, domain must be Some
}
```

With derived `Deserialize`, a JSON payload like `{"bind_address": "0.0.0.0:0", "use_tls_proxy": true}` would create an invalid object. The port is 0 (dynamic assignment not supported), and TLS is enabled without a domain. Both violate business invariants.

This affects:

1. **Loading persisted state** - Environment state files (`data/*/environment.json`)
2. **Configuration parsing** - Application DTOs (handled separately)
3. **API responses** - If we ever expose HTTP endpoints

## Decision

We adopt a **custom `Deserialize` implementation pattern** for domain types that have invariants:

```rust
/// Internal struct for serde deserialization that bypasses validation
#[derive(Deserialize)]
struct HttpApiConfigRaw {
    bind_address: SocketAddr,
    admin_token: ApiToken,
    #[serde(default)]
    domain: Option<DomainName>,
    use_tls_proxy: bool,
}

impl<'de> Deserialize<'de> for HttpApiConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = HttpApiConfigRaw::deserialize(deserializer)?;

        Self::new(
            raw.bind_address,
            raw.admin_token,
            raw.domain,
            raw.use_tls_proxy,
        )
        .map_err(serde::de::Error::custom)
    }
}
```

### Pattern Requirements

1. **Raw struct**: A private struct with identical field names, using derived `Deserialize`
2. **Custom impl**: `Deserialize` implementation that calls the validated constructor
3. **Error mapping**: Domain errors converted to serde errors via `serde::de::Error::custom`
4. **Serialize unchanged**: `#[derive(Serialize)]` can remain on the main type

### When to Apply This Pattern

Apply custom deserialization when a domain type has:

- Validated constructor (`new()` that returns `Result`)
- Private fields (invariants that need protection)
- Business rules beyond simple type conversion

### When NOT to Apply

Do not apply this pattern for:

- Simple value objects with no invariants (e.g., wrapper newtypes)
- Types where construction always succeeds
- Application DTOs (these use different validation strategies)

## Consequences

### Benefits

1. **Guaranteed validity**: Domain objects are always valid, even after deserialization
2. **Single validation point**: All construction goes through `new()`, no duplication
3. **Clear error messages**: Domain errors with `help()` methods propagate through serde
4. **Consistent pattern**: Same approach across all invariant-bearing domain types

### Trade-offs

1. **More boilerplate**: Each type needs a Raw struct and custom impl (~20 lines)
2. **Field duplication**: Raw struct mirrors the main struct's fields
3. **Maintenance cost**: Field additions require updating both structs

### Risks

1. **Raw struct drift**: If fields are added to main type but not Raw, deserialization breaks (compile error, so caught early)
2. **Serde attribute mismatch**: Attributes like `#[serde(default)]` must be on Raw struct

## Alternatives Considered

### Alternative 1: `#[serde(try_from)]` Attribute

```rust
#[derive(Deserialize)]
#[serde(try_from = "HttpApiConfigRaw")]
pub struct HttpApiConfig { ... }
```

**Rejected because:**

- Requires `TryFrom` implementation, similar boilerplate
- Error type constraints are less flexible
- The explicit `Deserialize` impl is clearer about what's happening

### Alternative 2: Post-Deserialization Validation

```rust
let config: HttpApiConfig = serde_json::from_str(json)?;
config.validate()?;  // Caller must remember to call
```

**Rejected because:**

- Violates "always valid" DDD principle
- Invalid objects can exist temporarily
- Easy to forget the validation call

### Alternative 3: Validation in Getters

Defer validation to accessor methods.

**Rejected because:**

- Moves errors to usage time, not construction time
- Invalid objects exist in memory
- Harder to reason about object state

### Alternative 4: Serde Validation Crate

Use `validator` or similar crate with serde integration.

**Rejected because:**

- Adds external dependency for this specific pattern
- Our invariants often involve multiple fields (aggregate validation)
- We already have `new()` constructors; adding another validation layer is redundant

## Related Decisions

- [Secrecy Crate for Sensitive Data](./secrecy-crate-for-sensitive-data.md) - Another domain type pattern
- [Configuration DTO Layer Placement](./configuration-dto-layer-placement.md) - Where validation happens in application layer
- [Port Zero Not Supported](./port-zero-not-supported.md) - Example invariant this pattern enforces

## References

- [Refactoring Plan: Strengthen Domain Invariant Enforcement](../refactors/plans/strengthen-domain-invariant-enforcement.md)
- [DDD Practices Guide](../contributing/ddd-practices.md)
- [serde Custom Deserialization](https://serde.rs/impl-deserialize.html)
