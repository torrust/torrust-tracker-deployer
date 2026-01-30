# SQLite Backup Performance for Large Databases

**Date**: 2026-01-30
**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This document analyzes backup performance for large SQLite databases based on
real-world testing with the Torrust Demo production database.

## Test Environment

| Setting          | Value                             |
| ---------------- | --------------------------------- |
| Database         | Torrust Tracker Demo (production) |
| Database Size    | **17 GB**                         |
| Backup Method    | SQLite `.backup` command          |
| Journal Mode     | WAL (changed during test)         |
| Concurrent Usage | Active tracker with real traffic  |

### SQLite File Sizes

```bash
$ ls -lh sqlite3.db*
-rwxrwx--- 1 torrust torrust  17G Jan 30 08:43 sqlite3.db
-rwxrwx--- 1 torrust torrust  64K Jan 30 08:44 sqlite3.db-shm
-rwxrwx--- 1 torrust torrust  20M Jan 30 08:44 sqlite3.db-wal
```

The WAL file (20 MB) is small relative to the main database, indicating
checkpointing is working properly. The 17 GB is mostly in the main db file.

## Observations

### Test Timeline

```text
Start:   Jan 29 17:43 - Backup size: 1.2 GB
         Jan 29 22:17 - Backup size: 1.5 GB  (+0.3 GB in 4.5 hrs)
         Jan 29 22:28 - Backup size: 1.6 GB  (+0.1 GB in 11 min)
         Jan 30 07:11 - Backup size: 1.7 GB  (+0.1 GB in 8.7 hrs)
         Jan 30 09:31 - Backup size: 1.7 GB  (no change in 2.3 hrs, ~15.8 hrs elapsed)
Status:  Still running after ~15.8 hours, ~10% complete
```

**Note**: WAL mode was enabled during the test (Jan 29). The backup continues to
run but progress has stalled, likely due to constant WAL file modifications
causing backup restarts.

### Performance Metrics

| Metric               | Value           |
| -------------------- | --------------- |
| Elapsed time         | ~13.5 hours     |
| Data copied          | ~1.7 GB         |
| Progress             | ~10%            |
| Effective rate       | **~37 MB/hour** |
| Estimated completion | **~17 days**    |

### Why So Slow?

SQLite's `.backup` command:

1. **Page-by-page copy**: Reads source database page by page
2. **Respects locks**: Waits when the tracker is actively writing
3. **Single-threaded**: No parallelism, sequential I/O
4. **Competes with application**: The tracker is using the same database
5. **Restart on modification**: If source changes during backup, may restart

For a heavily-used 17GB database, this creates a near-infinite backup scenario.

### Disk I/O Is NOT the Bottleneck

Raw disk read test on the same server:

```bash
$ dd if=sqlite3.db of=/dev/null bs=4M status=progress
17305698304 bytes (17 GB, 16 GiB) copied, 39 s, 444 MB/s
```

| Method               | Speed        | Time for 17 GB |
| -------------------- | ------------ | -------------- |
| Raw disk read (`dd`) | **445 MB/s** | ~40 seconds    |
| SQLite `.backup`     | ~37 MB/hour  | ~17 days       |

**The disk is 43,000x faster than the observed backup rate.** This proves the
bottleneck is entirely within SQLite's backup mechanism - lock contention,
page-level copying, and restart-on-modification behavior.

### Maintenance Window Test (SUCCESS)

After killing the stalled `.backup` process, we tested a simple file copy with
the tracker stopped:

```bash
$ docker compose stop tracker
$ time cp sqlite3.db /home/torrust/backups/backup_maintenance_test.db

real    1m12.271s
user    0m0.338s
sys     0m36.584s

$ docker compose start tracker
```

**Result: 72 seconds total** for a complete 17GB backup.

| Method                 | Time           | Speed         | Downtime   |
| ---------------------- | -------------- | ------------- | ---------- |
| `.backup` (live)       | ~17 days       | ~37 MB/hour   | None       |
| `cp` (tracker stopped) | **72 seconds** | **~240 MB/s** | 72 seconds |

**Conclusion**: For large SQLite databases, a brief maintenance window (~1-2
minutes) is far more practical than attempting online backup with `.backup`.

## Scalability Conclusions

| Database Size   | `.backup` Viability | Expected Duration | Recommendation            |
| --------------- | ------------------- | ----------------- | ------------------------- |
| < 100 MB        | ✅ Excellent        | Seconds           | Use `.backup`             |
| 100 MB - 500 MB | ✅ Good             | 1-5 minutes       | Use `.backup`             |
| 500 MB - 1 GB   | ⚠️ Acceptable       | 5-30 minutes      | Consider alternatives     |
| 1 GB - 5 GB     | ⚠️ Slow             | Hours             | Plan maintenance window   |
| 5 GB - 10 GB    | ❌ Impractical      | Many hours        | Use alternatives          |
| > 10 GB         | ❌ Unusable         | Days/weeks        | **Must use alternatives** |

## Alternative Approaches for Large Databases

### 1. Filesystem Snapshots (Recommended for Large DBs)

**LVM Snapshots** or **ZFS Snapshots**:

