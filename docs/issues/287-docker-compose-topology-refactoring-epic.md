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

## Tasks

Implementation follows a 5-PR strategy:

- [x] [#288](https://github.com/torrust/torrust-tracker-deployer/issues/288) - [ADR] Bind Mount Standardization for Docker Compose (ADR-01)
- [x] [#290](https://github.com/torrust/torrust-tracker-deployer/issues/290) - [BUG] Remove invalid "Grafana without Prometheus" template branch (BUG-01)
- [x] [#292](https://github.com/torrust/torrust-tracker-deployer/issues/292) - [Refactor] Phase 0: Convert volumes to bind mounts with domain type (P0.1, P0.2)
- [x] [#294](https://github.com/torrust/torrust-tracker-deployer/issues/294) - [Refactor] Phase 1: Create Network domain types (P1.1, P1.2)
- [ ] #X - [Refactor] Phase 2: Create DockerComposeTopology aggregate (P2.1, P2.2)

(Tasks will be created and linked as work progresses)

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
```

## Key Decisions

- Use bind mounts exclusively (9 reasons documented in ADR task)
- Type-safe `MountOption` enum (ReadOnly, SELinux)
- Domain-driven network derivation (single source of truth)
- ~47 domain rules identified with test specifications

## Related

- Plan Review: #285 (merged)
- Refactoring Plan: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)
