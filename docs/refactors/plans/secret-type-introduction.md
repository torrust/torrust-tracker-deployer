# Secret Type Introduction

## 📋 Overview

Introduce wrapper types based on the `secrecy` crate's `SecretString` to replace primitive `String` types for sensitive data throughout the codebase. This refactoring enhances security by clearly identifying secret values, preventing accidental exposure through logging or debugging, and ensuring secrets are securely wiped from memory when dropped.

**Decision**: After evaluating custom implementation vs industry-standard solutions, we chose to use `secrecy::SecretString` as the foundation with thin wrapper types (see [ADR](../../decisions/secrecy-crate-for-sensitive-data.md)).

**Implementation Note**: `SecretString` (which is `SecretBox<str>`) provides debug redaction and memory zeroing but cannot implement `Serialize` (requires `SerializableSecret` trait which `str` doesn't implement due to orphan rules). Our wrapper types (`ApiToken`, `Password`) add serialization support needed for config file generation in this deployment tool.

**Target Files:**

- `Cargo.toml` (add secrecy dependency)
- `src/shared/secret.rs` (new file - re-exports and type aliases)
- `src/domain/provider/hetzner.rs` (API token)
- `src/domain/tracker/database/mysql.rs` (database password)
- `src/application/command_handlers/create/config/tracker/http_api_section.rs` (admin token)
- `src/domain/tracker/http_api.rs` (admin token)
- All files using the above types
- `docs/decisions/secrecy-crate-for-sensitive-data.md` (ADR - already created)
- `AGENTS.md` (new essential rule)

**Scope:**

- Add `secrecy` crate dependency to project (with `serde` feature)
- Create `src/shared/secret.rs` module with `ApiToken` and `Password` wrappers around `SecretString`
- Wrappers add: serialization support, PartialEq/Eq for testing, domain-specific types
- Replace all `String` fields containing secrets with `ApiToken`/`Password` types
- Update all tests to use `ApiToken::from()` / `Password::from()` and `.expose_secret()`
- Verify debug output redacts secrets (provided by `SecretString`)
- Document decision in ADR (already completed)
- Update AI agent instructions

## 📊 Progress Tracking

**Total Active Proposals**: 10
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 9
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Core Setup (High Impact, Low Effort)**: ✅ 2/2 completed (100%)
- **Phase 1 - Provider Secrets (High Impact, Medium Effort)**: ✅ 2/2 completed (100%)
- **Phase 2 - Database Secrets (High Impact, Medium Effort)**: ✅ 2/2 completed (100%)
- **Phase 3 - Documentation and Guidance (High Impact, Low Effort)**: ✅ 2/2 completed (100%) - ADR and AGENTS.md completed
- **Phase 4 - Future Enhancements (Low Impact, Medium Effort)**: ✅ 2/3 completed (67%) - Proposals #9 and #10 verified

### Discarded Proposals

None

### Postponed Proposals

None - the decision to use `secrecy` crate addresses all initial requirements plus memory zeroing.

## 📋 Secret Locations Inventory

Comprehensive list of all locations where secrets are currently stored as plain `String` types and need to be converted to `Secret<T>`:

### Domain Layer (Core Business Logic)

| File                                   | Type/Field                   | Secret Type | Layer  | Status         | Priority | Proposal |
| -------------------------------------- | ---------------------------- | ----------- | ------ | -------------- | -------- | -------- |
| `src/domain/provider/hetzner.rs`       | `HetznerConfig::api_token`   | API Token   | Domain | ⏳ Not Started | P1       | #2       |
| `src/domain/tracker/database/mysql.rs` | `MysqlConfig::password`      | Password    | Domain | ⏳ Not Started | P2       | #4       |
| `src/domain/tracker/config.rs`         | `HttpApiConfig::admin_token` | API Token   | Domain | ⏳ Not Started | P2       | TBD      |

### Application Layer (DTOs & Config)

| File                                                                             | Type/Field                         | Secret Type | Layer       | Status         | Priority | Proposal |
| -------------------------------------------------------------------------------- | ---------------------------------- | ----------- | ----------- | -------------- | -------- | -------- |
| `src/application/command_handlers/create/config/provider/hetzner.rs`             | `HetznerSection::api_token`        | API Token   | Application | ⏳ Not Started | P1       | #2       |
| `src/application/command_handlers/create/config/tracker/http_api_section.rs`     | `HttpApiSection::admin_token`      | API Token   | Application | ⏳ Not Started | P2       | TBD      |
| `src/application/command_handlers/create/config/tracker/tracker_core_section.rs` | `DatabaseSection::Mysql::password` | Password    | Application | ⏳ Not Started | P2       | #4       |

### Infrastructure Layer (Templates & External Tools)

| File                                                                                                | Type/Field                                  | Secret Type | Layer          | Status         | Priority | Proposal |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------- | ----------- | -------------- | -------------- | -------- | -------- |
| `src/infrastructure/templating/tofu/template/providers/hetzner/wrappers/variables/context.rs`       | `HetznerVariablesContext::hcloud_api_token` | API Token   | Infrastructure | ⏳ Not Started | P1       | #2       |
| `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs`                  | `TrackerConfigContext::mysql_password`      | Password    | Infrastructure | ⏳ Not Started | P2       | #4       |
| `src/infrastructure/templating/prometheus/template/wrapper/prometheus_config/context.rs`            | `TrackerScrapeConfig::api_token`            | API Token   | Infrastructure | ⏳ Not Started | P2       | TBD      |
| `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/database.rs` | `DatabaseContext::root_password`            | Password    | Infrastructure | ⏳ Not Started | P2       | #4       |
| `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/database.rs` | `DatabaseContext::password`                 | Password    | Infrastructure | ⏳ Not Started | P2       | #4       |
| `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`                     | `EnvContext::api_admin_token`               | API Token   | Infrastructure | ⏳ Not Started | P2       | TBD      |
| `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`                     | `EnvContext::root_password`                 | Password    | Infrastructure | ⏳ Not Started | P2       | #4       |
| `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`                     | `EnvContext::password`                      | Password    | Infrastructure | ⏳ Not Started | P2       | #4       |

### Testing Layer (Test Infrastructure)

| File                                           | Type/Field                  | Secret Type | Layer   | Status         | Priority | Proposal |
| ---------------------------------------------- | --------------------------- | ----------- | ------- | -------------- | -------- | -------- |
| `src/testing/integration/ssh_server/config.rs` | `SshServerConfig::password` | Password    | Testing | ⏳ Not Started | P4       | Future   |

### Summary Statistics

- **Total Secret Fields**: 16
- **Domain Layer**: 3 fields
- **Application Layer**: 3 fields
- **Infrastructure Layer**: 9 fields
- **Testing Layer**: 1 field

**By Secret Type**:

- **API Tokens**: 6 fields
- **Passwords**: 10 fields

**Priority Breakdown**:

- **P1 (Provider Secrets)**: 3 fields (Hetzner API token across layers)
- **P2 (Database & API Secrets)**: 12 fields (MySQL passwords, HTTP API tokens)
- **P4 (Testing Infrastructure)**: 1 field (test SSH server password)

### Notes

1. **Layered Approach**: Secrets flow from domain → application → infrastructure, so we must update all layers
2. **Template Contexts**: Infrastructure layer template contexts need updates to accept and handle `Secret<T>`
3. **Test Data**: Testing layer fields can be lower priority but should eventually be consistent
4. **Conversion Pattern**: Each field requires:
   - Type change: `String` → `Secret<String>`
   - Construction: Wrap with `Secret::new()`
   - Access: Call `.expose_secret()` when needed
   - Test updates: Use new type in all test fixtures

## 🎯 Key Problems Identified

### 1. Secret Exposure Risk

Secrets (API tokens, database passwords) are currently stored as plain `String` types, which means:

- They appear in full when printed with `Debug` formatting
- They may leak through error messages or logs
- No compile-time distinction between secret and non-secret data

### 2. Difficult Secret Tracking

Without a dedicated type, it's hard to:

- Locate all places where secrets are used
- Ensure consistent handling across the codebase
- Audit secret usage patterns

### 3. No Type-Level Security Guarantees

The type system doesn't enforce secure handling:

- Accidental `println!("{:?}", config)` exposes secrets
- Error context might include secret values
- No centralized place to add security enhancements
- Secrets remain in memory after being dropped (accessible via memory dumps)

## 🚀 Refactoring Phases

---

## Phase 0: Core Type Implementation (Highest Priority)

This phase creates the foundational `Secret` type that will be used throughout the codebase. Must be completed before any other phase.

### Proposal #0: Add `secrecy` Crate and Setup Module

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

There is no dedicated type for handling sensitive data. All secrets are currently stored as `String`, making them indistinguishable from regular strings and exposing them to potential leaks through debug output, logs, and error messages.

```rust
// Current problematic approach
pub struct HetznerConfig {
    pub api_token: String, // Secret exposed in debug output
}
```

#### Proposed Solution

Add the `secrecy` crate dependency and create a shared module with re-exports and type aliases:

```toml
# Cargo.toml
[dependencies]
secrecy = { version = "0.10", features = ["serde"] }
```

````rust
// src/shared/secret.rs
//! Secret types for handling sensitive data
//!
//! This module re-exports the `secrecy` crate and provides domain-specific
//! type aliases for clarity. The `secrecy` crate ensures:
//!
//! - Secrets are redacted in debug output
//! - Secrets are securely wiped from memory on drop (via zeroize)
//! - Explicit `expose_secret()` calls make secret access visible
//!
//! # Examples
//!
//! ```rust
//! use torrust_tracker_deployer_lib::shared::{ApiToken, ExposeSecret};
//! use secrecy::Secret;
//!
//! let token = Secret::new("my-secret-token".to_string());
//!
//! // Debug output is redacted
//! println!("{:?}", token); // Output: Secret([REDACTED])
//!
//! // Explicit access required
//! let token_str = token.expose_secret();
//! ```

use secrecy::SerializableSecret;

// Re-export secrecy types
pub use secrecy::{ExposeSecret, Secret, SecretString};

// Enable serialization for String secrets (required for config files)
// This is intentional - config files need actual values
impl SerializableSecret for String {}

// Domain-specific type aliases for clarity
pub type ApiToken = Secret<String>;
pub type Password = Secret<String>;

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::{ExposeSecret, Secret};

    #[test]
    fn it_should_create_secret_with_new() {
        let secret = Secret::new("my-secret".to_string());
        assert_eq!(secret.expose_secret(), "my-secret");
    }

    #[test]
    fn it_should_redact_debug_output() {
        let secret = Secret::new("api-token-12345".to_string());
        let debug_output = format!("{:?}", secret);
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("api-token-12345"));
    }

    #[test]
    fn it_should_expose_inner_value_when_explicitly_requested() {
        let secret = Secret::new("secret-value".to_string());
        assert_eq!(secret.expose_secret(), "secret-value");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let secret = Secret::new("my-token".to_string());
        let json = serde_json::to_string(&secret).unwrap();
        // Note: Serialization exposes value for configuration purposes
        assert_eq!(json, r#""my-token""#);
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = r#""my-token""#;
        let secret: Secret<String> = serde_json::from_str(json).unwrap();
        assert_eq!(secret.expose_secret(), "my-token");
    }

    #[test]
    fn it_should_support_cloning() {
        let secret = Secret::new("clone-me".to_string());
        let cloned = secret.clone();
        assert_eq!(secret.expose_secret(), cloned.expose_secret());
    }
}
````

