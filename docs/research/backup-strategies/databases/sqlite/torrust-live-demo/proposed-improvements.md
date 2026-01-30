# Proposed Improvements for Torrust Live Demo

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This document contains the text for GitHub issues to be opened on the [torrust-demo](https://github.com/torrust/torrust-demo) repository. These issues are based on findings from our backup strategies research.

The Torrust Live Demo serves as an invaluable production lab for testing these improvements before applying them to the Torrust Tracker Deployer.

## Created Issues

- **Issue #85**: [Use SQLite .backup command instead of cp for database backups](https://github.com/torrust/torrust-demo/issues/85)
- **Issue #86**: [Evaluate WAL journal mode for improved tracker performance](https://github.com/torrust/torrust-demo/issues/86)

## Traffic Profile

The Torrust Live Demo is a **high-traffic production system**:

- **~8,000-17,000 UDP requests per second** continuously
- **17 GB database**
- **No low-traffic periods** - constant traffic 24/7

---

## Issue 1: Use SQLite `.backup` Command Instead of `cp`

### Title

Use SQLite .backup command instead of cp for database backups

### Labels

enhancement, database

### Body

Copy the following markdown as the issue body:

---

#### Summary

The current backup script uses `cp` to copy the SQLite database file. This should be changed to use SQLite's `.backup` command for guaranteed backup consistency.

#### Current Implementation

File: `share/bin/tracker-db-backup.sh`

```bash
cp $DATABASE_FILE "$BACKUP_FILE"
```

#### Problem

Using `cp` to backup a SQLite database has a consistency risk: if the tracker writes to the database during the copy operation, the backup may be inconsistent or corrupted.

The Torrust Live Demo is a **high-traffic production system**:

- **~8,000-17,000 UDP requests per second** continuously
- **17 GB database**
- **No low-traffic periods** - the tracker receives constant traffic 24/7

With this traffic profile, the probability of a write occurring during the `cp` copy of a 17 GB file is essentially 100%. The backup files are very likely inconsistent.

Using SQLite's proper backup mechanism is not optional for this workload - it's critical.

#### Proposed Solution

Replace the `cp` command with SQLite's `.backup` command:

```bash
sqlite3 "$DATABASE_FILE" ".backup '$BACKUP_FILE'"
```

#### Why `.backup` is Better

| Aspect                    | `cp`                  | `.backup`                    |
| ------------------------- | --------------------- | ---------------------------- |
| Consistency guarantee     | ❌ Risk of corruption | ✅ Guaranteed                |
| Handles concurrent writes | ❌ No                 | ✅ Yes                       |
| WAL mode support          | ❌ Must copy 3 files  | ✅ Automatic                 |
| Locking                   | ❌ None               | ✅ Proper page-level locking |

#### How `.backup` Works

The `.backup` command uses SQLite's Online Backup API which:

- Acquires proper locks automatically
- Creates a consistent snapshot even while database is in use
- Handles WAL mode databases (produces single self-contained file)
- If source is modified mid-backup, backup automatically restarts

See: [SQLite Backup API Documentation](https://www.sqlite.org/backup.html)

#### Suggested Updated Script

```bash
#!/bin/bash
set -e

BACKUP_DIR="/home/torrust/backups"
DATABASE_FILE="/home/torrust/github/torrust/torrust-demo/storage/tracker/lib/database/sqlite3.db"
BACKUP_FILE="$BACKUP_DIR/tracker_backup_$(date +%Y-%m-%d_%H-%M-%S).db"

# Use SQLite backup command for consistency
sqlite3 "$DATABASE_FILE" ".backup '$BACKUP_FILE'"

# Find and remove backups older than 7 days
find "$BACKUP_DIR" -type f -name "tracker_backup_*.db" -mtime +7 -delete
```

#### Additional Improvements (Optional)

These could be added in the same PR or as follow-ups:

1. Add error handling: Check if backup succeeded
2. Add compression: `gzip` or `zstd` to reduce storage
3. Add verification: `PRAGMA integrity_check` on backup file
4. Fix variable quoting: Quote `$DATABASE_FILE` in all uses

#### Context

This issue is part of backup strategy research for the Torrust Tracker Deployer project. The Live Demo serves as a production lab to validate improvements before applying them more broadly.

Research documentation: <https://github.com/torrust/torrust-tracker-deployer/issues/310>

---

## Issue 2: Evaluate WAL Mode for Improved Performance

### Title

Evaluate WAL journal mode for improved tracker performance

### Labels

enhancement, database, performance

### Prerequisites

Depends on Issue 1 (the `.backup` command handles WAL mode automatically, so that should be implemented first).

### Body

Copy the following markdown as the issue body:

---

#### Summary

Evaluate whether switching the SQLite database from `delete` journal mode to `wal` (Write-Ahead Logging) mode improves tracker performance, and measure the impact using the existing Grafana dashboard.

#### Prerequisites

- Depends on: Use SQLite `.backup` command instead of `cp` (Issue TBD)

#### Current State

The tracker database currently uses `delete` journal mode:

```bash
$ sqlite3 ./storage/tracker/lib/database/sqlite3.db "PRAGMA journal_mode;"
delete
```

#### Why Consider WAL Mode?

WAL mode offers several potential benefits:

| Benefit            | Description                                              |
| ------------------ | -------------------------------------------------------- |
| Better concurrency | Readers don't block writers, writers don't block readers |
| Faster writes      | Writes are sequential to WAL file                        |
| Fewer fsync calls  | Better performance, especially on slower disks           |
| Read performance   | Readers see consistent snapshots without locking         |

#### Potential Drawbacks

| Drawback                   | Mitigation                                   |
| -------------------------- | -------------------------------------------- |
| Three files instead of one | `.backup` command handles this automatically |
| Shared memory requirement  | Not an issue (not using NFS)                 |
| WAL file can grow large    | Regular checkpointing (automatic)            |

#### Proposed Experiment

Phase 1: Baseline Metrics

Before making any changes, collect baseline metrics from Grafana:

- Request latency (announce, scrape)
- Requests per second
- Database operation timing (if available)
- Resource usage (CPU, memory, disk I/O)

Phase 2: Switch to WAL Mode

```bash
# Stop the tracker (or during maintenance window)
sqlite3 /path/to/sqlite3.db "PRAGMA journal_mode=WAL;"
# Restart the tracker
```

Phase 3: Collect Comparison Metrics

After running in WAL mode for a representative period (e.g., 1 week):

- Compare the same metrics from Phase 1
- Document any observed differences
- Note any issues or unexpected behavior

Phase 4: Decision

Based on the data:

- If performance improves: Keep WAL mode
- If no significant change: Keep WAL mode (still safer for concurrent access)
- If issues arise: Revert to delete mode

#### Expected Outcome

Document the measured performance difference (if any) between delete and WAL modes in a real production environment. This data will inform:

1. Whether to recommend WAL mode for Torrust Tracker deployments
2. Default settings in the Torrust Tracker Deployer project

#### Context

This issue is part of backup strategy research for the Torrust Tracker Deployer project. The Live Demo serves as a production lab to validate improvements before applying them more broadly.

Research documentation: <https://github.com/torrust/torrust-tracker-deployer/issues/310>

#### References

- [SQLite WAL Mode Documentation](https://www.sqlite.org/wal.html)
- [SQLite Backup API](https://www.sqlite.org/backup.html)

---

## How to Create These Issues

1. Go to <https://github.com/torrust/torrust-demo/issues/new>
2. Copy the **Title** into the title field
3. Copy the **Body** section (between the `---` lines) into the description field
4. Add the suggested labels
5. Submit

After creating Issue 1, update Issue 2's "Depends on" line with the actual issue number.