```bash
# LVM example
lvcreate --snapshot --size 1G --name sqlite_snap /dev/vg0/data
mount /dev/vg0/sqlite_snap /mnt/snapshot
cp /mnt/snapshot/sqlite3.db /backups/
umount /mnt/snapshot
lvremove /dev/vg0/sqlite_snap
```

- ✅ Instant snapshot (milliseconds)
- ✅ Copy-on-write, minimal overhead
- ✅ Works with any database size
- ❌ Requires LVM/ZFS setup

### 2. VACUUM INTO (SQLite 3.27+)

```bash
sqlite3 sqlite3.db "VACUUM INTO '/backups/backup.db'"
```

- ✅ Creates compacted, defragmented copy
- ✅ May be faster than `.backup` for fragmented DBs
- ⚠️ Still single-threaded
- ⚠️ Requires 2x disk space temporarily

### 3. Maintenance Window Approach (TESTED - RECOMMENDED)

```bash
# Stop tracker
docker compose stop tracker

# Quick file copy (safe when app is stopped)
cp sqlite3.db /backups/sqlite3_$(date +%Y%m%d).db

# Restart tracker
docker compose start tracker
```

- ✅ Fast (limited by I/O speed)
- ✅ Guaranteed consistent
- ✅ **Tested: 72 seconds for 17GB database**
- ❌ Requires downtime (~1-2 minutes for 17GB)

### 4. Litestream (Continuous Replication)

[Litestream](https://litestream.io/) provides real-time SQLite replication:

```yaml
# litestream.yml
dbs:
  - path: /data/sqlite3.db
    replicas:
      - url: s3://bucket/tracker
```

- ✅ Near-zero RPO (Point-in-time recovery)
- ✅ No backup windows needed
- ✅ Works with any database size
- ❌ Requires S3-compatible storage
- ❌ Additional complexity

### 5. WAL Mode with Checkpoint Control

```bash
# Enable WAL mode (one-time)
sqlite3 sqlite3.db "PRAGMA journal_mode=WAL;"

# Backup during low activity
sqlite3 sqlite3.db ".backup /backups/backup.db"

# Or checkpoint and copy
sqlite3 sqlite3.db "PRAGMA wal_checkpoint(TRUNCATE);"
cp sqlite3.db /backups/
cp sqlite3.db-wal /backups/  # if exists
cp sqlite3.db-shm /backups/  # if exists
```

- ✅ Better concurrent read/write
- ⚠️ Still slow for very large DBs
- ⚠️ Must backup all three files

## Recommendations by Use Case

### Small Deployments (< 1 GB)

Use SQLite `.backup` command - simple and reliable:

```bash
sqlite3 sqlite3.db ".backup /backups/backup_$(date +%Y%m%d).db"
```

### Medium Deployments (1-10 GB)

Consider:

1. **Scheduled maintenance window** - Stop app, copy, restart
2. **VACUUM INTO** - May be faster for fragmented databases
3. **Migrate to MySQL** - Better tooling for larger databases

### Large Deployments (> 10 GB)

**Strongly recommend**:

1. **Filesystem snapshots** (LVM/ZFS) - If infrastructure supports it
2. **Litestream** - If cloud storage is available
3. **Migrate to MySQL/PostgreSQL** - Better suited for large datasets

### Production Trackers (Like Torrust Demo)

For the 17GB Torrust Demo database, **tested and verified approach**:

```bash
# Maintenance window backup (TESTED: 72 seconds total)
docker compose stop tracker
time cp sqlite3.db /backups/backup_$(date +%Y%m%d).db
docker compose start tracker
```

**Actual results from production test:**

- Database size: 17 GB
- Copy time: 72 seconds
- Total downtime: ~90 seconds (including stop/start)
- Backup integrity: Verified

### Off-site Backup Transfer

After local backup, transferring to off-site storage:

```bash
$ scp backup_maintenance_test.db user@backup-server:~/backups/
backup_maintenance_test.db    100%   16GB  32.3MB/s   08:39
```

| Metric         | Value      |
| -------------- | ---------- |
| Transfer size  | 16-17 GB   |
| Transfer speed | 32.3 MB/s  |
| Transfer time  | ~9 minutes |

**Total backup cycle time**: ~10-11 minutes (72s local + 9min transfer)

## Summary: Complete Backup Workflow

For a 17GB SQLite database with off-site backup:

| Step                  | Time            | Downtime |
| --------------------- | --------------- | -------- |
| Stop tracker          | ~5 seconds      | Yes      |
| Copy database locally | 72 seconds      | Yes      |
| Start tracker         | ~10 seconds     | -        |
| **Total downtime**    | **~90 seconds** | -        |
| Transfer off-site     | ~9 minutes      | No       |
| **Total backup time** | **~11 minutes** | -        |

This is dramatically better than the `.backup` approach which never completed.

## Impact on Backup Strategy

This finding affects our backup strategy recommendations:

1. **Database driver choice matters**: MySQL/PostgreSQL have better large-DB
   backup tooling than SQLite
2. **Size monitoring needed**: Alert when database exceeds backup-friendly
   thresholds
3. **Backup method selection**: Should vary based on database size
4. **RPO expectations**: Large SQLite DBs may have longer backup windows

## Related Documents

- [SQLite Backup Approaches](backup-approaches.md)
- [Torrust Live Demo Analysis](torrust-live-demo/)
- [MySQL Backup Research](../mysql/)
