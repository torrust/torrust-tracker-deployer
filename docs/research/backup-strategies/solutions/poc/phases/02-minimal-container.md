# Phase 2: Minimal Backup Container

**Status**: 🔲 Not started
**Date**: -

## Goal

Build and run a backup container that does nothing but log a message every
2 minutes.

## Checklist

- [ ] Create `docker/backup-sidecar/Dockerfile`
- [ ] Create minimal entrypoint that logs every 2 minutes
- [ ] Manually add backup service to `docker-compose.yml` on the instance
- [ ] Verify container starts and logs messages

## Artifacts

- Dockerfile: [../artifacts/Dockerfile](../artifacts/Dockerfile)
- Entrypoint: [../artifacts/scripts/entrypoint.sh](../artifacts/scripts/entrypoint.sh)
- Docker Compose additions: [../artifacts/docker-compose-backup.yml](../artifacts/docker-compose-backup.yml)

## Commands Executed

<!-- Will be populated during implementation -->

## Validation

**Expected**: `docker compose logs backup` shows periodic log entries like:

```text
backup  | [2026-01-29 18:30:00] Backup service running...
backup  | [2026-01-29 18:32:00] Backup service running...
```

## Issues Encountered

<!-- Will be populated if issues arise -->

## Next Steps

Proceed to [Phase 3: MySQL Backup](03-mysql-backup.md).
