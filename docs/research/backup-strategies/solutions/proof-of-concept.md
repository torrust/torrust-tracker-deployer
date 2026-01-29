# Sidecar Container Backup - Proof of Concept

## Overview

This document tracks the practical implementation and validation of the
[sidecar container backup solution](sidecar-container.md).

**Goal**: Validate the sidecar container pattern by implementing it in a real
test environment, progressively adding features and documenting findings.

## Environment

| Setting         | Value                        |
| --------------- | ---------------------------- |
| Name            | `manual-test-sidecar-backup` |
| Provider        | LXD (local)                  |
| Database        | MySQL                        |
| Backup storage  | Local filesystem only        |
| Backup interval | 2 minutes (for testing)      |

## Implementation Plan

Each phase is a small, testable increment. We validate each phase works before
moving to the next.

### Phase 1: Environment Setup

**Goal**: Create and deploy a working tracker environment with MySQL.

- [ ] Create environment configuration file
- [ ] Run deployer commands: create → provision → configure → release → run
- [ ] Verify tracker is running and accessible
- [ ] Verify MySQL has tracker tables

**Validation**: Can access tracker API and MySQL shows InnoDB tables.

---

### Phase 2: Minimal Backup Container

**Goal**: Build and run a backup container that does nothing but log a message.

- [ ] Create `docker/backup-sidecar/Dockerfile`
- [ ] Create minimal entrypoint that logs every 2 minutes
- [ ] Manually add backup service to `docker-compose.yml` on the instance
- [ ] Verify container starts and logs messages

**Validation**: `docker compose logs backup` shows periodic log entries.

---

### Phase 3: MySQL Backup

**Goal**: Backup MySQL database using `mysqldump --single-transaction`.

- [ ] Install `mysql-client` in backup container
- [ ] Create backup script that runs `mysqldump`
- [ ] Mount backup output directory
- [ ] Configure MySQL credentials via environment variables
- [ ] Run backup and verify `.sql.gz` file is created

**Validation**: Backup file exists and can be inspected with `zcat`.

---

### Phase 4: Configuration Files Backup

**Goal**: Add configuration files to the backup.

- [ ] Mount config directories as read-only in backup container
- [ ] Update backup script to copy `.env` and `tracker.toml`
- [ ] Copy Prometheus and Grafana configs
- [ ] Create staging directory structure

**Validation**: Staging directory contains all expected config files.

---

### Phase 5: Archive Creation

**Goal**: Create timestamped tar.gz archives.

- [ ] Update script to create staging directory
- [ ] Stage all files (database dump + configs)
- [ ] Create compressed archive
- [ ] Implement 7-day local retention
- [ ] Clean up staging directory after archive

**Validation**: `backup_YYYYMMDD_HHMMSS.tar.gz` files in `/backups/`.

---

### Phase 6: Restore Validation

**Goal**: Verify backups can be restored.

- [ ] Extract archive to temp directory
- [ ] Verify all expected files are present
- [ ] Test MySQL restore to a test database
- [ ] Document restore procedure

**Validation**: Data can be read from restored backup.

---

### Phase 7: Documentation Update

**Goal**: Update research documentation with findings.

- [ ] Fix any errors found in sidecar-container.md
- [ ] Document final working configuration
- [ ] Add lessons learned
- [ ] Update implementation status checklist

---

## Directory Structure

```text
docker/
└── backup-sidecar/
    ├── Dockerfile
    ├── entrypoint.sh
    └── scripts/
        ├── backup-mysql.sh
        ├── backup-config.sh
        └── backup-all.sh
```

## Progress Log

### Phase 1: Environment Setup

**Status**: Not started

<!-- Log entries will be added here as we progress -->

---

## Commands Reference

### Deployer Commands

```bash
# Create environment
cargo run -- create environment --env-file envs/manual-test-sidecar-backup.json

# Provision (OpenTofu)
cargo run -- provision environment manual-test-sidecar-backup

# Configure (Ansible)
cargo run -- configure environment manual-test-sidecar-backup

# Release (Docker images)
cargo run -- release environment manual-test-sidecar-backup

# Run (Docker Compose up)
cargo run -- run environment manual-test-sidecar-backup

# SSH into instance
cargo run -- ssh environment manual-test-sidecar-backup
```

### Instance Commands (run inside VM)

```bash
# Check services
cd /opt/torrust
docker compose ps

# View backup logs
docker compose logs backup

# Check backup files
ls -la /opt/torrust/backups/

# Manual backup trigger
docker compose exec backup /scripts/backup-all.sh
```

### Cleanup

```bash
# Destroy environment when done
cargo run -- destroy environment manual-test-sidecar-backup
```

## Findings and Lessons Learned

<!-- Will be populated during implementation -->

## References

- [Sidecar Container Solution](sidecar-container.md)
- [MySQL Backup Approaches](../mysql/backup-approaches.md)
- [Manual E2E Testing Guide](../../../e2e-testing/manual-testing.md)
