# Decision: Use Secrecy Crate for Sensitive Data Handling

## Status

Accepted

## Date

2025-12-17

## Context

### The Problem

Sensitive data (API tokens, passwords, database credentials) is currently stored as plain `String` types throughout the codebase. This creates several security and maintainability issues:

1. **Accidental Exposure**: Secrets appear in full when printed with `Debug` formatting, potentially leaking through:

   - Debug logs during development
   - Error messages and stack traces
   - Panic outputs
   - Test output and CI logs

2. **No Type-Level Security**: The type system doesn't distinguish between secrets and regular strings:

   - No compile-time guarantee that secrets are handled carefully
   - Easy to accidentally log or print secret values
   - No centralized place to add security enhancements

3. **Difficult Auditing**: Hard to track secret usage:

   - Can't easily find all places where secrets are used
   - Can't grep for secret-specific types
   - No visibility into when/where secrets are exposed

4. **Memory Security Gap**: Secrets remain in memory even after being dropped, potentially accessible through:
   - Memory dumps
   - Core dumps
   - Swap files
   - Process memory inspection

### Examples of Current Problems

```rust
// Current problematic approach
#[derive(Debug)]
pub struct HetznerConfig {
    pub api_token: String, // Exposed in debug output!
}

// This accidentally logs the token
tracing::debug!("Config: {:?}", config);
// Output: Config: HetznerConfig { api_token: "hf_abc123..." }

pub struct MysqlConfig {
    pub password: String, // Visible in error messages!
}

// Error contains password
return Err(format!("Failed to connect to {:?}", config));
```

### Project Requirements

1. **Identification**: Clearly identify where secrets are used in the codebase
2. **Redacted Output**: Prevent accidental exposure through debug/display formatting
3. **Memory Security**: Wipe secrets from memory when no longer needed
4. **Maintainability**: Keep solution simple and well-documented
5. **Standards Compliance**: Follow Rust ecosystem best practices

## Decision

