# SQLite Backup Approaches

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This document explores different approaches for backing up SQLite databases that are actively in use. The core challenge is ensuring **consistency** - if you copy a database file while it's being written to, you may get a corrupted backup.

## The Core Challenge

SQLite stores everything in a single file (or multiple files with WAL mode). Unlike client-server databases, there's no built-in backup protocol - you're essentially backing up a file that may be actively modified.

## Backup Methods

### 1. SQLite Online Backup API (`.backup` command)

The safest method for backing up SQLite databases. SQLite provides a built-in backup API that handles consistency:

```bash
sqlite3 /path/to/database.db ".backup /path/to/backup.db"
```

**How it works:**

- Creates a consistent snapshot even while database is in use
- Handles locking internally at the page level
- Copies pages atomically
- Works with WAL mode databases
- Source database remains fully usable during backup

**Technical details** (from [SQLite Backup API documentation](https://www.sqlite.org/backup.html)):

- Source database is read-locked only during actual page reads, not continuously
- Destination database holds exclusive lock for entire backup duration
- If source is modified mid-backup, backup automatically restarts
- Result is a bit-wise identical snapshot of source at backup completion
- Can be done incrementally (N pages at a time) for large databases

**WAL mode handling:**

The `.backup` command does NOT copy three files - it does something better:

- Reads from **both** the main `.db` file AND the `-wal` file
- Creates a **single, complete** backup file
- The backup contains all committed changes (checkpointed or not)
- The resulting backup is a standalone `.db` file (no `-wal` or `-shm` needed)

| Approach               | Files Created | Self-Contained?     | Consistency                  |
| ---------------------- | ------------- | ------------------- | ---------------------------- |
| `.backup`              | 1 file        | ✅ Yes              | ✅ Guaranteed                |
| Manual `cp` of 3 files | 3 files       | ❌ No (needs all 3) | ⚠️ Risk if copied separately |

The backup file is "clean" - like a freshly vacuumed database that can be opened directly without needing any WAL or SHM files.

**Pros:**

- ✅ Safe and consistent
- ✅ Built into SQLite
- ✅ Handles concurrent writes
- ✅ Works with all journal modes

**Cons:**

- ❌ Requires `sqlite3` CLI tool on the system
- ❌ Slightly slower than simple file copy

**Example with error handling:**

```bash
#!/bin/bash
DATABASE="/path/to/database.db"
BACKUP="/path/to/backup.db"

if sqlite3 "$DATABASE" ".backup '$BACKUP'"; then
    echo "Backup successful"
else
    echo "Backup failed!" >&2
    exit 1
fi
```

### 2. VACUUM INTO (SQLite 3.27+)

Creates a compacted backup while also defragmenting:

```sql
VACUUM INTO '/path/to/backup.db';
```

**How it works:**

- Creates a fresh, compacted copy of the database
- Removes unused space (like VACUUM but to a new file)
- Takes an exclusive lock briefly during operation

**Pros:**

- ✅ Consistent backup
- ✅ Also optimizes/defragments
- ✅ Single SQL command

**Cons:**

- ❌ Requires SQLite 3.27+ (released 2019)
- ❌ Takes exclusive lock (brief write blocking)
- ❌ May be slower for large databases

**Example:**

```bash
sqlite3 /path/to/database.db "VACUUM INTO '/path/to/backup.db';"
```

### 3. Simple File Copy

Just copying the file with standard tools:

```bash
cp /path/to/database.db /path/to/backup.db
```

**How it works:**

- Standard filesystem copy operation
- No awareness of SQLite internals
- Relies on filesystem atomicity (or lack thereof)

**Risks:**

- ⚠️ If a write happens during copy, backup may be corrupted
- ⚠️ SQLite uses filesystem locking, but copy ignores it
- ⚠️ WAL mode files (`-wal`, `-shm`) are not automatically included

**When it works:**

- Database is not being written to during copy
- Database uses WAL mode (main file is more stable)
- Low write frequency reduces corruption probability
- Acceptable to lose occasional backup (non-critical data)

**Mitigation strategies:**

- Copy during low-activity periods (scheduled maintenance window)
- Use filesystem snapshots (LVM, ZFS, cloud provider snapshots)
- Accept occasional corruption for non-critical data
- Verify backup integrity after copy

### 4. Filesystem Snapshots

Use filesystem or cloud provider snapshot capabilities:

```bash
# LVM snapshot example
lvcreate -L 1G -s -n db_snapshot /dev/vg/db_volume

# Then mount and copy from snapshot
mount /dev/vg/db_snapshot /mnt/snapshot
cp /mnt/snapshot/database.db /backups/
umount /mnt/snapshot
lvremove /dev/vg/db_snapshot
```

**Pros:**

- ✅ Point-in-time consistent snapshot
- ✅ Very fast (copy-on-write)
- ✅ Works at block level

**Cons:**

- ❌ Requires specific filesystem (LVM, ZFS, btrfs)
- ❌ More complex setup
- ❌ May not be available in all cloud environments

## WAL Mode Considerations

When SQLite uses WAL (Write-Ahead Logging) mode, the database state is spread across multiple files:

| File              | Purpose                               |
| ----------------- | ------------------------------------- |
| `database.db`     | Main database file                    |
| `database.db-wal` | Write-ahead log (uncommitted changes) |
| `database.db-shm` | Shared memory file (index for WAL)    |

**Important:** If WAL mode is enabled, you must backup all three files together for a complete backup, OR use the `.backup` command which handles this automatically.

**Check if WAL mode is enabled:**

```bash
sqlite3 database.db "PRAGMA journal_mode;"
```

### WAL Mode Pros and Cons

#### Advantages

| Advantage           | Description                                                        |
| ------------------- | ------------------------------------------------------------------ |
| Better concurrency  | Readers don't block writers, writers don't block readers           |
| Faster writes       | Writes are sequential to WAL file (faster than random page writes) |
| Fewer fsync calls   | Better performance, especially on slower disks                     |
| Atomic transactions | Crash recovery is more robust                                      |
| Read performance    | Readers see consistent snapshots without locking                   |
| Persistent mode     | WAL mode persists across database close/reopen (see note below)    |

> **Note on WAL persistence:** Unlike other journal modes (DELETE, TRUNCATE, PERSIST), WAL mode is stored in the database file header and remains active across close/reopen cycles. You don't need to set it on every connection. However, you **can** switch back to other modes anytime with `PRAGMA journal_mode=DELETE;`.

#### Disadvantages

| Disadvantage               | Description                                                               |
| -------------------------- | ------------------------------------------------------------------------- |
| Three files instead of one | `.db`, `.db-wal`, `.db-shm` must stay together                            |
| Shared memory requirement  | `-shm` file uses shared memory; doesn't work on network filesystems (NFS) |
| WAL file can grow large    | If readers hold connections open, WAL can't be checkpointed               |
| Backup complexity          | Must copy all three files together (unless using `.backup` command)       |
| Not supported everywhere   | Some embedded/restricted environments don't support it                    |
| Read-only databases        | Was not possible before SQLite 3.22.0 (2018)                              |
| Large transactions         | Not ideal for transactions >100MB (though improved in SQLite 3.11.0+)     |

### Checkpointing

Checkpointing transfers content from the WAL file back into the main database file.

**Automatic checkpointing:**

- Default threshold: 1000 pages (~4MB WAL file size)
- Runs automatically on COMMIT when threshold is exceeded
- Can be configured or disabled via `PRAGMA wal_autocheckpoint`

**Checkpoint starvation:**

If there are always active readers, checkpoints cannot complete and the WAL file grows without bound. To avoid this:

- Ensure there are "reader gaps" (times when no processes are reading)
- Consider manual checkpoints with `SQLITE_CHECKPOINT_RESTART` or `SQLITE_CHECKPOINT_TRUNCATE`

**WAL file cleanup:**

When the last connection to a database closes, SQLite:

1. Runs a final checkpoint
2. Deletes the WAL and SHM files

If a process crashes without clean shutdown, the WAL file may remain on disk.

### SQLITE_BUSY in WAL Mode

While WAL mode provides better concurrency, `SQLITE_BUSY` can still occur in edge cases:

- Another connection has the database in exclusive locking mode
- Database is being cleaned up by a closing connection
- Recovery is running after a crash

Applications should still handle `SQLITE_BUSY` appropriately.

### When to Use WAL Mode

| Scenario                      | Recommendation                    |
| ----------------------------- | --------------------------------- |
| High read/write concurrency   | ✅ Use WAL                        |
| Network filesystem (NFS)      | ❌ Stay with delete/truncate      |
| Simple backup with `cp`       | ⚠️ WAL adds complexity            |
| Backup with `.backup` command | ✅ WAL handled automatically      |
| Single-threaded access        | Either works, WAL slightly faster |

### Switching Journal Modes

```bash
# Check current mode
sqlite3 database.db "PRAGMA journal_mode;"

# Switch to WAL (permanent until changed)
sqlite3 database.db "PRAGMA journal_mode=WAL;"

# Switch back to delete mode
sqlite3 database.db "PRAGMA journal_mode=DELETE;"
```

**Note**: Switching mode requires exclusive access - no other connections can be open.

## Compression Options

After creating a backup, compression can significantly reduce storage:

| Tool    | Speed     | Compression Ratio | Notes                         |
| ------- | --------- | ----------------- | ----------------------------- |
| `gzip`  | Fast      | Good (~70%)       | Standard, widely available    |
| `zstd`  | Very Fast | Better (~75%)     | Modern, excellent for backups |
| `xz`    | Slow      | Best (~80%)       | Best ratio, CPU intensive     |
| `bzip2` | Medium    | Good (~72%)       | Older, less common now        |

Note: `zstd` offers a good balance of speed and compression ratio for backup use cases.

```bash
# Backup with zstd compression
sqlite3 database.db ".backup /tmp/backup.db"
zstd -q -9 /tmp/backup.db -o /backups/backup_$(date +%Y%m%d).db.zst
rm /tmp/backup.db
```

## Backup Verification

**Critical**: Always verify backup integrity after creation. An unverified backup is not a backup.

### Basic Integrity Check

```bash
# Check integrity of backup file
sqlite3 /path/to/backup.db "PRAGMA integrity_check;"

# Expected output for healthy database:
# ok

# If corruption is found, output will show specific errors:
# *** in database main ***
# Page 123: btree page has invalid parent pointer
```

### Full Verification Process

For critical backups, perform a comprehensive verification:

```bash
#!/bin/bash
BACKUP_FILE="$1"

# 1. Check file exists and has content
if [ ! -s "$BACKUP_FILE" ]; then
    echo "ERROR: Backup file is empty or missing"
    exit 1
fi

# 2. Verify SQLite file header (first 16 bytes contain "SQLite format 3")
if ! head -c 16 "$BACKUP_FILE" | grep -q "SQLite format 3"; then
    echo "ERROR: Not a valid SQLite database file"
    exit 1
fi

# 3. Run integrity check
RESULT=$(sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;" 2>&1)
if [ "$RESULT" != "ok" ]; then
    echo "ERROR: Integrity check failed: $RESULT"
    exit 1
fi

# 4. Verify expected tables exist (application-specific)
TABLE_COUNT=$(sqlite3 "$BACKUP_FILE" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
if [ "$TABLE_COUNT" -eq 0 ]; then
    echo "ERROR: No tables found in backup"
    exit 1
fi

echo "SUCCESS: Backup verified"
```

### Quick Check vs Full Check

| Check Type  | Command                   | Speed  | Use Case                       |
| ----------- | ------------------------- | ------ | ------------------------------ |
| Quick check | `PRAGMA quick_check;`     | Fast   | Routine verification           |
| Full check  | `PRAGMA integrity_check;` | Slower | Critical/periodic verification |

The `quick_check` is faster but less thorough. Use `integrity_check` for critical backups.

### Verification of Compressed Backups

For compressed backups, decompress before verification:

```bash
# For zstd compressed backups
zstd -d backup.db.zst -o /tmp/backup_verify.db
sqlite3 /tmp/backup_verify.db "PRAGMA integrity_check;"
rm /tmp/backup_verify.db
```

## Restore Procedures

Restoring from backup is as important as creating backups. Test your restore procedure regularly.

### Basic Restore

```bash
# 1. Stop the application using the database
systemctl stop tracker

# 2. Backup the current (possibly corrupted) database
mv /path/to/database.db /path/to/database.db.corrupted

# 3. Copy backup to database location
cp /path/to/backup.db /path/to/database.db

# 4. If the backup was from WAL mode, ensure clean state
sqlite3 /path/to/database.db "PRAGMA wal_checkpoint(TRUNCATE);"

# 5. Verify the restored database
sqlite3 /path/to/database.db "PRAGMA integrity_check;"

# 6. Restart the application
systemctl start tracker
```

### Restore from Compressed Backup

```bash
# Decompress and restore in one step
zstd -d /backups/backup_20260128.db.zst -o /path/to/database.db

# Or with gzip
gunzip -c /backups/backup_20260128.db.gz > /path/to/database.db
```

### Point-in-Time Considerations

SQLite backups are **point-in-time snapshots**. When restoring:

| Consideration      | Impact                                       |
| ------------------ | -------------------------------------------- |
| Data since backup  | Lost - only data at backup time is restored  |
| Active connections | Must be closed before restore                |
| Journal mode       | Backup may have different mode than original |
| File permissions   | Must match application requirements          |

### Restore Verification Checklist

After restoring, verify:

```bash
# 1. Integrity check
sqlite3 /path/to/database.db "PRAGMA integrity_check;"

# 2. Check expected row counts (application-specific)
sqlite3 /path/to/database.db "SELECT COUNT(*) FROM torrents;"

# 3. Check database is writable
sqlite3 /path/to/database.db "PRAGMA user_version;"

# 4. Application-level verification
# Start application and verify it works correctly
```

### Disaster Recovery Testing

**Recommendation**: Periodically test restore procedures:

1. Create a backup
2. Restore to a test location
3. Verify the restored database
4. Run application smoke tests against restored data

This ensures backups are actually usable when needed.

## Comparison Summary

| Method              | Consistency  | Speed     | Complexity | Requirements  |
| ------------------- | ------------ | --------- | ---------- | ------------- |
| `.backup` command   | ✅ Excellent | Good      | Low        | sqlite3 CLI   |
| `VACUUM INTO`       | ✅ Excellent | Medium    | Low        | SQLite 3.27+  |
| Simple `cp`         | ⚠️ Risky     | Fast      | Very Low   | None          |
| Filesystem snapshot | ✅ Excellent | Very Fast | High       | LVM/ZFS/Cloud |

## Key Observations

Based on this exploration:

- **`.backup` command** provides the best consistency guarantees with low complexity
- **Simple `cp`** has corruption risks when the database is being written to
- **Compression** can significantly reduce storage needs (70-80% reduction)
- **Verification** with `PRAGMA integrity_check` can detect corrupted backups
- **WAL mode** requires special handling unless using `.backup` (which handles it automatically)

## Key Conclusion: `.backup` vs `cp`

### Is `.backup` safe?

**Yes.** The `.backup` command uses SQLite's Online Backup API which:

- Acquires proper locks automatically
- Handles concurrent writes correctly (restarts if source modified)
- Produces a guaranteed consistent snapshot
- Works with both `delete` and `wal` journal modes

### Does `.backup` solve the problems with `cp`?

**Yes.** Using `.backup` instead of `cp` solves the main consistency issue:

| Problem                                  | With `cp`                    | With `.backup`             |
| ---------------------------------------- | ---------------------------- | -------------------------- |
| Inconsistent backup if write during copy | ❌ Risk of corruption        | ✅ Safe - proper locking   |
| WAL files not handled                    | ❌ Must copy manually        | ✅ Automatic               |
| Active transactions                      | ❌ May capture partial state | ✅ Waits/handles correctly |

### Is WAL mode required for safe backups?

**No.** WAL mode is NOT required for safe backups.

The `.backup` command works correctly with `delete` mode (traditional journal). WAL mode offers **other benefits** unrelated to backup safety:

| WAL Mode Benefit              | Relevant to Backups?              |
| ----------------------------- | --------------------------------- |
| Better read/write concurrency | No - operational benefit          |
| Faster writes                 | No - operational benefit          |
| Safe backups                  | No - `.backup` handles both modes |

**Bottom line**: Switching from `cp` to `.backup` is the critical change for safe backups. WAL mode is optional and would be chosen for performance/concurrency reasons, not backup safety.

## References

- [SQLite Backup API](https://www.sqlite.org/backup.html)
- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [SQLite VACUUM INTO](https://www.sqlite.org/lang_vacuum.html)
- [zstd Compression](https://github.com/facebook/zstd)
