# Strengthen Domain Invariant Enforcement

**Issue**: [#281](https://github.com/torrust/torrust-tracker-deployer/issues/281)

## 📋 Overview

This refactoring plan addresses DDD violations where domain types fail to enforce business invariants. Currently, some domain configuration types have public fields and lack validated constructors, allowing the creation of invalid domain objects. This undermines the core DDD principle that domain entities should be "always valid" after construction.

**Target Files:**

- `src/domain/tracker/config/mod.rs` (TrackerConfig)
- `src/domain/tracker/config/udp.rs` (UdpTrackerConfig)
- `src/domain/tracker/config/http.rs` (HttpTrackerConfig)
- `src/domain/tracker/config/http_api.rs` (HttpApiConfig)
- `src/domain/tracker/config/health_check_api.rs` (HealthCheckApiConfig)
- `src/domain/tracker/config/core/mod.rs` (TrackerCoreConfig)
- `src/domain/grafana/config.rs` (GrafanaConfig)
- `src/domain/https/config.rs` (HttpsConfig)
- `src/domain/prometheus/config.rs` (PrometheusConfig)
- `src/application/command_handlers/create/config/tracker/*.rs` (DTOs)

**Scope:**

- Move validation logic from application layer to domain layer
- Replace public fields with private fields and getters
- Add validated constructors (factory methods) to domain types
- Ensure domain invariants are enforced at construction time
- Keep application DTOs as thin mapping layers

## 📊 Progress Tracking

**Total Active Proposals**: 6
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 2
**In Progress**: 0
**Not Started**: 4

### Phase Summary

- **Phase 0 - Foundation (High Impact, Low Effort)**: ✅ 2/2 completed (100%)
- **Phase 1 - Tracker Config Hardening (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
- **Phase 2 - Aggregate Invariants (Medium Impact, Low Effort)**: ⏳ 0/2 completed (0%)

### Discarded Proposals

None at this time.

### Postponed Proposals

None at this time.

## 🎯 Key Problems Identified

### 1. Domain Types with Public Fields

Domain configuration types have public fields that allow direct mutation and bypass validation:

```rust
// Current problematic pattern in domain layer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    pub bind_address: SocketAddr,  // ← Public field, can be set to any value
    pub domain: Option<DomainName>,
}

// Can create invalid objects directly:
let invalid = UdpTrackerConfig {
    bind_address: "0.0.0.0:0".parse().unwrap(), // Port 0 is invalid
    domain: None,
};
```

### 2. Validation Logic in Wrong Layer

Business rules like "port 0 is not supported" are enforced in the application layer DTOs instead of domain:

```rust
// Current: Validation in application layer DTO
impl HttpApiSection {
    pub fn to_http_api_config(&self) -> Result<HttpApiConfig, CreateConfigError> {
        // Port 0 rejection happens HERE in application layer
        if bind_address.port() == 0 {
            return Err(CreateConfigError::DynamicPortNotSupported { ... });
        }
        // Creates domain object without domain-level validation
        Ok(HttpApiConfig { ... })
    }
}
```

**DDD Violation**: Domain types should reject invalid construction, not rely on callers to validate.

### 3. TrackerConfig Validation is Post-Hoc

`TrackerConfig::validate()` exists but is called separately after construction, allowing invalid objects to exist temporarily:

```rust
// Current pattern - object can exist in invalid state
let config = TrackerConfig { ... }; // No validation at construction
config.validate()?;  // Validation happens later, optional call
```

### 4. Cross-Cutting Invariants in Application Layer

The "Grafana requires Prometheus" rule is checked in the application layer:

```rust
// In application layer (environment_config.rs)
if grafana_config.is_some() && prometheus_config.is_none() {
    return Err(CreateConfigError::GrafanaRequiresPrometheus);
}
```

This should be enforced as an aggregate invariant in the domain.

### 5. Inconsistent Constructor Patterns

Compare these two domain types:

```rust
// ✅ Good: EnvironmentName enforces invariants at construction
impl EnvironmentName {
    pub fn new(name: String) -> Result<Self, EnvironmentNameError> {
        Self::validate(&name)?;  // Validates BEFORE creating
        Ok(Self(name))
    }
}

// ❌ Bad: TrackerConfig has no validated constructor
pub struct TrackerConfig {
    pub core: TrackerCoreConfig,  // Public fields
    pub udp_trackers: Vec<UdpTrackerConfig>,
    // ...
}
// Created directly without validation
```

## 🚀 Refactoring Phases

---

## Phase 0: Foundation (Highest Priority)

Establish the pattern for validated domain types that will be applied throughout.

### Proposal #0: Add Validated Constructor Pattern to HttpApiConfig

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None

#### Problem

`HttpApiConfig` has public fields and no validated constructor. Port 0 validation happens in the application DTO:

```rust
// Current domain type - no validation
pub struct HttpApiConfig {
    pub bind_address: SocketAddr,
    pub admin_token: ApiToken,
    pub domain: Option<DomainName>,
    pub use_tls_proxy: bool,
}
```

#### Proposed Solution

Make fields private and add a validated constructor:

```rust
/// HTTP API configuration with domain invariants enforced at construction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiConfig {
    bind_address: SocketAddr,
    admin_token: ApiToken,
    domain: Option<DomainName>,
    use_tls_proxy: bool,
}

/// Errors for HTTP API configuration validation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum HttpApiConfigError {
    #[error("Dynamic port (0) is not supported for bind address '{0}'")]
    DynamicPortNotSupported(SocketAddr),

    #[error("TLS proxy requires a domain to be configured for bind address '{0}'")]
    TlsProxyRequiresDomain(SocketAddr),

    #[error("Localhost '{0}' cannot be used with TLS proxy (Caddy runs in separate container)")]
    LocalhostWithTls(SocketAddr),
}

impl HttpApiConfig {
    /// Creates a new HTTP API configuration with validation
    ///
    /// # Errors
    ///
    /// - `DynamicPortNotSupported` if port is 0
    /// - `TlsProxyRequiresDomain` if use_tls_proxy is true but domain is None
    /// - `LocalhostWithTls` if bind_address is localhost and use_tls_proxy is true
    pub fn new(
        bind_address: SocketAddr,
        admin_token: ApiToken,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> Result<Self, HttpApiConfigError> {
        // Invariant: Port 0 not supported
        if bind_address.port() == 0 {
            return Err(HttpApiConfigError::DynamicPortNotSupported(bind_address));
        }

        // Invariant: TLS requires domain
        if use_tls_proxy && domain.is_none() {
            return Err(HttpApiConfigError::TlsProxyRequiresDomain(bind_address));
        }

        // Invariant: Localhost cannot use TLS (Caddy in separate container)
        if use_tls_proxy && is_localhost(&bind_address) {
            return Err(HttpApiConfigError::LocalhostWithTls(bind_address));
        }

        Ok(Self {
            bind_address,
            admin_token,
            domain,
            use_tls_proxy,
        })
    }

    // Getters...
    pub fn bind_address(&self) -> SocketAddr { self.bind_address }
    pub fn admin_token(&self) -> &ApiToken { &self.admin_token }
    pub fn domain(&self) -> Option<&DomainName> { self.domain.as_ref() }
    pub fn use_tls_proxy(&self) -> bool { self.use_tls_proxy }
}
```

Update application DTO to use `TryFrom` trait for conversion:

> **ADR**: See [TryFrom for DTO to Domain Conversion](../../decisions/tryfrom-for-dto-to-domain-conversion.md) for the rationale.

```rust
use std::convert::TryFrom;

impl TryFrom<HttpApiSection> for HttpApiConfig {
    type Error = CreateConfigError;

    fn try_from(section: HttpApiSection) -> Result<Self, Self::Error> {
        let bind_address = section.bind_address.parse::<SocketAddr>()
            .map_err(|e| CreateConfigError::InvalidBindAddress { ... })?;

        let domain = section.domain
            .map(|d| DomainName::new(&d))
            .transpose()
            .map_err(|e| CreateConfigError::InvalidDomain { ... })?;

        // Delegate all business validation to domain
        HttpApiConfig::new(
            bind_address,
            section.admin_token.into(),
            domain,
            section.use_tls_proxy.unwrap_or(false),
        ).map_err(CreateConfigError::from)
    }
}

// Usage:
// let config: HttpApiConfig = section.try_into()?;
```

#### Rationale

- **Encapsulation**: Private fields prevent invalid state
- **Fail-fast**: Invalid configs rejected at construction, not later
- **Single Source of Truth**: Validation rules live in domain only
- **Cleaner Application Layer**: DTOs just parse and delegate

#### Benefits

- ✅ Domain types are always valid after construction
- ✅ Validation logic centralized in domain layer
- ✅ Application DTOs become simpler (just parsing)
- ✅ Better separation of concerns
- ✅ Impossible to create invalid domain objects

#### Implementation Checklist

- [x] Create `HttpApiConfigError` enum in domain
- [x] Make `HttpApiConfig` fields private
- [x] Add `HttpApiConfig::new()` with validation
- [x] Add getter methods for all fields
- [x] Implement `TryFrom<HttpApiSection> for HttpApiConfig` (replaces `to_http_api_config()` method)
- [x] Add `From<HttpApiConfigError>` impl for `CreateConfigError`
- [x] Update tests in both layers
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Implementation Notes (Completed)

**Key Files Modified:**

- `src/domain/tracker/config/http_api.rs` - Rewritten with validated constructor pattern
- `src/domain/tracker/config/mod.rs` - Added export, updated tests and docs
- `src/application/command_handlers/create/config/tracker/http_api_section.rs` - Simplified to delegate
- `src/application/command_handlers/create/config/errors.rs` - Added `HttpApiConfigInvalid` variant
- Multiple test files updated to use `HttpApiConfig::new()` instead of struct literals

**Key Decisions:**

1. Custom `Deserialize` implementation that validates through `new()` (see http_api.rs)
2. Tests for localhost+TLS validation moved from `TrackerConfig::validate()` to `HttpApiConfig::new()`
3. Added `test_http_api_config` and `test_http_api_config_with_tls` helper functions for tests

**Lessons Learned:**

- When making fields private, ALL code that creates the type must be updated
- Perl one-liners work well for bulk replacement across test files
- Some validation tests become redundant when invariants are enforced at construction

#### Testing Strategy

1. Unit tests for domain type: valid construction, each error case
2. Update application DTO tests to verify delegation
3. Integration tests remain unchanged (behavior preserved)

---

### Proposal #1: Apply Same Pattern to UdpTrackerConfig, HttpTrackerConfig, HealthCheckApiConfig

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P0
**Depends On**: Proposal #0 ✅

#### Problem

These tracker configs have the same issues as `HttpApiConfig`: public fields and no validated constructors.

#### Proposed Solution

Apply the same pattern established in Proposal #0 to each:

1. **UdpTrackerConfig**:
   - Validate port != 0
   - No TLS rules (UDP doesn't support TLS)

2. **HttpTrackerConfig**:
   - Validate port != 0
   - TLS requires domain
   - Localhost cannot use TLS
   - **Note**: Same validation as `HttpApiConfig` but for tracker announce endpoint

3. **HealthCheckApiConfig**:
   - Validate port != 0
   - TLS requires domain
   - Localhost cannot use TLS
   - **Note**: Same structure as `HttpApiConfig`, consider sharing error types or creating a trait

#### Implementation Approach (Based on Proposal #0)

For each config type, follow this exact pattern:

```rust
// 1. Create error enum with help() method
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UdpTrackerConfigError {
    #[error("Dynamic port (0) is not supported for UDP tracker bind address '{0}'")]
    DynamicPortNotSupported(SocketAddr),
}

impl UdpTrackerConfigError {
    #[must_use]
    pub fn help(&self) -> &'static str { ... }
}

// 2. Create raw struct for deserialization
#[derive(Deserialize)]
struct UdpTrackerConfigRaw { ... }

// 3. Make fields private, add validated constructor
impl UdpTrackerConfig {
    pub fn new(bind_address: SocketAddr, domain: Option<DomainName>)
        -> Result<Self, UdpTrackerConfigError> { ... }

    // 4. Add getter methods
    pub fn bind_address(&self) -> SocketAddr { self.bind_address }
    pub fn domain(&self) -> Option<&DomainName> { self.domain.as_ref() }
}

// 5. Implement custom Deserialize
impl<'de> Deserialize<'de> for UdpTrackerConfig { ... }
```

#### Key Differences from HttpApiConfig

| Type                 | Has admin_token | Has use_tls_proxy | Validation Rules     |
| -------------------- | --------------- | ----------------- | -------------------- |
| UdpTrackerConfig     | ❌              | ❌                | Port != 0 only       |
| HttpTrackerConfig    | ❌              | ✅                | Port != 0, TLS rules |
| HealthCheckApiConfig | ❌              | ✅                | Port != 0, TLS rules |
| HttpApiConfig        | ✅              | ✅                | Port != 0, TLS rules |

#### Rationale

Consistent pattern across all tracker configuration types.

#### Benefits

- ✅ All tracker configs enforce invariants
- ✅ Consistent API across config types
- ✅ Application layer simplified uniformly

#### Implementation Checklist

- [x] **UdpTrackerConfig**:
  - [x] Create `UdpTrackerConfigError` enum with `help()` method
  - [x] Create `UdpTrackerConfigRaw` struct for deserialization
  - [x] Make fields private
  - [x] Add `UdpTrackerConfig::new()` with port validation
  - [x] Add getter methods
  - [x] Implement custom `Deserialize`
  - [x] Implement `TryFrom<UdpTrackerSection> for UdpTrackerConfig` (standard trait conversion)
  - [x] Add `From<UdpTrackerConfigError>` for `CreateConfigError`
  - [x] Update tests

- [x] **HttpTrackerConfig**:
  - [x] Create `HttpTrackerConfigError` enum (same variants as HttpApiConfigError minus admin_token)
  - [x] Create `HttpTrackerConfigRaw` struct
  - [x] Make fields private
  - [x] Add `HttpTrackerConfig::new()` with full validation
  - [x] Add getter methods
  - [x] Implement custom `Deserialize`
  - [x] Implement `TryFrom<HttpTrackerSection> for HttpTrackerConfig` (standard trait conversion)
  - [x] Add `From<HttpTrackerConfigError>` for `CreateConfigError`
  - [x] Update tests

- [x] **HealthCheckApiConfig**:
  - [x] Create `HealthCheckApiConfigError` enum
  - [x] Create `HealthCheckApiConfigRaw` struct
  - [x] Make fields private
  - [x] Add `HealthCheckApiConfig::new()` with full validation
  - [x] Add getter methods
  - [x] Implement custom `Deserialize`
  - [x] Implement `TryFrom<HealthCheckApiSection> for HealthCheckApiConfig` (standard trait conversion)
  - [x] Add `From<HealthCheckApiConfigError>` for `CreateConfigError`
  - [x] Update tests

- [x] **Final verification**:
  - [x] Remove redundant localhost+TLS tests from `TrackerConfig::validate()` tests
  - [x] Verify all tests pass
  - [x] Run clippy and fix any issues
  - [x] Run doc tests

#### Estimated Effort

Based on Proposal #0 experience:

- UdpTrackerConfig: ~2 hours (simpler, fewer validation rules)
- HttpTrackerConfig: ~3 hours (TLS validation, more tests)
- HealthCheckApiConfig: ~3 hours (similar to HttpTrackerConfig)
- Total: ~8 hours (1 day)

#### Testing Strategy

1. Unit tests for each domain type: valid construction, each error case
2. Verify error messages and help text
3. Update application DTO tests to verify delegation
4. Check that `TrackerConfig::validate()` still catches aggregate-level issues
5. Integration tests remain unchanged (behavior preserved)

#### Implementation Notes (Completed)

**Key Files Modified:**

- `src/domain/tracker/config/udp.rs` - Rewritten with validated constructor pattern
- `src/domain/tracker/config/http.rs` - Rewritten with validated constructor pattern
- `src/domain/tracker/config/health_check_api.rs` - Rewritten with validated constructor pattern
- `src/domain/tracker/config/mod.rs` - Updated `TrackerConfig::default()` to use `::new()` constructors, updated tests with helper functions, removed redundant localhost+TLS tests
- `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs` - Added `TryFrom` impl
- `src/application/command_handlers/create/config/tracker/http_tracker_section.rs` - Added `TryFrom` impl
- `src/application/command_handlers/create/config/tracker/health_check_api_section.rs` - Added `TryFrom` impl
- `src/application/command_handlers/create/config/tracker/tracker_section.rs` - Updated to use `.try_into()` for conversions
- `src/application/command_handlers/create/config/errors.rs` - Added three new error variants with `From` implementations
- Multiple infrastructure test files updated to use `::new()` constructors

**Key Decisions:**

1. Custom `Deserialize` implementation via serde `deserialize_with` attribute (same pattern as HttpApiConfig)
2. Test helper functions added: `test_udp_tracker_config()`, `test_http_tracker_config()`, `test_health_check_api_config()` (and TLS variants)
3. Localhost+TLS validation tests removed from `TrackerConfig::validate()` tests since these invariants are now enforced at construction time by individual config types
4. All field accesses updated to use getter methods (`.bind_address()`, `.domain()`, `.use_tls_proxy()`)

**Lessons Learned:**

- Bulk replacement of field accesses (`.field` → `.field()`) required updates across many files
- Test code changes were more extensive than production code changes
- The `HasBindAddress` trait implementations needed fully-qualified method calls to avoid ambiguity

---

## Phase 1: Tracker Config Hardening (High Priority)

Strengthen the aggregate root TrackerConfig.

### Proposal #2: TrackerConfig Validates at Construction

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposal #1 ✅

#### Problem

`TrackerConfig` has a `validate()` method that must be called separately:

```rust
let config = TrackerConfig { ... }; // May be invalid
config.validate()?;  // Caller must remember to call this
```

This violates the DDD principle that entities should always be valid.

#### What Validation Remains After Phase 0?

After Proposals #0 and #1, individual component validation (port, TLS rules) is handled at construction time. `TrackerConfig::validate()` will only contain **aggregate-level** checks:

1. **Socket address conflicts**: Same IP:port:protocol used by multiple services
2. **Localhost + TLS for child types**: Already moved to child constructors in Phase 0

**Current `check_localhost_with_tls()` implementation note**:
After Proposal #0, the HTTP API localhost+TLS check became redundant because `HttpApiConfig::new()` already prevents this. Similar cleanup will happen for other types in Proposal #1.

#### Proposed Solution

Move validation into constructor:

```rust
pub struct TrackerConfig {
    core: TrackerCoreConfig,
    udp_trackers: Vec<UdpTrackerConfig>,
    http_trackers: Vec<HttpTrackerConfig>,
    http_api: HttpApiConfig,
    health_check_api: HealthCheckApiConfig,
}

impl TrackerConfig {
    /// Creates a new TrackerConfig with all invariants validated
    ///
    /// # Errors
    ///
    /// - Socket address conflicts (same IP:port:protocol used by multiple services)
    pub fn new(
        core: TrackerCoreConfig,
        udp_trackers: Vec<UdpTrackerConfig>,
        http_trackers: Vec<HttpTrackerConfig>,
        http_api: HttpApiConfig,
        health_check_api: HealthCheckApiConfig,
    ) -> Result<Self, TrackerConfigError> {
        let config = Self {
            core,
            udp_trackers,
            http_trackers,
            http_api,
            health_check_api,
        };

        // Aggregate-level validation only
        // (Child components are already validated at their construction)
        config.check_socket_address_conflicts()?;

        Ok(config)
    }

    fn check_socket_address_conflicts(&self) -> Result<(), TrackerConfigError> {
        // Existing logic from validate() - collects all bindings and checks for duplicates
    }

    // Getters for each field...
}
```

**Note**: After Phase 0 completion, `check_localhost_with_tls()` can be removed entirely since each child config now validates this at construction.

#### Rationale

- **Always Valid**: TrackerConfig is guaranteed valid after construction
- **No Forgotten Validation**: Impossible to forget to validate
- **Simpler API**: No separate validate() call needed

#### Benefits

- ✅ TrackerConfig is always valid after construction
- ✅ Socket conflict validation happens automatically
- ✅ Simpler, more robust API

#### Implementation Checklist

- [ ] Make TrackerConfig fields private
- [ ] Create `TrackerConfig::new()` with aggregate validation
- [ ] Remove public `validate()` method (internalize it)
- [ ] Add getter methods for all fields
- [ ] Implement `TryFrom<TrackerSection> for TrackerConfig` (standard trait conversion)
- [ ] Update all tests
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

---

### Proposal #3: TrackerCoreConfig and DatabaseConfig Validation

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P1
**Depends On**: None

#### Problem

`TrackerCoreConfig` and `DatabaseConfig` have public fields without validation:

```rust
pub struct TrackerCoreConfig {
    pub database: DatabaseConfig,
    pub private: bool,
}

pub enum DatabaseConfig {
    Sqlite(SqliteConfig),
    Mysql(MysqlConfig),
}

pub struct SqliteConfig {
    pub database_name: String,  // Could be empty string
}
```

#### Proposed Solution

Add validation for database configurations:

```rust
impl SqliteConfig {
    pub fn new(database_name: impl Into<String>) -> Result<Self, DatabaseConfigError> {
        let name = database_name.into();
        if name.is_empty() {
            return Err(DatabaseConfigError::EmptyDatabaseName);
        }
        // Could add more validation: must end in .db, valid filename chars, etc.
        Ok(Self { database_name: name })
    }
}

impl MysqlConfig {
    pub fn new(
        database_name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, DatabaseConfigError> {
        let name = database_name.into();
        if name.is_empty() {
            return Err(DatabaseConfigError::EmptyDatabaseName);
        }
        if port == 0 {
            return Err(DatabaseConfigError::InvalidPort(0));
        }
        // ...
        Ok(Self { ... })
    }
}
```

#### Rationale

Database configuration should be validated to prevent runtime errors.

#### Benefits

- ✅ Database configs are always valid
- ✅ Catches configuration errors early
- ✅ Consistent with other validated domain types

#### Implementation Checklist

- [ ] Create `DatabaseConfigError` enum
- [ ] Add `SqliteConfig::new()` with validation
- [ ] Add `MysqlConfig::new()` with validation
- [ ] Make fields private, add getters
- [ ] Update application DTOs
- [ ] Update tests
- [ ] Verify all tests pass

---

## Phase 2: Aggregate Invariants (Medium Priority)

Move cross-cutting validation to domain aggregates.

### Proposal #4: Add Validated Constructor to UserInputs

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P2
**Depends On**: Proposals #2, #3

#### Problem

Cross-cutting invariants like "Grafana requires Prometheus" are checked in the application layer:

```rust
// In application layer (environment_config.rs) - should be in domain
if grafana_config.is_some() && prometheus_config.is_none() {
    return Err(CreateConfigError::GrafanaRequiresPrometheus);
}
```

This validation belongs in the domain because it's a business rule about valid deployment configurations.

#### Design Analysis

**Why not create a new `ServicesConfig` aggregate?**

The original proposal suggested creating a new `ServicesConfig` type to hold `tracker`, `prometheus`, `grafana`, and `https`. However, analysis revealed:

1. **`UserInputs` already exists**: It already holds these four fields together as part of `EnvironmentContext`
2. **`Environment` is the true aggregate**: The `Environment<S>` entity is the aggregate root for deployments
3. **Avoid unnecessary indirection**: Adding `ServicesConfig` would require `UserInputs` to contain it, adding complexity

**The correct DDD pattern being followed:**

```text
Application Layer (DTO)        Domain Layer
─────────────────────────      ──────────────────────────────────
EnvironmentCreationConfig  →   Build partial domain objects:
                               - TrackerConfig (validates its invariants)
                               - PrometheusConfig
                               - GrafanaConfig
                               - HttpsConfig
                                        ↓
                               Pass to UserInputs::new()
                                        ↓
                               UserInputs validates cross-cutting invariants
                                        ↓
                               Environment::with_working_dir_and_tracker()
                               creates the complete aggregate
```

**Why validate in `UserInputs::new()` vs `Environment` constructor?**

- `UserInputs` is the semantic container for these related configurations
- `Environment` is already complex (state machine + context)
- Keeps validation close to the data it protects
- Single responsibility: `UserInputs` validates user-provided configuration coherence

#### Proposed Solution

Add a validated constructor to `UserInputs` that enforces cross-service invariants:

```rust
/// Errors for user inputs validation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UserInputsError {
    #[error("Grafana requires Prometheus to be configured as its data source")]
    GrafanaRequiresPrometheus,

    #[error("HTTPS section is defined but no service has TLS configured")]
    HttpsSectionWithoutTlsServices,

    #[error("At least one service has TLS configured but HTTPS section is missing")]
    TlsServicesWithoutHttpsSection,
}

impl UserInputsError {
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::GrafanaRequiresPrometheus => {
                "Add a 'prometheus' section to your configuration, or remove the 'grafana' section."
            }
            Self::HttpsSectionWithoutTlsServices => {
                "Either remove the 'https' section, or set 'use_tls_proxy: true' on at least one service."
            }
            Self::TlsServicesWithoutHttpsSection => {
                "Add an 'https' section with 'admin_email' for Let's Encrypt certificate management."
            }
        }
    }
}

impl UserInputs {
    /// Creates a new UserInputs with cross-service invariant validation
    ///
    /// # Errors
    ///
    /// - `GrafanaRequiresPrometheus` if Grafana is configured without Prometheus
    /// - `HttpsSectionWithoutTlsServices` if HTTPS section exists but no service uses TLS
    /// - `TlsServicesWithoutHttpsSection` if a service uses TLS but HTTPS section is missing
    pub fn new(
        name: EnvironmentName,
        instance_name: InstanceName,
        provider_config: ProviderConfig,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        tracker: TrackerConfig,
        prometheus: Option<PrometheusConfig>,
        grafana: Option<GrafanaConfig>,
        https: Option<HttpsConfig>,
    ) -> Result<Self, UserInputsError> {
        // Cross-service invariant: Grafana requires Prometheus as data source
        if grafana.is_some() && prometheus.is_none() {
            return Err(UserInputsError::GrafanaRequiresPrometheus);
        }

        // Cross-service invariant: HTTPS section requires at least one TLS service
        let has_tls = tracker.has_any_tls_configured();
        if https.is_some() && !has_tls {
            return Err(UserInputsError::HttpsSectionWithoutTlsServices);
        }

        // Inverse: TLS services require HTTPS section
        if has_tls && https.is_none() {
            return Err(UserInputsError::TlsServicesWithoutHttpsSection);
        }

        Ok(Self {
            name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker,
            prometheus,
            grafana,
            https,
        })
    }
}
```

Then update `EnvironmentContext` to use the validated constructor:

```rust
impl EnvironmentContext {
    pub fn with_working_dir_and_tracker(
        // ... params ...
    ) -> Result<Self, UserInputsError> {
        let user_inputs = UserInputs::new(
            name,
            instance_name,
            provider_config,
            ssh_credentials,
            ssh_port,
            tracker,
            prometheus,
            grafana,
            https,
        )?;

        Ok(Self {
            created_at,
            user_inputs,
            internal_config,
            runtime_outputs,
        })
    }
}
```

#### Rationale

- **Existing structure**: `UserInputs` already groups these fields - no new type needed
- **Semantic fit**: These are all "user inputs" that must be coherent together
- **Layered validation**: Individual configs validate themselves, `UserInputs` validates cross-cutting rules
- **Simpler than alternative**: Avoids creating an intermediate `ServicesConfig` type

#### Benefits

- ✅ Cross-service invariants moved to domain layer
- ✅ Application layer becomes thinner (removes validation logic)
- ✅ No new types needed - uses existing `UserInputs`
- ✅ Clear separation: individual validation vs. cross-cutting validation
- ✅ Consistent with DDD aggregate pattern

#### Implementation Checklist

- [ ] Create `UserInputsError` enum with `help()` method in `user_inputs.rs`
- [ ] Change `UserInputs::new()` to return `Result<Self, UserInputsError>`
- [ ] Add cross-service validation to `UserInputs::new()`
- [ ] Update `EnvironmentContext::new()` and `with_working_dir_and_tracker()` to propagate errors
- [ ] Update `Environment::new()` and `with_working_dir_and_tracker()` to return `Result`
- [ ] Add `From<UserInputsError>` impl for `CreateConfigError` in application layer
- [ ] Remove duplicate validation from `EnvironmentCreationConfig::to_environment_params()`
- [ ] Update all call sites that create `UserInputs` directly
- [ ] Update tests
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Migration Notes

This change makes `Environment::with_working_dir_and_tracker()` fallible, which is a breaking change. However:

1. The function is primarily called from `CreateCommandHandler::execute()` which already handles errors
2. Test helpers may need updating to use `.expect()` or propagate errors
3. The change aligns with the "always valid" principle - aggregates should validate at construction

---

### Proposal #5: Move HTTPS Validation to Domain

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P2
**Depends On**: Proposal #4

#### Problem

`HttpsConfig` doesn't validate the email at construction:

```rust
// Current: HttpsConfig accepts any string
impl HttpsConfig {
    pub fn new(admin_email: impl Into<String>, use_staging: bool) -> Self {
        Self {
            admin_email: admin_email.into(),  // No validation!
            use_staging,
        }
    }
}
```

Email validation happens in the application layer.

#### Proposed Solution

Add email validation to `HttpsConfig`:

```rust
impl HttpsConfig {
    pub fn new(
        admin_email: &Email,  // Already validated Email type
        use_staging: bool,
    ) -> Self {
        Self {
            admin_email: admin_email.to_string(),
            use_staging,
        }
    }

    // Alternative: validate string directly
    pub fn from_string(
        admin_email: impl Into<String>,
        use_staging: bool,
    ) -> Result<Self, HttpsConfigError> {
        let email_str = admin_email.into();
        let _ = Email::new(&email_str)
            .map_err(|_| HttpsConfigError::InvalidEmail(email_str.clone()))?;
        Ok(Self {
            admin_email: email_str,
            use_staging,
        })
    }
}
```

#### Rationale

If domain stores an email, it should ensure it's valid.

#### Benefits

- ✅ HttpsConfig is always valid
- ✅ Email validation in domain
- ✅ Consistent with other validated types

#### Implementation Checklist

- [ ] Add email validation to `HttpsConfig::new()` or create `from_validated_email`
- [ ] Create `HttpsConfigError` if needed
- [ ] Update application layer
- [ ] Update tests
- [ ] Verify all tests pass

---

## 📈 Timeline

**Based on Proposal #0 implementation experience:**

- **Start Date**: January 21, 2026
- **Estimated Total Duration**: 3-4 days of focused work

### Phase Breakdown

| Phase   | Proposals | Estimated Time | Notes                                      |
| ------- | --------- | -------------- | ------------------------------------------ |
| Phase 0 | #0, #1    | ~1.5 days      | #0 complete, #1 similar pattern            |
| Phase 1 | #2, #3    | ~1 day         | Aggregate validation simpler after Phase 0 |
| Phase 2 | #4, #5    | ~1 day         | Cross-service invariants                   |

### Actual Time Spent

| Proposal                 | Estimated | Actual   | Notes                                     |
| ------------------------ | --------- | -------- | ----------------------------------------- |
| #0 HttpApiConfig         | 3-4 hours | ~4 hours | First implementation, learning curve      |
| #1 Other tracker configs | 8 hours   | TBD      | Should be faster with established pattern |

### Key Time Sinks (Lessons Learned)

1. **Test updates**: 40% of time spent updating test fixtures
2. **Finding all usages**: IDE grep + cargo check cycle
3. **Doc example updates**: Often forgotten, caught by doc tests
4. **Redundant test removal**: Identifying which tests are now obsolete

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals reviewed by team
- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Aligns with [DDD Layer Placement Guide](../../contributing/ddd-layer-placement.md)
- [ ] Implementation plan is clear and actionable

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../../development-principles.md)
- [DDD Layer Placement Guide](../../contributing/ddd-layer-placement.md)
- [DDD Practices Guide](../../contributing/ddd-practices.md) - Domain patterns including validated deserialization
- [Error Handling Guide](../../contributing/error-handling.md)
- [Testing Conventions](../../contributing/testing/)
- [Config DTOs README](../../../src/application/command_handlers/create/config/README.md)
- [ADR: Validated Deserialization for Domain Types](../../decisions/validated-deserialization-for-domain-types.md)
- [ADR: TryFrom for DTO to Domain Conversion](../../decisions/tryfrom-for-dto-to-domain-conversion.md)

## 💡 Notes

### Breaking Changes

This refactoring will cause breaking changes in:

- Domain type constructors (new signature)
- Application DTO conversion methods (delegate to domain)

All changes are internal; JSON configuration format remains unchanged.

### Backward Compatibility for Deserialization

The `#[derive(Deserialize)]` on domain types currently works for JSON persistence. After making fields private, we need custom deserialization that calls `new()`.

> **📖 Full Documentation**: See the [Validated Deserialization ADR](../../decisions/validated-deserialization-for-domain-types.md) for the complete decision record, and the [DDD Practices Guide](../../contributing/ddd-practices.md) for implementation patterns.

**Implemented Pattern** (from Proposal #0):

```rust
/// Internal struct for serde deserialization that bypasses validation
#[derive(Deserialize)]
struct HttpApiConfigRaw {
    #[serde(deserialize_with = "deserialize_socket_addr")]
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

**Why this pattern works:**

1. Deserialization parses JSON into raw struct (no validation)
2. `new()` is called with raw values, enforcing all invariants
3. Error is converted to serde error for proper reporting
4. `Serialize` derive can remain unchanged (just outputs fields)

---

## 📖 Lessons Learned from Proposal #0

This section documents insights from implementing the `HttpApiConfig` validated constructor pattern. Use this guidance for subsequent proposals.

### High-Impact Changes

**Making fields private has cascading effects:**

- Every place that creates the type with struct literal syntax must be updated
- Test files are the biggest source of changes (many test fixtures)
- Doc comments with code examples need updating too
- The `Default` implementation must use `new().expect()` pattern

**Estimated file changes per type:**

- Domain file: Complete rewrite (~600 lines for HttpApiConfig)
- Application DTO: Minor changes (~20 lines)
- Tests in domain: Major updates (many fixtures)
- Tests across codebase: Moderate updates (10-20 files typically)

### Custom Deserialize Pattern Details

**The Raw struct approach is essential because:**

1. Derived `Deserialize` would require public fields or `#[serde(default)]` hacks
2. It allows using the same field names and structure as JSON
3. The deserialize_with helper can handle type conversions (e.g., string to SocketAddr)
4. Error messages from `new()` propagate correctly through serde

**Important implementation detail:**

```rust
// Use #[serde(default)] for optional fields in the Raw struct
#[serde(default)]
domain: Option<DomainName>,
```

### Test Helper Functions

**Create helper functions in test modules:**

```rust
fn test_http_api_config(bind_address: &str, admin_token: &str) -> HttpApiConfig {
    HttpApiConfig::new(
        bind_address.parse().expect("valid address"),
        admin_token.to_string().into(),
        None,
        false,
    )
    .expect("test values should be valid")
}

fn test_http_api_config_with_tls(
    bind_address: &str,
    admin_token: &str,
    domain: Option<DomainName>,
    use_tls_proxy: bool,
) -> HttpApiConfig {
    HttpApiConfig::new(
        bind_address.parse().expect("valid address"),
        admin_token.to_string().into(),
        domain,
        use_tls_proxy,
    )
    .expect("test values should be valid")
}
```

**Benefits:**

- Tests remain readable
- Easy to create configs with different parameter combinations
- Changes to constructor signature only need updating in one place

### Validation Tests Become Redundant

**When invariants move to construction time:**

Some tests that previously validated errors from `TrackerConfig::validate()` became redundant because the invalid state can no longer be constructed.

**Example:** Tests for "HTTP API localhost + TLS" rejection were removed from `TrackerConfig` tests because `HttpApiConfig::new()` now prevents this at construction time.

**Action:** Remove redundant tests and add a comment explaining why:

```rust
// NOTE: Tests for HTTP API localhost + TLS rejection have been removed because
// this validation is now enforced at construction time by HttpApiConfig::new().
// See http_api.rs for the corresponding tests.
```

### Bulk Replacement Strategy

For large-scale test updates, use perl one-liners:

```bash
# Replace struct literals with constructor calls across all test files
perl -i -0pe 's/http_api: HttpApiConfig \{\s+bind_address: "([^"]+)"\.parse\(\)\.unwrap\(\),\s+admin_token: "([^"]+)"\.to_string\(\)\.into\(\),\s+domain: None,\s+use_tls_proxy: false,\s+\}/http_api: test_http_api_config("$1", "$2")/g' src/domain/tracker/config/mod.rs
```

**Limitations:**

- Pattern must be exact (whitespace sensitive)
- Won't catch struct literals with different field orders
- Review changes manually after bulk replacement

### Error Type Design

**Include help() method for all domain errors:**

```rust
impl HttpApiConfigError {
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::DynamicPortNotSupported(_) => {
                "Dynamic port assignment (port 0) is not supported.\n\
                 \n\
                 Why: Port 0 tells the operating system to assign a random available port.\n\
                 This is not suitable for deployment where ports must be known in advance.\n\
                 \n\
                 Fix: Specify an explicit port number (e.g., 1212, 8080, 3000)."
            }
            // ... other variants
        }
    }
}
```

**Error message structure:**

1. Brief message in `Display` impl (what went wrong)
2. Detailed help in `help()` method (why + how to fix)

### Application Layer Error Conversion

**Use From trait for clean error conversion:**

```rust
// In CreateConfigError enum
#[error("Invalid HTTP API configuration")]
HttpApiConfigInvalid(#[from] HttpApiConfigError),

// With help delegation
impl CreateConfigError {
    pub fn help(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::HttpApiConfigInvalid(e) => Some(Cow::Borrowed(e.help())),
            // ... other variants
        }
    }
}
```

**Benefits:**

- `?` operator works naturally: `HttpApiConfig::new(...)?.into()`
- Help messages propagate from domain to application
- Single source of truth for error guidance

### Files Changed Summary (Proposal #0)

For reference, here are all files modified for `HttpApiConfig`:

| File                                                                               | Changes                                     |
| ---------------------------------------------------------------------------------- | ------------------------------------------- |
| `src/domain/tracker/config/http_api.rs`                                            | Complete rewrite with validated constructor |
| `src/domain/tracker/config/mod.rs`                                                 | Exports, tests, doc examples, Default impl  |
| `src/domain/tracker/mod.rs`                                                        | Exports, doc example                        |
| `src/application/command_handlers/create/config/errors.rs`                         | Added error variant                         |
| `src/application/command_handlers/create/config/tracker/http_api_section.rs`       | Simplified delegation                       |
| `src/application/command_handlers/create/config/tracker/tracker_section.rs`        | Test updates                                |
| `src/application/command_handlers/show/info/tracker.rs`                            | Getter method calls                         |
| `src/application/command_handlers/test/handler.rs`                                 | Getter method calls                         |
| `src/application/steps/rendering/docker_compose_templates.rs`                      | Getter method calls                         |
| `src/domain/environment/context.rs`                                                | Getter method calls                         |
| `src/domain/environment/runtime_outputs.rs`                                        | Getter method calls                         |
| `src/infrastructure/templating/ansible/template/wrappers/variables/context.rs`     | Constructor + tests                         |
| `src/infrastructure/templating/prometheus/template/renderer/project_generator.rs`  | Constructor + tests                         |
| `src/infrastructure/templating/tracker/template/renderer/project_generator.rs`     | Constructor in tests                        |
| `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs` | Doc example, getter, tests                  |

Total: 15 files modified

### Order of Implementation

The proposals should be implemented in order due to dependencies:

1. Phase 0 first (individual configs)
2. Phase 1 second (aggregate TrackerConfig)
3. Phase 2 last (cross-service aggregate)

---

**Created**: January 21, 2026
**Last Updated**: January 21, 2026
**Status**: 🚧 In Progress (Phase 0)
