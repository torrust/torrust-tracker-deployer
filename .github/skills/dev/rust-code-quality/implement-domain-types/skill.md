---
name: implement-domain-types
description: Guide for implementing domain types (value objects and entities) with invariants in this Rust project. Covers the "always valid" pattern with private fields and validated constructors, custom Deserialize implementations to prevent bypass, and expressive error types with actionable help text. Use when creating or modifying domain value objects, entities, or types with business rules. Triggers on "domain type", "value object", "entity", "invariant", "validated constructor", "domain entity", "DDD practices", "domain model", or "implement domain".
metadata:
  author: torrust
  version: "1.0"
---

# Implementing Domain Types with Invariants

## Core Rule: Domain Objects Are Always Valid

After construction, a domain object must satisfy all its invariants. **Invalid objects must be impossible to create.**

## Pattern 1: Value Object with Private Fields + Validated Constructor

```rust
// ✅ Correct
pub struct EnvironmentName(String); // private field

impl EnvironmentName {
    pub fn new(name: String) -> Result<Self, EnvironmentNameError> {
        if name.is_empty() {
            return Err(EnvironmentNameError::Empty);
        }
        if name.contains(char::is_uppercase) {
            return Err(EnvironmentNameError::UppercaseNotAllowed(name));
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

```rust
// ❌ Wrong - allows invalid state
pub struct EnvironmentName {
    pub name: String, // public field, anyone can bypass validation
}
```

## Pattern 2: Custom Deserialize (Required for Serde)

`#[derive(Deserialize)]` bypasses the constructor — **always implement custom `Deserialize`**:

```rust
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]  // Serialize can be derived
pub struct HttpApiConfig {
    bind_address: SocketAddr,        // private fields
    use_tls_proxy: bool,
}

#[derive(Deserialize)]
struct HttpApiConfigRaw {            // mirror struct for deserialization
    bind_address: SocketAddr,
    use_tls_proxy: bool,
}

impl<'de> Deserialize<'de> for HttpApiConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = HttpApiConfigRaw::deserialize(deserializer)?;
        HttpApiConfig::new(raw.bind_address, raw.use_tls_proxy)
            .map_err(serde::de::Error::custom)
    }
}
```

## Pattern 3: Expressive Error Types with Help Text

```rust
#[derive(Debug, thiserror::Error)]
pub enum HttpApiConfigError {
    #[error("Dynamic port (port 0) is not supported: {0}")]
    DynamicPortNotSupported(SocketAddr),

    #[error("TLS proxy requires a domain to be specified")]
    TlsProxyRequiresDomain,
}

impl HttpApiConfigError {
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DynamicPortNotSupported(_) =>
                "Specify an explicit port number (e.g., 1212, 8080).",
            Self::TlsProxyRequiresDomain =>
                "Set a domain name or disable TLS proxy.",
        }
    }
}
```

## Checklist

- [ ] Fields are private (no `pub` on struct fields)
- [ ] Validated constructor returns `Result<Self, Error>`
- [ ] Custom `Deserialize` impl using a Raw mirror struct
- [ ] Error type uses `thiserror::Error` with `help()` method
- [ ] Single point of validation — not duplicated in setters/validators

## Reference

Full guide: [`docs/contributing/ddd-practices.md`](../../docs/contributing/ddd-practices.md)
ADR: [`docs/decisions/validated-deserialization-for-domain-types.md`](../../docs/decisions/validated-deserialization-for-domain-types.md)
