# MySQL Backup Research

This directory contains research on MySQL backup approaches for the Torrust
Tracker Deployer.

## Context

Torrust Tracker supports MySQL as a database backend. Understanding MySQL
backup options is essential for:

- Production deployments using MySQL/MariaDB
- High-availability scenarios
- Point-in-time recovery requirements

## Documents

| Document                                     | Description                                 |
| -------------------------------------------- | ------------------------------------------- |
| [backup-approaches.md](backup-approaches.md) | Comprehensive guide to MySQL backup methods |

## Key Findings

### Comparison with SQLite

| Aspect                 | SQLite                | MySQL (InnoDB)                                      |
| ---------------------- | --------------------- | --------------------------------------------------- |
| **Lock-free backup**   | `.backup` command     | `--single-transaction` option                       |
| **Safe while running** | ✅ Yes (with .backup) | ✅ Yes (for InnoDB tables)                          |
| **Hot backup tools**   | Built-in              | Percona XtraBackup (free) / MySQL Enterprise Backup |
| **Complexity**         | Simple                | More options/complexity                             |
| **Incremental backup** | Not native            | Binary log / XtraBackup                             |

### Recommended Approach

For containerized Torrust Tracker with MySQL:

1. **Simple deployments**: Use `mysqldump --single-transaction`
2. **Large databases**: Consider Percona XtraBackup
3. **Backup tool**: Restic (same as SQLite)

## Related Documents

- [SQLite Backup Approaches](../sqlite/backup-approaches.md)
- [Restic Integration](../tools/restic.md)
