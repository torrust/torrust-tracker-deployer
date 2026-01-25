# [Refactor] Phase 2: Create DockerComposeTopology Aggregate

**Issue**: [#296](https://github.com/torrust/torrust-tracker-deployer/issues/296)
**Parent Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Related**:

- [Refactoring Plan](../refactors/plans/docker-compose-topology-domain-model.md#phase-2-topology-aggregate--network-derivation)
- [Phase 1 PR #295](https://github.com/torrust/torrust-tracker-deployer/pull/295) - Network domain types

## Overview

This task creates the `DockerComposeTopology` aggregate and derives required networks from service configurations. This is PR 5 in the 5-PR refactoring strategy and completes Epic #287.

## Goals

- [ ] Create `DockerComposeTopology` aggregate with service topology collection
- [ ] Create `Service` enum for type-safe service identification
- [ ] Derive required networks from service configurations (single source of truth)
- [ ] Remove conditional network logic from docker-compose template
- [ ] Enforce invariant: "if a service uses a network, that network must be defined"

## 🏗️ Architecture Requirements

**DDD Layer**: Domain for types, Infrastructure for context builder
**Module Path**: `src/domain/topology/` (extend existing module)
**Pattern**: Aggregate Root with derived collections

### Module Structure Requirements

- [ ] Add `Service` enum to `src/domain/topology/`
- [ ] Create `DockerComposeTopology` aggregate in `src/domain/topology/`
- [ ] Add `NetworkDefinition` type for template context
- [ ] Extend `DockerComposeContext` with `required_networks` field

### Architectural Constraints

- [ ] Domain types must be independent of infrastructure concerns
- [ ] Network derivation happens in domain layer, not template
- [ ] Template becomes pure rendering (no conditionals for networks)
- [ ] All existing tests must continue to pass
- [ ] E2E tests must verify behavioral equivalence

### Anti-Patterns to Avoid

- ❌ Template conditionals for network definitions
- ❌ Hardcoded network lists in multiple places
- ❌ Service names as strings (use `Service` enum)
- ❌ Orphan networks (networks defined but not used)

## Specifications

### P2.1: Create DockerComposeTopology Aggregate

Create domain types for topology management:

```rust
// src/domain/topology/service.rs

/// Services in the Docker Compose deployment
///
/// This enum provides type-safe service identification, preventing typos
/// and enabling exhaustive matching in domain logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Service {
    Tracker,
    MySQL,
    Prometheus,
    Grafana,
    Caddy,
}

impl Service {
    /// Returns the service name as used in docker-compose.yml
    pub fn name(&self) -> &'static str {
        match self {
            Service::Tracker => "tracker",
            Service::MySQL => "mysql",
            Service::Prometheus => "prometheus",
            Service::Grafana => "grafana",
            Service::Caddy => "caddy",
        }
    }
}
```

```rust
// src/domain/topology/aggregate.rs

/// Docker Compose deployment topology
///
/// This aggregate ensures all invariants are maintained:
/// - Networks used by services are derived and always defined
/// - Service dependencies are explicitly modeled
pub struct DockerComposeTopology {
    services: Vec<ServiceTopology>,
}

/// Topology information for a single service
pub struct ServiceTopology {
    pub service: Service,
    pub networks: Vec<Network>,
}

impl DockerComposeTopology {
    /// Returns all networks required by enabled services
    ///
    /// This is the single source of truth - the template's `networks:` section
    /// should iterate over this, not use conditionals.
    pub fn required_networks(&self) -> Vec<Network> {
        let unique: HashSet<Network> = self.services.iter()
            .flat_map(|s| s.networks.iter().copied())
            .collect();

        // Return in deterministic order for template stability
        let mut networks: Vec<Network> = unique.into_iter().collect();
        networks.sort_by_key(|n| n.name());
        networks
    }
}
```

#### Unit Tests Required (P2.1)

Service enum tests:

- `it_should_return_correct_service_name_for_tracker`
- `it_should_return_correct_service_name_for_mysql`
- `it_should_return_correct_service_name_for_prometheus`
- `it_should_return_correct_service_name_for_grafana`
- `it_should_return_correct_service_name_for_caddy`
- `it_should_display_service_as_name`

Aggregate invariant tests:

- `it_should_derive_required_networks_from_all_services` (INV-01)
- `it_should_not_have_orphan_networks` (INV-02)
- `it_should_return_networks_in_deterministic_order`

### P2.2: Derive Required Networks in Context

Add `required_networks` to `DockerComposeContext`:

```rust
/// A network definition for the global `networks:` section
#[derive(Serialize)]
pub struct NetworkDefinition {
    pub name: String,
    pub driver: String,
}

impl From<Network> for NetworkDefinition {
    fn from(net: Network) -> Self {
        Self {
            name: net.name().to_string(),
            driver: net.driver().to_string(),
        }
    }
}

pub struct DockerComposeContext {
    // ... existing service fields ...

    /// All networks required by enabled services (derived)
    pub required_networks: Vec<NetworkDefinition>,
}
```

Update template to use derived networks:

```yaml
{%- if required_networks | length > 0 %}
networks:
{%- for net in required_networks %}
  {{ net.name }}:
    driver: {{ net.driver }}
{%- endfor %}
{%- endif %}
```

#### Unit Tests Required (P2.2)

Network derivation tests (from Domain Rules Analysis):

- `it_should_include_database_network_when_mysql_enabled` (NET-01)
- `it_should_include_metrics_network_when_prometheus_enabled` (NET-02)
- `it_should_include_visualization_network_when_grafana_enabled` (NET-03)
- `it_should_include_proxy_network_when_caddy_enabled` (NET-04)
- `it_should_include_all_service_networks_in_required_networks` (NET-14)
- `it_should_not_include_unused_networks_in_required_networks` (NET-15)
- `it_should_not_include_database_network_when_mysql_disabled`
- `it_should_not_include_metrics_network_when_prometheus_disabled`
- `it_should_not_include_visualization_network_when_grafana_disabled`
- `it_should_not_include_proxy_network_when_caddy_disabled`

Configuration combination tests:

- `it_should_configure_minimal_deployment` (no networks needed)
- `it_should_configure_deployment_with_mysql` (database_network only)
- `it_should_configure_deployment_with_monitoring` (metrics + visualization)
- `it_should_configure_full_http_deployment` (database + metrics + visualization)
- `it_should_configure_full_https_deployment` (all four networks)
- `it_should_configure_https_minimal_deployment` (proxy_network only)

## Implementation Checklist

### P2.1: DockerComposeTopology Aggregate

- [ ] Create `Service` enum in `src/domain/topology/service.rs`
- [ ] Create `DockerComposeTopology` aggregate in `src/domain/topology/aggregate.rs`
- [ ] Create `ServiceTopology` struct
- [ ] Implement `required_networks()` method with deduplication and sorting
- [ ] Add Display trait for Service
- [ ] Export types in `src/domain/topology/mod.rs`
- [ ] Write unit tests for Service enum
- [ ] Write unit tests for aggregate invariants

### P2.2: Derive Required Networks in Context

- [ ] Add `NetworkDefinition` type to context module
- [ ] Add `required_networks: Vec<NetworkDefinition>` to `DockerComposeContext`
- [ ] Implement `derive_required_networks()` in builder
- [ ] Update template to iterate over `required_networks`
- [ ] Remove all `{%- if mysql %}` style conditionals from networks section
- [ ] Write unit tests for network derivation
- [ ] Write configuration combination tests

### Verification

- [ ] All unit tests pass
- [ ] E2E infrastructure lifecycle tests pass
- [ ] E2E deployment workflow tests pass
- [ ] Verify rendered output matches current behavior (behavioral equivalence)
- [ ] Run all linters
- [ ] Update progress documentation

## Acceptance Criteria

1. ✅ `Service` enum exists with all 5 services
2. ✅ `DockerComposeTopology` aggregate derives required networks
3. ✅ `required_networks` field added to `DockerComposeContext`
4. ✅ Template uses `required_networks` loop instead of conditionals
5. ✅ No conditional network logic remains in template
6. ✅ All invariants enforced (no orphan networks, deterministic order)
7. ✅ All existing tests pass
8. ✅ E2E tests verify behavioral equivalence

## Notes

### Behavioral Equivalence

The rendered docker-compose.yml output must be functionally identical before and after this change. The order of networks in the output may change (now alphabetically sorted), but the same networks must be defined.

### Future Work

This phase completes the Epic #287 scope. Future phases may address:

- Service dependencies (`ServiceDependency` domain type)
- Port exposure rules (`PortMapping` domain type)
- Service inclusion logic

These are documented in the refactoring plan but are out of scope for this PR.
