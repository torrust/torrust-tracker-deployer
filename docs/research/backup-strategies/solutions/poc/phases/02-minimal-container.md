# Phase 2: Minimal Backup Container

**Status**: ✅ Complete
**Date**: 2026-01-29

## Goal

Build and run a backup container that does nothing but log a message every
2 minutes.

## Checklist

- [x] Create `Dockerfile` for backup sidecar
- [x] Create minimal entrypoint that logs every 2 minutes
- [x] Manually add backup service to `docker-compose.yml` on the instance
- [x] Verify container starts and logs messages

## Artifacts

- Dockerfile: [../artifacts/Dockerfile](../artifacts/Dockerfile)
- Entrypoint: [../artifacts/entrypoint.sh](../artifacts/entrypoint.sh)
- Original docker-compose: [../artifacts/docker-compose-original.yml](../artifacts/docker-compose-original.yml)
- Docker Compose with backup: [../artifacts/docker-compose-with-backup.yml](../artifacts/docker-compose-with-backup.yml)

## Commands Executed

### 1. Create backup directory on instance

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "sudo mkdir -p /opt/torrust/backup && sudo chown torrust:torrust /opt/torrust/backup"
```

### 2. Copy Dockerfile and entrypoint

```bash
scp -i fixtures/testing_rsa \
  docs/research/backup-strategies/solutions/poc/artifacts/Dockerfile \
  torrust@10.140.190.35:/opt/torrust/backup/Dockerfile

scp -i fixtures/testing_rsa \
  docs/research/backup-strategies/solutions/poc/artifacts/entrypoint.sh \
  torrust@10.140.190.35:/opt/torrust/backup/entrypoint.sh
```

### 3. Copy new docker-compose.yml

```bash
scp -i fixtures/testing_rsa \
  docs/research/backup-strategies/solutions/poc/artifacts/docker-compose-with-backup.yml \
  torrust@10.140.190.35:/tmp/docker-compose.yml

ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "sudo cp /tmp/docker-compose.yml /opt/torrust/docker-compose.yml && \
   sudo chown torrust:torrust /opt/torrust/docker-compose.yml"
```

### 4. Build and start backup container

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "cd /opt/torrust && docker compose up -d --build backup"
```

**Output**:

```text
#1 [backup internal] load build definition from Dockerfile
...
Container mysql  Running
Container backup  Created
Container backup  Started
```

## Validation

### Docker Compose Services

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "cd /opt/torrust && docker compose ps"
```

**Output**:

```text
NAME         IMAGE                     STATUS
backup       torrust-backup            Up 15 seconds
grafana      grafana/grafana:12.3.1    Up 20 minutes (healthy)
mysql        mysql:8.4                 Up 20 minutes (healthy)
prometheus   prom/prometheus:v3.5.0    Up 20 minutes (healthy)
tracker      torrust/tracker:develop   Up 20 minutes (healthy)
```

✅ All 5 services running.

### Backup Container Logs

```bash
ssh -i fixtures/testing_rsa torrust@10.140.190.35 \
  "cd /opt/torrust && docker compose logs backup"
```

**Output**:

```text
backup  | [2026-01-29 18:23:49] Backup sidecar starting...
backup  | [2026-01-29 18:23:49] Backup interval: 120 seconds
backup  | [2026-01-29 18:23:49] Backup service running...
```

✅ Backup container logging messages as expected.

## Issues Encountered

### Permission denied for docker-compose.yml

**Problem**: Direct scp to `/opt/torrust/docker-compose.yml` failed with
permission denied.

**Solution**: Copy to `/tmp/` first, then use `sudo cp` to move it.

### SSH host key changed

**Problem**: LXD VM had different host key from previous session.

**Solution**: `ssh-keygen -f ~/.ssh/known_hosts -R 10.140.190.35`

## Next Steps

Proceed to [Phase 3: MySQL Backup](03-mysql-backup.md).