#### Rationale

- **Industry standard**: `secrecy` crate is the de facto standard in Rust ecosystem
- **Memory zeroing**: Secrets are securely wiped from memory on drop (via `zeroize`)
- **Battle-tested**: Used by major projects (diesel, sqlx, etc.) with security audits
- **Explicit `expose_secret()` method**: Makes secret access visible in code reviews
- **Serde support**: Built-in with `SerializableSecret` marker trait
- **Type aliases**: Domain-specific aliases (`ApiToken`, `Password`) improve clarity
- **Minimal complexity**: Single dependency, simple API

#### Benefits

- ✅ Clear identification of all secret values in codebase
- ✅ Prevents accidental exposure through `Debug` output (auto-redacted)
- ✅ **Memory security**: Secrets wiped from memory on drop
- ✅ Type-safe distinction between secrets and regular strings
- ✅ Easy to locate all secret usage with IDE search
- ✅ Industry-standard solution recognized by Rust developers
- ✅ Foundation for future security enhancements
- ✅ Small, stable dependency (~70KB total with zeroize)

#### Implementation Checklist

- [ ] Add `secrecy = { version = "0.10", features = ["serde"] }` to `Cargo.toml`
- [ ] Create `src/shared/secret.rs` with re-exports and type aliases
- [ ] Implement `SerializableSecret for String`
- [ ] Add type aliases: `ApiToken`, `Password`
- [ ] Write unit tests (redaction, serialization, expose_secret)
- [ ] Export from `src/shared/mod.rs`
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Verify `cargo tree` shows secrecy dependency

