# [Refactor] Phase 1: Create Network Domain Types

**Issue**: [#294](https://github.com/torrust/torrust-tracker-deployer/issues/294)
**Parent Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Related**:

- [Refactoring Plan](../refactors/plans/docker-compose-topology-domain-model.md#phase-1-domain-network-types-core-infrastructure)
- [Phase 0 PR #293](https://github.com/torrust/torrust-tracker-deployer/pull/293) - Bind mount foundation

## Overview

This task creates domain types for Docker Compose networks and migrates service configurations from `Vec<String>` to type-safe `Vec<Network>`. This is PR 4 in the 5-PR refactoring strategy.

## Goals

- [x] Create type-safe `Network` enum to eliminate string-based network references
- [x] Migrate all service configs to use domain network types
- [x] Establish foundation for Phase 2 (automatic network derivation)

## Implementation

**PR**: [#295](https://github.com/torrust/torrust-tracker-deployer/pull/295)
**Commit**: `11ed1b0e`

### Changes Made

- Created `Network` enum in `src/domain/topology/network.rs` with 4 variants:
  - `Database` - for database services
  - `Metrics` - for Prometheus/exporters
  - `Visualization` - for Grafana
  - `Proxy` - for Caddy reverse proxy
- Implemented `name()`, `driver()`, `all()`, `Display`, custom `Serialize`
- Migrated all 5 service configs to use `Vec<Network>`:
  - `TrackerServiceConfig`
  - `MysqlServiceConfig`
  - `CaddyServiceConfig`
  - `PrometheusServiceConfig`
  - `GrafanaServiceConfig`
- Added comprehensive unit tests (16 for Network, plus per-service tests)
- All 1953 unit tests pass
- E2E infrastructure and deployment tests pass

## 🏗️ Architecture Requirements

**DDD Layer**: Domain
**Module Path**: `src/domain/deployment/topology/`
**Pattern**: Value Object (enum with behavior)

### Module Structure Requirements

- [ ] Create `network.rs` in `src/domain/deployment/topology/`
- [ ] Export via `mod.rs` in topology module
- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../../codebase-architecture.md))

### Architectural Constraints

- [ ] Domain types must be independent of infrastructure concerns
- [ ] Serialization uses network name strings for template compatibility
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Hardcoding network names in multiple places
- ❌ Infrastructure layer creating domain types
- ❌ Template logic computing network assignments

## Specifications

### P1.1: Network Domain Type

Create a domain enum representing Docker Compose networks:

```rust
// src/domain/deployment/topology/network.rs

/// Docker Compose networks used for service isolation
///
/// Each network serves a specific security purpose:
/// - Database: Isolates database access to only the tracker
/// - Metrics: Allows Prometheus to scrape tracker metrics
/// - Visualization: Allows Grafana to query Prometheus
/// - Proxy: Allows Caddy to reverse proxy to backend services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Network {
    /// Network for database access (Tracker ↔ MySQL)
    Database,
    /// Network for metrics scraping (Tracker ↔ Prometheus)
    Metrics,
    /// Network for visualization queries (Prometheus ↔ Grafana)
    Visualization,
    /// Network for TLS proxy (Caddy ↔ backend services)
    Proxy,
}

impl Network {
    /// Returns the network name as used in docker-compose.yml
    pub fn name(&self) -> &'static str {
        match self {
            Network::Database => "database_network",
            Network::Metrics => "metrics_network",
            Network::Visualization => "visualization_network",
            Network::Proxy => "proxy_network",
        }
    }

    /// Returns the network driver (always "bridge" for now)
    pub fn driver(&self) -> &'static str {
        "bridge"
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
```

### P1.2: Migrate Service Configs

Update service configuration structs to use `Vec<Network>` instead of `Vec<String>`:

```rust
use crate::domain::deployment::topology::Network;

pub struct TrackerServiceConfig {
    // ...
    pub networks: Vec<Network>,
}
```

Network assignment rules (from domain analysis):

| Service    | Networks                       | Conditions                            |
| ---------- | ------------------------------ | ------------------------------------- |
| Tracker    | `Database`, `Metrics`, `Proxy` | Based on MySQL, Prometheus, TLS flags |
| MySQL      | `Database`                     | Always (when enabled)                 |
| Prometheus | `Metrics`, `Visualization`     | Based on Grafana flag                 |
| Grafana    | `Visualization`, `Proxy`       | Based on TLS flag                     |
| Caddy      | `Proxy` + others               | Based on proxied services             |

### Template Integration

Create serialization wrapper for network names:

```rust
/// Wrapper for serializing Network to its name string for templates
#[derive(Serialize)]
#[serde(transparent)]
pub struct NetworkRef(String);

impl From<Network> for NetworkRef {
    fn from(net: Network) -> Self {
        Self(net.name().to_string())
    }
}
```

Or implement custom `Serialize` to emit the network name directly.

## Implementation Plan

### Phase 1: Create Network Domain Type (P1.1) (~30 min)

- [ ] Create `src/domain/deployment/topology/network.rs`
- [ ] Add `Network` enum with all four variants
- [ ] Implement `name()` method returning network names
- [ ] Implement `driver()` method returning "bridge"
- [ ] Implement `Display` trait for template rendering
- [ ] Add `Serialize` implementation (emit name string)
- [ ] Export from `src/domain/deployment/topology/mod.rs`
- [ ] Add unit tests for Network enum

### Phase 2: Migrate Service Configs (P1.2) (~1-2 hours)

- [ ] Create `NetworkRef` wrapper or custom serialization
- [ ] Update `TrackerServiceConfig` to use `Vec<Network>`
- [ ] Update `MysqlServiceConfig` to use `Vec<Network>`
- [ ] Update `PrometheusServiceConfig` to use `Vec<Network>`
- [ ] Update `GrafanaServiceConfig` to use `Vec<Network>`
- [ ] Update `CaddyServiceConfig` to use `Vec<Network>`
- [ ] Update network computation logic in each service config
- [ ] Verify template rendering produces identical output
- [ ] Add unit tests for network assignment rules

### Phase 3: Verification (~30 min)

- [ ] Run all unit tests
- [ ] Run E2E tests
- [ ] Verify docker-compose.yml output matches previous behavior
- [ ] Run linters and fix any issues

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `Network` enum exists with `Database`, `Metrics`, `Visualization`, `Proxy` variants
- [ ] All service configs use `Vec<Network>` instead of `Vec<String>`
- [ ] Generated docker-compose.yml is identical to before refactoring
- [ ] Unit tests cover all network assignment rules from domain analysis
- [ ] No string literals for network names outside the `Network` enum

## Unit Tests Required

### Network Enum Tests

- `it_should_return_correct_network_name_for_database`
- `it_should_return_correct_network_name_for_metrics`
- `it_should_return_correct_network_name_for_visualization`
- `it_should_return_correct_network_name_for_proxy`
- `it_should_return_bridge_driver_for_all_networks`
- `it_should_serialize_network_to_name_string`
- `it_should_display_network_as_name`

### Service Network Assignment Tests

**Tracker:**

- `it_should_connect_tracker_to_database_network_when_mysql_enabled`
- `it_should_connect_tracker_to_metrics_network_when_prometheus_enabled`
- `it_should_connect_tracker_to_proxy_network_when_tracker_needs_tls`
- `it_should_not_connect_tracker_to_database_network_when_mysql_disabled`
- `it_should_not_connect_tracker_to_metrics_network_when_prometheus_disabled`
- `it_should_not_connect_tracker_to_proxy_network_when_tracker_has_no_tls`

**MySQL:**

- `it_should_connect_mysql_to_database_network`

**Prometheus:**

- `it_should_connect_prometheus_to_metrics_network`
- `it_should_connect_prometheus_to_visualization_network_when_grafana_enabled`
- `it_should_not_connect_prometheus_to_visualization_network_when_grafana_disabled`

**Grafana:**

- `it_should_connect_grafana_to_visualization_network`
- `it_should_connect_grafana_to_proxy_network_when_grafana_has_tls`
- `it_should_not_connect_grafana_to_proxy_network_when_grafana_has_no_tls`

**Caddy:**

- `it_should_connect_caddy_to_all_networks_of_proxied_services`

## Related Documentation

- [Refactoring Plan - Phase 1](../refactors/plans/docker-compose-topology-domain-model.md#phase-1-domain-network-types-core-infrastructure)
- [Codebase Architecture](../../codebase-architecture.md)
- [DDD Layer Placement Guide](../../contributing/ddd-layer-placement.md)
- [Unit Testing Conventions](../../contributing/testing/unit-testing.md)

## Notes

- This phase can technically start in parallel with Phase 0, but depends on Phase 0 for the bind mount patterns and domain type conventions
- The `Network` enum is designed to be extended in Phase 2 for automatic network derivation
- Template output must remain identical to verify no regression
