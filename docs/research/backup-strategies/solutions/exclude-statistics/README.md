# Exclude Statistics Backup

**Status**: 🔬 Proposed

## Summary

Backup only essential data (whitelist, keys, metrics) and exclude torrents that
have never been completed. This dramatically reduces backup size while
preserving valuable data.

## Problem

The Torrust Tracker database can grow very large due to torrent tracking data.
Analysis of the [Torrust Demo production database](torrust-demo-analysis.md)
(17 GB) revealed:

| Table                     | Rows        | Size    | Value |
| ------------------------- | ----------- | ------- | ----- |
| torrents                  | 161,454,700 | ~8 GB   | Mixed |
| whitelist                 | 30,076      | ~1.4 MB | High  |
| keys                      | 10          | ~0.5 KB | High  |
| torrent_aggregate_metrics | 1           | ~0.1 KB | Low   |

**Key finding**: 96.9% of torrents (156M rows) have `completed = 0`, meaning
they were announced but never fully downloaded by anyone.

## Solution

Perform selective database dumps that exclude never-completed torrents. This
reduces backup from 17 GB to ~247 MB (98.5% reduction).

### What Gets Backed Up

- ✅ Whitelist entries (essential configuration)
- ✅ Authentication keys (essential configuration)
- ✅ Aggregate metrics (small, useful)
- ✅ Torrents with `completed > 0` (5M rows with actual activity)

### What Gets Excluded

- ❌ Torrents with `completed = 0` (156M rows, no download activity)

## Trade-offs

| Advantage                             | Disadvantage                               |
| ------------------------------------- | ------------------------------------------ |
| 98.5% backup size reduction           | Never-completed torrents lost              |
| Smaller off-site transfer             | Requires selective SQL export              |
| Preserves all completed torrent data  | More complex than full backup              |
| Excluded data repopulates on announce | Not a "full" backup                        |
| Faster restore time                   | **Does NOT reduce backup generation time** |

## ⚠️ Critical Limitation

**This solution reduces backup SIZE but NOT backup TIME under load.**

The SQLite locking problem observed with `.backup` (stalling at 10% after 16+
hours) is caused by lock contention with the running tracker, not by the amount
of data being backed up.

Even backing up only 5M rows (completed torrents) instead of 161M rows still
requires:

1. Acquiring read locks on the database
2. Reading pages while tracker is writing
3. Competing with announce requests for database access

**What this solution DOES help with:**

- ✅ Storage space (247 MB vs 17 GB)
- ✅ Off-site transfer time (~8 seconds vs ~9 minutes at 32 MB/s)
- ✅ Restore time (faster to import smaller backup)
- ✅ Backup file management (easier to store/rotate smaller files)

**What this solution does NOT help with:**

- ❌ Backup generation time under heavy load
- ❌ SQLite locking contention
- ❌ The stalling issue observed in large database testing

**Conclusion**: This solution is complementary to other approaches. Use it
together with:

- **Maintenance window backup** - Stop tracker, export, restart (solves locking)
- **Low-traffic scheduling** - Run backup during quiet periods

## Implementation Notes

### SQLite Selective Export

```bash
# Export essential tables (full)
sqlite3 tracker.db ".dump whitelist" > backup.sql
sqlite3 tracker.db ".dump keys" >> backup.sql
sqlite3 tracker.db ".dump torrent_aggregate_metrics" >> backup.sql

# Export only completed torrents
sqlite3 tracker.db <<EOF >> backup.sql
.mode insert torrents
SELECT * FROM torrents WHERE completed > 0;
EOF
```

### MySQL Selective Export

```bash
# Dump whitelist and keys tables completely
mysqldump -u user -p tracker whitelist keys torrent_aggregate_metrics > backup.sql

# For torrents, use WHERE clause
mysqldump -u user -p tracker torrents --where="completed > 0" >> backup.sql
```

## Research Documents

- [Torrust Demo Analysis](torrust-demo-analysis.md) - Detailed database
  analysis showing table sizes and data patterns
