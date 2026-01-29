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

## Open Questions

These questions need answers as research continues:

- [ ] What is the minimum backup frequency needed to meet "no data loss" requirement?
- [ ] What retention period is appropriate? (7 days like demo? longer?)
- [ ] Should backups be compressed? (saves space but adds complexity)
- [ ] Should backup success/failure be reported? How? (logs, metrics, alerts?)
- [ ] Should we support multiple backup types simultaneously? (e.g., DB backup + selective config backup)
- [ ] How to restore from backups? (manual process? deployer command?)
