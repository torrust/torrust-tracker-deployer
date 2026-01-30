# SQLite Backup Strategies Research

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)
**Parent Epic**: [#309 - Add backup support](https://github.com/torrust/torrust-tracker-deployer/issues/309)

## Overview

This folder contains research documentation for SQLite database backup strategies in the context of Torrust Tracker deployments.

## Documents

| Document                                          | Description                                         |
| ------------------------------------------------- | --------------------------------------------------- |
| [Backup Approaches](backup-approaches.md)         | Comparison of different SQLite backup methods       |
| [Large Database Backup](large-database-backup.md) | **Critical**: Performance for databases >1GB        |
| [Torrust Live Demo](torrust-live-demo/)           | Research related to the production demo environment |

See also: [General Requirements Notes](../requirements-notes.md) for requirements that apply to all backup types.

## ⚠️ Database Size Considerations

**Critical finding from production testing**: SQLite's `.backup` command does not
scale well for large databases under concurrent load.

| Database Size | `.backup` Viability | Recommendation            |
| ------------- | ------------------- | ------------------------- |
| < 100 MB      | ✅ Excellent        | Use `.backup`             |
| 100 MB - 1 GB | ✅ Good             | Use `.backup`             |
| 1 GB - 5 GB   | ⚠️ Slow             | Consider alternatives     |
| 5 GB - 10 GB  | ❌ Impractical      | Use alternatives          |
| > 10 GB       | ❌ Unusable         | **Must use alternatives** |

**Real-world example**: A 17GB production database took ~37 MB/hour with `.backup`,
resulting in an estimated **17 days** to complete a single backup.

See [Large Database Backup](large-database-backup.md) for:

- Detailed performance analysis
- Alternative approaches (filesystem snapshots, Litestream, maintenance windows)
- Recommendations by deployment size

## Key Findings Summary

### Current State

- The Torrust Live Demo uses a simple file copy approach (`cp`) for daily SQLite backups with 7-day retention
- The demo database uses **delete** journal mode (not WAL)
- This approach works but has limitations around consistency and compression

### Key Learnings

- SQLite's `.backup` command provides safe, consistent backups
- Simple `cp` has corruption risks when database is being written to
- WAL mode requires backing up three files (unless using `.backup`)
- Compression can reduce backup size by 70-80%
- Backup integrity can be verified with `PRAGMA integrity_check`

### Critical Conclusion

**Using `.backup` instead of `cp` solves the main backup safety issue.** The `.backup` command:

- Uses SQLite's Online Backup API with proper locking
- Handles concurrent writes safely (auto-restarts if source modified)
- Works with both `delete` and `wal` journal modes
- Produces guaranteed consistent snapshots

**WAL mode is NOT required for safe backups.** It offers performance/concurrency benefits but is unrelated to backup safety.

## Related Resources

- [SQLite Backup API Documentation](https://www.sqlite.org/backup.html)
- [SQLite WAL Mode Documentation](https://www.sqlite.org/wal.html)
- [Torrust Live Demo Repository](https://github.com/torrust/torrust-demo)

## Status

- [x] Document backup approaches
- [x] Analyze current Torrust Live Demo implementation
- [x] Document requirements from discussions
- [ ] Research compression options in detail
- [ ] Research retention strategies
- [ ] Test backup approaches in containerized environment
