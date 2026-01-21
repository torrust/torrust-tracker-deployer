# DDD Practices Guide

This guide documents Domain-Driven Design practices adopted in the Torrust Tracker Deployer project. These patterns ensure domain objects are always valid and business rules are consistently enforced.

## Overview

DDD emphasizes that the domain layer should encapsulate business logic and maintain invariants. This guide covers specific implementation patterns we use to achieve these goals in Rust.

For layer placement guidelines (which code belongs where), see [DDD Layer Placement Guide](./ddd-layer-placement.md).

## Core Principles

### 1. Domain Objects Are Always Valid

After construction, a domain object must satisfy all its business invariants. Invalid objects should be impossible to create.

**Bad - Public fields allow invalid state:**

```rust
pub struct HttpApiConfig {
    pub bind_address: SocketAddr,  // Anyone can set port to 0
    pub use_tls_proxy: bool,       // Can enable TLS without domain
    pub domain: Option<DomainName>,
}
```

**Good - Validated constructor enforces invariants:**

```rust
pub struct HttpApiConfig {
    bind_address: SocketAddr,  // Private
    use_tls_proxy: bool,
    domain: Option<DomainName>,
}

impl HttpApiConfig {
    pub fn new(
        bind_address: SocketAddr,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> Result<Self, HttpApiConfigError> {
        if bind_address.port() == 0 {
            return Err(HttpApiConfigError::DynamicPortNotSupported(bind_address));
        }
        if use_tls_proxy && domain.is_none() {
            return Err(HttpApiConfigError::TlsProxyRequiresDomain);
        }
        Ok(Self { bind_address, domain, use_tls_proxy })
    }
}
```

### 2. Single Point of Validation

All validation happens in the constructor. Don't duplicate validation in:

- Separate `validate()` methods
- Setters
- Getters
- Serialization/deserialization code

### 3. Expressive Error Types

Domain errors should explain what went wrong and how to fix it:

```rust
#[derive(Debug, thiserror::Error)]
pub enum HttpApiConfigError {
    #[error("Dynamic port assignment (port 0) is not supported: {0}")]
    DynamicPortNotSupported(SocketAddr),

    #[error("TLS proxy requires a domain to be specified")]
    TlsProxyRequiresDomain,
}

impl HttpApiConfigError {
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DynamicPortNotSupported(_) => {
                "Dynamic port assignment (port 0) is not supported.\n\
                 \n\
                 Why: Port 0 tells the OS to assign a random available port.\n\
                 This is not suitable for deployment where ports must be known.\n\
                 \n\
                 Fix: Specify an explicit port number (e.g., 1212, 8080)."
            }
            Self::TlsProxyRequiresDomain => {
                "TLS proxy mode requires a domain name.\n\
                 \n\
                 Why: TLS certificates are issued for specific domain names.\n\
                 \n\
                 Fix: Either set a domain name or disable TLS proxy."
            }
        }
    }
}
```

## Validated Deserialization Pattern

When domain types use serde for serialization/deserialization, we must ensure deserialization also enforces invariants.

> **ADR**: See [Validated Deserialization for Domain Types](../decisions/validated-deserialization-for-domain-types.md) for the full decision record.

### The Problem

Derived `#[derive(Deserialize)]` bypasses the validated constructor:

```rust
#[derive(Deserialize)]  // ❌ Creates object without validation
pub struct HttpApiConfig { ... }

// This JSON creates an invalid object:
let config: HttpApiConfig = serde_json::from_str(
    r#"{"bind_address": "0.0.0.0:0", "use_tls_proxy": true}"#
)?;
```

### The Solution

Use a custom `Deserialize` implementation with a Raw struct:

