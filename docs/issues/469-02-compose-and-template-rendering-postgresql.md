# Vertical Slice B: Managed PostgreSQL Service (Compose + Release)

**Issue**: TBD (`469-02` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `templates/docker-compose/.env.tera`
- `templates/docker-compose/docker-compose.yml.tera`
- `templates/ansible/variables.yml.tera`
- `templates/ansible/create-mysql-storage.yml`
- `src/application/services/rendering/docker_compose.rs`
- `src/application/command_handlers/release/workflow.rs`
- `src/application/command_handlers/release/steps/mysql.rs`
- `src/domain/topology/service.rs`
- `src/domain/environment/state/release_failed.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`

## Overview

Extend the external PostgreSQL support from Slice A to a fully managed local PostgreSQL deployment mode, including compose service definition, topology wiring, and release-time remote storage preparation.

Generated artifacts and workflow should be sufficient to deploy PostgreSQL alongside tracker on the target host.

## Deployable Outcome

After this slice, users can run the full workflow using PostgreSQL with local service provisioning:

- compose file includes `postgresql` service
- tracker depends on healthy `postgresql`
- release prepares PostgreSQL storage directory on remote host
- `show`/status-level service metadata includes PostgreSQL image when applicable

## Goals

- [ ] Render `.env` with PostgreSQL DSN overrides.
- [ ] Render `docker-compose.yml` with PostgreSQL service and healthcheck.
- [ ] Add release workflow steps to prepare PostgreSQL storage on remote host.
- [ ] Add topology/service model support for PostgreSQL.
- [ ] Preserve current SQLite/MySQL behavior.

## Inside-Out Execution Order

Implement this slice from runtime internals toward user-facing contract:

1. Templating/runtime internals: implement managed PostgreSQL compose/env rendering and validate by running generated artifacts manually.
2. Automation and workflow: add ansible storage prep and release step wiring for managed PostgreSQL.
3. Command wiring: ensure command handlers route managed PostgreSQL mode through new workflow branches.
4. Presentation and errors: expose service/image/status and actionable failure messages.
5. Schema/contract refinement: confirm user-facing config contract remains aligned with implemented managed mode.
6. Slice gate: tests plus one managed PostgreSQL end-to-end run before closing.

## Code-Level Implementation Details

### 0. Existing Code Paths To Extend

- Compose env context: `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`
- Compose builder/database context: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/database.rs`, `.../builder.rs`, `.../mod.rs`
- Compose rendering service: `src/application/services/rendering/docker_compose.rs`
- Topology service enum: `src/domain/topology/service.rs`
- Release workflow + step modules: `src/application/command_handlers/release/workflow.rs`, `src/application/command_handlers/release/steps/mysql.rs`
- Release state steps: `src/domain/environment/state/release_failed.rs`
- Ansible templates + static copy list: `templates/ansible/variables.yml.tera`, `templates/ansible/create-mysql-storage.yml`, `src/infrastructure/templating/ansible/template/renderer/project_generator.rs`
- Show command docker image logic: `src/application/command_handlers/show/handler.rs`

- Correct tracker database driver override
- Correct PostgreSQL DSN override
- Optional local PostgreSQL service definition
- Correct service dependency and healthcheck behavior

## Architecture Requirements

**DDD Layer**: Application + Infrastructure
**Module Path**: `src/application/services/rendering/`, `src/infrastructure/templating/docker_compose/`, `templates/`
**Pattern**: Context builder + Tera templating

### Architectural Constraints

- [ ] No secrets hardcoded into template files.
- [ ] DSN generation escapes reserved URL characters in user/password.
- [ ] Generated output remains deterministic in field ordering and optional blocks.

## Specifications

### 1. EnvContext + DSN

Extend `EnvContext` with PostgreSQL mode:

- Add optional `postgresql` service config block analogous to `mysql`.
- Add constructor similar to `new_with_mysql`, e.g. `new_with_postgresql(...)`.
- Ensure DSN format uses `postgresql://user:pass@host:port/database`.
- Keep percent-encoding behavior for user/password.

### 2. Docker Compose Context + Templates

Extend builder/context to support PostgreSQL service setup:

- Add PostgreSQL setup config type.
- Add `with_postgresql(...)` on builder.
- Add optional `postgresql` service context in final compose context.

### 3. Template Updates

#### `.env.tera`

- Render `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER='postgresql'` when PostgreSQL selected.
- Render `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` with PostgreSQL DSN.
- Add PostgreSQL service env vars (container-specific) when local PostgreSQL service is enabled.

#### `docker-compose.yml.tera`

- Add `postgresql` service block with:
  - image
  - environment
  - volume mount path
  - healthcheck
  - database network
- Make tracker `depends_on` target `postgresql` in PostgreSQL mode.
- Ensure MySQL and PostgreSQL blocks are mutually exclusive per selected driver.

### 3. Topology + Show Metadata

- Add `Service::PostgreSQL` in `src/domain/topology/service.rs` and include serialization/name/all() updates.
- Update topology derivations that currently check `Service::MySQL` when behavior should apply to any managed SQL service.
- Update show command (`show/handler.rs`) to include PostgreSQL image when tracker uses managed PostgreSQL.

### 4. Release + Ansible Storage Preparation

- Add `postgresql_enabled` in `templates/ansible/variables.yml.tera`.
- Add `templates/ansible/create-postgresql-storage.yml` (parallel to mysql playbook).
- Register new playbook in static template copy list (`project_generator.rs`).
- Add release step module for PostgreSQL storage prep (parallel to `steps/mysql.rs`).
- Add state step and error mapping for PostgreSQL storage failures.

### 5. Rendering Service Wiring

Update `DockerComposeTemplateRenderingService` decision tree to handle 3-way DB selection:

- sqlite -> existing path
- mysql -> existing path
- postgresql -> new path (managed mode)

## Implementation Plan

### Phase 1: Compose Context + Templates (1 day)

- [ ] Task 1.1: Extend env context and compose context builders for managed PostgreSQL.
- [ ] Task 1.2: Update `.env.tera` and `docker-compose.yml.tera` blocks.
- [ ] Task 1.3: Add rendering tests for managed PostgreSQL snapshots.

### Phase 2: Release + Ansible (1 day)

- [ ] Task 2.1: Add `create-postgresql-storage.yml` + ansible variable wiring.
- [ ] Task 2.2: Add PostgreSQL release step + workflow integration + error mapping.
- [ ] Task 2.3: Add release step unit tests (skip/execute/failure mapping).

### Phase 3: Command + Topology + Show (0.5 day)

- [ ] Task 3.1: Add `Service::PostgreSQL` and update topology-related tests.
- [ ] Task 3.2: Update show command docker image reporting.
- [ ] Task 3.3: Verify command-handler routing for managed PostgreSQL path.

### Phase 4: Presentation + Contract Check + E2E Gate (0.5 day)

- [ ] Task 4.1: Validate PostgreSQL-specific release failure messages are actionable.
- [ ] Task 4.2: Confirm schema/docs contract is still aligned with managed mode behavior.
- [ ] Task 4.3: Run one managed PostgreSQL end-to-end scenario.

## Acceptance Criteria

- [ ] Managed PostgreSQL deployments render valid `.env` and compose files with healthy dependency checks.
- [ ] Release workflow creates required PostgreSQL storage directories on remote host.
- [ ] Topology and show output include PostgreSQL where appropriate.
- [ ] SQLite/MySQL snapshots and release behavior remain unchanged unless intentionally updated.
- [ ] New/updated tests for templates, topology, and release step are passing.