#### Testing Strategy

Unit tests should verify:

- Secret creation and value exposure
- Debug output is redacted
- Display output is redacted
- Serialization/deserialization works correctly
- Equality and cloning work as expected

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: ~50 (re-exports, type aliases, tests)
- **Net Change**: +50 lines
- **Dependencies Added**: `secrecy@0.10`, `zeroize@1.8` (transitive)
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

### Proposal #1: Export Secret Types from `src/shared/mod.rs`

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #0

#### Problem

The secret types need to be accessible throughout the codebase via the `shared` module.

#### Proposed Solution

Add module declaration and public exports in `src/shared/mod.rs`:

```rust
// Add to src/shared/mod.rs
pub mod secret;

pub use secret::{ApiToken, ExposeSecret, Password, Secret, SecretString};
```

#### Rationale

Standard Rust module system pattern for making types publicly available.

#### Benefits

- ✅ Makes `Secret` type accessible across the codebase
- ✅ Allows users to import with `use crate::shared::{Secret, SecretString}`
- ✅ Maintains clean module organization

#### Implementation Checklist

- [ ] Add `pub mod secret;` to `src/shared/mod.rs`
- [ ] Add `pub use secret::{Secret, SecretString};`
- [ ] Verify compilation succeeds
- [ ] Test that type can be imported in other modules

