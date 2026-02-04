# Phase 4: Documentation & E2E Testing Plan

**Issue**: #315 - Implement Backup Support
**Phase**: 4 (Documentation and Final Testing)
**Date**: February 4, 2026
**Status**: ~95% Complete (Parts 1 & 2 nearly finished, Part 2.2 remaining)

## Overview

Phase 4 completes the backup feature implementation by adding comprehensive user documentation and automated E2E tests. This ensures users can effectively use the backup feature and that future changes don't break backup functionality.

---

## Part 1: Documentation

All documentation should follow the existing project structure. Do NOT duplicate content in `docs/console-commands.md`.

### 1.1 Create `docs/user-guide/backup.md` ✅ COMPLETE

**Purpose**: Comprehensive user guide for the backup feature

**Content to cover**:

- Overview: What backup does, why it's important
- Key features: automatic, scheduled, configurable retention
- Supported database types: MySQL, SQLite
- Configuration options: schedule and retention_days fields with constraints
- How it works: two-phase backup system (initial + scheduled)
- Backup file storage and naming patterns
- Monitoring and verification procedures
- Troubleshooting common issues
- Recovery procedures (note: future enhancement)
- Configuration examples (default, multiple backups, weekly, disabled)

### 1.2 Update command documentation ✅ COMPLETE

Add backup configuration examples to existing command files:

- **`docs/user-guide/commands/create.md`**: Add backup configuration section to the `create environment` command ✅
- **`docs/user-guide/commands/release.md`**: Document how backup service is deployed during release ✅
- **`docs/user-guide/commands/run.md`**: Document initial backup behavior during `run` command ✅

### 1.3 Update `docs/user-guide/README.md` ✅ COMPLETE

Add navigation link to the new backup guide:

- Link to `backup.md`
- Brief description: "Automatic database and configuration backups with configurable retention"

### 1.4 Update configuration documentation ✅ COMPLETE

Update existing configuration schema/reference documentation to include backup section:

- Backup configuration in `schemas/environment-config.json` ✅ (auto-generated from Rust types)
- Comprehensive backup configuration guide in `docs/user-guide/backup.md` ✅
- Configuration field constraints documented in schema ✅
- Examples for different use cases provided in multiple locations ✅

---

## Part 2: E2E Tests ✅ COMPLETE

### 2.1 Simple Backup Verification ✅ COMPLETE

**Test Suite**: Simple backup verification integrated into existing E2E tests

#### Integration Approach

Rather than creating complex standalone test scenarios, add backup validation to the existing E2E deployment workflow:

**Key Implementation**:

1. **Update `run_run_validation()` function** in `src/testing/e2e/tasks/run_run_validation.rs`:
   - Add optional backup validation parameter
   - When enabled, verify backup files exist in `/opt/torrust/storage/backup/`
   - Check for at least one backup file (config, mysql, or sqlite)
   - Return helpful error message if no backups found

2. **Update `src/bin/e2e_deployment_workflow_tests.rs`**:
   - Enable backup validation in the `run_run_validation` call
   - This verifies backups are created as part of the full deployment workflow

**What this validates**:

- Initial backup was created during release phase
- Backup directory structure was created
- At least one backup file exists (proves backup container ran)
- Works for both MySQL and SQLite configurations

**Simple and effective**: The existing E2E deployment workflow (create → provision → configure → release → run) already exercises the full backup feature. By adding one verification step to check backup files exist, we validate that backup functionality is working without needing complex standalone tests.

---

### 2.2 Update Existing E2E Tests

- **`tests/e2e_integration.rs`**: Ensure backup doesn't break integration tests
- **`tests/template_integration.rs`**: Verify `create template` generates backup section with correct defaults

### 2.3 Update Manual Testing Documentation ✅ COMPLETE

Update `docs/e2e-testing/manual/backup-verification.md` with step-by-step procedures:

- Prepare environment ✅
- Deploy stack ✅
- Verify initial backup ✅
- Check crontab installation ✅
- Trigger manual backup ✅
- Monitor logs ✅
- Verify database backup ✅
- Test retention cleanup ✅

---

## Implementation Order

Recommended sequence:

1. Create `docs/user-guide/backup.md` (comprehensive guide)
2. Update `docs/user-guide/commands/create.md` (backup configuration)
3. Update `docs/user-guide/commands/release.md` (backup deployment)
4. Update `docs/user-guide/commands/run.md` (initial backup behavior)
5. Update `docs/user-guide/README.md` (navigation)
6. Update configuration documentation (schema)
7. Add backup validation to `run_run_validation()` (automated verification)
8. Update `e2e_deployment_workflow_tests.rs` to enable backup validation
9. Update existing E2E tests (integration verification)
10. Update manual testing docs (verification procedures)
11. Run all tests and fix issues
12. Commit and document completion

---

## Success Criteria

**Documentation**:

- ✅ Users can understand how to enable backup
- ✅ Users have complete configuration reference
- ✅ Users have troubleshooting guide for common issues
- ✅ Users have examples of different configurations
- ✅ Command documentation includes backup examples
- ✅ No duplication in `docs/console-commands.md`

**E2E Tests**:

- ✅ Backup verification integrated into existing E2E deployment workflow
- ✅ Initial backup creation validated after full deployment
- ✅ Works with both MySQL and SQLite configurations
- ✅ Simple and maintainable - verifies happy path
- ✅ Existing tests still pass (no regressions)

**Manual Testing**:

- ✅ Users have step-by-step guide to verify backup
- ✅ Troubleshooting section covers common problems
- ✅ Log inspection procedures documented
- ✅ Recovery procedures documented

---

## Notes

- **Documentation Structure**: Follow existing project structure - use `docs/user-guide/` for user guides and `docs/user-guide/commands/` for command-specific documentation
- **No Console Commands Duplication**: Update individual command files instead of `docs/console-commands.md`
- **Retention Cleanup**: Automatically runs after each backup - users should verify logs
- **Downtime**: Backup requires briefly stopping tracker (10-15 seconds) - schedule accordingly
- **Storage**: Backups are compressed but can consume significant space over time
- **Database-Specific**: MySQL and SQLite have different backup procedures - documentation should clearly explain differences