**Adopt the `secrecy` crate** (https://crates.io/crates/secrecy) as the standard solution for handling sensitive data throughout the codebase.

### Implementation Details

**Core Type**: `Secret<T>` from the `secrecy` crate

```rust
// src/shared/secret.rs
pub use secrecy::{ExposeSecret, Secret, SecretString};
use secrecy::SerializableSecret;

// Enable serialization for String secrets (required for config files)
impl SerializableSecret for String {}

// Domain-specific type aliases for clarity
pub type ApiToken = Secret<String>;
pub type Password = Secret<String>;
```

**Usage Pattern**:

```rust
use crate::shared::{ApiToken, ExposeSecret};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HetznerConfig {
    pub api_token: ApiToken, // Automatically redacted in Debug
    // ...
}

// Creating with secret
let config = HetznerConfig {
    api_token: Secret::new("token".to_string()),
};

// Debug output is safe
println!("{:?}", config); // HetznerConfig { api_token: Secret([REDACTED]) }

// Explicit exposure when needed
let token_str = config.api_token.expose_secret();
```

**Locations to Apply**:

1. **Provider Secrets**:

   - `HetznerConfig.api_token` → `ApiToken`

2. **Database Secrets**:

   - `MysqlConfig.password` → `Password`

3. **API Secrets**:
   - `HttpApiSection.admin_token` → `ApiToken`
   - `HttpApiConfig.admin_token` → `ApiToken`

## Rationale

### Why `secrecy` Crate Over Custom Implementation?

1. **Battle-Tested Security**:

   - Used by major Rust projects (diesel, sqlx, etc.)
   - Audited and maintained by security-conscious community
   - Implements best practices from cryptography experts

2. **Memory Zeroing**:

   - Uses `zeroize` crate to securely wipe memory on drop
   - Prevents secrets from lingering in memory/swap/core dumps
   - Hard to implement correctly in custom solution

3. **Industry Standard**:

   - De facto standard for secret handling in Rust
   - Well-documented with extensive examples
   - Future maintainers will recognize the pattern

4. **Minimal Complexity**:

   - Single dependency (`secrecy` + transitive `zeroize`)
   - Simple API: `Secret::new()` and `expose_secret()`
   - Type aliases reduce boilerplate

5. **Future-Proof**:
   - If security requirements evolve (audits, compliance), infrastructure is ready
   - Can easily add more advanced features if needed
   - No need to retrofit memory zeroing later

### Why Not a Custom Type?

**Pros of Custom**:

- Zero dependencies
- Full control
- Simpler initial implementation

**Cons of Custom** (Why we rejected it):

- ❌ No memory zeroing (significant security gap)
- ❌ Need to implement everything ourselves
- ❌ Risk of security mistakes in implementation
- ❌ Reinventing a well-solved problem
- ❌ Future maintainers less likely to understand custom approach

### Why Not a Hybrid Wrapper?

A custom wrapper around `secrecy` would add:

- Extra abstraction layers
- More code to maintain
- Learning curve for contributors
- No significant benefits over direct usage

The `secrecy` API is already simple and well-designed - wrapping it adds unnecessary complexity.

## Consequences

### Positive

✅ **Security Improvements**:

- Secrets automatically redacted in debug output
- Memory securely wiped on drop (via `zeroize`)
- Type-safe secret handling at compile time
- Industry-standard security practices

✅ **Code Quality**:

- Clear identification of all secret values (grep for `Secret<T>`)
- Explicit `expose_secret()` calls visible in code review
- Type aliases improve readability (`ApiToken` vs `String`)
- Consistent pattern across codebase

✅ **Maintainability**:

- Standard solution recognized by Rust developers
- Extensive documentation and examples available
- Community support and updates
- Easy to audit secret usage

✅ **Minimal Overhead**:

- Small dependency (single crate + zeroize)
- No runtime performance impact
- `no_std` compatible (if we ever need it)
- Well-maintained and stable

### Negative

⚠️ **Learning Curve**:

- Contributors need to learn `expose_secret()` pattern
- Must implement `SerializableSecret` marker trait per type
- Slightly more verbose than plain `String`

**Mitigation**: Add examples to `AGENTS.md` and create comprehensive ADR (this document).

⚠️ **Serialization Boilerplate**:

- Need `impl SerializableSecret for String {}` once
- Intentional friction to prevent accidental serialization

**Mitigation**: Single implementation covers all `Secret<String>` uses.

⚠️ **Dependency Addition**:

- Adds `secrecy` (~50KB) and `zeroize` (~20KB) to dependency tree

**Mitigation**: Tiny, stable dependencies with strong security track record.

### Migration Impact

**Affected Modules** (requires updates):

- `src/domain/provider/hetzner.rs` (API token)
- `src/domain/tracker/database/mysql.rs` (password)
- `src/application/command_handlers/create/config/tracker/http_api_section.rs` (admin token)
- `src/domain/tracker/http_api.rs` (admin token)
- All tests using these types

**Breaking Changes**: None (internal refactoring only)

**Timeline**: Estimated 2-3 sprints for complete migration

## Alternatives Considered

### Alternative 1: Custom `Secret<T>` Type

```rust
// Custom implementation
pub struct Secret<T> {
    inner: T,
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}
```

**Rejected because**:

- No memory zeroing on drop (major security gap)
- Would need to add `zeroize` dependency anyway
- Reinventing already-solved problem
- More code to maintain and audit
- Less trustworthy than community-vetted solution

### Alternative 2: Manual Conventions

Use comments and documentation to mark secret fields:

```rust
pub struct Config {
    /// SECRET: Never log this field
    pub api_token: String,
}
```

**Rejected because**:

- No type-safety or compile-time guarantees
- Easy to accidentally violate conventions
- No automatic redaction of debug output
- No memory security
- Impossible to audit automatically

### Alternative 3: `secrets` Crate

Alternative crate with more advanced features (mlock, mprotect).

**Rejected because**:

- Requires `std` and `libc` (not `no_std` compatible)
- Heavier dependency
- More complexity than we currently need
- `secrecy` is more widely adopted

## Related Decisions

- [Error Context Strategy](./error-context-strategy.md) - Errors must not expose secret values
- [Actionable Error Messages](./actionable-error-messages.md) - Error messages must redact secrets
- [Development Principles](../development-principles.md) - Security and observability principles

## References

- **Secrecy Crate**: https://docs.rs/secrecy/latest/secrecy/
- **Zeroize Crate**: https://docs.rs/zeroize/latest/zeroize/
- **Security Best Practices**: https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Related Issue**: [Secret Type Introduction Refactor Plan](../refactors/plans/secret-type-introduction.md)

## Implementation Notes

### Phase 1: Setup (Priority: P0)

1. Add `secrecy` dependency to `Cargo.toml`
2. Create `src/shared/secret.rs` module
3. Export types and implement `SerializableSecret` for `String`
4. Add type aliases: `ApiToken`, `Password`, `SecretString`

### Phase 2: Provider Secrets (Priority: P1)

1. Update `HetznerConfig.api_token` to use `ApiToken`
2. Update all Hetzner-related tests
3. Verify no secrets in debug output

### Phase 3: Database Secrets (Priority: P2)

1. Update `MysqlConfig.password` to use `Password`
2. Update all MySQL-related tests
3. Verify template rendering works correctly

### Phase 4: API Secrets (Priority: P2)

1. Update `HttpApiSection.admin_token` to use `ApiToken`
2. Update `HttpApiConfig.admin_token` to use `ApiToken`
3. Update all HTTP API tests

### Phase 5: Documentation (Priority: P3)

1. Update `AGENTS.md` with secret handling rule
2. Add examples to module documentation
3. Update contributing guidelines

### Testing Verification

For each phase, verify:

- ✅ All unit tests pass
- ✅ Debug output shows `[REDACTED]` instead of actual values
- ✅ Serialization/deserialization works correctly
- ✅ Error messages don't expose secrets
- ✅ All linters pass (clippy, rustfmt, etc.)

---

**Last Updated**: 2025-12-17
