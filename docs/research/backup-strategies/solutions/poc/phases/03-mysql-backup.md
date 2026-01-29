# Phase 3: MySQL Backup

**Status**: ✅ Complete
**Date**: 2026-01-29

## Goal

Backup MySQL database using `mysqldump --single-transaction`.

## Checklist

- [x] Install `mysql-client` in backup container
- [x] Create backup script that runs `mysqldump`
- [x] Mount backup output directory
- [x] Configure MySQL credentials via environment variables
- [x] Run backup and verify `.sql.gz` file is created

## Artifacts

- Dockerfile: [../artifacts/backup-container/Dockerfile](../artifacts/backup-container/Dockerfile)
- Backup script: [../artifacts/backup-container/backup-mysql.sh](../artifacts/backup-container/backup-mysql.sh)
- Docker Compose: [../artifacts/docker-compose-with-backup.yml](../artifacts/docker-compose-with-backup.yml)

## Commands Executed

### 1. Copy updated files to VM

```bash
scp -i fixtures/testing_rsa \
  docs/research/backup-strategies/solutions/poc/artifacts/Dockerfile \
  docs/research/backup-strategies/solutions/poc/artifacts/entrypoint.sh \
  docs/research/backup-strategies/solutions/poc/artifacts/backup-mysql.sh \
  torrust@10.140.190.35:/opt/torrust/backup/

scp -i fixtures/testing_rsa \
  docs/research/backup-strategies/solutions/poc/artifacts/docker-compose-with-backup.yml \
  torrust@10.140.190.35:/tmp/docker-compose.yml

ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "sudo cp /tmp/docker-compose.yml /opt/torrust/docker-compose.yml && \
   sudo chown torrust:torrust /opt/torrust/docker-compose.yml"
```

### 2. Create backup storage directory

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "sudo mkdir -p /opt/torrust/storage/backup && \
   sudo chown torrust:torrust /opt/torrust/storage/backup"
```

### 3. Rebuild and restart backup container

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "cd /opt/torrust && docker compose up -d --build backup"
```

## Validation

### Backup Container Logs

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "cd /opt/torrust && docker compose logs backup --tail 10"
```

**Output**:

```text
backup  | [2026-01-29 18:38:24] Backup sidecar starting...
backup  | [2026-01-29 18:38:24] Backup interval: 120 seconds
backup  | [2026-01-29 18:38:24] Running initial backup...
backup  | [2026-01-29 18:38:24] Starting MySQL backup...
backup  | [2026-01-29 18:38:24] Database: torrust_tracker@mysql:3306
backup  | [2026-01-29 18:38:24] ✅ Backup complete: /backups/mysql_20260129_183824.sql.gz (4.0K)
```

### Backup File Contents

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "ls -la /opt/torrust/storage/backup/ && \
   zcat /opt/torrust/storage/backup/mysql_20260129_183824.sql.gz | head -30"
```

**Output**:

```text
-rw-r--r-- 1 root root 964 Jan 29 18:38 mysql_20260129_183824.sql.gz

-- MariaDB dump 10.19  Distrib 10.11.14-MariaDB
-- Host: mysql    Database: torrust_tracker
-- Server version       8.4.8

DROP TABLE IF EXISTS `keys`;
CREATE TABLE `keys` (
  `id` int NOT NULL AUTO_INCREMENT,
  `key` varchar(32) NOT NULL,
  ...
```

✅ Backup contains valid SQL dump with table definitions.

## Issues Encountered

### Alpine's mysql-client is MariaDB

**Problem**: Alpine's `mysql-client` package is actually MariaDB client, which
doesn't support MySQL 8's `caching_sha2_password` authentication.

**Error**: `Plugin caching_sha2_password could not be loaded`

**Solution**: Switch to Debian-based image (`debian:bookworm-slim`) with
`default-mysql-client` which works with MySQL 8.

### SSL Certificate Error

**Problem**: TLS/SSL error with self-signed certificate in Docker network.

**Solution**: Added `--ssl=0` to disable SSL for internal Docker network
communication.

### PROCESS Privilege Warning

**Problem**: `Access denied; you need PROCESS privilege for tablespaces`

**Solution**: Added `--no-tablespaces` flag. Tablespace info is not needed for
logical restore.

## Key Findings

1. **MariaDB client tools are fully compatible with MySQL**: We use
   `mariadb-dump` from Debian's `default-mysql-client` package to backup MySQL
   8.4 databases. The tools are fully compatible despite being from different
   projects. Note: The tracker uses MySQL as the database engine (there's an
   existing ADR about choosing MySQL over MariaDB that will be added to this
   repo).

2. **Use `mariadb-dump` not `mysqldump`**: The `mysqldump` command is a symlink
   that shows a deprecation warning. Using `mariadb-dump` directly avoids this.

3. **Docker network SSL**: Internal Docker networks don't need SSL; disabling
   it with `--ssl=0` simplifies connectivity.

4. **Minimal privileges**: Standard database user doesn't need PROCESS
   privilege for logical backups. Use `--no-tablespaces` to avoid the warning.

## Next Steps

Proceed to [Phase 4: Configuration Files Backup](04-config-backup.md).
