# Exclude Statistics Backup

**Status**: 🔬 Proposed

## Summary

Backup only essential data (users, torrents, whitelists) and exclude statistics
tables that can grow very large.

## Problem

Statistics data (announce counts, peer history) causes database bloat, making
full backups slow or impractical for large deployments.

## Solution

Perform selective database dumps that exclude `torrust_*_stats` tables and other
non-essential data. This dramatically reduces backup size and time.

## Trade-offs

| Advantage                           | Disadvantage                                |
| ----------------------------------- | ------------------------------------------- |
| Dramatically smaller backup size    | Historical statistics lost on restore       |
| Enables online backup for large DBs | Users needing stats must manage own backups |
| Statistics can be regenerated       | Partial data recovery                       |

## Implementation Notes

- MySQL: Use `mysqldump --ignore-table=db.stats_table`
- SQLite: Selective export with custom SQL queries
- Warn users to perform manual full backups if statistics are important
