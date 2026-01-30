# Backup Requirements Notes

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)
**Parent Epic**: [#309 - Add backup support](https://github.com/torrust/torrust-tracker-deployer/issues/309)

## Overview

This document captures requirements and constraints collected during research discussions. These notes inform the research direction but do not define implementation decisions.

## Backup Types

The deployer may support different backup strategies with different trade-offs:

### 1. Storage Backup (Complete)

Backs up the entire storage directory including configuration files and database files.

| Aspect              | Description                                                                |
| ------------------- | -------------------------------------------------------------------------- |
| **What's included** | Everything: config files, database files, logs, etc.                       |
| **Pros**            | Simple, complete, easy to restore                                          |
| **Cons**            | Larger backups, database files may be inconsistent if copied during writes |
| **Use case**        | Full system restore, disaster recovery                                     |

### 2. Database Backup

Backs up only the database using database-specific tools that ensure consistency.

#### SQLite

| Aspect       | Description                                         |
| ------------ | --------------------------------------------------- |
| **Tool**     | `sqlite3 database.db ".backup backup.db"`           |
| **Pros**     | Safe, consistent, handles WAL mode                  |
| **Cons**     | Requires `sqlite3` CLI                              |
| **Research** | See [sqlite/](sqlite/) folder for detailed research |

#### MySQL

| Aspect       | Description                        |
| ------------ | ---------------------------------- |
| **Tool**     | `mysqldump`, hot backup tools      |
| **Pros**     | Database-aware, consistent         |
| **Cons**     | More complex, requires credentials |
| **Research** | To be documented                   |

### 3. Selective Backup (Partial Storage)

Backs up only specific parts of the storage directory.

| Aspect              | Description                                                            |
| ------------------- | ---------------------------------------------------------------------- |
| **What's included** | Selected files only (e.g., config files, but not raw DB files)         |
| **Pros**            | Smaller backups, faster, avoids redundancy                             |
| **Cons**            | More complex to configure, requires coordination with database backups |
| **Use case**        | When database backup is handled separately                             |

**Example**: If database backups are done with `sqlite3 .backup`, the selective backup could copy only:

- Configuration files (TOML, JSON)
- SSL certificates
- Custom scripts
- But NOT the raw `.db` files (already backed up safely)

## General Requirements

These requirements apply to all backup types.

### Data Loss Tolerance

**Requirement**: No data loss is acceptable.

**Context**: The tracker persists critical data including:

- Torrent keys
- Statistics
- Other persistent state

**Notes**:

- Some data types are more critical than others (keys vs stats)
- Even less critical data should not be lost
- This requirement suggests strong consistency guarantees are needed

### Backup Storage Location

**Requirement**: User provides a path in the environment configuration.

**Context**:

- The deployer should not know or care about the physical location of storage
- Path could be a simple directory on the VM disk
- Path could be a directory mounted from an attached volume
- This abstraction allows users to leverage cloud provider features (volume snapshots) without deployer complexity

**Design notes**:

- Deployer configures: backup path (e.g., `/backups/tracker`)
- User responsibility: ensure path has adequate storage, configure mounting if needed
- This keeps the deployer simple and provider-agnostic

**Example scenarios**:

| Scenario        | User Configuration                 | Deployer Sees                |
| --------------- | ---------------------------------- | ---------------------------- |
| Local disk      | No extra config                    | `/backups/tracker` directory |
| Attached volume | User mounts volume at `/backups`   | Same directory path          |
| Network storage | User mounts NFS/CIFS at `/backups` | Same directory path          |

### Backup Complexity

**Requirement**: Keep backup configuration simple.

**Context**:

- Avoid complexity related to specific cloud providers
- Users can leverage cloud-specific features externally (snapshots, S3 sync, etc.)
- Deployer should focus on creating reliable local backups

**What deployer handles**:

- Running backup commands at configured times
- Storing backups at the configured path
- Managing retention (deleting old backups)
- Basic verification

**What deployer does NOT handle**:

- Cloud provider APIs
- S3/object storage integration
- Volume management
- Off-site replication

### Backup Safety

**Requirement**: Safety is a priority over simplicity.

**Context**: When choosing between backup methods, prefer safer approaches even if more complex.

**Implications**:

- Simple `cp` of database files is not acceptable for production (corruption risk)
- Database-specific backup tools are preferred
- Verification of backup integrity should be included

## Database-Specific Requirements

### SQLite

- Must handle active database (writes during backup)
- Should handle WAL mode (multiple files)
- Verification with `PRAGMA integrity_check`

### MySQL

- Must handle credentials securely
- Should support hot backups (no downtime)
- May need to coordinate with Docker container

## Performance and Scalability

### Large Database Considerations

**Context**: The tracker database can grow significantly large (17 GB or more in
production). This has important implications for backup strategies.

**Challenges**:

| Challenge              | Description                                                   |
| ---------------------- | ------------------------------------------------------------- |
| Backup duration        | Daily backup of 17 GB can take 20-50 minutes with `mysqldump` |
| Disk space consumption | Storing 7 daily backups = ~120 GB just for database backups   |
| Network bandwidth      | If pushing to remote storage, significant upload time         |
| I/O impact             | Backup process competes with production workload              |

**Largest tables**: The `torrent_aggregate_metrics` table (stats about torrents)
is typically the largest and grows continuously. Other tables (`keys`, `torrents`,
`whitelist`) are usually much smaller.

### Selective Table Backup

**Requirement (nice-to-have)**: Allow users to selectively choose which tables
to backup.

**Use cases**:

- Backup only critical tables (`keys`, `torrents`, `whitelist`) daily
- Backup stats table (`torrent_aggregate_metrics`) weekly or not at all
- Exclude regenerable data to reduce backup size significantly

**Example**:

```bash
# Backup only critical tables (small, fast)
mysqldump --single-transaction tracker_db keys torrents whitelist > critical.sql

# Full backup (large, slow) - run weekly
mysqldump --single-transaction tracker_db > full.sql
```

## Backup Execution Flexibility

### Disable/Enable Backups

**Requirement (nice-to-have)**: Users should be able to deploy with backup
infrastructure in place but with automatic backups disabled.

**Use cases**:

- Initial deployment: get system running first, enable backups later
- Resource-constrained environments: run backups manually during off-peak hours
- Troubleshooting: temporarily disable backups to isolate issues

**Implementation options**:

| Option                 | Description                                     |
| ---------------------- | ----------------------------------------------- |
| Environment variable   | `BACKUP_ENABLED=false` disables cron schedule   |
| Remove cron schedule   | Container runs but no automatic backups         |
| Manual trigger command | `docker compose exec backup /scripts/backup.sh` |

### Manual Backup Execution

**Requirement (nice-to-have)**: Even with automatic backups disabled, users
should be able to trigger backups manually.

**Benefits**:

- Run backups during maintenance windows
- Immediate backup before risky operations
- On-demand backup when needed

**Command example**:

```bash
# Manual backup trigger (works even if cron is disabled)
docker compose exec backup /scripts/backup-all.sh
```

## Open Questions

These questions need answers as research continues:

- [x] What is the minimum backup frequency needed to meet "no data loss" requirement?
- [x] What retention period is appropriate? (7 days like demo? longer?)
- [x] Should backups be compressed? (saves space but adds complexity)
- [x] Should backup success/failure be reported? How? (logs, metrics, alerts?)
- [x] Should we support multiple backup types simultaneously? (e.g., DB backup + selective config backup)
- [x] How to restore from backups? (manual process? deployer command?)
- [x] Should selective table backup be configurable? Which tables are critical vs optional?
- [x] Should backup be enabled/disabled via environment config or runtime toggle?

## Answers to Open Questions

### Backup Frequency

**Decision**: User-configurable (default: daily or hourly).

There will always be some data loss without replication. The frequency depends
on:

- How long the backup takes
- Acceptable data loss window (RPO - Recovery Point Objective)

The frequency will be a configuration value in the initial environment config
and injected into the backup container as an environment variable (or config
file if complex).

### Retention Period

**Decision**: User-configurable (default: 7 days).

Depends on available disk space. Users provide this in configuration and can
change it later in production while the system is running.

### Compression

**Decision**: Not in first iteration.

If we use Restic to handle backups, compression is an option the backup
software provides. We don't need to implement it ourselves.

### Success/Failure Reporting

**Decision**: Logs only (for now).

Simple logging to container stdout/stderr. No metrics or alerts in the first
implementation.

### Multiple Backup Types

**Decision**: Nice-to-have, but start simple.

Goal is to provide at least one backup solution out of the box, extremely
simple. Multiple backup strategies can be added later.

### Restore Process

**Decision**: Manual process, documented.

No deployer command for restore. Users follow documentation to restore from
backups manually.

### Selective Table Backup

**Decision**: Not yet, easy to add later.

Note that database usage varies significantly:

- Stats can be disabled in the tracker → smaller database
- Public trackers don't need keys or whitelist
- If no persistence needed, database may not even exist

### Enable/Disable Backups

**Decision**: Via initial environment config + easy to disable in production.

- Initial config determines if backup container is deployed
- To disable in production: stop stack, remove backup service from
  docker-compose.yml, restart
- Backup container has no dependents (other services don't depend on it),
  so removal is safe
- This approach reduces resource usage and attack surface when backups
  are not needed