#### Testing Strategy

Verify that imports work correctly in a test file:

```rust
use crate::shared::{Secret, SecretString};
```

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: 2
- **Net Change**: +2 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

## Phase 1: Provider Secrets

Replace `String` with `Secret` types in provider configurations. Hetzner API tokens are particularly sensitive.

### Proposal #2: Replace Hetzner API Token with `ApiToken`

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposals #0, #1

#### Problem

Hetzner API token is currently a plain `String`, exposed in debug output:

```rust
// src/domain/provider/hetzner.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerConfig {
    /// Note: Future improvement could use a validated `ApiToken` type.
    pub api_token: String, // Exposed in Debug output!
    pub server_type: String,
    pub location: String,
    pub image: String,
}
```

#### Proposed Solution

Replace `api_token` field with `ApiToken` type:

```rust
use crate::shared::ApiToken;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerConfig {
    /// Hetzner API token for authentication.
    ///
    /// This value is kept secure and not exposed in debug output.
    pub api_token: ApiToken,
    pub server_type: String,
    pub location: String,
    pub image: String,
}
```

Update all usages:

```rust
// Creating config
use secrecy::Secret;

let config = HetznerConfig {
    api_token: Secret::new("your-token".to_string()),
    // ...
};

// Accessing token (when needed for API calls)
use secrecy::ExposeSecret;
let token_str = config.api_token.expose_secret();
```

#### Rationale

API tokens are highly sensitive and should never appear in logs or debug output. Using `ApiToken` type makes this guarantee at compile time.

#### Benefits

- ✅ API token never appears in debug output
- ✅ Clear identification of sensitive field
- ✅ Type safety for secret handling
- ✅ Audit trail of where tokens are exposed

