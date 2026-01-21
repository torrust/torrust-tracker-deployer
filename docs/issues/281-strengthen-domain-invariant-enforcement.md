# Strengthen Domain Invariant Enforcement (DDD Refactor)

**Issue**: [#281](https://github.com/torrust/torrust-tracker-deployer/issues/281)
**Type**: Refactor
**Status**: In Progress

## Overview

Apply the DDD validated constructor pattern to all domain configuration types, ensuring domain invariants are enforced at construction time rather than validated post-hoc.

## Detailed Plan

See: [`docs/refactors/plans/strengthen-domain-invariant-enforcement.md`](../refactors/plans/strengthen-domain-invariant-enforcement.md)

## Related ADRs

- [Validated Deserialization for Domain Types](../decisions/validated-deserialization-for-domain-types.md)
- [TryFrom for DTO to Domain Conversion](../decisions/tryfrom-for-dto-to-domain-conversion.md)

## Reference Implementation

`HttpApiConfig` has been refactored as the reference implementation (Phase 0, Proposal #0):

- Location: `src/domain/tracker/config/http_api.rs`
- DTO: `src/application/command_handlers/create/config/tracker/http_api_section.rs`

## Implementation Checklist

### Phase 0: HTTP API Config ✅

- [x] `HttpApiConfig` validated constructor with `HttpApiConfigError`
- [x] Private fields with getter methods
- [x] Custom `Deserialize` using `HttpApiConfigRaw`
- [x] `TryFrom<HttpApiSection> for HttpApiConfig`
- [x] Documentation and ADRs

### Phase 1: Tracker Configuration Types

- [ ] `UdpTrackerConfig` - validated constructor, private fields, getters
- [ ] `HttpTrackerConfig` - validated constructor, private fields, getters
- [ ] `HealthCheckApiConfig` - validated constructor, private fields, getters
- [ ] `TryFrom` implementations for each DTO section

### Phase 2: Cross-Cutting Invariants

- [ ] `TrackerCoreConfig` - database configuration validation
- [ ] `TrackerConfig` - validates at construction (socket conflicts)
- [ ] `UserInputs` - validated constructor (Grafana requires Prometheus)

## Acceptance Criteria

- [ ] All domain configuration types use validated constructors
- [ ] All fields are private with getter methods
- [ ] All types implement custom `Deserialize` with validation
- [ ] All DTO→Domain conversions use `TryFrom` trait
- [ ] Validation logic moved from application to domain layer
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

## Contributing Guide

See [`docs/contributing/ddd-practices.md`](../contributing/ddd-practices.md) for implementation patterns.
