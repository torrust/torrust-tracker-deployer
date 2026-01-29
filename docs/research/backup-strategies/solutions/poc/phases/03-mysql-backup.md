# Phase 3: MySQL Backup

**Status**: 🔲 Not started
**Date**: -

## Goal

Backup MySQL database using `mysqldump --single-transaction`.

## Checklist

- [ ] Install `mysql-client` in backup container
- [ ] Create backup script that runs `mysqldump`
- [ ] Mount backup output directory
- [ ] Configure MySQL credentials via environment variables
- [ ] Run backup and verify `.sql.gz` file is created

## Artifacts

- Backup script: [../artifacts/scripts/backup-mysql.sh](../artifacts/scripts/backup-mysql.sh)

## Commands Executed

<!-- Will be populated during implementation -->

## Validation

**Expected**: Backup file exists and can be inspected:

```bash
ls -la /opt/torrust/backups/
zcat /opt/torrust/backups/mysql_*.sql.gz | head -50
```

## Issues Encountered

<!-- Will be populated if issues arise -->

## Next Steps

Proceed to [Phase 4: Configuration Files Backup](04-config-backup.md).