#### Implementation Checklist

- [ ] Update `HetznerConfig.api_token` field type to `ApiToken`
- [ ] Add import `use crate::shared::ApiToken;`
- [ ] Update struct instantiation in tests (use `ApiToken::new()`)
- [ ] Update all places where token is accessed (use `.expose()`)
- [ ] Update documentation comments
- [ ] Search codebase for all `hetzner.api_token` usages
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Run existing tests and ensure:

- All Hetzner config tests pass
- Debug output doesn't contain token value
- Serialization/deserialization works correctly

#### Results (if completed)

- **Lines Removed**: ~10
- **Lines Added**: ~15
- **Net Change**: +5 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

### Proposal #3: Update Provider Config Enum

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P1  
**Depends On**: Proposal #2

#### Problem

`ProviderConfig` enum uses `HetznerConfig` which now has `ApiToken`. Need to ensure the enum propagates the change correctly.

#### Proposed Solution

Verify `ProviderConfig` in `src/domain/provider/config.rs` works with updated `HetznerConfig`. No direct changes needed since it uses the Hetzner type, but tests may need updates.

```rust
// src/domain/provider/config.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "provider", content = "config")]
pub enum ProviderConfig {
    #[serde(rename = "lxd_vm")]
    LxdVm(LxdVmConfig),
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig), // This now contains ApiToken
}
```

Update tests that create `ProviderConfig::Hetzner` variants.

#### Rationale

Ensures the enum layer correctly handles the secret type.

#### Benefits

- ✅ Consistent secret handling across provider types
- ✅ Tests verify enum serialization with secrets

#### Implementation Checklist

- [ ] Review `ProviderConfig` enum definition
- [ ] Update all tests creating `ProviderConfig::Hetzner`
- [ ] Verify serialization/deserialization tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Run existing provider config tests and verify enum variants work correctly.

#### Results (if completed)

- **Lines Removed**: ~5
- **Lines Added**: ~5
- **Net Change**: ±0 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

## Phase 2: Database Secrets

Replace database password strings with `Password` type.

### Proposal #4: Replace MySQL Password with `Password` Type

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposals #0, #1

#### Problem

MySQL database password is stored as plain `String`:

```rust
// src/domain/tracker/database/mysql.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String, // Exposed in debug output!
}
```

#### Proposed Solution

Replace `password` field with `Password` type:

```rust
use crate::shared::Password;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    /// Database password (redacted in debug output)
    pub password: Password,
}
```

Update all usages:

```rust
// Creating config
use secrecy::Secret;

let config = MysqlConfig {
    password: Secret::new("secure_password".to_string()),
    // ...
};

// Accessing password (when building connection string)
use secrecy::ExposeSecret;
let password_str = config.password.expose_secret();
```

#### Rationale

Database passwords are extremely sensitive and must not leak through logs or error messages.

#### Benefits

- ✅ Database password never appears in debug output
- ✅ Type safety for password handling
- ✅ Clear identification of sensitive field
- ✅ Consistent with API token approach

#### Implementation Checklist

- [ ] Update `MysqlConfig.password` field type to `Password`
- [ ] Add import `use crate::shared::Password;`
- [ ] Update struct instantiation in tests (use `Password::new()`)
- [ ] Update all places where password is accessed (use `.expose()`)
- [ ] Search for all MySQL config creation points
- [ ] Update Docker Compose template if password is rendered
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Run existing tests and ensure:

- All MySQL config tests pass
- Debug output doesn't contain password
- Serialization/deserialization works correctly
- Template rendering works correctly

#### Results (if completed)

- **Lines Removed**: ~10
- **Lines Added**: ~15
- **Net Change**: +5 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

### Proposal #5: Update Database Config Enum

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #4

#### Problem

`DatabaseConfig` enum uses `MysqlConfig` which now has `Password` type.

#### Proposed Solution

Verify `DatabaseConfig` in `src/domain/tracker/database/mod.rs` works with updated `MysqlConfig`:

