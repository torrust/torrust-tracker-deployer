# Docker Compose Topology Domain Model Refactoring Plan

**Issue**: [#285](https://github.com/torrust/torrust-tracker-deployer/issues/285)
**Parent Epic**: N/A _(standalone documentation issue)_
**Related**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)

## Overview

This issue tracks the review and approval of the refactoring plan for moving Docker Compose topology logic from Tera templates to the Rust domain layer. The plan document has been created and needs contributor review before implementation begins.

Once this plan is approved and merged, a separate Epic issue will be created to track the actual implementation work.

## Goals

- [ ] Get contributor review of the refactoring plan
- [ ] Validate the phased approach and task breakdown
- [ ] Ensure the plan aligns with project architecture principles
- [ ] Merge the plan documentation to main branch

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (documentation only)
**Module Path**: `docs/refactors/plans/`
**Pattern**: Refactoring Plan Document

### Module Structure Requirements

- [x] Follow refactoring documentation conventions (see [docs/refactors/README.md](../refactors/README.md))
- [x] Plan is registered in [docs/refactors/active-refactorings.md](../refactors/active-refactorings.md)
- [x] DDD layer placement guidance referenced for proposed code locations

### Architectural Constraints

- [x] Plan proposes domain-first approach (domain types before infrastructure changes)
- [x] Plan respects dependency flow rules
- [x] Plan includes unit test specifications for domain invariants

### Anti-Patterns to Avoid

- ❌ Proposing business logic in templates
- ❌ Proposing infrastructure layer depending on presentation
- ❌ Big-bang refactoring without incremental phases

## Specifications

### Plan Document Location

The complete refactoring plan is at:
[docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)

### Plan Summary

The plan addresses architectural issues where Docker Compose topology rules (networks, volumes, dependencies) are scattered between Rust code and Tera templates. It proposes:

1. **Phase 0**: Convert named volumes to bind mounts with new `BindMount` domain type
2. **Phase 1**: Create `Network` enum and `NetworkSet` for type-safe network management
3. **Phase 2**: Create `DockerComposeTopology` aggregate that derives required networks from services

### Key Decisions Documented

- Use bind mounts exclusively (9 reasons documented in ADR task)
- Type-safe `MountOption` enum (ReadOnly, SELinux)
- Domain-driven network derivation (single source of truth)
- ~47 domain rules identified with test specifications

### Implementation Strategy

The plan proposes 5 separate PRs:

| PR   | Scope                                  |
| ---- | -------------------------------------- |
| PR 1 | ADR: Bind Mount Standardization        |
| PR 2 | BUG-01: Remove invalid template branch |
| PR 3 | Phase 0: Bind mount foundation         |
| PR 4 | Phase 1: Network domain types          |
| PR 5 | Phase 2: Topology aggregate            |

### Issues Discovered During Planning

- **BUG-01**: Template handles invalid "Grafana without Prometheus" case
- **ISSUE-01**: ADR `grafana-integration-pattern.md` recommends named volumes (will be superseded)
- **ISSUE-03**: ADR says Grafana has no healthcheck but template now has one

## Implementation Plan

### Phase 1: Plan Review (this issue)

- [ ] Task 1.1: Create GitHub issue for plan review
- [ ] Task 1.2: Create branch and PR for plan documentation
- [ ] Task 1.3: Get contributor review and feedback
- [ ] Task 1.4: Address review comments if any
- [ ] Task 1.5: Merge plan to main branch

### Phase 2: Implementation Tracking (after merge)

- [ ] Task 2.1: Create Epic issue for implementation
- [ ] Task 2.2: Create first child issue (ADR-01)
- [ ] Task 2.3: Begin implementation following the plan

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Plan document is complete with all phases defined
- [ ] Domain rules are documented with test specifications
- [ ] Implementation strategy (PRs) is clear
- [ ] Bugs and inconsistencies discovered are documented
- [ ] Plan follows refactoring documentation conventions
- [ ] Plan is registered in active-refactorings.md
- [ ] All markdown linting passes

## Related Documentation

- [docs/refactors/README.md](../refactors/README.md) - Refactoring process
- [docs/refactors/active-refactorings.md](../refactors/active-refactorings.md) - Active refactorings index
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) - DDD layer guidance
- [docs/decisions/grafana-integration-pattern.md](../decisions/grafana-integration-pattern.md) - ADR to be superseded
- [templates/docker-compose/docker-compose.yml.tera](../../templates/docker-compose/docker-compose.yml.tera) - Current template

## Notes

- This is a **documentation-only** issue for reviewing the plan
- The actual implementation will be tracked by a separate Epic issue after this plan is merged
- The plan was developed iteratively, discovering bugs and inconsistencies along the way
- Once merged, the Epic issue will reference this plan as the source of truth
