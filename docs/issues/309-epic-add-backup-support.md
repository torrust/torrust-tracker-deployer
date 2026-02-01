# Add Backup Support

**Issue**: #309 (Epic)
**Parent Epic**: N/A (Independent feature)
**Related**:

- Roadmap: [Task 7 - Add backup support](../../roadmap.md#7-add-backup-support)
- Subtask #310: Research database backup strategies
- Reference: [Torrust Live Demo](https://github.com/torrust/torrust-tracker-live-demo) - Current SQLite backup implementation

## Overview

This epic implements backup support for Torrust Tracker deployments, enabling operators to safely backup databases (SQLite and MySQL), configuration files, and deployment state. The implementation follows an incremental approach: research → design → implementation for each backup target type.

## Problem Statement

Currently, the Torrust Tracker Deployer provides no built-in backup capabilities. Tracker operators must manually implement backup solutions, which:

- Requires technical expertise in database backup tools
- Risks data corruption if backups are performed incorrectly
- Lacks standardization across different deployment scenarios
- May not handle database concurrency issues properly

The Torrust Live Demo has a working SQLite backup implementation that provides a baseline, but it's not integrated into the deployer and doesn't cover MySQL or configuration files.

## Backup Targets

The epic covers three types of backup targets:

### 1. SQLite Database Backups

- **Scope**: SQLite database file used by the tracker
- **Challenges**: Safe copying while database is in use, no locking mechanism
- **Reference**: Torrust Live Demo implementation (daily backups, compression, 7-day rotation)
- **Considerations**: File-based backup, minimal downtime requirements

### 2. MySQL Database Backups

- **Scope**: MySQL database used by the tracker (when configured)
- **Challenges**: Concurrency control, transaction consistency, containerized environment
- **Tools**: mysqldump, mysqlpump, physical backup tools
- **Considerations**: Locking vs. hot backup trade-offs

### 3. Configuration File Backups

- **Scope**: Deployment configuration files
- **Files**: Docker Compose files, tracker config (TOML), Caddy config, Prometheus/Grafana configs
- **Challenges**: Identifying critical files, versioning, organization
- **Considerations**: Lightweight, version control integration

## Goals

- [x] Research and document backup strategies for each target type (Task 7.1) ✅ Completed
- [ ] Implement backup support with all features (Task 7.2 - consolidates original 7.2-7.5)
  - Backup container templates (Dockerfile, backup.sh)
  - Docker Compose integration (backup service)
  - Configuration schema extension
  - Ansible deployment playbooks
  - Crontab installation for scheduled backups
  - MySQL, SQLite, and config file backup support

## Research Findings Summary

The research phase (Issue #310, PR #312) recommended the **maintenance-window
hybrid approach**:

- **95% Container**: Backup logic in Docker container (portable, tested)
- **5% Host Script**: Simple crontab + shell script for scheduling
- **Architecture**: Crontab → Stop tracker → Run backup container → Start tracker
- **Artifacts**: 58 unit tests, production-ready scripts and Dockerfile

Key decision: Use maintenance-window backups for all production deployments.
The always-running sidecar approach is only viable for small databases (< 1GB).

See [Research Conclusions](../research/backup-strategies/conclusions.md) for
detailed findings.

## Implementation Approach

### Phase 1: Research ✅ COMPLETED (Task 7.1)

Research documented backup approaches for SQLite, MySQL, and configuration files:

- Evaluated safe backup techniques for each database type
- Tested with real 17GB production database (Torrust Demo)
- Built and tested POC with 58 unit tests
- Recommended maintenance-window hybrid approach

**Deliverable**: Research documentation in `docs/research/backup-strategies/` ✅

### Phase 2: Implementation (Task 7.2)

Based on research findings, implement backup support as a single integrated feature.
See [implementation spec](315-implement-backup-support.md) for the detailed implementation
plan with incremental steps.

## Context: Torrust Live Demo Implementation

The Torrust Live Demo currently implements SQLite backups with:

1. **Daily Schedule**: Automated backups once per day
2. **No Locking**: Tracker continues running during backup
3. **Compression**: Backup files are compressed after generation
4. **Rotation**: 7-day retention (keeps 1 week of backups)
5. **Local Storage**: Backup folder on same instance (convenient but not redundant)
6. **Infrastructure Backup**: Droplet snapshots provide additional layer

**Reference**: <https://github.com/torrust/torrust-tracker-live-demo>

This provides a proven baseline for SQLite backups in production, but opportunities exist for improvement:

- Redundancy: Store backups on separate volumes or remote storage
- Scope: Extend to MySQL and configuration files
- Integration: Built into the deployer rather than separate scripts

## Tasks

- [x] #310 - Research database backup strategies ✅ Completed
- [ ] #315 - Implement backup support (consolidates original tasks 7.2-7.5)

**Note**: Based on research findings (PR #312), the original tasks 7.2-7.5 have been
consolidated into a single implementation issue (see [spec](315-implement-backup-support.md)). The research provided
production-ready POC artifacts and clear implementation guidance that makes
incremental sub-issues unnecessary.

## Scope Boundaries

### In Scope

- ✅ Database backup (SQLite and MySQL)
- ✅ Configuration file backup
- ✅ User-provided backup destination paths
- ✅ Compression support
- ✅ Backup retention cleanup (configurable days)
- ✅ Scheduled backups via crontab (deployer installs crontab)
- ✅ Integration with existing command structure

### Out of Scope

- ❌ Backup encryption (future enhancement)
- ❌ Automated restore functionality (future enhancement)
- ❌ Backup testing/verification (future enhancement)
- ❌ Multi-region backup replication (users handle with volumes/cloud services)
- ❌ Volume management (users provide mounted locations)
- ❌ Backup notifications/alerts (future enhancement)

**Note**: Volume management is explicitly out of scope. Users are responsible for:

- Provisioning backup storage (cloud volumes, attached disks, etc.)
- Mounting volumes at backup destination paths
- Managing volume lifecycle, snapshots, and replication

The deployer will assume backup destinations are already mounted and accessible.

## Success Criteria

This epic is complete when:

- [x] Research is documented for all backup target types
- [ ] Backup container templates are integrated into deployer
- [ ] Backup configuration is part of environment creation schema
- [ ] Backup artifacts are deployed via Ansible playbooks
- [ ] Crontab is installed for scheduled backups
- [ ] SQLite backup is implemented and tested
- [ ] MySQL backup is implemented and tested
- [ ] Configuration file backup is implemented and tested
- [ ] All E2E tests pass
- [ ] Documentation is updated (user guide, commands reference)
- [ ] Pre-commit checks pass for all changes

## Related Documentation

- [Roadmap](../../roadmap.md) - See task 7 for complete backup feature roadmap
- [Torrust Live Demo](https://github.com/torrust/torrust-tracker-live-demo) - Reference implementation
- [DDD Architecture](../codebase-architecture.md) - Architectural guidance for implementation
- [Command Structure](../console-commands.md) - Integration with existing commands
- [Error Handling](../contributing/error-handling.md) - Error handling conventions
- [Testing Guide](../contributing/testing/) - Testing conventions and patterns

## Notes

### Why Backup Matters

The Torrust Tracker stores critical data:

- **Torrent statistics**: Upload/download counters, peer counts, torrent metadata
- **Authentication tokens**: User credentials and access control
- **Whitelisted torrents**: List of allowed torrents (if private tracker)

**Note**: Peer information (active peers and swarm state) is kept in memory only
and is not persisted to the database, so it cannot be backed up.

Losing this data would be catastrophic for tracker operators. Backups must be:

- **Safe**: Don't corrupt the database during backup
- **Reliable**: Actually work when you need to restore
- **Automated**: Don't rely on manual intervention
- **Redundant**: Survive infrastructure failures

### Design Principles

The backup implementation will follow these principles:

1. **Safety First**: Never risk data corruption during backup
2. **User Control**: Users provide backup destinations and manage storage
3. **Flexibility**: Support different backup scopes and strategies
4. **Integration**: Fit naturally into existing command structure
5. **Transparency**: Clear logging and error reporting

### Future Enhancements

After the initial implementation, future enhancements could include:

- Automated restore functionality
- Backup encryption (at-rest and in-transit)
- Backup verification and testing
- Point-in-time recovery
- Backup retention policies
- Cross-region replication support
- Backup scheduling integration

These are out of scope for the initial epic but documented for future reference.
