# Backup Tools Evaluation

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This folder contains evaluations of backup tools that could be integrated into the Torrust Tracker Deployer. The goal is to find a well-tested, reliable tool rather than implementing backup logic from scratch.

## Design Preferences

Based on initial analysis:

1. **Use external tools** - More reliable, less maintenance burden
2. **Centralized backups** - Acceptable if the tool is flexible
3. **Docker-based** - Easy to move the entire stack, consistent with our approach
4. **Pre-built images preferred** - Avoid maintaining custom backup containers

## Evaluation Criteria

| Criterion          | Priority | Description                              |
| ------------------ | -------- | ---------------------------------------- |
| Docker support     | High     | Official or well-maintained Docker image |
| SQLite support     | High     | Via `.backup` command pre-hook           |
| MySQL support      | High     | Via `mysqldump` pre-hook                 |
| Local backups      | High     | Backup to local volume/disk              |
| Remote backups     | Medium   | S3, B2, SFTP, etc.                       |
| Encryption         | Medium   | Data encryption at rest                  |
| Deduplication      | Medium   | Efficient storage for large DBs          |
| Retention policies | Medium   | Automatic cleanup of old backups         |
| Verification       | Medium   | Integrity checking of backups            |
| Compression        | Low      | Usually handled by the tool              |
| Incremental        | Low      | Nice to have for large files             |

## Tools Under Evaluation

| Tool                        | Status           | Summary                                   |
| --------------------------- | ---------------- | ----------------------------------------- |
| [Restic](restic.md)         | ✅ Recommended   | Modern, encrypted, deduplicated backups   |
| [Kopia](restic-vs-kopia.md) | ⚠️ Alternative   | Newer, more features, less mature         |
| Borg                        | ❓ Not evaluated | Similar to Restic, needs rclone for cloud |
| Duplicati                   | ❓ Not evaluated | GUI-focused, .NET-based                   |

### Discarded Tools

| Tool                             | Reason                                                                                                                                                  |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Rustic](https://rustic.cli.rs/) | Beta status - "not recommended for production backups, yet". Rust reimplementation of Restic with same repo format. Worth revisiting if it reaches 1.0. |

## Key Question

**Does the tool backup databases directly, or does it backup files?**

Most backup tools (Restic, Borg, etc.) backup **files**, not databases. For databases:

```text
┌─────────────────────────────────────────────────────────┐
│                  Two-Phase Backup                       │
│                                                         │
│  Phase 1: Database Dump (pre-hook script)               │
│  ─────────────────────────────────────────              │
│  - SQLite: sqlite3 db.db ".backup /tmp/dump.db"         │
│  - MySQL:  mysqldump -h host -u user -p db > dump.sql   │
│                                                         │
│  Phase 2: File Backup (Restic/Borg/etc.)                │
│  ─────────────────────────────────────────              │
│  - Backup the dump files                                │
│  - Handle encryption, deduplication, transfer           │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

This is actually a **good design**:

- Separation of concerns (database tools know databases, backup tools know storage)
- Each tool does what it's best at
- Pre-hooks are simple scripts we control
