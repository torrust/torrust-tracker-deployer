# Vertical Slice F: Documentation and Examples Parity

**Issue**: TBD (`469-06` planned child subissue)
**Parent Epic**: #469 - Add PostgreSQL Support To Tracker Deployer
**Related**:

- `docs/user-guide/README.md`
- `docs/user-guide/commands/*.md`
- `docs/user-guide/backup.md`
- `docs/user-guide/security.md`
- `docs/external-issues/tracker/database-driver-double-specification.md`

## Overview

Update user and contributor documentation so deployer database support is consistently documented as `sqlite3`, `mysql`, and `postgresql`, including examples, operational notes, and limitations.

This slice is independently deployable because it ensures users can adopt PostgreSQL features from slices A-E without relying on tribal knowledge.

## Deployable Outcome

After this slice, a new user can choose between sqlite3/mysql/postgresql and complete the workflow using only repository documentation.

## Goals

- [ ] Replace two-driver wording with three-driver wording where applicable.
- [ ] Add PostgreSQL examples in key user workflows.
- [ ] Update backup/security docs for PostgreSQL internals.
- [ ] Clarify tracker image/version dependency for PostgreSQL support.

## Inside-Out Execution Order

Document this slice from implementation reality outward:

1. Validate final runtime/template/workflow behavior from completed implementation slices.
2. Update operational docs (commands, backup, security) tied to verified behavior.
3. Update examples/config snippets for external and managed PostgreSQL.
4. Perform consistency pass against schema/templates/tests to remove stale statements.
5. Final gate: walkthrough where a new user can complete a PostgreSQL flow using docs only.

## Code/Artifact References To Keep In Sync

- Schema model: `schemas/environment-config.json`
- Compose templates: `templates/docker-compose/.env.tera`, `templates/docker-compose/docker-compose.yml.tera`
- Tracker template: `templates/tracker/tracker.toml.tera`
- Backup templates/runtime: `templates/backup/backup.conf.tera`, `docker/backup/backup.sh`
- Release workflow + ansible playbooks for managed DB storage

## Architecture Requirements

**DDD Layer**: Documentation (cross-cutting)
**Pattern**: User guide parity + operational actionability

### Constraints

- [ ] Keep docs accurate to implemented behavior only.
- [ ] Mark assumptions/dependencies explicitly when dependent on tracker image versions.
- [ ] Avoid contradicting existing security model docs.

## Specifications

### 1. User Guide Core

Update references in:

- top-level user guide summaries
- commands create/release/render/run
- services/security sections

Target docs should include at minimum:

- `docs/user-guide/README.md`
- `docs/user-guide/commands/`
- `docs/console-commands.md`
- `docs/deployment-overview.md`

Examples should include PostgreSQL where side-by-side driver comparison is presented.

### 2. Backup Documentation

Update backup docs to include:

- PostgreSQL backup file naming/location
- PostgreSQL troubleshooting section
- restore note(s) for PostgreSQL dump format

Target docs:

- `docs/user-guide/backup/` (or nearest backup user docs)
- `docker/backup/README.md`

### 3. Security Documentation

Update network/isolation examples to include PostgreSQL service in database network narrative.

Target docs:

- `docs/user-guide/security/` or equivalent security guides
- `docs/security/` notes if they reference DB topology

### 4. External Issue Notes

Review and update tracker-entrypoint note doc if now stale after tracker PostgreSQL support:

- if still relevant, update to include PostgreSQL branch
- if obsolete, mark historical with status note and current behavior

### 5. Example Configs

Add/mention PostgreSQL environment config example in docs and quick references.

Examples should cover:

- external PostgreSQL mode
- managed PostgreSQL mode
- backup-enabled PostgreSQL mode (if enabled by implementation)

## Implementation Plan

### Phase 1: Inventory and Global Wording Updates (0.5 day)

- [ ] Task 1.1: Find all SQLite/MySQL-only statements.
- [ ] Task 1.2: Update to three-driver language where true.

### Phase 2: Examples and How-To Updates (0.5 day)

- [ ] Task 2.1: Add PostgreSQL example snippets in create/render docs.
- [ ] Task 2.2: Update backup and security examples.

### Phase 3: Validation and Consistency Pass (0.5 day)

- [ ] Task 3.1: Ensure docs reflect actual generated file/service names.
- [ ] Task 3.2: Cross-check with implemented behavior and tests.

## Acceptance Criteria

- [ ] User-facing docs consistently describe three database drivers.
- [ ] PostgreSQL appears in concrete examples for external and managed workflows.
- [ ] Backup and security docs include PostgreSQL operational notes.
- [ ] No stale statement remains claiming deployer supports only sqlite/mysql.
