# [Refactor] Phase 3: Create Port Topology Domain Model

**Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Related Plan**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../../refactors/plans/docker-compose-topology-domain-model.md)
**Status**: Draft

## Overview

Move port exposure logic from Tera templates to the domain layer, following the same pattern used for networks in Phase 1-2. This creates type-safe port definitions with descriptions that render as inline YAML comments for sysadmin self-documentation.

## Background

This phase was originally part of the refactoring plan (see PORT-01 through PORT-11 rules in the plan) but was not included in the initial 5-PR strategy. The port exposure rules identified in the plan are currently scattered between Rust code and Tera templates, creating the same dual-source-of-truth problem that was solved for networks.

## Problem Statement

### Current State

Port exposure logic is split across multiple locations:

1. **Template conditionals** - Complex `{%- if %}` blocks for TLS-based port exposure
2. **Rust filtering** - `http_tracker_ports_without_tls` computed in context builder
3. **Hardcoded ports** - Caddy (80, 443), Prometheus (9090), MySQL (none)

Example from template:

```tera
{%- if tracker.needs_ports_section %}
    ports:
      {%- for udp_tracker in tracker.udp_trackers %}
      - "{{ udp_tracker.port }}:{{ udp_tracker.port }}/udp"
      {%- endfor %}
      {%- for http_tracker_port in tracker.http_tracker_ports_without_tls %}
      - "{{ http_tracker_port }}:{{ http_tracker_port }}"
      {%- endfor %}
      {%- if not tracker.http_api_has_tls %}
      - "{{ tracker.http_api_port }}:{{ tracker.http_api_port }}"
      {%- endif %}
{%- endif %}
```

### Problems

1. **Logic in templates** - Port exposure decisions based on TLS configuration
2. **No cross-service validation** - Cannot detect host port conflicts across services
3. **Missing documentation** - Rendered YAML lacks context for what each port does
4. **Existing validation is tracker-only** - `TrackerConfig` validates internal socket conflicts but not Docker Compose host port conflicts

### Desired State

```yaml
services:
  tracker:
    ports:
      # BitTorrent UDP announce (external)
      - "6969:6969/udp"
      # HTTP tracker announce (no TLS)
      - "7070:7070"
      # REST API for stats/whitelist (no TLS)
      - "1212:1212"
```

## Proposed Solution

### Domain Types

Create port topology types that complement the existing network topology:

```rust
// src/domain/topology/port.rs

/// Network protocol for port bindings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// A port binding in Docker Compose
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortBinding {
    /// Host port (None = internal only, not exposed)
    host_port: Option<u16>,
    /// Container port
    container_port: u16,
    /// Protocol (TCP or UDP)
    protocol: Protocol,
    /// Host IP to bind to (None = 0.0.0.0)
    host_ip: Option<IpAddr>,
}

impl PortBinding {
    /// Short description of what this port does
    pub fn description(&self) -> &'static str {
        // Derived from context (service type + port purpose)
    }
}
```

### ServiceTopology Extension

Extend `ServiceTopology` to include port definitions:

```rust
// src/domain/topology/aggregate.rs

pub struct ServiceTopology {
    service: Service,
    networks: Vec<Network>,
    ports: Vec<PortBinding>,  // NEW
}

impl ServiceTopology {
    /// Derives ports from service configuration
    pub fn ports(&self) -> &[PortBinding] {
        &self.ports
    }
}
```

### Aggregate Validation

Add cross-service port conflict detection to `DockerComposeTopology`:

```rust
impl DockerComposeTopology {
    /// Validates no host port conflicts across all services
    fn validate_port_uniqueness(&self) -> Result<(), TopologyError> {
        // Collect all exposed (host_port, protocol) pairs
        // Error if duplicates found
    }
}
```

### Relationship with TrackerConfig Validation

| Validation                                                     | Location                            | Scope                        |
| -------------------------------------------------------------- | ----------------------------------- | ---------------------------- |
| Socket address uniqueness (UDP tracker vs HTTP tracker vs API) | `TrackerConfig::new()`              | Tracker application internal |
| Host port uniqueness (Tracker vs Grafana vs Caddy)             | `DockerComposeTopology::validate()` | Docker Compose deployment    |

**Key insight**: `TrackerConfig` validates what the Torrust Tracker binary can bind to. `DockerComposeTopology` validates what Docker Compose can expose to the host.

## Port Rules from Original Plan

