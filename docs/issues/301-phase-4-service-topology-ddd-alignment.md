# [Refactor] Phase 4: Service Topology DDD Layer Alignment

**Issue**: [#301](https://github.com/torrust/torrust-tracker-deployer/issues/301)
**Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Related Plan**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)
**Predecessor**: [#300](https://github.com/torrust/torrust-tracker-deployer/issues/300) - Phase 3 Port Topology Template Integration

## Overview

Move port derivation and network computation logic from the infrastructure layer to the domain layer, ensuring proper DDD layer separation. This phase was identified during Phase 3 implementation when we discovered business rules incorrectly placed in infrastructure.

## Problem Statement

The current architecture has domain logic (port derivation, network computation) incorrectly placed in the infrastructure layer:

```text
infrastructure/context/port_derivation.rs  ← Business rules about port exposure
infrastructure/context/tracker.rs          ← compute_networks() method
infrastructure/context/grafana.rs          ← compute_networks() method
infrastructure/context/prometheus.rs       ← compute_networks() method
```

These are business rules that should be in the domain layer:

- "UDP ports are always exposed (no TLS for UDP)" - PORT-02
- "HTTP ports hidden when TLS enabled" - PORT-03, PORT-04
- "Tracker joins metrics_network when Prometheus is enabled"

## Goals

- [x] Move port derivation logic to domain layer using `PortDerivation` trait
- [x] Move network computation logic to domain layer using `NetworkDerivation` trait
- [x] Convert infrastructure context types to pure DTOs (use domain traits, no `compute_*()` methods)
- [x] Maintain all existing functionality and E2E tests passing

## 🏗️ Architecture Requirements

**DDD Layer**: Domain (for business logic) + Infrastructure (for DTOs)
**Module Paths**:

- `src/domain/topology/traits.rs` - `PortDerivation`, `NetworkDerivation` traits
- `src/domain/topology/enabled_services.rs` - `EnabledServices` topology context
- `src/domain/topology/fixed_ports.rs` - Caddy/MySQL port functions

**Pattern**: Trait-based port and network derivation + `EnabledServices` for topology context

### Design Principles Applied

1. **Open/Closed Principle**: Port derivation is local to each service config. Adding a new service doesn't require modifying existing services.

2. **DDD Layer Separation**:
   - **Domain**: Business rules, invariants, rich objects
   - **Infrastructure**: DTOs, template rendering, format conversion

3. **Two Levels of Logic**:
   - **Service-Local**: Can be computed from service's own configuration (ports)
   - **Topology-Level**: Requires knowledge of other services (networks)

4. **Trait-Based Extensibility**: Services implement a trait to participate in topology, making it easy to add new services in the future.

### Layer Diagram

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                              DOMAIN LAYER                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ SERVICE CONFIGS (Level 1: Local Port Derivation)                    │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                     │    │
│  │  domain/tracker/config.rs                                           │    │
│  │    impl PortDerivation for TrackerConfig {                          │    │
│  │        fn derive_ports(&self) -> Vec<PortBinding>                   │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  │  domain/grafana/config.rs                                           │    │
│  │    impl PortDerivation for GrafanaConfig {                          │    │
│  │        fn derive_ports(&self) -> Vec<PortBinding>                   │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  │  domain/prometheus/config.rs                                        │    │
│  │    impl PortDerivation for PrometheusConfig {                       │    │
│  │        fn derive_ports(&self) -> Vec<PortBinding>                   │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ TOPOLOGY MODULE (Level 2: Network Composition)                      │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                     │    │
│  │  domain/topology/traits.rs (NEW)                                    │    │
│  │    pub trait PortDerivation {                                       │    │
│  │        fn derive_ports(&self) -> Vec<PortBinding>;                  │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  │  domain/topology/builder.rs (NEW)                                   │    │
│  │    pub struct DockerComposeTopologyBuilder {                        │    │
│  │        // Knows which services are enabled                          │    │
│  │        // Computes networks based on inter-service dependencies     │    │
│  │        // Creates ServiceTopology with correct networks             │    │
│  │        // Uses trait to get ports from each config                  │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  │  domain/topology/aggregate.rs (existing)                            │    │
│  │    pub struct DockerComposeTopology {                               │    │
│  │        // Validates cross-service invariants (port conflicts)       │    │
│  │        // Derives required_networks from all services               │    │
│  │        // Always-valid aggregate                                    │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                          INFRASTRUCTURE LAYER                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ TEMPLATE CONTEXT (Pure DTOs - No Business Logic)                    │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                     │    │
│  │  context/tracker.rs                                                 │    │
│  │    pub struct TrackerServiceContext {                               │    │
│  │        // Template-friendly fields only                             │    │
│  │        // NO compute_networks(), NO derive_ports()                  │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  │  context/builder.rs                                                 │    │
│  │    pub struct DockerComposeContextBuilder {                         │    │
│  │        // Receives DockerComposeTopology from domain                │    │
│  │        // Converts ServiceTopology → ServiceContext DTOs            │    │
│  │        // Converts PortBinding → PortDefinition                     │    │
│  │        // Adds template-specific formatting                         │    │
│  │    }                                                                │    │
│  │                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```text
                         DOMAIN LAYER
                              │
    ┌─────────────────────────┼─────────────────────────┐
    │                         │                         │
    ▼                         ▼                         ▼
TrackerConfig           GrafanaConfig            PrometheusConfig
    │                         │                         │
    │ derive_ports()          │ derive_ports()          │ derive_ports()
    │                         │                         │
    ▼                         ▼                         ▼
Vec<PortBinding>        Vec<PortBinding>         Vec<PortBinding>
    │                         │                         │
    └─────────────────────────┼─────────────────────────┘
                              │
                              ▼
              DockerComposeTopologyBuilder
                              │
                              │ compute_networks() for each service
                              │ (uses knowledge of which services exist)
                              │
                              ▼
                    DockerComposeTopology
                    (validated aggregate)
                              │
    ──────────────────────────┼────────────────────────────
                              │
                    INFRASTRUCTURE LAYER
                              │
                              ▼
                DockerComposeContextBuilder
                              │
                              │ Convert to DTOs
                              │ Format for templates
                              │
                              ▼
                    DockerComposeContext
                    (template-ready DTO)
```

## Specifications

### Port Derivation Trait

**Location**: `src/domain/topology/traits.rs`

```rust
use super::PortBinding;

/// Trait for services that can derive their port bindings
///
/// This trait enables the Open/Closed principle: each service
/// encapsulates its own port derivation logic without requiring
/// changes to other services or the topology builder.
pub trait PortDerivation {
    /// Derives port bindings based on service configuration
    ///
    /// The implementation should apply all PORT-* rules relevant
    /// to this service (e.g., hiding ports when TLS is enabled).
    fn derive_ports(&self) -> Vec<PortBinding>;
}
```

### Config Implementations

Each service config implements the trait locally:

```rust
// domain/tracker/config.rs
impl PortDerivation for TrackerConfig {
    fn derive_ports(&self) -> Vec<PortBinding> {
        let mut ports = Vec::new();

        // PORT-02: UDP ports always exposed
        for udp_config in &self.udp_trackers {
            ports.push(PortBinding::udp(
                udp_config.binding_address().port(),
                "BitTorrent UDP announce"
            ));
        }

        // PORT-03/04: HTTP ports only if no TLS
        for http_config in &self.http_trackers {
            if !http_config.use_tls_proxy() {
                ports.push(PortBinding::tcp(
                    http_config.binding_address().port(),
                    "HTTP tracker announce"
                ));
            }
        }

        // PORT-05/06: API only if no TLS
        if !self.http_api.use_tls_proxy() {
            ports.push(PortBinding::tcp(
                self.http_api.binding_address().port(),
                "HTTP API (stats/whitelist)"
            ));
        }

        ports
    }
}
```

### Fixed Port Services

Services without configuration (Caddy, MySQL) use free functions:

**Location**: `src/domain/topology/fixed_ports.rs`

```rust
/// PORT-09: Caddy always exposes 80, 443, 443/udp
pub fn caddy_ports() -> Vec<PortBinding> {
    vec![
        PortBinding::tcp(80, "HTTP (ACME HTTP-01 challenge)"),
        PortBinding::tcp(443, "HTTPS"),
        PortBinding::udp(443, "HTTP/3 (QUIC)"),
    ]
}

/// PORT-11: MySQL has no exposed ports
pub fn mysql_ports() -> Vec<PortBinding> {
    vec![]
}
```

### Infrastructure Context (Pure DTO)

**Location**: `src/infrastructure/.../context/tracker.rs`

```rust
/// Tracker service context for Docker Compose template
///
/// This is a pure DTO for template rendering. All business logic
/// (port derivation, network computation) happens in the domain layer.
#[derive(Serialize, Debug, Clone)]
pub struct TrackerServiceContext {
    /// Port bindings for Docker Compose (from domain)
    pub ports: Vec<PortDefinition>,
    /// Networks (from domain topology)
    pub networks: Vec<NetworkDefinition>,
}

impl TrackerServiceContext {
    /// Creates context from domain topology
    pub fn from_topology(topology: &ServiceTopology) -> Self {
        Self {
            ports: topology.ports().iter().map(PortDefinition::from).collect(),
            networks: topology.networks().iter().map(NetworkDefinition::from).collect(),
        }
    }
}
```

## Implementation Plan

> **Approach**: Single PR with incremental commits. Each step is a logical commit point.
> Progress tracked with checkboxes below.

### Step 1: Create PortDerivation Trait Foundation

- [x] 1.1 Create `src/domain/topology/traits.rs` with `PortDerivation` trait
- [x] 1.2 Export trait from `src/domain/topology/mod.rs`

### Step 2: Implement PortDerivation for Prometheus (Simplest)

- [x] 2.1 Implement `PortDerivation` for `PrometheusConfig` in domain
- [x] 2.2 Add unit tests for Prometheus port derivation
- [x] 2.3 Update infrastructure `PrometheusServiceConfig` to use domain trait
- [x] 2.4 Remove `derive_prometheus_ports()` calls from infrastructure

### Step 3: Implement PortDerivation for Grafana

- [x] 3.1 Implement `PortDerivation` for `GrafanaConfig` in domain
- [x] 3.2 Add unit tests for Grafana port derivation
- [x] 3.3 Update infrastructure `GrafanaServiceConfig` to use domain trait
- [x] 3.4 Remove `derive_grafana_ports()` calls from infrastructure

### Step 4: Implement PortDerivation for Tracker (Most Complex)

- [x] 4.1 Implement `PortDerivation` for `TrackerConfig` in domain
- [x] 4.2 Add unit tests for Tracker port derivation
- [x] 4.3 Update infrastructure `TrackerServiceConfig` to use domain trait
- [x] 4.4 Remove `derive_tracker_ports()` calls from infrastructure
- [x] 4.5 Remove `TrackerServiceConfig::new()` - all callers migrated to `from_domain_config()`
  - Application layer (`docker_compose_templates.rs`) now uses domain `TrackerConfig` directly
  - All test code updated with domain config helper functions
  - Deleted `port_derivation.rs` entirely

### Step 5: Fixed Port Services (Caddy, MySQL)

- [x] 5.1 ~~Create `src/domain/topology/fixed_ports.rs` with `caddy_ports()` and `mysql_ports()`~~ - **Replaced**: Created proper domain types instead
- [x] 5.2 ~~Add unit tests for fixed port functions~~ - **Replaced**: Domain types have their own unit tests
- [x] 5.3 Update infrastructure to use domain fixed port functions
- [x] 5.4 Remove `derive_caddy_ports()` and `derive_mysql_ports()` from infrastructure
- [x] 5.5 Create `src/domain/caddy/config.rs` with `CaddyConfig` implementing `PortDerivation` and `NetworkDerivation`
- [x] 5.6 Create `src/domain/mysql/config.rs` with `MysqlServiceConfig` implementing `PortDerivation` and `NetworkDerivation`
- [x] 5.7 Delete `src/domain/topology/fixed_ports.rs` - no longer needed
- [x] 5.8 Update infrastructure `CaddyDockerServiceConfig` and `MysqlDockerServiceConfig` to use `from_domain_config()`

**Rationale for change**: Even though Caddy and MySQL have fixed port/network behavior, they should follow the same trait-based patterns as other services for consistency and Open/Closed compliance.

### Step 6: Network Computation - Domain Topology Builder

- [x] 6.1 Create `NetworkDerivation` trait for network assignment logic
- [x] 6.2 Implement `NetworkDerivation` for `TrackerConfig`
- [x] 6.3 Implement `NetworkDerivation` for `PrometheusConfig`
- [x] 6.4 Implement `NetworkDerivation` for `GrafanaConfig`
- [x] 6.5 Create `EnabledServices` abstraction (renamed from `TopologyContext`)
  - Uses `HashSet<Service>` for Open/Closed compliance
  - Provides only `has(Service)` method - no convenience methods
- [x] 6.6 Add unit tests for `EnabledServices` (10 tests covering constructor, has method, Default, Clone)
- [x] 6.7 ~~Create `DockerComposeTopologyBuilder`~~ - **Not needed**: Caddy/MySQL have static networks (NET-08, NET-09), infrastructure builder handles collection
- [x] 6.8 ~~Move Caddy network computation~~ - **Not needed**: Caddy always connects to Proxy network only (no conditional logic)
- [x] 6.9 ~~Move MySQL network computation~~ - **Not needed**: MySQL always connects to Database network only (no conditional logic)
- [x] 6.10 ~~Add builder unit tests~~ - **Not needed**: Existing infrastructure tests cover network derivation

### Step 7: Refactor Infrastructure to Pure DTOs

- [x] 7.1 Remove `compute_networks()` from `TrackerServiceConfig` - **Done**: Uses `NetworkDerivation` trait
- [x] 7.2 Remove `compute_networks()` from `PrometheusServiceConfig` - **Done**: Uses `NetworkDerivation` trait
- [x] 7.3 Remove `compute_networks()` from `GrafanaServiceConfig` - **Done**: Uses `NetworkDerivation` trait
- [x] 7.4 Update `DockerComposeContextBuilder` to use domain traits - **Done**: Passes `EnabledServices` to `from_domain_config`
- [x] 7.5 Rename infrastructure service config types to `*Context` - **Done**
  - **What**: Renamed infrastructure DTOs to better reflect their purpose as template contexts
  - **Renamed**: `TrackerServiceConfig` → `TrackerServiceContext`, `GrafanaServiceConfig` → `GrafanaServiceContext`, `PrometheusServiceConfig` → `PrometheusServiceContext`, `CaddyDockerServiceConfig` → `CaddyServiceContext`, `MysqlDockerServiceConfig` → `MysqlServiceContext`
  - **Benefit**: Clear naming distinction between domain configs (`*Config`) and infrastructure contexts (`*Context`)
  - **No backward compatibility aliases**: Clean break for readability (project not in production yet)

### Step 8: Cleanup and Verification

- [x] 8.1 Delete `src/infrastructure/.../context/port_derivation.rs`
- [x] 8.2 Remove unused imports and dead code
- [x] 8.3 Run full E2E test suite
- [x] 8.4 Run pre-commit checks: `./scripts/pre-commit.sh`

## Files Changed

### New Files

| File                                      | Purpose                                                         |
| ----------------------------------------- | --------------------------------------------------------------- |
| `src/domain/topology/traits.rs`           | `PortDerivation`, `NetworkDerivation` traits                    |
| `src/domain/topology/enabled_services.rs` | `EnabledServices` set for topology queries                      |
| `src/domain/caddy/config.rs`              | `CaddyConfig` with `PortDerivation`, `NetworkDerivation`        |
| `src/domain/mysql/config.rs`              | `MysqlServiceConfig` with `PortDerivation`, `NetworkDerivation` |

### Modified Files

| File                                           | Change                                                              |
| ---------------------------------------------- | ------------------------------------------------------------------- |
| `src/domain/mod.rs`                            | Export new `caddy` and `mysql` modules                              |
| `src/domain/topology/mod.rs`                   | Export new modules                                                  |
| `src/domain/tracker/config.rs`                 | Implement `PortDerivation`, `NetworkDerivation`                     |
| `src/domain/grafana/config.rs`                 | Implement `PortDerivation`, `NetworkDerivation`                     |
| `src/domain/prometheus/config.rs`              | Implement `PortDerivation`, `NetworkDerivation`                     |
| `src/infrastructure/.../context/caddy.rs`      | Renamed to `CaddyServiceContext`, use `from_domain_config()`        |
| `src/infrastructure/.../context/mysql.rs`      | Renamed to `MysqlServiceContext`, use `from_domain_config()`        |
| `src/infrastructure/.../context/tracker.rs`    | Renamed to `TrackerServiceContext`, removed `compute_networks()`    |
| `src/infrastructure/.../context/grafana.rs`    | Renamed to `GrafanaServiceContext`, removed `compute_networks()`    |
| `src/infrastructure/.../context/prometheus.rs` | Renamed to `PrometheusServiceContext`, removed `compute_networks()` |
| `src/infrastructure/.../context/builder.rs`    | Receive domain topology, use new `*ServiceContext` types            |

### Deleted Files

| File                                                | Reason                                        |
| --------------------------------------------------- | --------------------------------------------- |
| `src/infrastructure/.../context/port_derivation.rs` | Logic moved to domain                         |
| `src/domain/topology/fixed_ports.rs`                | Replaced by proper domain types (Caddy/MySQL) |

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [x] `PortDerivation` trait defined in `domain/topology/traits.rs`
- [x] All service configs (`TrackerConfig`, `GrafanaConfig`, `PrometheusConfig`, `CaddyConfig`, `MysqlServiceConfig`) implement `PortDerivation`
- [x] `NetworkDerivation` trait defined in `domain/topology/traits.rs` for network computation
- [x] Infrastructure context types are pure DTOs with no `compute_*()` methods
- [x] `port_derivation.rs` deleted from infrastructure
- [x] All existing E2E tests pass (2060 tests)
- [x] Unit tests cover port derivation for each service
- [x] Unit tests cover network derivation for `EnabledServices`

## Design Decisions (Resolved)

| Question                                                                          | Decision                                               | Rationale                                                                                                                           |
| --------------------------------------------------------------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| Should `PortDerivation` trait be in `domain/topology/` or a shared traits module? | `domain/topology/traits.rs`                            | The trait exists for topology purposes. Consumer (builder) defines it, implementers import it. Keeps topology concerns cohesive.    |
| Should we rename infrastructure context types to `*Context` now or defer?         | Done - renamed to `*ServiceContext`                    | Clearer naming: domain uses `*Config`, infrastructure uses `*ServiceContext`. No backward compatibility aliases for readability.    |
| Should fixed-port services (Caddy, MySQL) use `fixed_ports.rs` or domain types?   | Domain types with `PortDerivation`/`NetworkDerivation` | Consistency with other services. All services follow same trait-based pattern, even if behavior is static. Open/Closed compliance.  |
| Should we create `DockerComposeTopologyBuilder` for network computation?          | Not needed                                             | Caddy/MySQL have static networks (NET-08, NET-09). Trait-based approach with `NetworkDerivation` + `EnabledServices` is sufficient. |

## Related Documentation

- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Docker Compose Topology Domain Model Plan](../refactors/plans/docker-compose-topology-domain-model.md)
- [Epic #287](https://github.com/torrust/torrust-tracker-deployer/issues/287)
