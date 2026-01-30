# Torrust Demo Database Analysis

**Date**: January 30, 2026
**Database**: `backup_maintenance_test.db` (17 GB)
**Source**: Torrust Live Demo production database

## Purpose

Analyze the database structure to determine if excluding certain data can
make backups practical for large databases.

## Database Schema

```sql
-- 4 tables in the database
whitelist (id, info_hash)                    -- Whitelisted torrents
keys (id, key, valid_until)                  -- Authentication keys
torrents (id, info_hash, completed)          -- Torrent tracking data
torrent_aggregate_metrics (id, metric_name, value)  -- Aggregate metrics
```

## Table Analysis

| Table                     | Rows        | Estimated Size | % of DB |
| ------------------------- | ----------- | -------------- | ------- |
| torrents                  | 161,454,700 | ~8 GB          | 99.8%   |
| whitelist                 | 30,076      | ~1.4 MB        | < 0.01% |
| keys                      | 10          | ~0.5 KB        | < 0.01% |
| torrent_aggregate_metrics | 1           | ~0.1 KB        | < 0.01% |

**Key Finding**: The `torrents` table contains **161 million rows** and accounts
for virtually all database space.

## Torrents Table Breakdown

The `torrents` table tracks every info_hash that has been announced to the
tracker, with a `completed` counter for download completions.

| Category                                  | Rows        | Percentage | Data Value |
| ----------------------------------------- | ----------- | ---------- | ---------- |
| Never completed (`completed = 0`)         | 156,482,966 | 96.9%      | Low        |
| Completed at least once (`completed > 0`) | 4,971,734   | 3.1%       | Higher     |

### Observations

1. **96.9% of torrents have never been fully downloaded** - These are
   essentially "ephemeral" entries from peers announcing torrents that
   never completed.

2. **Only 3.1% have meaningful completion data** - These represent torrents
   that at least one peer has fully downloaded.

3. **Max completed count: 120,194** - Some popular torrents have been
   downloaded over 100,000 times.

## Backup Size Scenarios

| Scenario                | What's Included       | Estimated Size | Backup Time            |
| ----------------------- | --------------------- | -------------- | ---------------------- |
| Full backup             | Everything            | ~17 GB         | Hours (with `.backup`) |
| Whitelist + keys only   | Essential config      | ~1.4 MB        | Seconds                |
| Completed torrents only | `WHERE completed > 0` | ~247 MB        | Minutes                |

## Implications for Exclude Statistics Solution

### What Can Be Excluded?

The data in `torrents` table is **not traditional statistics** - it's the
core tracking data. However, most of it (97%) represents torrents that were
announced but never completed by anyone.

### Possible Approaches

#### Approach A: Exclude All Torrents

- **Backup**: Only `whitelist`, `keys`, `torrent_aggregate_metrics`
- **Size**: ~1.4 MB
- **Trade-off**: All torrent tracking data lost; tracker restarts "empty"
- **Recovery**: Torrents repopulate as peers announce

#### Approach B: Exclude Never-Completed Torrents

- **Backup**: `WHERE completed > 0` (5M rows)
- **Size**: ~247 MB
- **Trade-off**: Lose torrents that were never fully downloaded
- **Recovery**: These repopulate on next announce anyway

#### Approach C: Exclude Old/Inactive Torrents

- **Backup**: Recent torrents only (e.g., last 30 days)
- **Size**: Depends on activity
- **Trade-off**: Lose historical torrent data
- **Note**: Requires adding `last_seen` timestamp column

## Recommendation

**Approach B (Exclude Never-Completed)** offers the best balance:

- Preserves all torrents that have actual download activity
- Reduces backup from 17 GB to ~247 MB (98.5% reduction)
- Lost data (never-completed torrents) has minimal value and repopulates
  automatically on next announce

### ⚠️ Important Limitation

**This approach reduces backup SIZE but NOT backup TIME under heavy load.**

The SQLite locking issue (backup stalling at 10%) is caused by lock contention
with the running tracker, not by data volume. Even backing up 5M rows instead
of 161M rows still requires acquiring locks and competing with announce
requests.

**This solution helps with:**

- Storage space (247 MB vs 17 GB)
- Off-site transfer time
- Restore time

**This solution does NOT help with:**

- Backup generation time under load
- SQLite locking contention

**Recommended combination**: Use selective backup WITH maintenance window
or low-traffic scheduling to address both size and locking issues.

## Implementation Notes

### SQLite Selective Export

```bash
# Export only valuable data
sqlite3 tracker.db ".dump whitelist" > backup.sql
sqlite3 tracker.db ".dump keys" >> backup.sql
sqlite3 tracker.db ".dump torrent_aggregate_metrics" >> backup.sql

# For completed torrents, use custom query
sqlite3 tracker.db "SELECT * FROM torrents WHERE completed > 0;" \
  | # Convert to INSERT statements
```

### Considerations

1. **No `last_seen` column**: The current schema doesn't track when a torrent
   was last announced, limiting time-based filtering options.

2. **Auto-increment gaps**: After restore, new torrent IDs will continue from
   where they left off (sqlite_sequence table).

3. **Consistency**: Selective backup is inherently consistent since we're
   reading from a single database file.

## Raw Query Results

```text
Database: backup_maintenance_test.db
Size: 17 GB (4,297,957 pages × 4,096 bytes)

Tables:
- whitelist:                30,076 rows
- keys:                     10 rows
- torrents:                 161,454,700 rows
- torrent_aggregate_metrics: 1 row

Torrents breakdown:
- completed = 0:  156,482,966 (96.9%)
- completed > 0:  4,971,734 (3.1%)
- Max completed:  120,194
- Avg completed:  5.39 (for completed > 0)
```
