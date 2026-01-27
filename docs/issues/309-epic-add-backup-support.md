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

- [ ] Research and document backup strategies for each target type (Task 7.1)
- [ ] Design backup feature specification with configuration schema (Task 7.2)
- [ ] Implement configuration file backup functionality (Task 7.3)
- [ ] Implement SQLite database backup functionality (Task 7.4)
- [ ] Implement MySQL database backup functionality (Task 7.5)

## Implementation Approach

### Phase 1: Research (Task 7.1)

Research and document backup approaches for SQLite, MySQL, and configuration files. Focus on understanding:

- Available tools and techniques
- Trade-offs between different approaches
- Cloud redundancy strategies
- Current Torrust Live Demo implementation

**Deliverable**: Research documentation in `docs/research/backup-strategies/`

### Phase 2: Specification (Task 7.2)

Design the backup feature specification based on research findings:

- Backup configuration schema (per-service-type enablement)
- User-provided destination paths
- Integration with existing command structure
- DDD architecture planning

**Deliverable**: Feature specification document

### Phase 3: Configuration Backups (Task 7.3)

Implement backup support for configuration files:

- Identify and document all configuration files
- Create backup commands/steps
- Support user-provided backup destinations
- Test with E2E scenarios

**Deliverable**: Working configuration backup functionality

### Phase 4: SQLite Backups (Task 7.4)

Implement SQLite backup based on research and Live Demo approach:

- Safe file copying mechanism
- Compression support
- Integration with deployment workflow
- E2E testing

**Deliverable**: Working SQLite backup functionality

### Phase 5: MySQL Backups (Task 7.5)

Implement MySQL backup based on research findings:

- Tool selection (mysqldump, physical backup, etc.)
- Concurrency handling
- Docker container integration
- E2E testing with MySQL deployments

**Deliverable**: Working MySQL backup functionality

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

- [ ] #310 - Research database backup strategies
- [ ] #TBD - Define backup feature specification
- [ ] #TBD - Implement configuration file backups
- [ ] #TBD - Implement SQLite database backups
- [ ] #TBD - Implement MySQL database backups

(Tasks will be created and linked as work progresses)

## Scope Boundaries

### In Scope

- ✅ Database backup (SQLite and MySQL)
- ✅ Configuration file backup
- ✅ User-provided backup destination paths
- ✅ Compression support
- ✅ Integration with existing command structure

### Out of Scope

- ❌ Automated backup scheduling (users can use cron/systemd timers)
- ❌ Backup encryption (future enhancement)
- ❌ Automated restore functionality (future enhancement)
- ❌ Backup testing/verification (future enhancement)
- ❌ Multi-region backup replication (users handle with volumes/cloud services)
- ❌ Backup retention policies (users manage retention)
- ❌ Volume management (users provide mounted locations)

**Note**: Volume management is explicitly out of scope. Users are responsible for:

- Provisioning backup storage (cloud volumes, attached disks, etc.)
- Mounting volumes at backup destination paths
- Managing volume lifecycle, snapshots, and replication

The deployer will assume backup destinations are already mounted and accessible.

## Success Criteria

This epic is complete when:

- [ ] Research is documented for all backup target types
- [ ] Feature specification is complete and approved
- [ ] Configuration file backup is implemented and tested
- [ ] SQLite backup is implemented and tested (compatible with Live Demo approach)
- [ ] MySQL backup is implemented and tested
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

- **Peer information**: Active peers and swarm state
- **Torrent statistics**: Upload/download counters, peer counts
- **Authentication tokens**: User credentials and access control

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
