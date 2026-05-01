# Vertical Slice A: BYO PostgreSQL (External DB) End-To-End

**Issue**: TBD (`469-01` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `schemas/environment-config.json`
- `src/application/command_handlers/create/config/tracker/tracker_core_section.rs`
- `src/domain/tracker/config/core/database/mod.rs`

## Overview

Deliver the minimum independently deployable PostgreSQL feature: users can configure the deployer to use an existing external PostgreSQL server (Bring Your Own DB), render valid artifacts, and run tracker without managed local PostgreSQL service support.

This slice intentionally excludes local PostgreSQL container provisioning and backup support. Those are delivered in later slices.

## Deployable Outcome

After this slice, a user can provide:

- `driver = postgresql`
- `host`, `port`, `database_name`, `username`, `password`

and successfully run `create -> provision -> configure -> release -> run` using an external PostgreSQL endpoint.

## Goals

- [ ] JSON schema validates PostgreSQL database configuration.
- [ ] DTO conversion supports PostgreSQL.
- [ ] Domain database enum includes PostgreSQL variant and constants.
- [ ] Rendered `.env` and `tracker.toml` support PostgreSQL DSN override path.
- [ ] Validation errors are actionable and consistent with existing style.

## Inside-Out Execution Order

Implement this slice from runtime internals toward user-facing contract:

1. Templating/runtime internals: add PostgreSQL DSN and driver mapping in rendering contexts and manually validate rendered artifacts with a controlled input.
2. Automation and workflow: ensure release flow does not accidentally require managed DB service for external PostgreSQL mode.
3. Command wiring: complete DTO/domain conversion so runtime path is exercised through command handlers.
4. Presentation and errors: ensure user-visible output and validation errors are actionable.
5. Schema hardening: finalize JSON schema branch after runtime behavior is proven.
6. Slice gate: run tests plus one end-to-end external PostgreSQL scenario before closing.

## Architecture Requirements

**DDD Layer**: Domain + Application
**Module Path**: `src/domain/tracker/config/core/database/`, `src/application/command_handlers/create/config/tracker/`
**Pattern**: Value objects + DTO-to-domain conversion

### Module Structure Requirements

- [ ] Keep database-driver-specific validation in domain config modules.
- [ ] Keep schema/DTO constraints aligned with domain invariants.
- [ ] Do not leak infrastructure concerns into domain validation.

### Architectural Constraints

- [ ] No breaking changes to serialized format for existing environments.
- [ ] Existing persisted environment files remain loadable.
- [ ] Error text stays user-oriented and action-guiding.

## Code-Level Implementation Details

### 0. Existing Code Paths To Extend

- Schema: `schemas/environment-config.json`
- DTO enum + conversion: `src/application/command_handlers/create/config/tracker/tracker_core_section.rs`
- Domain DB enum: `src/domain/tracker/config/core/database/mod.rs`
- Domain DB types: `src/domain/tracker/config/core/database/mysql.rs` and `sqlite.rs` (reference patterns)
- Tracker context driver mapping: `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs`
- Env DSN generation: `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`
- Compose rendering service branch: `src/application/services/rendering/docker_compose.rs`

### 1. Schema Changes

Extend `DatabaseSection` oneOf in `schemas/environment-config.json` with a PostgreSQL branch:

- `driver`: `postgresql`
- Required fields:
  - `host`
  - `port`
  - `database_name`
  - `username`
  - `password`
- No `ssl_mode` field in this slice.

Rationale: keep DTO/domain/rendering aligned with currently supported tracker override shape and avoid introducing config keys not consumed end-to-end.

### 2. DTO Changes (`tracker_core_section.rs`)

Update `DatabaseSection` enum in `tracker_core_section.rs` to add `Postgresql { ... }`, mirroring MySQL layout with PostgreSQL naming.

Implement `TryFrom<DatabaseSection> for DatabaseConfig` branch to produce domain `DatabaseConfig::Postgresql(...)`.

### 3. Domain Changes (`core/database`)

In `src/domain/tracker/config/core/database/`:

- Add `postgresql.rs` with `PostgresqlConfig` and `PostgresqlConfigError`.
- Add `DRIVER_POSTGRESQL` constant.
- Add `DatabaseConfig::Postgresql(PostgresqlConfig)` variant.
- Update helper methods:
  - `driver_name()`
  - `database_name()`
  - `docker_image()` behavior returns `None` in this slice for PostgreSQL, because this slice is external DB only.

Validation should mirror MySQL quality:

- non-empty host
- port != 0
- non-empty database_name
- non-empty username
- non-empty password
- no reserved username policy in this slice

### 4. Rendering Changes (External DB Mode)

- Extend `DatabaseDriver` enum in tracker context (`tracker_config/context.rs`) with `Postgresql` and map `DatabaseConfig::Postgresql(..)`.
- Extend env context (`env/context.rs`) with PostgreSQL DSN constructor (parallel to `new_with_mysql`) but without PostgreSQL service block.
- Update `DockerComposeTemplateRenderingService` (`rendering/docker_compose.rs`) to route PostgreSQL DB to env/tracker context creation and to avoid enabling DB service dependency.
- Keep `templates/tracker/tracker.toml.tera` as SQL-driver DSN override model (`path` only for sqlite3).

### 5. Explicit Non-Goals In This Slice

- No `postgresql` service section in `docker-compose.yml.tera`.
- No release step creating PostgreSQL storage directories.
- No backup support for PostgreSQL.

### 6. Backward Compatibility

Confirm existing environment files deserialize unchanged for:

- `driver=sqlite3`
- `driver=mysql`

## Implementation Plan

### Phase 1: Runtime Rendering Internals (0.5 day)

- [ ] Task 1.1: Extend tracker context driver enum/mapping.
- [ ] Task 1.2: Add PostgreSQL DSN constructor in env context.
- [ ] Task 1.3: Extend docker compose rendering service branch logic for external mode.

### Phase 2: Domain + Command Wiring (1 day)

- [ ] Task 2.1: Add `postgresql.rs` domain config with invariants.
- [ ] Task 2.2: Add `DatabaseSection::Postgresql` in DTO enum.
- [ ] Task 2.3: Extend DTO-to-domain conversion.
- [ ] Task 2.4: Confirm workflow path treats this slice as external DB mode.

### Phase 3: Presentation + Schema Finalization (0.5 day)

- [ ] Task 3.1: Ensure actionable validation/output text for PostgreSQL config errors.
- [ ] Task 3.2: Add PostgreSQL branch to `schemas/environment-config.json` as final contract exposure.

### Phase 4: Tests + Regression Safety (1 day)

- [ ] Task 4.1: Add DTO serde/try_from tests in `tracker_core_section.rs` test module.
- [ ] Task 4.2: Add domain config tests in `core/database/mod.rs` + `postgresql.rs`.
- [ ] Task 4.3: Add env context DSN encoding tests in `env/context.rs`.
- [ ] Task 4.4: Add tracker context mapping tests in `tracker_config/context.rs`.
- [ ] Task 4.5: Re-run existing sqlite/mysql tests touched by enum branching.
- [ ] Task 4.6: Run one end-to-end external PostgreSQL scenario.

## Acceptance Criteria

- [ ] `EnvironmentCreationConfig` accepts valid PostgreSQL external-db config.
- [ ] Rendering produces PostgreSQL DSN override for tracker.
- [ ] Deployment works when PostgreSQL is externally available.
- [ ] Invalid PostgreSQL config yields clear validation errors.
- [ ] Existing SQLite/MySQL config parsing and rendering remain unchanged.
- [ ] Unit/integration tests for added branches are present and passing.

## Notes

Keep naming canonical as `postgresql` (not `postgres`) unless tracker integration proves otherwise.
