# Vertical Slice D: PostgreSQL Failure UX, State Mapping, and Diagnostics

**Issue**: TBD (`469-04` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `src/domain/environment/state/release_failed.rs`
- `src/application/command_handlers/release/workflow.rs`
- `src/application/command_handlers/release/steps/mysql.rs`
- `src/presentation/` output handlers related to error presentation
- `docs/user-guide/commands/`
- `docs/e2e-testing/manual-testing.md`

## Overview

Deliver actionable PostgreSQL operational behavior when deployment/release fails: users should know what failed, where it failed, and what to do next.

This slice focuses on runtime ergonomics and troubleshooting quality, not on adding new core provisioning capabilities.

## Deployable Outcome

After this slice, PostgreSQL-related failures in release and startup paths are surfaced with clear, step-specific messages and troubleshooting guidance equivalent to sqlite/mysql quality.

## Goals

- [ ] Add PostgreSQL-specific failure step(s) in release failure state.
- [ ] Ensure CLI output is explicit and actionable for PostgreSQL failures.
- [ ] Add operational diagnostics checks to detect common misconfigurations.
- [ ] Keep existing error output style and compatibility.

## Inside-Out Execution Order

Implement this slice from internals toward user-facing contract:

1. Domain/application internals: add PostgreSQL-specific failure states and workflow mapping first.
2. Automation/diagnostics: add deterministic early checks for common misconfigurations.
3. Command/presentation wiring: surface clear CLI errors with remediation actions.
4. Contract/docs check: ensure operational guide text and command docs match failure behavior.
5. Slice gate: state-mapping tests plus one representative failure-path integration test.

## Code-Level Implementation Details

### 0. Existing Code Paths To Extend

- Failure step enum and message mapping: `src/domain/environment/state/release_failed.rs`
- Release step orchestration: `src/application/command_handlers/release/workflow.rs`
- SQL storage step patterns to mirror: `src/application/command_handlers/release/steps/mysql.rs`
- User output formatting layer: `src/presentation/` modules used by command handlers

### 1. Failure Step Coverage

- Add step variant(s) for PostgreSQL-specific failures such as:
  - storage directory preparation failure
  - postgresql service readiness failure
- Ensure these variants are used by workflow transitions and state persistence.

### 2. Error Message Quality

For PostgreSQL failures, messages should include:

- failed step name
- expected remote path or service
- likely causes
- next command or check to run (example: docker logs, storage path inspection)

### 3. Operational Diagnostics

- Add or improve checks where misconfiguration can be detected early (before opaque runtime failures):
  - missing PostgreSQL storage path when managed mode is selected
  - invalid host/port combinations for external mode
- Ensure diagnostics are deterministic and covered by tests.

## Architecture Requirements

**DDD Layer**: Domain state + Application workflow + Presentation output
**Pattern**: Actionable error handling / user friendliness principle

### Constraints

- [ ] No sensitive values exposed in errors.
- [ ] Error messages follow existing tone and actionability standards.
- [ ] Existing failure mapping for mysql/sqlite stays unchanged.

## Implementation Plan

### Phase 1: State + Workflow (0.5 day)

- [ ] Task 1.1: Add PostgreSQL release failure steps in state model.
- [ ] Task 1.2: Wire workflow transitions to those steps.

### Phase 2: Diagnostics Checks (0.5 day)

- [ ] Task 2.1: Add deterministic pre-failure checks for common PostgreSQL misconfigurations.
- [ ] Task 2.2: Ensure diagnostics are stable and non-sensitive.

### Phase 3: User-Facing Errors (0.5 day)

- [ ] Task 3.1: Improve message templates for PostgreSQL failure cases.
- [ ] Task 3.2: Ensure action-oriented remediation text is present.

### Phase 4: Tests + Operational Guide Checks (0.5 day)

- [ ] Task 4.1: Add unit tests for state mapping and error messages.
- [ ] Task 4.2: Add integration-style test for one representative PostgreSQL failure path.
- [ ] Task 4.3: Validate no regressions in mysql/sqlite failure message tests.

## Acceptance Criteria

- [ ] PostgreSQL release failures are mapped to explicit state steps.
- [ ] CLI output includes actionable guidance for PostgreSQL failures.
- [ ] No secret values are leaked in logs/errors.
- [ ] Existing mysql/sqlite error behavior remains intact.