These rules (PORT-01 through PORT-11) will be moved to domain:

| Rule ID | Rule Description                                                          | Target Location                         |
| ------- | ------------------------------------------------------------------------- | --------------------------------------- |
| PORT-01 | Tracker needs ports section if UDP OR HTTP without TLS OR API without TLS | `ServiceTopology::ports()`              |
| PORT-02 | UDP ports always exposed (no TLS)                                         | `PortBinding` for UDP tracker           |
| PORT-03 | HTTP ports WITHOUT TLS exposed directly                                   | `PortBinding` with host_port            |
| PORT-04 | HTTP ports WITH TLS NOT exposed (Caddy handles)                           | No `PortBinding` created                |
| PORT-05 | API exposed only when no TLS                                              | Conditional `PortBinding`               |
| PORT-06 | API NOT exposed when TLS                                                  | No `PortBinding` created                |
| PORT-07 | Grafana 3000 exposed only without TLS                                     | Conditional `PortBinding`               |
| PORT-08 | Grafana 3000 NOT exposed with TLS                                         | No `PortBinding` created                |
| PORT-09 | Caddy always exposes 80, 443, 443/udp                                     | Fixed `PortBinding`s                    |
| PORT-10 | Prometheus 9090 on localhost only                                         | `PortBinding` with `host_ip: 127.0.0.1` |
| PORT-11 | MySQL no exposed ports                                                    | Empty `ports` in `ServiceTopology`      |

## Template Simplification

After refactoring:

```tera
services:
  {{ service.name }}:
{%- if service.ports | length > 0 %}
    ports:
{%- for port in service.ports %}
      # {{ port.description }}
      - "{{ port.binding }}"
{%- endfor %}
{%- endif %}
```

## Tasks

### P3.1: Create Port Domain Types

- [ ] Create `Protocol` enum (or reuse from `domain/tracker`)
- [ ] Create `PortBinding` struct with description support
- [ ] Add `PortDefinition` context type for template rendering
- [ ] Add unit tests for port binding creation

### P3.2: Extend ServiceTopology with Ports

- [ ] Add `ports: Vec<PortBinding>` to `ServiceTopology`
- [ ] Implement port derivation for each service type:
  - Tracker: UDP ports, HTTP ports (TLS-dependent), API (TLS-dependent)
  - Caddy: 80, 443, 443/udp (fixed)
  - Prometheus: 9090 (localhost)
  - Grafana: 3000 (TLS-dependent)
  - MySQL: none
- [ ] Add unit tests for each service's port derivation

### P3.3: Add Cross-Service Port Validation

- [ ] Implement `DockerComposeTopology::validate_port_uniqueness()`
- [ ] Create `TopologyError::PortConflict` variant
- [ ] Add tests for conflict detection scenarios

### P3.4: Update Template and Context

- [ ] Create `PortDefinition` with `binding()` and `description()` for template
- [ ] Update `DockerComposeContext` to use derived ports
- [ ] Simplify `docker-compose.yml.tera` port sections
- [ ] Remove conditional port logic from template

## Acceptance Criteria

- [ ] All PORT-\* rules from refactoring plan are implemented in domain
- [ ] Port descriptions render as YAML comments in output
- [ ] Cross-service port conflicts are detected and reported with actionable error
- [ ] Template has no conditional port logic (just loops)
- [ ] No duplication with `TrackerConfig` validation (different scopes)
- [ ] All existing E2E tests pass
- [ ] Pre-commit checks pass

## Testing Strategy

### Unit Tests

- Port derivation for each service type
- TLS-dependent port inclusion/exclusion
- Port conflict detection
- Description generation

### Integration Tests

- Context builder produces correct port definitions
- Template renders ports with descriptions

### E2E Tests

- Existing tests continue to pass (no behavioral change)

## Out of Scope

- Volume topology (no cross-service invariants currently)
- Service dependency ordering (separate concern)
- Health check configuration

## References

- [Original refactoring plan - Port rules](../../refactors/plans/docker-compose-topology-domain-model.md#port-exposure-rules)
- [TrackerConfig validation](../../../src/domain/tracker/config/mod.rs) - Internal socket conflict validation
- [Network topology implementation (Phase 1-2)](./287-docker-compose-topology-refactoring-epic.md)
- [UDP/TCP port sharing documentation](../../external-issues/tracker/udp-tcp-port-sharing-allowed.md)
