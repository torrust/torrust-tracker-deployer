Title: #469 Add PostgreSQL Support To Tracker Deployer

## Overview

This epic tracks all changes required in the deployer repository to support PostgreSQL as a first-class tracker persistence backend, aligned with tracker issue https://github.com/torrust/torrust-tracker/issues/1723.

This plan is intentionally organized as vertical slices. Every sub-issue must be independently deployable, independently testable, and provide user-facing value without waiting for later slices.

Current deployer behavior supports `sqlite3` and `mysql` only across:

- Environment config DTOs and schema
- Domain tracker database model
- Template rendering (`tracker.toml`, `.env`, `docker-compose.yml`)
- Release workflow and Ansible storage preparation
- Backup image/script/config rendering
- User docs and examples

The goal is parity: deployer users can select `postgresql` with the same usability and operational quality currently available for `sqlite3` and `mysql`.

## Why This Epic

The tracker now includes PostgreSQL support in progress/merged work under issue #1723 and linked PRs. If deployer remains two-driver only, we create a mismatch:

- Tracker supports PostgreSQL, deployer blocks it at schema/DTO level.
- Deployer templates and release workflow cannot provision/use PostgreSQL service.
- Backup and docs remain incomplete for PostgreSQL deployments.

This epic closes that gap.

## Scope

In scope:

- Add `postgresql` driver support in configuration model and schema.
- Add PostgreSQL service rendering in compose and env templates.
- Add PostgreSQL-aware release/ansible preparation workflow.
- Add PostgreSQL-aware backup rendering and backup runtime support (`pg_dump`).
- Add tests, fixtures, and docs for PostgreSQL.

Out of scope:

- Tracker internal database implementation (handled in tracker repo).
- Data migration from existing MySQL/SQLite deployments to PostgreSQL.
- Production hardening beyond existing deployer baseline conventions.

## Dependencies And Assumptions

- Tracker container image used by deployer supports:
  - `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=postgresql`
  - PostgreSQL default container config selection in entrypoint
- Driver string for tracker/deployer integration is exactly `postgresql`.
- Existing deployer behavior for `sqlite3` and `mysql` must remain backward compatible.

## Proposed Sub-Issues (Vertical Slices)

- [ ] 1. Vertical Slice A: BYO PostgreSQL (External DB) End-To-End
  - Child identifier: `469-01`
  - Local spec: `docs/issues/469-01-config-model-and-schema-postgresql-driver.md`
  - Outcome: users can deploy tracker against an existing external PostgreSQL endpoint.

- [ ] 2. Vertical Slice B: Managed PostgreSQL Service (Compose + Release)
  - Child identifier: `469-02`
  - Local spec: `docs/issues/469-02-compose-and-template-rendering-postgresql.md`
  - Outcome: deployer can provision and run a local PostgreSQL container in deployment artifacts.

- [ ] 3. Vertical Slice C: PostgreSQL Backup End-To-End
  - Child identifier: `469-03`
  - Local spec: `docs/issues/469-03-release-workflow-and-ansible-postgresql.md`
  - Outcome: backup subsystem supports PostgreSQL dumps and retention policies.

- [ ] 4. Vertical Slice D: PostgreSQL Failure UX, State Mapping, and Operational Guide Quality
  - Child identifier: `469-04`
  - Local spec: `docs/issues/469-04-backup-pipeline-postgresql.md`
  - Outcome: PostgreSQL-related failures are actionable in CLI/state/errors and operational workflows.

- [ ] 5. Vertical Slice E: Test Matrix and E2E Coverage for PostgreSQL
  - Child identifier: `469-05`
  - Local spec: `docs/issues/469-05-tests-fixtures-e2e-postgresql.md`
  - Outcome: stable regression protection across sqlite3/mysql/postgresql for each critical workflow.

- [ ] 6. Vertical Slice F: Documentation and Examples Parity
  - Child identifier: `469-06`
  - Local spec: `docs/issues/469-06-documentation-postgresql.md`
  - Outcome: users can confidently choose PostgreSQL from docs alone.

## Vertical Slice Rules

- Each slice must leave `main` releasable.
- Each slice must include tests for changed behavior.
- Each slice must preserve backward compatibility for `sqlite3` and `mysql`.
- Each slice must include at least one user-visible validation artifact (rendered file, command behavior, or documented workflow).

## Implementation Style Policy

