# Phase 4: Configuration Files Backup

**Status**: 🔲 Not started
**Date**: -

## Goal

Add configuration files to the backup.

## Checklist

- [ ] Mount config directories as read-only in backup container
- [ ] Update backup script to copy `.env` and `tracker.toml`
- [ ] Copy Prometheus and Grafana configs
- [ ] Create staging directory structure

## Artifacts

- Config backup script: [../artifacts/scripts/backup-config.sh](../artifacts/scripts/backup-config.sh)

## Commands Executed

<!-- Will be populated during implementation -->

## Validation

**Expected**: Staging directory contains all config files:

```text
/backups/staging/
├── config/
│   ├── .env
│   ├── tracker.toml
│   ├── prometheus.yml
│   └── grafana-provisioning/
└── ...
```

## Issues Encountered

<!-- Will be populated if issues arise -->

## Next Steps

Proceed to [Phase 5: Archive Creation](05-archive-creation.md).
