# [Epic] Docker Compose Topology Domain Model Refactoring

**Issue**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287)
**Related**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)

## Overview

This epic tracks the implementation of the Docker Compose Topology Domain Model refactoring, moving topology logic from Tera templates to the Rust domain layer.

## Refactoring Plan Reference

From [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md):

> This refactoring plan addresses architectural issues in the docker-compose template rendering where infrastructure topology rules (networks, volumes, dependencies) are scattered between Rust code and Tera templates. The goal is to move all topology decisions to the domain layer, enforcing invariants and making the template a pure rendering layer.

## Scope

- Convert all named volumes to bind mounts (unified `./storage/` directory)
- Standardize bind mount representation in template context
- Create domain types for Docker Compose topology (networks, dependencies)
- Derive global network lists from service configurations (single source of truth)
- Move topology decision logic from template conditionals to domain layer
- Enforce invariants like "if a service uses a network, that network must be defined"
- Move port exposure logic from templates to domain (Phase 3 - added post-completion)

## Tasks

Implementation follows a 5-PR strategy (original scope), with Phase 3 added as a continuation:

### Original 5-PR Strategy (Completed)

- [x] [#288](https://github.com/torrust/torrust-tracker-deployer/issues/288) - [ADR] Bind Mount Standardization for Docker Compose (ADR-01)
- [x] [#290](https://github.com/torrust/torrust-tracker-deployer/issues/290) - [BUG] Remove invalid "Grafana without Prometheus" template branch (BUG-01)
- [x] [#292](https://github.com/torrust/torrust-tracker-deployer/issues/292) - [Refactor] Phase 0: Convert volumes to bind mounts with domain type (P0.1, P0.2)
- [x] [#294](https://github.com/torrust/torrust-tracker-deployer/issues/294) - [Refactor] Phase 1: Create Network domain types (P1.1, P1.2)
- [x] [#296](https://github.com/torrust/torrust-tracker-deployer/issues/296) - [Refactor] Phase 2: Create DockerComposeTopology aggregate (P2.1, P2.2) → [PR #297](https://github.com/torrust/torrust-tracker-deployer/pull/297)

### Phase 3: Port Topology (Added Extension)

> **Note**: Phase 3 was part of the original refactoring plan analysis (see PORT-01 through PORT-11 rules in [docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md#port-exposure-rules)) but was not included in the initial 5-PR strategy. It follows the same pattern as networks and completes the topology domain model.
>
> **Implementation Note**: Phase 3 is split into two PRs for better reviewability. PR #298 delivers the domain layer foundation (P3.1-P3.3), while a follow-up PR will integrate ports into the template (P3.4).

- [ ] [#298](https://github.com/torrust/torrust-tracker-deployer/issues/298) - [Refactor] Phase 3: Port Topology Domain Model - Foundation (P3.1, P3.2, P3.3) → [Spec](298-phase-3-port-topology-domain-model.md)
  - P3.1: Create Port domain types (`PortBinding`, reuse `Protocol`)
  - P3.2: Extend `ServiceTopology` with ports field
  - P3.3: Add cross-service port conflict validation with `help()` method
- [ ] TBD - [Refactor] Phase 3: Port Topology Template Integration (P3.4)
  - P3.4: Update template to use derived ports with descriptions

## PR Dependencies

```text
PR 1 (ADR-01)
    │
    ▼
PR 3 (Phase 0) ◄─── PR 2 (BUG-01) can be merged independently
    │
    ▼
PR 4 (Phase 1)
    │
    ▼
PR 5 (Phase 2)
    │
    ▼
PR 6 (Phase 3 Foundation) ◄─── Domain types: PortBinding, validation, help()
    │
    ▼
PR 7 (Phase 3 Template) ◄─── Template integration (P3.4) - follow-up
```

## Key Decisions

- Use bind mounts exclusively (9 reasons documented in ADR task)
- Type-safe `MountOption` enum (ReadOnly, SELinux)
- Domain-driven network derivation (single source of truth)
- ~47 domain rules identified with test specifications
- Network descriptions rendered as YAML comments for sysadmin documentation
- Port topology follows same pattern as networks (Phase 3)
- Phase 3 split into foundation (domain) and integration (template) PRs for reviewability
- `PortConflict` error includes `help()` method per DDD error handling conventions

## Related

- Plan Review: #285 (merged)
- Refactoring Plan: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)
- Phase 3 Spec: [298-phase-3-port-topology-domain-model.md](298-phase-3-port-topology-domain-model.md)