```rust
// src/domain/tracker/database/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "driver", content = "config")]
pub enum DatabaseConfig {
    #[serde(rename = "sqlite3")]
    Sqlite(SqliteConfig),
    #[serde(rename = "mysql")]
    Mysql(MysqlConfig), // This now contains Password
}
```

Update tests that create `DatabaseConfig::Mysql` variants.

#### Rationale

Ensures consistent secret handling across database configurations.

#### Benefits

- ✅ Enum correctly propagates secret types
- ✅ Tests verify serialization with passwords

#### Implementation Checklist

- [ ] Review `DatabaseConfig` enum definition
- [ ] Update all tests creating `DatabaseConfig::Mysql`
- [ ] Verify serialization/deserialization tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Run existing database config tests and verify enum variants work correctly.

#### Results (if completed)

- **Lines Removed**: ~5
- **Lines Added**: ~5
- **Net Change**: ±0 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

## Phase 3: Documentation and Guidance

Create ADR and update AI agent instructions to ensure consistent secret handling.

### Proposal #6: Reference ADR for Secrecy Crate Decision

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P3  
**Depends On**: None  
**Completed**: 2025-12-17  
**Commit**: TBD

#### Problem

The decision to use `secrecy` crate and the rationale needs to be documented for future maintainers.

#### Solution

✅ **Already completed**: Created comprehensive ADR at `docs/decisions/secrecy-crate-for-sensitive-data.md`.

The ADR documents:

- Context: Security issues with plain strings
- Decision: Use `secrecy` crate over custom implementation
- Rationale: Battle-tested, memory zeroing, industry standard
- Consequences: Benefits and trade-offs
- Alternatives considered: Custom type, manual conventions, `secrets` crate
- Implementation phases and affected files

#### Benefits

- ✅ Documented rationale for using `secrecy` crate
- ✅ Guidance for maintainers and contributors
- ✅ Reference for similar decisions
- ✅ Security best practices documented
- ✅ Comparison with alternatives preserved

#### Results

- **Lines Removed**: 0
- **Lines Added**: ~400 (ADR document)
- **Net Change**: +400 lines
- **Tests**: ✅ N/A
- **Linters**: ✅ Passed

---

### Proposal #7: Update AGENTS.md with Secret Handling Rule

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P3  
**Depends On**: Proposal #6

#### Problem

AI agents (GitHub Copilot) need clear instructions on how to handle secrets in code.

#### Proposed Solution

Add a new essential rule to `AGENTS.md` in the "Essential Rules" section:

```markdown
## 🔧 Essential Rules

[...existing rules...]

XX. **When handling sensitive data (secrets)**: Read [`docs/decisions/secrecy-crate-for-sensitive-data.md`](docs/decisions/secrecy-crate-for-sensitive-data.md) for the complete guide on secret handling. **CRITICAL**: Never use `String` for sensitive data like API tokens, passwords, private keys, or database credentials. Always use the `secrecy` crate's `Secret<T>` type from `src/shared/secret.rs`:

    - **API Tokens**: Use `Secret<String>` or `ApiToken` type alias
    - **Passwords**: Use `Secret<String>` or `Password` type alias
    - **Database Credentials**: Use `Password` for passwords in database configs
    - **Access Pattern**: Call `.expose_secret()` only when the actual value is needed
    - **Never**: Log, print, or include secret values in error messages
    - **Debug Output**: `Secret` types automatically redact in `Debug` output
    - **Memory Security**: Secrets are wiped from memory on drop (via `zeroize`)

    **Examples of secret fields**:
    - `HetznerConfig.api_token` (use `ApiToken`)
    - `MysqlConfig.password` (use `Password`)
    - Any field containing authentication credentials

    **Rule**: If a field contains data that should not appear in logs or debug output, it MUST use `Secret<T>` or one of its type aliases.
```

#### Rationale

AI agents need explicit instructions to maintain consistent patterns. This rule ensures Copilot will suggest the correct type when creating new secret fields.

#### Benefits

- ✅ AI agents suggest correct secret types
- ✅ Consistent codebase patterns
- ✅ Reduced chance of security mistakes
- ✅ Clear examples for reference

#### Implementation Checklist

