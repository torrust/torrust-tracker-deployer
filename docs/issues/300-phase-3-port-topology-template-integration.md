# [Refactor] Phase 3: Port Topology Template Integration (P3.4)

**Issue**: [#300](https://github.com/torrust/torrust-tracker-deployer/issues/300)
**Parent Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Foundation**: [#298](https://github.com/torrust/torrust-tracker-deployer/issues/298) - Phase 3 Foundation (P3.1-P3.3) ✅ Merged
**Related Plan**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../../refactors/plans/docker-compose-topology-domain-model.md)

## Overview

Integrate the `PortBinding` domain types (created in #298) into the Docker Compose template rendering. This completes Phase 3 of the topology refactoring by moving port exposure logic from Tera template conditionals to the domain layer.

## Problem Statement

### Current State

Port exposure logic is split across multiple locations with complex conditionals:

**Template (`docker-compose.yml.tera`)**:

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

**Context Builder (`DockerComposeContext`)**:

- `http_tracker_ports_without_tls` computed with TLS filtering
- `needs_ports_section` computed from multiple conditions
- Port logic duplicated for each service

### Problems

1. **Logic in templates** - Port exposure decisions based on TLS configuration
2. **No descriptions** - Rendered YAML lacks context for what each port does
3. **Duplication** - Similar port patterns repeated across services
4. **Hard to test** - Template logic not unit-testable

### Desired State

Simple template loops with domain-derived ports:

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

Output with inline documentation:

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

## Port Rules (from Refactoring Plan)

These PORT-\* rules will be implemented in the port derivation logic:

| Rule ID | Rule Description                                                          | Implementation                     |
| ------- | ------------------------------------------------------------------------- | ---------------------------------- |
| PORT-01 | Tracker needs ports section if UDP OR HTTP without TLS OR API without TLS | Derive from `TrackerConfig`        |
| PORT-02 | UDP ports always exposed (no TLS for UDP)                                 | Always create `PortBinding::udp()` |
| PORT-03 | HTTP ports WITHOUT TLS exposed directly                                   | Conditional `PortBinding::tcp()`   |
| PORT-04 | HTTP ports WITH TLS NOT exposed (Caddy handles)                           | No `PortBinding` created           |
| PORT-05 | API exposed only when no TLS                                              | Conditional `PortBinding::tcp()`   |
| PORT-06 | API NOT exposed when TLS                                                  | No `PortBinding` created           |
| PORT-07 | Grafana 3000 exposed only without TLS                                     | Conditional based on HTTPS config  |
| PORT-08 | Grafana 3000 NOT exposed with TLS                                         | No `PortBinding` created           |
| PORT-09 | Caddy always exposes 80, 443, 443/udp                                     | Fixed `PortBinding`s               |
| PORT-10 | Prometheus 9090 on localhost only                                         | `PortBinding::localhost_tcp(9090)` |
| PORT-11 | MySQL no exposed ports                                                    | Empty `ports` in `ServiceTopology` |

## Implementation Plan

### P3.4.1: Create Port Derivation Functions

**Location**: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/`

- [x] Create `port_derivation.rs` module with functions:
  - `derive_tracker_ports(udp_ports, http_ports_without_tls, http_api_port, http_api_has_tls) -> Vec<PortBinding>`
  - `derive_caddy_ports() -> Vec<PortBinding>`
  - `derive_prometheus_ports() -> Vec<PortBinding>`
  - `derive_grafana_ports(has_tls: bool) -> Vec<PortBinding>`
  - `derive_mysql_ports() -> Vec<PortBinding>` (returns empty)
- [x] Add unit tests for each derivation function (15 tests)
- [x] Test TLS-dependent behavior (HTTP/API ports hidden when TLS enabled)

### P3.4.2: Create Template Context Types

**Location**: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/`

- [x] Create `PortDefinition` struct for template rendering:

  ```rust
  pub struct PortDefinition {
      binding: String,      // e.g., "6969:6969/udp"
      description: String,  // e.g., "BitTorrent UDP announce"
  }
  ```

- [x] Add `ports: Vec<PortDefinition>` to service context types
- [x] Implement `From<&PortBinding>` for `PortDefinition`

### P3.4.3: Update Context Builder

**Location**: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/mod.rs`

- [x] Build service configs with derived ports for each service
- [x] Call `try_build()` with port validation before rendering
- [x] Convert `PortBinding` to `PortDefinition` using `From` trait for template context
- [ ] Remove legacy port computation (`http_tracker_ports_without_tls`, etc.) - DEFERRED (backward compatibility)

### P3.4.4: Simplify Template

**Location**: `templates/docker-compose/docker-compose.yml.tera`

- [x] Replace conditional port logic with simple loops
- [x] Add description comments using `# {{ port.description }}`
- [x] Remove `needs_ports_section` checks (replaced by `ports | length > 0`)
- [x] Apply same pattern to all services (Tracker, Caddy, Prometheus, Grafana)

### P3.4.5: Validation Integration

- [x] Add `try_build()` method to `DockerComposeContextBuilder` with port validation
- [x] Surface `PortConflict` errors with `help()` message via `PortConflictError`
- [x] DDD "always valid" pattern: `DockerComposeTopology::new()` returns `Result<Self, PortConflict>`
- [x] Used idiomatic Rust `From` trait instead of helper functions

## Acceptance Criteria

- [x] All PORT-\* rules from refactoring plan implemented in port derivation functions
- [x] Port descriptions render as YAML comments in generated output
- [x] Template uses loops over `service.ports` (conditional logic simplified)
- [ ] Legacy port computation removed from context builder - DEFERRED (backward compatibility)
- [x] Cross-service port conflicts detected before rendering via `try_build()`
- [x] All existing E2E tests pass (no behavioral change)
- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

## Testing Strategy

### Unit Tests

- [x] Port derivation for each service type (15 tests in `port_derivation.rs`)
- [x] TLS-dependent port inclusion/exclusion
- [x] Description generation for each port type
- [x] `PortDefinition` conversion via `From` trait

### Integration Tests

- [x] Context builder produces correct port definitions
- [x] Template renders ports with descriptions
- [x] Validation called before rendering

### E2E Tests

- [x] Existing tests continue to pass (no behavioral change)
- [x] Generated `docker-compose.yml` has correct ports section

## Files to Modify

| File                                                                                               | Change                               |
| -------------------------------------------------------------------------------------------------- | ------------------------------------ |
| `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/mod.rs`     | Add port derivation, call validation |
| `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/tracker.rs` | Remove legacy port logic             |
| `templates/docker-compose/docker-compose.yml.tera`                                                 | Simplify to loops                    |
| NEW: `port_derivation.rs` or similar                                                               | Port derivation functions            |
| NEW: `port_definition.rs` or similar                                                               | Template context type                |

## Out of Scope

- Volume topology (no cross-service invariants)
- Service dependency ordering
- Health check configuration
- Changes to `PortBinding` domain type (already complete in #298)

## References

- [Refactoring Plan - Port Rules](../../refactors/plans/docker-compose-topology-domain-model.md#port-exposure-rules)
- [Phase 3 Foundation Spec](./298-phase-3-port-topology-domain-model.md)
- [Network Topology Implementation (Phase 1-2)](./287-docker-compose-topology-refactoring-epic.md)
- [UDP/TCP Port Sharing Documentation](../../external-issues/tracker/udp-tcp-port-sharing-allowed.md)
