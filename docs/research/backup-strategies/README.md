# Backup Strategies Research

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)
**Parent Epic**: [#309 - Add backup support](https://github.com/torrust/torrust-tracker-deployer/issues/309)

## Overview

This folder contains research documentation for backup strategies in the context of Torrust Tracker deployments. The research covers different backup types, database-specific approaches, and general requirements.

## Backup Types

The research is organized around three backup types:

| Type                 | Description                                     | Status      |
| -------------------- | ----------------------------------------------- | ----------- |
| **Storage Backup**   | Complete backup of entire storage (config + DB) | Not started |
| **Database Backup**  | Database-specific backup using safe tools       | In progress |
| **Selective Backup** | Partial storage backup (e.g., config only)      | Not started |

See [requirements-notes.md](requirements-notes.md) for detailed explanation of each type.

## Documents

### General

| Document                                    | Description                                                      |
| ------------------------------------------- | ---------------------------------------------------------------- |
| [Requirements Notes](requirements-notes.md) | Collected requirements, constraints, and backup type definitions |

### Database-Specific

| Folder             | Description                                                          |
| ------------------ | -------------------------------------------------------------------- |
| [sqlite/](sqlite/) | SQLite backup research (approaches, current implementation analysis) |
| mysql/             | MySQL backup research (to be created)                                |

## Research Status

### SQLite

- [x] Document backup approaches
- [x] Analyze Torrust Live Demo implementation
- [x] Investigate journal mode
- [ ] Test in containerized environment

### MySQL

- [ ] Research mysqldump approaches
- [ ] Research hot backup tools
- [ ] Research Docker volume strategies

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
