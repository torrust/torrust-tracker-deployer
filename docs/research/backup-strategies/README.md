# Backup Strategies Research

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)
**Parent Epic**: [#309 - Add backup support](https://github.com/torrust/torrust-tracker-deployer/issues/309)

## Overview

This folder contains research documentation for backup strategies in the context of Torrust Tracker deployments. The research covers different backup types, database-specific approaches, and general requirements.

> **📋 See [Conclusions](conclusions.md) for a summary of key findings and recommendations.**

## Backup Types

The research is organized around three backup types:

| Type                 | Description                                     | Status      |
| -------------------- | ----------------------------------------------- | ----------- |
| **Storage Backup**   | Complete backup of entire storage (config + DB) | Not started |
| **Database Backup**  | Database-specific backup using safe tools       | In progress |
| **Selective Backup** | Partial storage backup (e.g., config only)      | Not started |

See [requirements.md](requirements.md) for detailed explanation of each type.

## Documents

### General

| Document                        | Description                                                      |
| ------------------------------- | ---------------------------------------------------------------- |
| [Requirements](requirements.md) | Collected requirements, constraints, and backup type definitions |
| [Conclusions](conclusions.md)   | Summary of key findings and recommendations                      |

### Database-Specific

| Folder                                 | Description                                                          |
| -------------------------------------- | -------------------------------------------------------------------- |
| [databases/sqlite/](databases/sqlite/) | SQLite backup research (approaches, current implementation analysis) |
| [databases/mysql/](databases/mysql/)   | MySQL backup research (mysqldump, hot backups, locking behavior)     |

### Architectures

| Document                                                                   | Description                                  |
| -------------------------------------------------------------------------- | -------------------------------------------- |
| [architectures/container-patterns.md](architectures/container-patterns.md) | Container-based backup architecture patterns |

### Tools

| Document         | Description                        |
| ---------------- | ---------------------------------- |
| [tools/](tools/) | Backup tool research (restic, etc) |

### Solutions

| Folder                   | Description                                          |
| ------------------------ | ---------------------------------------------------- |
| [solutions/](solutions/) | Proposed backup solutions and architectural patterns |

**⭐ Recommended**: [Sidecar Container Pattern](solutions/sidecar-container/) - A dedicated backup container in the Docker Compose stack that handles all backup operations portably.

## Research Status

### SQLite

- [x] Document backup approaches
- [x] Analyze Torrust Live Demo implementation
- [x] Investigate journal mode
- [ ] Test in containerized environment

### MySQL

- [x] Research mysqldump approaches
- [x] Research hot backup tools (Percona XtraBackup)
- [x] Document locking behavior (no lock needed for InnoDB)

### General

- [x] Document backup types
- [x] Collect requirements from discussions
- [ ] Research compression strategies
- [ ] Research retention policies
- [ ] Research restore procedures

## Key Decisions Captured

From research discussions:

1. **No data loss acceptable** - Safety is priority over simplicity
2. **User provides backup path** - Deployer is storage-location agnostic
3. **Keep complexity low** - No cloud provider APIs, focus on local backups
4. **Database-aware backups** - Simple `cp` not acceptable for databases
