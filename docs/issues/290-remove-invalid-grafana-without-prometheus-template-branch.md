# [BUG] Remove invalid "Grafana without Prometheus" template branch

**Issue**: [#290](https://github.com/torrust/torrust-tracker-deployer/issues/290)
**Parent Epic**: #287 - [Epic] Docker Compose Topology Domain Model Refactoring
**Related**:

- Refactoring Plan: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md#bug-01-template-handles-invalid-grafana-without-prometheus-case)
- Grafana Integration ADR: [docs/decisions/grafana-integration-pattern.md](../decisions/grafana-integration-pattern.md)

## Overview

Remove dead code from the Docker Compose template that handles an impossible configuration state: Grafana enabled without Prometheus. This template branch can never be executed because environment validation should reject this configuration. The code misleads readers into thinking this is a valid configuration.

## Problem

The template has an `{%- else %}` branch for when Grafana is enabled but Prometheus is NOT:

```yaml
grafana:
  depends_on:
{%- if prometheus %}
    prometheus:
      condition: service_healthy
{%- else %}
    - tracker  # <-- This branch handles an INVALID case
{%- endif %}
```

### Why it's a bug

- Grafana requires Prometheus as its data source - it has no purpose without it
- The environment creation should fail validation if `grafana.enabled = true` and `prometheus.enabled = false`
- This template branch can NEVER be reached in valid configurations
- The code is dead code that misleads readers into thinking this is a valid configuration

## Goals

- [ ] Verify environment validation rejects `grafana.enabled && !prometheus.enabled`
- [ ] Remove the dead `{%- else %}` branch from the Docker Compose template
- [ ] Simplify template to always use Prometheus dependency when Grafana is enabled

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (template) + Application (validation)
**Module Path**: `templates/docker-compose/docker-compose.yml.tera` + validation in `src/application/`
**Pattern**: Template cleanup + validation verification

### Architectural Constraints

- [ ] No new business logic in templates - templates should be pure rendering
- [ ] Validation belongs in Application layer (DTOs) or Domain layer (entities)
- [ ] Template conditionals should only handle VALID configuration combinations

### Anti-Patterns to Avoid

- ❌ Adding more conditional branches for invalid states
- ❌ "Defensive" template code that handles impossible cases
- ❌ Silent fallbacks for invalid configurations

## Specifications

### Current Template State

File: `templates/docker-compose/docker-compose.yml.tera`

```yaml
grafana:
  depends_on:
{%- if prometheus %}
    prometheus:
      condition: service_healthy
{%- else %}
    - tracker
{%- endif %}
```

### Expected Template State After Fix

```yaml
grafana:
  depends_on:
    prometheus:
      condition: service_healthy
```

Since Grafana can only exist when Prometheus exists (enforced by validation), no conditional is needed.

### Validation Verification

Verify that the environment configuration DTO or domain type enforces this invariant:

```rust
// Expected validation behavior
// If grafana.enabled == true, then prometheus.enabled MUST be true
```

## Implementation Plan

### Phase 1: Verify Validation (5 minutes)

- [ ] Search codebase for existing validation of Grafana/Prometheus relationship
- [ ] Locate where `grafana.enabled` and `prometheus.enabled` are validated
- [ ] Confirm validation rejects `grafana.enabled && !prometheus.enabled`
- [ ] If validation is missing, add it to the environment configuration DTO

### Phase 2: Remove Dead Code (5 minutes)

- [ ] Edit `templates/docker-compose/docker-compose.yml.tera`
- [ ] Remove the `{%- if prometheus %}...{%- else %}...{%- endif %}` conditional
- [ ] Replace with unconditional Prometheus dependency

### Phase 3: Verification (5 minutes)

- [ ] Run all linters: `./scripts/pre-commit.sh`
- [ ] Verify E2E tests still pass
- [ ] Manually review the generated docker-compose.yml for an environment with Grafana enabled

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Dead `{%- else %}` branch removed from Grafana `depends_on` section
- [ ] Validation exists that rejects `grafana.enabled && !prometheus.enabled`
- [ ] No regression in generated docker-compose.yml for valid configurations
- [ ] E2E tests pass

## Related Documentation

- [Docker Compose Topology Refactoring Plan](../refactors/plans/docker-compose-topology-domain-model.md)
- [Grafana Integration Pattern ADR](../decisions/grafana-integration-pattern.md)
- [Tera Template Guide](../contributing/templates/tera.md)

## Notes

This is a quick cleanup task that:

1. Removes misleading dead code
2. Makes the template cleaner and easier to understand
3. Prepares for Phase 0/1 refactoring by simplifying the template

The fix is independent of other refactoring tasks and can be merged at any time.
