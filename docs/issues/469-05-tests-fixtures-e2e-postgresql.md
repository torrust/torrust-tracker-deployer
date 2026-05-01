# Vertical Slice E: PostgreSQL Test Matrix and E2E Coverage

**Issue**: TBD (`469-05` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `envs/`
- `src/application/services/rendering/docker_compose.rs`
- `src/application/services/rendering/backup.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`
- `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs`
- `src/bin/e2e-complete-workflow-tests.rs`
- `src/bin/e2e-infrastructure-lifecycle-tests.rs`
- `src/bin/e2e-deployment-workflow-tests.rs`
- `tests/`
- `docker/backup/backup_test.bats`

## Overview

Add comprehensive automated validation for PostgreSQL support across unit, integration, and E2E levels while preserving existing sqlite/mysql quality gates.

This slice is independently deployable because it introduces enforceable quality gates required to safely release PostgreSQL functionality from prior slices.

## Deployable Outcome

After this slice, PostgreSQL support is guarded by repeatable test gates at rendering, workflow, and end-to-end levels, making regressions visible before release.

## Goals

- [ ] Add/extend unit tests in DTO/domain/template contexts for PostgreSQL branches.
- [ ] Add fixture environments for PostgreSQL in `envs/` for local E2E.
- [ ] Ensure split E2E workflows validate PostgreSQL provisioning+deployment paths.
- [ ] Keep existing test runtime stable and deterministic.

## Inside-Out Execution Order

Implement this quality slice from low-level checks to full workflow gates:

1. Unit-level branch coverage for new PostgreSQL logic.
2. Rendering/integration assertions for generated artifacts and strategy branching.
3. Fixture creation for external and managed PostgreSQL modes.
4. Command-level and E2E workflow scenarios.
5. Final gate: 3-driver matrix report and flakiness baseline.

## Code-Level Implementation Details

### 0. Existing Test Surfaces To Extend

- Unit/integration tests near changed modules in `src/application/`, `src/domain/`, `src/infrastructure/`
- E2E executables in `src/bin/`
- Integration test suite in `tests/`
- Environment fixtures in `envs/`
- Backup runtime tests in `docker/backup/backup_test.bats`

### 1. Driver Matrix Assertions

For each critical branch introduced in slices A-D, enforce 3-driver matrix checks:

- sqlite3
- mysql
- postgresql

Target file groups include:

- `src/application/services/rendering/docker_compose.rs` tests
- `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs` tests
- `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs` tests
- `src/application/services/rendering/backup.rs` tests

### 2. Fixture Coverage

- Add dedicated PostgreSQL env fixtures in `envs/` (mirroring existing e2e fixture style).
- Include both external PostgreSQL and managed PostgreSQL fixture variants.
- Ensure fixture values are deterministic and compatible with local E2E execution.

### 3. E2E Coverage

- Extend E2E flow coverage for `create -> provision -> configure -> release -> run` with PostgreSQL.
- Verify generated artifacts contain expected PostgreSQL values.
- Validate service readiness and key lifecycle commands.

## Architecture Requirements

**DDD Layer**: Cross-layer tests (domain/application/infrastructure/presentation)
**Pattern**: Deterministic matrix testing with reusable fixtures

### Constraints

- [ ] Tests remain deterministic and isolated.
- [ ] New tests do not require external network by default.
- [ ] Existing sqlite/mysql suites remain unchanged and green.

## Implementation Plan

### Phase 1: Unit + Integration Matrix (1 day)

- [ ] Task 1.1: Expand DTO/domain tests for PostgreSQL branches.
- [ ] Task 1.2: Expand rendering context tests (`.env`, compose, tracker, backup config).
- [ ] Task 1.3: Add backup script bats coverage for PostgreSQL branch.

### Phase 2: Fixtures + E2E Paths (1 day)

- [ ] Task 2.1: Add PostgreSQL env fixtures in `envs/`.
- [ ] Task 2.2: Extend E2E tests to execute PostgreSQL scenarios.
- [ ] Task 2.3: Validate cleanup and idempotency behavior for repeated runs.

### Phase 3: Quality Gate Integration (0.5 day)

- [ ] Task 3.1: Ensure test/lint docs mention PostgreSQL expectations.
- [ ] Task 3.2: Capture baseline runtime and flakiness notes for new scenarios.

## Acceptance Criteria

- [ ] PostgreSQL branches are covered in unit/integration tests for changed modules.
- [ ] External and managed PostgreSQL scenarios are each covered by at least one automated test path.
- [ ] Existing sqlite/mysql E2E scenarios continue to pass.
- [ ] No flaky behavior introduced in lifecycle or backup tests.
