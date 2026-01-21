# Decision: Use TryFrom Trait for DTO to Domain Conversions

## Status

Accepted

## Date

2026-01-21

## Context

When converting application layer DTOs (Data Transfer Objects) to domain entities, we need a consistent pattern. The conversion is fallible because:

1. DTOs contain primitive types (e.g., `String`) that need parsing
2. Domain constructors validate business invariants and can reject invalid data

Currently, we use custom methods like `to_http_api_config()`:

```rust
impl HttpApiSection {
    pub fn to_http_api_config(&self) -> Result<HttpApiConfig, CreateConfigError> {
        // Parse and validate...
    }
}

// Usage
let config = section.to_http_api_config()?;
```

This approach works but has limitations:

- **Discoverability**: Developers must know the specific method name exists
- **Inconsistency**: Different types might use different naming conventions (`to_xxx`, `into_xxx`, `as_xxx`)
- **Generic code**: Cannot easily use these types in generic contexts that require conversion traits
- **Self-documentation**: The type relationship is not expressed in the type system

## Decision

Use Rust's standard `TryFrom` and `TryInto` traits for fallible DTO-to-domain conversions.

```rust
impl TryFrom<HttpApiSection> for HttpApiConfig {
    type Error = CreateConfigError;

    fn try_from(section: HttpApiSection) -> Result<Self, Self::Error> {
        let bind_address = section.bind_address.parse::<SocketAddr>()
            .map_err(|e| CreateConfigError::InvalidBindAddress {
                value: section.bind_address.clone(),
                reason: e.to_string(),
            })?;

        let domain = section.domain
            .map(|d| DomainName::new(&d))
            .transpose()
            .map_err(|e| CreateConfigError::InvalidDomain {
                value: section.domain.clone().unwrap_or_default(),
                reason: e.to_string(),
            })?;

        HttpApiConfig::new(
            bind_address,
            section.admin_token.into(),
            domain,
            section.use_tls_proxy.unwrap_or(false),
        ).map_err(CreateConfigError::from)
    }
}

// Usage - multiple idiomatic options
let config = HttpApiConfig::try_from(section)?;
let config: HttpApiConfig = section.try_into()?;
```

### Guidelines

1. **Implement on the domain type**: `impl TryFrom<DTO> for DomainType`
2. **Error type is the application error**: `type Error = CreateConfigError`
3. **Consume the DTO**: `TryFrom` takes ownership; if borrowing is needed, implement `TryFrom<&DTO>`
4. **Delegate to domain constructor**: The `try_from` body should parse primitives then call the domain's `new()`

### When to Use

| Scenario                       | Use                                           |
| ------------------------------ | --------------------------------------------- |
| DTO → Domain (fallible)        | `TryFrom`                                     |
| Domain → DTO (infallible)      | `From`                                        |
| Multiple conversion strategies | Named methods (`to_strict()`, `to_lenient()`) |
| Borrowing required             | `TryFrom<&T>` or named method                 |

## Consequences

### Benefits

- **Self-documenting**: Type relationships are explicit in the trait bounds
- **Discoverable**: IDE autocomplete shows `try_into()` on any type; developers can search for `TryFrom` implementations
- **Ecosystem alignment**: Uses Rust's standard conversion traits
- **Generic compatibility**: Works with `T: TryInto<Target>` bounds
- **Consistency**: All DTO conversions follow the same pattern

### Trade-offs

- **Ownership**: `TryFrom` consumes the input; use `TryFrom<&T>` if borrowing is needed
- **Single conversion**: Only one `TryFrom` impl per (Source, Target, Error) triple
- **Migration effort**: Existing `to_xxx()` methods need refactoring

### Migration Path

1. Add `TryFrom` implementation alongside existing method
2. Update call sites to use `.try_into()?`
3. Remove the old `to_xxx()` method
4. Update documentation and tests

## Alternatives Considered

### Keep Custom Methods (`to_xxx()`)

```rust
impl HttpApiSection {
    pub fn to_http_api_config(&self) -> Result<HttpApiConfig, CreateConfigError> { ... }
}
```

**Rejected because**:

- Ad-hoc naming reduces discoverability
- Cannot use in generic contexts
- Type relationship not expressed in type system
- Inconsistent with Rust idioms

### Use `From` with `Result` Return

Not possible - `From` is for infallible conversions only.

### Use Both Trait and Method

```rust
impl TryFrom<HttpApiSection> for HttpApiConfig { ... }

impl HttpApiSection {
    pub fn to_http_api_config(self) -> Result<HttpApiConfig, CreateConfigError> {
        self.try_into()
    }
}
```

**Rejected because**:

- Redundant API surface
- Maintenance burden
- Confusion about which to use

## Related Decisions

- [Validated Deserialization for Domain Types](./validated-deserialization-for-domain-types.md) - Custom Deserialize for domain invariants
- [Configuration DTO Layer Placement](./configuration-dto-layer-placement.md) - DTOs belong in application layer

## References

- [Rust std::convert::TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)
- [Rust API Guidelines - Conversions](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv-traits)
- [DDD Practices Guide](../contributing/ddd-practices.md)
