# Research Database Backup Strategies

**Issue**: #310
**Parent Epic**: #309 - Add backup support
**Related**: [Roadmap Task 7.1](../../roadmap.md#7-add-backup-support)

## Overview

Research and document backup strategies for SQLite databases, MySQL databases, and configuration files used in Torrust Tracker deployments. This is a pure research task focused on **learning and collecting information** about available tools, techniques, and approaches. The goal is to understand how to safely backup data in production without proposing specific implementations yet.

## Goals

### SQLite Backup

- [x] Learn how to backup SQLite files safely while used in production (no locks, safe copying)
- [x] Research tools and techniques for copying and compressing SQLite backups
- [x] Investigate redundancy strategies for SQLite backups (cloud volumes, S3, backup services, snapshots)
- [x] Document current Torrust Live Demo SQLite backup implementation

### MySQL Backup

- [x] Research MySQL backup approaches for containerized deployments
- [x] Learn about MySQL-specific backup tools (mysqldump, hot backup, volume snapshots)
- [x] Investigate compression and redundancy strategies for MySQL backups

### Complete Storage Folder Backup

- [x] Research approaches for backing up the entire deployment storage folder
- [x] Learn about tools for full directory backups (tar, rsync, volume snapshots)
- [x] Understand trade-offs between full storage backup and selective approaches

### Selective Files Backup

- [x] Identify which configuration files and directories need backup
- [x] Research strategies for backing up specific files (docker-compose, tracker config, etc.)
- [x] Learn about version control and organization for selective backups

### General Research

- [x] Explore different backup scope strategies and their trade-offs
- [x] Document all findings in `docs/research/backup-strategies/` (to be created during research)

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (Research task - documentation only)
**Module Path**: `docs/research/backup-strategies/`
**Pattern**: Research documentation

### Research Scope

This is a **pure research and learning task** - no code implementation or specific design proposals are required. The goal is to gather information about available tools, techniques, and approaches. Implementation decisions will be made in task 7.2 after completing this research.

**Focus Areas:**

1. **Learning**: Understand how backup tools and techniques work
2. **Collection**: Gather information about available options
3. **Documentation**: Document findings for future reference

**Out of Scope:**

- ❌ Proposing specific implementation approaches
- ❌ Designing backup configuration schemas
- ❌ Planning DDD layer placement
- ❌ Creating Ansible playbooks or code

## Specifications

### Context: Current Torrust Live Demo Implementation

The Torrust Live Demo currently implements SQLite backups with the following approach:

1. **Backup Script**: Generates SQLite DB copies on-the-fly while the database is being used
2. **No Locking**: The tracker doesn't know the backup is happening - no lock mechanism to stop using the DB
3. **Schedule**: Runs once per day
4. **Rotation**: Keeps only 1 week of copies (7 daily backups)
5. **Compression**: Copies are compressed after generation
6. **Storage**: Copies stored in a backup folder (convenience for easy access, not independent volume)
7. **Infrastructure Backup**: The droplet also has daily snapshots at the infrastructure level

**Reference**: <https://github.com/torrust/torrust-tracker-live-demo>

This implementation provides a baseline for understanding practical backup approaches in production.

### SQLite Backup Research

Research and document approaches for backing up SQLite databases safely in production:

#### 1. Safe Backup While In Use

- **Question**: How can we backup SQLite files while they're actively being used?
- **Topics**:
  - File locking mechanisms in SQLite
  - Copy-on-write approaches
  - Online backup techniques
  - WAL (Write-Ahead Logging) mode and its impact
  - Risk assessment: backup without locks vs locked backups

#### 2. Copy and Compression

- **Question**: What tools and techniques exist for copying and compressing SQLite backups?
- **Topics**:
  - File copy commands (`cp`, `rsync`)
  - SQLite-specific backup tools (`.backup` command, `sqlite3` CLI)
  - Compression tools (`gzip`, `bzip2`, `xz`, `zstd`)
  - Compression timing (during or after backup)
  - Trade-offs between compression speed and ratio

#### 3. Redundancy and Storage

- **Question**: How can we ensure backup redundancy in cloud environments?
- **Topics**:
  - Attaching additional volumes to cloud instances
  - Copying backups to remote storage (S3, Backblaze B2, etc.)
  - Cloud provider snapshots vs file-level backups
  - Multi-region redundancy strategies
  - Cost considerations for different storage options

#### 4. Torrust Live Demo Analysis

- **Task**: Document and analyze the current Live Demo implementation
- **Questions**:
  - How does the script work technically?
  - What are the risks of copying without locks?
  - Why is this approach acceptable for the Live Demo?
  - What improvements could be considered?

### MySQL Backup Research

Research and document approaches for backing up MySQL databases in containerized environments:

#### 1. Logical Backup Tools

- **Question**: What tools exist for MySQL logical backups?
- **Topics**:
  - `mysqldump` usage and options
  - `mysqlpump` for parallel dumping
  - Locking behavior and transaction consistency
  - Performance impact on running database
  - Executing commands in Docker container context

#### 2. Physical Backup Approaches

- **Question**: What are the options for physical MySQL backups?
- **Topics**:
  - Docker volume copying strategies
  - Hot backup tools (Percona XtraBackup, MySQL Enterprise Backup)
  - When to stop/lock the database vs continuous operation
  - InnoDB-specific considerations

#### 3. Compression and Storage

- **Question**: How to compress and store MySQL backups?
- **Topics**:
  - Piping mysqldump to compression tools
  - Compressing volume backups
  - Storage destinations (volumes, S3, backup services)

#### 4. Redundancy for Containers

- **Question**: How to ensure MySQL backup redundancy in containerized deployments?
- **Topics**:
  - Docker volume snapshots
  - Copying volumes to attached storage
  - Remote storage integration
  - Container orchestration backup patterns

### Configuration Backup Research

Research approaches for backing up configuration files and deployment state:

#### 1. What to Backup

- **Question**: Which files and directories need backup?
- **Topics**:
  - Docker Compose files and environment files
  - Tracker configuration (TOML files)
  - Ansible playbooks and variables
  - Infrastructure configuration (Caddy, Prometheus, Grafana)
  - Build artifacts vs source templates

#### 2. File Copy Techniques

- **Question**: What tools and techniques exist for copying configuration files?
- **Topics**:
  - Simple file copy vs archive tools (`tar`, `zip`)
  - Directory synchronization (`rsync`)
  - Compression for config archives
  - Preserving permissions and ownership

#### 3. Storage Strategies

- **Question**: Where and how to store configuration backups?
- **Topics**:
  - Local backup directories
  - Attached volumes
  - Remote storage (S3, Git repositories)
  - Version control for configs

### Backup Scope Strategies

Research different approaches to defining backup scope:

#### Strategy Comparison

- **Question**: What are the trade-offs of different backup scopes?
- **Options to Research**:
  1. **Full Storage Folder**: Backup entire deployment directory
     - Pros: Complete state, simple to restore
     - Cons: Large size, includes unnecessary files
  2. **Database Only**: Backup only database files
     - Pros: Minimal size, fast backup
     - Cons: Requires manual configuration restoration
  3. **Database + Selective Config**: Backup DB and specific config files
     - Pros: Balance between completeness and size
     - Cons: Requires identifying critical configs
  4. **Layered Approach**: Different schedules/retention for different components
     - Pros: Optimized storage and recovery
     - Cons: More complex to implement

## Implementation Plan

### Phase 1: SQLite Research (estimated 4-6 hours)

- [x] Read SQLite backup documentation
- [x] Research safe file copy approaches while database is in use
- [x] Investigate SQLite locking mechanisms and WAL mode
- [x] Research compression tools and techniques
- [x] Learn about cloud volume attachment and snapshot strategies
- [x] Study S3 and backup service integration options
- [x] Analyze Torrust Live Demo backup script implementation
- [x] Document all findings in `docs/research/backup-strategies/sqlite-backup-strategies.md` (create folder and file)

### Phase 2: MySQL Research (estimated 4-6 hours)

- [x] Read MySQL backup documentation
- [x] Research `mysqldump` usage and locking behavior
- [x] Investigate physical backup tools (Percona XtraBackup)
- [x] Learn about Docker volume backup strategies
- [x] Research compression techniques for MySQL dumps
- [x] Study cloud redundancy options for MySQL backups
- [x] Test basic mysqldump in Docker container (optional hands-on)
- [x] Document all findings in `docs/research/backup-strategies/mysql-backup-strategies.md`

### Phase 3: Configuration Research (estimated 2-3 hours)

- [x] Identify all configuration files and directories
- [x] Research file copy and archive tools (`tar`, `rsync`)
- [x] Learn about compression options and trade-offs
- [x] Study configuration storage strategies
- [x] Research version control for config backups
- [x] Document all findings in `docs/research/backup-strategies/configuration-backup-strategies.md`

### Phase 4: Backup Scope Strategies (estimated 2-3 hours)

- [x] Research full storage backup approaches
- [x] Compare database-only backup patterns
- [x] Study selective backup strategies
- [x] Learn about layered backup approaches
- [x] Document trade-offs for each strategy
- [x] Document all findings in `docs/research/backup-strategies/backup-scope-strategies.md`

### Phase 5: Documentation Review (estimated 1 hour)

- [x] Review all research documents for completeness
- [x] Create README in research folder with overview
- [x] Ensure all research questions are addressed
- [x] Cross-reference with Torrust Live Demo implementation
- [x] Run linters and ensure documentation quality
- [x] Update issue with any follow-up questions or findings

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Research Documentation**:

- [x] SQLite backup approaches documented (safe copying, compression, redundancy)
- [x] MySQL backup approaches documented (tools, techniques, containerization)
- [x] Configuration backup approaches documented
- [x] Backup scope strategies compared
- [x] Torrust Live Demo implementation analyzed and documented
- [x] All research questions addressed with sufficient detail
- [x] Cloud redundancy strategies documented (volumes, S3, snapshots)
- [x] Compression techniques compared

**Research Completeness**:

- [x] All research questions in specifications section answered
- [x] Tools and techniques identified for each backup type
- [x] Trade-offs documented for different approaches
- [x] References to official documentation included
- [x] Findings organized in `docs/research/backup-strategies/` folder
- [x] README created in research folder with overview

**Documentation Quality**:

- [x] Markdown linting passes (markdownlint)
- [x] Spell checking passes (cspell)
- [x] All links valid and properly formatted
- [x] Code examples properly formatted with syntax highlighting (if any)

## Related Documentation

- [Roadmap](../../roadmap.md) - See task 7 for complete backup feature roadmap
- [Torrust Live Demo](https://github.com/torrust/torrust-tracker-live-demo) - Reference implementation
- [SQLite Backup Documentation](https://www.sqlite.org/backup.html)
- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [MySQL Backup Documentation](https://dev.mysql.com/doc/refman/8.0/en/backup-and-recovery.html)
- [mysqldump Documentation](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Error Handling Guide](../contributing/error-handling.md)

## Notes

### Research Focus

This is a **pure research task** focused on **learning and collecting information**. The goal is to understand:

- What tools and techniques are available
- How they work technically
- What trade-offs exist between different approaches
- What the Torrust Live Demo currently does and why

**Do not**:

- Propose specific implementation approaches
- Design configuration schemas
- Plan command architecture
- Write code or Ansible playbooks

Those activities belong to task 7.2 (Define backup feature specification) and will be informed by this research.

### Key Questions to Answer

1. **Safety**: How to backup databases safely while in production use?
2. **Tools**: What tools exist for copying, compressing, and storing backups?
3. **Redundancy**: How to ensure backups are stored redundantly in cloud environments?
4. **Scope**: What are the trade-offs of different backup scopes (full vs selective)?
5. **Current Practice**: How does the Torrust Live Demo handle backups today?

### Context: Why This Matters

The Torrust Tracker stores critical data (peer information, torrent stats, authentication tokens). Losing this data would be catastrophic for tracker operators. Backups must be:

- **Safe**: Don't corrupt the database during backup
- **Reliable**: Actually work when you need to restore
- **Automated**: Don't rely on manual intervention
- **Redundant**: Survive infrastructure failures
- **Cost-effective**: Balance storage costs with retention needs

### Out of Scope

The following are explicitly out of scope for this research task:

- ❌ Implementation of backup commands or logic
- ❌ Writing Ansible playbooks for backup operations
- ❌ Designing backup configuration schemas
- ❌ Planning DDD architecture or command structure
- ❌ Backup scheduling or retention policies (those are implementation details)
- ❌ Backup encryption (future enhancement)
- ❌ Automated backup testing (belongs to implementation phase)
- ❌ Specific implementation recommendations (that's for task 7.2)

This research should focus solely on understanding and documenting what exists, not on deciding what we should build.

### Dependencies

This research task (7.1) must be completed before starting:

- Task 7.2: Define backup feature specification (will use research findings to design)
- Task 7.3: Implement configuration file backups
- Task 7.4: Implement SQLite database backups
- Task 7.5: Implement MySQL database backups

The findings from this research will inform all subsequent backup-related tasks.