- [ ] Add new rule to `AGENTS.md` Essential Rules section
- [ ] Number the rule correctly (next sequential number)
- [ ] Include clear examples
- [ ] Reference the ADR for detailed information
- [ ] List common secret types and their aliases
- [ ] Add "CRITICAL" marker for visibility
- [ ] Run markdown linter
- [ ] Run spell checker
- [ ] Test that rule is clear and actionable

#### Testing Strategy

Review rule clarity with team. Verify markdown formatting is correct.

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: ~25
- **Net Change**: +25 lines
- **Tests**: ✅ N/A
- **Linters**: ✅ Pending

---

## Phase 4: Future Enhancements

Optional improvements that can be done later if needed.

### Proposal #8: Add Debug Tracing for Secret Access

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P4  
**Depends On**: Proposals #0-#7

**Note**: This would require wrapping `secrecy::Secret` in a custom newtype to add tracing. Consider if the added complexity is worth it.

#### Problem

It would be useful to track when and where secrets are exposed during debugging, especially in complex execution flows.

#### Proposed Solution

Add optional tracing to the `expose()` method using the `tracing` crate (which we already use):

```rust
impl<T> Secret<T> {
    #[must_use]
    pub fn expose(&self) -> &T {
        #[cfg(debug_assertions)]
        tracing::trace!("Secret exposed at: {}", std::panic::Location::caller());

        &self.inner
    }
}
```

This would log the location where secrets are exposed, but only in debug builds.

#### Rationale

During development, it's useful to see where secrets are being accessed. This helps audit secret usage patterns without impacting production performance.

#### Benefits

- ✅ Visibility into secret access patterns during development
- ✅ Helps identify unnecessary secret exposure
- ✅ No performance impact in release builds
- ✅ Uses existing tracing infrastructure

#### Implementation Checklist

- [ ] Add tracing to `expose()` method
- [ ] Use `#[cfg(debug_assertions)]` for debug-only
- [ ] Add location tracking with `std::panic::Location::caller()`
- [ ] Test that tracing output appears in debug builds
- [ ] Verify no overhead in release builds
- [ ] Update documentation
- [ ] Run linter and fix any issues

#### Testing Strategy

Build in debug mode and verify trace messages appear. Build in release mode and verify no overhead.

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: ~5
- **Net Change**: +5 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

### Proposal #9: Add `expose_str()` Convenience Method

**Status**: ✅ Completed (Already Implemented)  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P4  
**Depends On**: Proposal #0

#### Problem

When working with `Secret<String>`, we often need `&str` instead of `&String`. Currently requires chaining: `secret.expose_secret().as_str()`.

#### Proposed Solution

Create an extension trait for convenience:

```rust
// src/shared/secret.rs
use secrecy::{ExposeSecret, Secret};

pub trait ExposeSecretStr {
    fn expose_str(&self) -> &str;
}

impl ExposeSecretStr for Secret<String> {
    /// Exposes the secret as a string slice
    ///
    /// # Security Warning
    ///
    /// Use with the same caution as `expose_secret()`.
    fn expose_str(&self) -> &str {
        self.expose_secret().as_str()
    }
}
```

#### Rationale

Common use case deserves convenient API. Reduces boilerplate in code.

#### Benefits

- ✅ Cleaner API for string secrets
- ✅ Reduces boilerplate
- ✅ Still requires explicit method call

#### Implementation Checklist

- [ ] Add `expose_str()` method to `Secret<String>` impl
- [ ] Add documentation
- [ ] Add unit tests
- [ ] Run linter and fix any issues

#### Testing Strategy

Test that method returns correct `&str` reference.

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: ~15 (including tests)
- **Net Change**: +15 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

### Proposal #10: Verify JSON Schema Support for `Secret`

**Status**: ✅ Verified  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P4  
**Depends On**: Proposal #0

#### Problem

Configuration schemas (used for JSON Schema generation) need to work with `Secret` types. We use `schemars` for schema generation.

#### Proposed Solution

Verify that `schemars` works with `secrecy::Secret<T>` out of the box. If not, implement `JsonSchema` trait:

