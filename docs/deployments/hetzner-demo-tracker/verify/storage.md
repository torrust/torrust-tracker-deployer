# Storage Volume Verification

**Status**: ✅ Verified (2026-03-04)

## Overview

Verifies that all persistent data is written to the attached Hetzner volume
(`/dev/sdb`, 50 GB) and **not** to the server's internal disk (`/dev/sda`).

## How to Verify

### 1. Confirm the volume is mounted

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "lsblk && df -h | grep -v tmpfs | grep -v udev"
```

### 2. Confirm the storage directory is on the volume

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "df -h /opt/torrust/storage"
```

### 3. Inspect the storage tree

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  find /opt/torrust/storage -maxdepth 3 | sort
  sudo du -sh /opt/torrust/storage/mysql/
"
```

## Results (2026-03-04)

### Block devices

```text
NAME    MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
sda       8:0    0 152.6G  0 disk
|-sda1    8:1    0 152.3G  0 part /
|-sda14   8:14   0     1M  0 part
`-sda15   8:15   0   256M  0 part /boot/efi
sdb       8:16   0    50G  0 disk /opt/torrust/storage
```

`sdb` (the attached Hetzner volume) is mounted at `/opt/torrust/storage`.
`sda` (the internal disk) only holds the OS (`/`) and the EFI partition.

### Filesystem usage

```text
Filesystem  Size  Used  Avail  Use%  Mounted on
/dev/sda1   150G  4.1G   140G    3%  /
/dev/sda15  253M  146K   252M    1%  /boot/efi
/dev/sdb     49G  261M    47G    1%  /opt/torrust/storage
```

### Docker Compose volume mounts

The `docker-compose.yml` uses **relative paths** (`./storage/...`) for all
volume mounts. Because the working directory is `/opt/torrust` and
`/opt/torrust/storage` is on `sdb`, every bind mount resolves to the Hetzner
volume:

| Service    | Host path (→ sdb)                | Container path              | Purpose            |
| ---------- | -------------------------------- | --------------------------- | ------------------ |
| caddy      | `./storage/caddy/etc/Caddyfile`  | `/etc/caddy/Caddyfile`      | Config (read-only) |
| caddy      | `./storage/caddy/data`           | `/data`                     | TLS certificates   |
| caddy      | `./storage/caddy/config`         | `/config`                   | Runtime config     |
| tracker    | `./storage/tracker/lib`          | `/var/lib/torrust/tracker`  | Tracker library    |
| tracker    | `./storage/tracker/log`          | `/var/log/torrust/tracker`  | Logs               |
| tracker    | `./storage/tracker/etc`          | `/etc/torrust/tracker`      | Config             |
| mysql      | `./storage/mysql/data`           | `/var/lib/mysql`            | MySQL data files   |
| prometheus | `./storage/prometheus/etc`       | `/etc/prometheus`           | Config             |
| grafana    | `./storage/grafana/data`         | `/var/lib/grafana`          | Grafana database   |
| grafana    | `./storage/grafana/provisioning` | `/etc/grafana/provisioning` | Dashboards/DS      |

### Data on the volume

```text
/opt/torrust/storage/backup/etc/backup.conf
/opt/torrust/storage/caddy/data/caddy/           ← TLS certificates
/opt/torrust/storage/caddy/etc/Caddyfile
/opt/torrust/storage/grafana/data/grafana.db     ← Grafana database
/opt/torrust/storage/grafana/provisioning/
/opt/torrust/storage/mysql/                      ← 209 MB MySQL data
/opt/torrust/storage/mysql/data/torrust_tracker/ ← tracker database
/opt/torrust/storage/prometheus/etc/prometheus.yml
/opt/torrust/storage/tracker/etc/tracker.toml
/opt/torrust/storage/tracker/lib/database/       ← 8 KB (SQLite stub)
```

MySQL occupies ~209 MB on the volume. The volume has 47 GB free (1% used).

## Conclusion

All persistent data — MySQL, TLS certificates, Grafana database, tracker
config, and logs — is written to the attached 50 GB Hetzner volume (`sdb`).
The server's internal disk is used only for the OS. If the server is destroyed
and a new one is created with the same volume attached and mounted at
`/opt/torrust/storage`, all data will be preserved.