```rust
use serde::{Deserialize, Deserializer, Serialize};

/// The domain type with private fields
#[derive(Debug, Clone, Serialize)]  // Serialize can be derived
pub struct HttpApiConfig {
    bind_address: SocketAddr,
    admin_token: ApiToken,
    domain: Option<DomainName>,
    use_tls_proxy: bool,
}

/// Internal struct for deserialization (mirrors the main type)
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
        D: Deserializer<'de>,
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

### When to Use This Pattern

Apply custom deserialization when a domain type has **all** of these:

- ✅ Private fields
- ✅ Validated constructor (`new()` returning `Result`)
- ✅ Business invariants to protect
- ✅ Needs serde deserialization (persisted or parsed from JSON)

### Checklist for Implementation

1. [ ] Create `TypeNameRaw` struct with identical fields
2. [ ] Add `#[derive(Deserialize)]` only to Raw struct
3. [ ] Keep `#[derive(Serialize)]` on main type (serialization doesn't need validation)
4. [ ] Implement `Deserialize` trait calling `new()`
5. [ ] Map domain errors with `serde::de::Error::custom`
6. [ ] Add `#[serde(default)]` to Raw struct for optional fields

## Getters for Private Fields

Provide getter methods for all private fields:

```rust
impl HttpApiConfig {
    #[must_use]
    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    #[must_use]
    pub fn domain(&self) -> Option<&DomainName> {
        self.domain.as_ref()
    }

    #[must_use]
    pub fn use_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }
}
```

Guidelines:

- Use `#[must_use]` on all getters
- Return references for owned types (`&str`, `&DomainName`)
- Return copies for `Copy` types (`bool`, `SocketAddr`, `u16`)
- Name getters after the field (no `get_` prefix)

## Default Implementations

When implementing `Default` for domain types with validated constructors:

```rust
impl Default for HttpApiConfig {
    fn default() -> Self {
        Self::new(
            "0.0.0.0:1212".parse().expect("default address is valid"),
            "MyAccessToken".to_string().into(),
            None,
            false,
        )
        .expect("default configuration is valid")
    }
}
```

The `expect()` is acceptable here because:

1. Default values are compile-time constants
2. Tests verify the default is valid
3. If it fails, it's a programmer error, not a user error

## Test Helper Functions

For tests that need to create domain objects with various configurations:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_http_api_config(bind_address: &str, admin_token: &str) -> HttpApiConfig {
        HttpApiConfig::new(
            bind_address.parse().expect("test address is valid"),
            admin_token.to_string().into(),
            None,
            false,
        )
        .expect("test configuration is valid")
    }

    fn test_http_api_config_with_tls(
        bind_address: &str,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> HttpApiConfig {
        HttpApiConfig::new(
            bind_address.parse().expect("test address is valid"),
            "token".to_string().into(),
            domain,
            use_tls_proxy,
        )
        .expect("test configuration is valid")
    }
}
```

## Application Layer DTOs

Application layer DTOs bridge external data (JSON) to domain types. We use Rust's standard `TryFrom` trait for these conversions.

> **ADR**: See [TryFrom for DTO to Domain Conversion](../decisions/tryfrom-for-dto-to-domain-conversion.md) for the full decision record.

### Why TryFrom?

Using standard traits makes the code self-documenting:

- **Discoverable**: Developers can search for `TryFrom` implementations
- **Consistent**: All conversions follow the same pattern
- **Generic-friendly**: Works with `T: TryInto<Target>` bounds
- **IDE support**: Autocomplete shows `.try_into()` on any type

### Implementation Pattern

```rust
use std::convert::TryFrom;

// Application DTO - accepts primitives from JSON
#[derive(Deserialize)]
pub struct HttpApiSection {
    pub bind_address: String,      // String from JSON
    pub admin_token: String,
    pub domain: Option<String>,
    pub use_tls_proxy: Option<bool>,
}

// Implement TryFrom on the domain type
impl TryFrom<HttpApiSection> for HttpApiConfig {
    type Error = CreateConfigError;

    fn try_from(section: HttpApiSection) -> Result<Self, Self::Error> {
        // 1. Parse primitives to domain types
        let bind_address: SocketAddr = section.bind_address.parse()
            .map_err(|e| CreateConfigError::InvalidBindAddress {
                value: section.bind_address.clone(),
                reason: e.to_string(),
            })?;

        let domain = section.domain
            .map(|d| DomainName::new(&d))
            .transpose()
            .map_err(|e| CreateConfigError::InvalidDomain { ... })?;

        // 2. Delegate to domain constructor
        HttpApiConfig::new(
            bind_address,
            section.admin_token.into(),
            domain,
            section.use_tls_proxy.unwrap_or(false),
        ).map_err(CreateConfigError::from)  // Domain errors convert via From trait
    }
}
```

### Usage

```rust
// Both forms are equivalent - use whichever reads better in context
let config = HttpApiConfig::try_from(section)?;
let config: HttpApiConfig = section.try_into()?;
```

### Key Points

- **DTOs use primitive types** (`String`, not `SocketAddr`)
- **Parsing** (string → typed value) happens in `TryFrom` implementation
- **Validation** (business rules) happens in domain constructor
- **Error propagation** via `?` and `From` trait implementations
- **Ownership**: `TryFrom` consumes the DTO; use `TryFrom<&T>` if borrowing is needed

### When to Use Which

| Scenario                  | Use           |
| ------------------------- | ------------- |
| DTO → Domain (fallible)   | `TryFrom`     |
| Domain → DTO (infallible) | `From`        |
| Need to borrow            | `TryFrom<&T>` |
| Multiple strategies       | Named methods |

## Related Documentation

- [DDD Layer Placement Guide](./ddd-layer-placement.md) - Which code belongs in which layer
- [Error Handling Guide](./error-handling.md) - Error design patterns
- [Validated Deserialization ADR](../decisions/validated-deserialization-for-domain-types.md) - Custom Deserialize for domain invariants
- [TryFrom for DTO Conversion ADR](../decisions/tryfrom-for-dto-to-domain-conversion.md) - Standard traits for DTO→Domain conversion
- [Refactoring Plan](../refactors/plans/strengthen-domain-invariant-enforcement.md) - Active refactoring work
