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

- [ ] Move port derivation logic to domain layer using `PortDerivation` trait
- [ ] Move network computation logic to domain `DockerComposeTopologyBuilder`
- [ ] Convert infrastructure context types to pure DTOs (no business logic)
- [ ] Maintain all existing functionality and E2E tests passing

## 🏗️ Architecture Requirements

**DDD Layer**: Domain (for business logic) + Infrastructure (for DTOs)
**Module Paths**:

- `src/domain/topology/traits.rs` - `PortDerivation` trait
- `src/domain/topology/builder.rs` - `DockerComposeTopologyBuilder`
- `src/domain/topology/fixed_ports.rs` - Caddy/MySQL port functions

**Pattern**: Trait-based port derivation + Builder for topology construction

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

### P4.1: Add Trait and Implement in Domain

- [ ] Create `src/domain/topology/traits.rs` with `PortDerivation` trait
- [ ] Implement `PortDerivation` for `TrackerConfig`
- [ ] Implement `PortDerivation` for `GrafanaConfig`
- [ ] Implement `PortDerivation` for `PrometheusConfig`
- [ ] Create `src/domain/topology/fixed_ports.rs` for Caddy and MySQL
- [ ] Add unit tests for each implementation

### P4.2: Create Domain Topology Builder

- [ ] Create `src/domain/topology/builder.rs` with `DockerComposeTopologyBuilder`
- [ ] Move network computation logic from infrastructure to domain builder
- [ ] Wire up port derivation via trait calls
- [ ] Add integration tests

### P4.3: Refactor Infrastructure to Pure DTOs

- [ ] Remove `compute_networks()` from `TrackerServiceConfig`
- [ ] Remove `compute_networks()` from `GrafanaServiceConfig`
- [ ] Remove `compute_networks()` from `PrometheusServiceConfig`
- [ ] Rename types to `*Context` (e.g., `TrackerServiceContext`)
- [ ] Update `DockerComposeContextBuilder` to receive domain topology
- [ ] Update tests

### P4.4: Cleanup

- [ ] Delete `src/infrastructure/.../context/port_derivation.rs`
- [ ] Remove any remaining business logic from infrastructure
- [ ] Update documentation
- [ ] Run full E2E test suite

## Files Changed

### New Files

| File                                 | Purpose                        |
| ------------------------------------ | ------------------------------ |
| `src/domain/topology/traits.rs`      | `PortDerivation` trait         |
| `src/domain/topology/builder.rs`     | `DockerComposeTopologyBuilder` |
| `src/domain/topology/fixed_ports.rs` | Caddy/MySQL port functions     |

### Modified Files

| File                                           | Change                                  |
| ---------------------------------------------- | --------------------------------------- |
| `src/domain/topology/mod.rs`                   | Export new modules                      |
| `src/domain/tracker/config.rs`                 | Implement `PortDerivation`              |
| `src/domain/grafana/config.rs`                 | Implement `PortDerivation`              |
| `src/domain/prometheus/config.rs`              | Implement `PortDerivation`              |
| `src/infrastructure/.../context/tracker.rs`    | Remove `compute_networks()`, become DTO |
| `src/infrastructure/.../context/grafana.rs`    | Remove `compute_networks()`, become DTO |
| `src/infrastructure/.../context/prometheus.rs` | Remove `compute_networks()`, become DTO |
| `src/infrastructure/.../context/builder.rs`    | Receive domain topology                 |

### Deleted Files

| File                                                | Reason                |
| --------------------------------------------------- | --------------------- |
| `src/infrastructure/.../context/port_derivation.rs` | Logic moved to domain |

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `PortDerivation` trait defined in `domain/topology/traits.rs`
- [ ] All service configs (`TrackerConfig`, `GrafanaConfig`, `PrometheusConfig`) implement `PortDerivation`
- [ ] `DockerComposeTopologyBuilder` computes networks in domain layer
- [ ] Infrastructure context types are pure DTOs with no `compute_*()` methods
- [ ] `port_derivation.rs` deleted from infrastructure
- [ ] All existing E2E tests pass
- [ ] Unit tests cover port derivation for each service

## Design Decisions (Resolved)

| Question                                                                          | Decision                                         | Rationale                                                                                                                        |
| --------------------------------------------------------------------------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| Should `PortDerivation` trait be in `domain/topology/` or a shared traits module? | `domain/topology/traits.rs`                      | The trait exists for topology purposes. Consumer (builder) defines it, implementers import it. Keeps topology concerns cohesive. |
| Should we rename infrastructure context types to `*Context` now or defer?         | Phase 4 (P4.3)                                   | Directly related to "refactor to pure DTOs" goal. One coherent refactoring story.                                                |
| Should `fixed_ports.rs` functions be in the builder or a separate module?         | Separate module `domain/topology/fixed_ports.rs` | Keeps builder focused on orchestration. Single responsibility. Easy to find, extend, test.                                       |

## Related Documentation

- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Docker Compose Topology Domain Model Plan](../refactors/plans/docker-compose-topology-domain-model.md)
- [Epic #287](https://github.com/torrust/torrust-tracker-deployer/issues/287)
