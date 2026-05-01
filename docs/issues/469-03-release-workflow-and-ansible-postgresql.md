# Vertical Slice C: PostgreSQL Backup End-To-End

**Issue**: TBD (`469-03` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `templates/backup/backup.conf.tera`
- `src/application/services/rendering/backup.rs`
- `src/infrastructure/templating/backup/template/wrapper/backup_config/context.rs`
- `docker/backup/backup.sh`
- `docker/backup/Dockerfile`
- `docker/backup/backup_test.bats`

## Overview

Deliver independently deployable PostgreSQL backup support for environments using PostgreSQL, including rendered configuration, backup runtime behavior, retention cleanup, and tests.

## Deployable Outcome

After this slice, environments using PostgreSQL can run backup jobs that produce compressed SQL dump artifacts with retention cleanup behavior equivalent to existing mysql/sqlite support.

## Goals

- [ ] Render backup config with PostgreSQL DB settings.
- [ ] Run PostgreSQL dumps via backup container.
- [ ] Keep cleanup/retention behavior deterministic and idempotent.
- [ ] Preserve existing mysql/sqlite behavior.

## Inside-Out Execution Order

Implement this slice from runtime internals toward user-facing contract:

1. Runtime tooling internals: implement PostgreSQL branch in backup script and Docker image dependencies.
2. Rendering/automation: wire backup config rendering for PostgreSQL.
3. Command/workflow wiring: ensure release/run workflows activate backup path correctly.
4. Presentation and diagnostics: ensure logs/errors are actionable and non-sensitive.
5. Contract/docs check: validate user-facing backup behavior matches docs/config expectations.
6. Slice gate: tests plus one real PostgreSQL backup/restore smoke scenario before closing.

## Code-Level Implementation Details

### 0. Existing Code Paths To Extend

- Backup context enum: `src/infrastructure/templating/backup/template/wrapper/backup_config/context.rs`
- DB mapping function: `src/application/services/rendering/backup.rs` (`convert_database_config_to_backup`)
- Backup template: `templates/backup/backup.conf.tera`
- Backup runtime: `docker/backup/backup.sh`
- Backup image deps: `docker/backup/Dockerfile`
- Backup tests: `docker/backup/backup_test.bats`

### 1. Render Layer

- Add `BackupDatabaseConfig::Postgresql { host, port, database, user, password }`.
- Extend `convert_database_config_to_backup` to map `DatabaseConfig::Postgresql`.
- Update backup template to render:
  - `DB_TYPE=postgresql`
  - `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`

### 2. Runtime Backup Script

In `backup.sh`:

- Add `postgresql` case in `case "$DB_TYPE"`.
- Add `validate_postgresql_config` checking required env vars.
- Add `backup_postgresql` using `pg_dump` piped to `gzip`.
- Add backup directory + cleanup function for PostgreSQL artifacts.

File naming convention:

- `/backups/postgresql/postgresql_YYYYMMDD_HHMMSS.sql.gz`

### 3. Backup Image

In `docker/backup/Dockerfile`:

- Install package providing `pg_dump` (e.g. `postgresql-client`).
- Create `/backups/postgresql` directory with correct ownership.

### 4. Test Requirements

In `backup_test.bats`:

- Add configuration-load tests for `DB_TYPE=postgresql`.
- Add validation tests for missing PostgreSQL vars.
- Add dispatch tests ensuring PostgreSQL branch is selected.
- Keep mysql/sqlite tests unchanged and green.

## Architecture Requirements

**DDD Layer**: Application + Infrastructure + Runtime shell tooling
**Pattern**: Config rendering + runtime DB strategy switch

### Constraints

- [ ] No secret values printed in logs.
- [ ] Cleanup logic must remain safe if directories are missing.
- [ ] Backward compatibility for mysql/sqlite backup naming and behavior.

## Implementation Plan

### Phase 1: Runtime Support (1 day)

- [ ] Task 1.1: Add PostgreSQL config validation.
- [ ] Task 1.2: Add `backup_postgresql` operation.
- [ ] Task 1.3: Add retention cleanup branch for PostgreSQL artifacts.

### Phase 2: Image + Render Support (0.5 day)

- [ ] Task 2.1: Add PostgreSQL client package and backup directory.
- [ ] Task 2.2: Extend backup context enum.
- [ ] Task 2.3: Extend DB mapping logic.
- [ ] Task 2.4: Update backup template conditionals.

### Phase 3: Workflow + Presentation Checks (0.5 day)

- [ ] Task 3.1: Verify command/workflow path selects PostgreSQL backup branch correctly.
- [ ] Task 3.2: Ensure error logs remain actionable and do not leak secrets.

### Phase 4: Tests + E2E Gate (0.5 day)

- [ ] Task 4.1: Extend `backup_test.bats` for PostgreSQL branch.
- [ ] Task 4.2: Re-run mysql/sqlite backup tests to confirm no regressions.
- [ ] Task 4.3: Run one PostgreSQL backup smoke test and verify artifact convention.

## Acceptance Criteria

- [ ] Backup config renders valid PostgreSQL settings.
- [ ] Backup runtime can generate PostgreSQL dumps when `DB_TYPE=postgresql`.
- [ ] Cleanup removes expired PostgreSQL backups using retention policy.
- [ ] Existing mysql/sqlite backup behavior remains unchanged and tested.
