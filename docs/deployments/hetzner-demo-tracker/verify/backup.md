# Backup Verification

**Status**: ✅ Verified (2026-03-04)

## Architecture

Backups run daily at 03:00 UTC via a host cron job that orchestrates a
graceful maintenance window:

```text
/etc/cron.d/tracker-backup (03:00 daily)
  └─▶ /usr/local/bin/maintenance-backup.sh
        ├─ stop tracker container
        ├─ docker compose --profile backup run --rm backup
        │    ├─ mysqldump torrust_tracker → mysql_<timestamp>.sql.gz
        │    └─ tar config files      → config_<timestamp>.tar.gz
        └─ start tracker container
```

The backup container uses the `backup` Docker Compose profile, so it is
**not** started on `docker compose up` — it only runs when explicitly invoked.

### Retention

Old backups older than **7 days** are deleted automatically at the end of each
backup cycle.

## What Gets Backed Up

### MySQL dump

The full `torrust_tracker` database is exported with `mysqldump` and compressed
with gzip. Output: `storage/backup/mysql/mysql_<YYYYMMDD_HHMMSS>.sql.gz`.

### Config files

The following files are archived into a tarball:
`storage/backup/config/config_<YYYYMMDD_HHMMSS>.tar.gz`.

| File                                                | Description                     |
| --------------------------------------------------- | ------------------------------- |
| `storage/tracker/etc/tracker.toml`                  | Tracker configuration           |
| `storage/prometheus/etc/prometheus.yml`             | Prometheus configuration        |
| `storage/grafana/provisioning/datasources/*.yml`    | Grafana datasource provisioning |
| `storage/grafana/provisioning/dashboards/*.yml`     | Grafana dashboard provisioning  |
| `storage/grafana/provisioning/dashboards/torrust/*` | Dashboard JSON definitions      |
| `storage/caddy/etc/Caddyfile`                       | Caddy reverse-proxy config      |

## How to Trigger a Manual Backup

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  cd /opt/torrust
  sudo docker compose --profile backup run --rm backup
"
```

## How to List Existing Backups

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  sudo find /opt/torrust/storage/backup -type f | sort
"
```

## How to Inspect a Backup

```bash
# List SQL dump tables
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  sudo zcat /opt/torrust/storage/backup/mysql/<dump>.sql.gz | grep '^CREATE TABLE'
"

# List config archive contents
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  sudo tar -tzf /opt/torrust/storage/backup/config/<archive>.tar.gz
"
```

## Issues Found and Fixed During Verification

### Oversight: backup.conf not updated after manual credentials fix

During the initial deployment the `run` command failed because the MySQL root
user password was not URL-encoded in `tracker.toml` (see Bug 1 and Bug 3 in
`commands/improvements.md`). That was fixed manually — the templates were
updated and the `run` command was retried.

However, `backup.conf` was also generated at the same time with the original
credentials, and was **not updated** when the tracker credentials were fixed.
As a result the backup container could not authenticate to MySQL:

| Setting   | Value in backup.conf | Required value    |
| --------- | -------------------- | ----------------- |
| `DB_USER` | `root`               | `torrust`         |
| `DB_NAME` | `torrust`            | `torrust_tracker` |

The fix was to align `backup.conf` with the credentials that were already
working for the tracker:

```bash
sudo sed -i \
  -e 's/^DB_USER=root/DB_USER=torrust/' \
  -e 's/^DB_NAME=torrust$/DB_NAME=torrust_tracker/' \
  /opt/torrust/storage/backup/etc/backup.conf
```

This is a process gap: whenever credentials are changed or fixed manually
after a deployment, all configuration files that reference those credentials
must be updated together — including `backup.conf`.

### Known warning: PROCESS privilege

The backup container uses `mariadb-dump` (from the MariaDB Docker image)
against a MySQL 8.4 server. MariaDB's dump client emits the following
non-critical warning:

```text
mysqldump: Error: 'Access denied; you need (at least one of) the PROCESS privilege(s)
for this operation' when trying to dump tablespaces
```

This only affects tablespace metadata, not the actual table data. All table
structures and rows are dumped correctly. The `torrust` user would need
`GRANT PROCESS ON *.* TO 'torrust'@'%'` or the `--no-tablespaces` flag added
to the `mysqldump` command to suppress this warning.

## Results (2026-03-04)

### Test run output

```text
[2026-03-04 16:07:58] Torrust Backup Container starting
[2026-03-04 16:07:58] Loading configuration from: /etc/backup/backup.conf
[2026-03-04 16:07:58] Configuration:
[2026-03-04 16:07:58]   Retention: 7 days
[2026-03-04 16:07:58]   Database: mysql
[2026-03-04 16:07:58]   Config paths file: /etc/backup/backup-paths.txt
[2026-03-04 16:07:58] Starting backup cycle
[2026-03-04 16:07:58] Starting MySQL backup: torrust_tracker@mysql:3306
[2026-03-04 16:07:58] MySQL backup completed: /backups/mysql/mysql_20260304_160758.sql.gz
[2026-03-04 16:07:59]   Size: 4.0K
[2026-03-04 16:07:59] Starting config files backup
[2026-03-04 16:07:59] Config backup completed: /backups/config/config_20260304_160759.tar.gz
[2026-03-04 16:07:59]   Files backed up: 4
[2026-03-04 16:07:59]   Size: 8.0K
[2026-03-04 16:07:59] Cleaning up backups older than 7 days
[2026-03-04 16:07:59]   No old backups to delete
[2026-03-04 16:07:59] Backup cycle completed successfully
```

### SQL dump content

```text
-- Host: mysql    Database: torrust_tracker
CREATE TABLE `keys` (...)
CREATE TABLE `torrent_aggregate_metrics` (...)
CREATE TABLE `torrents` (...)
CREATE TABLE `whitelist` (...)
```

### Config archive content

```text
data/storage/tracker/etc/tracker.toml
data/storage/prometheus/etc/prometheus.yml
data/storage/grafana/provisioning/datasources/prometheus.yml
data/storage/grafana/provisioning/dashboards/torrust.yml
data/storage/grafana/provisioning/dashboards/torrust/stats.json
data/storage/grafana/provisioning/dashboards/torrust/metrics.json
data/storage/caddy/etc/Caddyfile
```

### Verification summary

| Check                            | Result                                                  |
| -------------------------------- | ------------------------------------------------------- |
| Cron job installed               | ✅ `/etc/cron.d/tracker-backup`                         |
| Schedule                         | ✅ Daily at 03:00 UTC                                   |
| Maintenance script present       | ✅ `/usr/local/bin/maintenance-backup.sh`               |
| Manual test run                  | ✅ Exit code 0                                          |
| MySQL dump created               | ✅ `mysql_20260304_160758.sql.gz` (4 KB)                |
| All 4 tables present in dump     | ✅ keys, torrent_aggregate_metrics, torrents, whitelist |
| Config archive created           | ✅ `config_20260304_160759.tar.gz` (8 KB, 4 paths)      |
| Retention policy                 | ✅ 7 days                                               |
| Backups stored on volume (`sdb`) | ✅ `/opt/torrust/storage/backup/`                       |
| Wrong DB_USER in backup.conf     | ⚠️ Oversight — fixed (root → torrust)                   |
| Wrong DB_NAME in backup.conf     | ⚠️ Oversight — fixed (torrust → torrust_tracker)        |
| PROCESS privilege warning        | ⚠️ Non-critical (tablespaces only)                      |