```rust
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, gen::SchemaGenerator, schema::Schema};

#[cfg(feature = "schemars")]
impl<T: JsonSchema> JsonSchema for Secret<T> {
    fn schema_name() -> String {
        format!("Secret_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        T::json_schema(gen)
    }
}
```

This makes `Secret<T>` transparent in JSON Schema generation - it will use the schema of the inner type.

#### Rationale

Configuration validation and schema generation should work seamlessly with secret types.

#### Benefits

- ✅ JSON Schema generation works with secrets
- ✅ Configuration validation remains functional
- ✅ Type-safe schema generation

#### Implementation Checklist

- [ ] Add `schemars` dependency (check if already present)
- [ ] Implement `JsonSchema` trait for `Secret<T>`
- [ ] Add feature flag if needed
- [ ] Test schema generation
- [ ] Update schemas in `schemas/` directory
- [ ] Verify validation still works
- [ ] Run linter and fix any issues

#### Testing Strategy

Generate JSON schemas and verify secret fields appear correctly. Test configuration validation.

#### Results (if completed)

- **Lines Removed**: 0
- **Lines Added**: ~20
- **Net Change**: +20 lines
- **Tests**: ✅ Pending
- **Linters**: ✅ Pending

---

## 📈 Timeline

- **Start Date**: December 17, 2025
- **Estimated Duration**: 2-3 sprints
  - Phase 0: 1-2 days (core type implementation)
  - Phase 1: 2-3 days (provider secrets)
  - Phase 2: 2-3 days (database secrets)
  - Phase 3: 1-2 days (documentation)
  - Phase 4: 3-5 days (optional enhancements)
- **Actual Completion**: _Pending_

## 🔍 Review Process

### Approval Criteria

- [x] All proposals reviewed by project maintainer
- [x] Technical feasibility validated
- [x] Aligns with [Development Principles](../development-principles.md) (Observability, Security)
- [x] Implementation plan is clear and actionable
- [x] Phased approach minimizes risk

### Completion Criteria

- [ ] All Phase 0-3 proposals implemented
- [ ] All tests passing
- [ ] All linters passing
- [ ] ADR created and reviewed
- [ ] AGENTS.md updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch
- [ ] Optional: Phase 4 enhancements implemented

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Security and observability principles
- [Contributing Guidelines](../contributing/README.md) - General contribution process
- [Error Handling Guide](../contributing/error-handling.md) - How errors should work with secrets
- [Module Organization](../contributing/module-organization.md) - Where to place shared types
- [ADR Template](../decisions/README.md) - How to write ADRs

## 💡 Notes

### Security Considerations

1. **Serialization Security**: `Secret` serializes the actual value (not `[REDACTED]`) because configuration files need real values. This is acceptable since config files should have proper permissions (0600). The `SerializableSecret` marker trait makes this explicit.

2. **Memory Security**: ✅ **Addressed by `secrecy` crate** - Secrets are automatically wiped from memory on drop using the `zeroize` crate. This protects against memory dumps, core dumps, and swap file exposure.

3. **Logging**: All logging/error code must be audited to ensure secrets aren't included in context. The explicit `expose_secret()` method makes this audit easier and more visible in code review.

4. **Test Data**: Test secrets should be clearly fake (e.g., "test-token-12345") to avoid confusion with real secrets.

### Implementation Notes

- Start with Phase 0 (core type) and validate it works before proceeding
- Phase 1 and 2 can be done in parallel if multiple developers are available
- Phase 3 (documentation) should be done immediately after Phase 2
- Phase 4 is optional and can be done as separate PRs later

### Future Improvements

If requirements expand, we can consider:

- Extension traits for convenience methods (`expose_str()`, etc.)
- Debug tracing for secret access patterns (development only)
- Integration with secret management systems (e.g., HashiCorp Vault)
- Constant-time comparison support
- Custom Display trait implementation (currently only Debug is redacted)
- Migration to `secrets` crate if we need mlock/mprotect features

---

**Created**: December 17, 2025  
**Last Updated**: December 17, 2025  
**Status**: 📋 Planning