Within each vertical slice, implement inside-out:

1. Runtime/template internals first, manually validated.
2. Automation/workflow wiring second (ansible/release/steps).
3. Command + presentation wiring next.
4. User-facing schema/docs exposure last.
5. Close slice only after tests and one representative end-to-end gate.

## Code-Validated Risks And Mitigations

This section is based on direct inspection of current implementation code paths.

### Risk 1: MySQL-Centric Branching Is Hard-Coded In Multiple Layers

Observed in:

- release workflow and mysql-only release step module
- docker compose rendering service branch matching only `Sqlite`/`Mysql`
- topology/service enum and show command image reporting

Mitigation:

- Introduce explicit PostgreSQL branches in all `match`/`uses_mysql()` decision points before exposing final schema contract.
- Add a shared `uses_managed_sql_service()` concept where appropriate to reduce duplicated mysql-only checks.

### Risk 2: Template Conditionals Are Coupled To `mysql` Presence

Observed in docker compose and env templates where DSN/path and `depends_on` are guarded by `mysql` blocks.

Mitigation:

- Add PostgreSQL-specific template blocks and explicit mutual exclusivity checks.
- Add snapshot assertions for sqlite/mysql/postgresql template rendering.

### Risk 3: Release Step Progress And Failure Taxonomy Are Fixed To Current Step Set

Observed in release step count, release step enum, and release error variants.

Mitigation:

- Add PostgreSQL release step(s) together with step-count and state-failure updates in one commit.
- Add tests for progress numbering and state serialization to avoid regressions in persisted state files.

### Risk 4: Ansible Static Playbook Copy List Is Closed

Observed in ansible project generator static playbook list and mysql storage playbook wiring.

Mitigation:

- Add `create-postgresql-storage.yml` and register it in static copy list and variables context/template at the same time.
- Add one generator test/assertion to ensure playbook presence in build output.

### Risk 5: Backup Pipeline Supports Only `mysql|sqlite|none`

Observed in backup config enum/template, backup rendering conversion, backup shell strategy switch, and backup image dependencies.

Mitigation:

- Add PostgreSQL support across enum/template/script/image in one slice, with bats coverage for config validation and branch dispatch.
- Add non-regression checks for mysql/sqlite artifacts and cleanup behavior.

### Risk 6: Domain/Application Types And Schema Must Evolve In Lockstep

Observed in DTO enum tags, domain `DatabaseConfig` variants, and schema `oneOf` branch definitions.

Mitigation:

- During inside-out implementation, keep schema exposure as the last phase of each slice, but never merge with mismatched DTO/domain tags.
- Add serde roundtrip tests in DTO and domain modules before final schema changes land.

## Pre-Merge Safety Checklist Per Slice

- [ ] sqlite/mysql regression tests still pass.
- [ ] new PostgreSQL branch has direct unit coverage in changed module(s).
- [ ] one representative end-to-end validation was executed for the slice outcome.
- [ ] no user-facing docs/schema statement claims behavior not yet implemented.

## Milestones

### Milestone A: External PostgreSQL Support

- Config accepts `postgresql`.
- Render/release/run works for external PostgreSQL deployments.
- Existing SQLite/MySQL paths remain green.

### Milestone B: Managed Service + Backup Parity

- Managed PostgreSQL service can be deployed with compose artifacts.
- Backup supports PostgreSQL.
- Show/status output and docker image reporting include PostgreSQL.
- Core unit/integration tests include PostgreSQL paths.

### Milestone C: Documentation Parity

- User docs show all three drivers consistently.
- Examples include PostgreSQL environment file.
- Known limitations and assumptions documented.

## Acceptance Criteria

- [ ] Deployer accepts PostgreSQL in JSON schema and runtime DTO conversion.
- [ ] Generated `tracker.toml`, `.env`, and `docker-compose.yml` support PostgreSQL correctly.
- [ ] Release workflow prepares required remote PostgreSQL storage path(s).
- [ ] Backup configuration and backup container perform PostgreSQL backups successfully.
- [ ] Existing SQLite/MySQL workflows and tests are not regressed.
- [ ] User-facing docs are updated to three-driver language and examples.

## Related

- EPIC issue: https://github.com/torrust/torrust-tracker-deployer/issues/469
- Tracker parent issue: https://github.com/torrust/torrust-tracker/issues/1723
- Local epic draft (this file)
- Local sub-issue drafts listed above
