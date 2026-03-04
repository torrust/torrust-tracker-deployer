# MySQL Database Verification

**Status**: ✅ Verified (2026-03-04)

## Overview

Verifies that the MySQL database is reachable from the tracker container and
that reads and writes work correctly. The tracker uses MySQL as its persistent
store for whitelisted torrents, authentication keys, torrent stats, and
aggregate metrics.

## Database Schema

Connect to MySQL using the `MYSQL_PWD` environment variable to avoid shell
escaping issues with the password (which contains a `/`):

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  cd /opt/torrust
  MYSQL_PWD=\$(sudo grep MYSQL_PASSWORD .env | cut -d= -f2 | tr -d \"'\")
  sudo docker compose exec -e MYSQL_PWD=\"\$MYSQL_PWD\" mysql \
    mysql -u torrust torrust_tracker -e 'SHOW TABLES;'
"
```

### Tables

```text
+-----------------------------+
| Tables_in_torrust_tracker   |
+-----------------------------+
| keys                        |
| torrent_aggregate_metrics   |
| torrents                    |
| whitelist                   |
+-----------------------------+
```

| Table                       | Description                                                        |
| --------------------------- | ------------------------------------------------------------------ |
| `keys`                      | Authentication keys (id, key varchar(32), valid_until int)         |
| `torrent_aggregate_metrics` | Named aggregate counters (id, metric_name varchar(50), value int)  |
| `torrents`                  | Persisted torrent stats (id, info_hash varchar(40), completed int) |
| `whitelist`                 | Whitelisted info hashes (id, info_hash varchar(40))                |

## Read/Write Verification

Verifying the tracker→MySQL connection requires an actual write. The simplest
approach is to add a torrent to the whitelist via the API and confirm it
appears in the `whitelist` table immediately.

> **Note**: The tracker is running in open mode (`listed = false`), so the
> whitelist table is not used for access control here. The entry can be safely
> deleted after the test.

### 1. Add a torrent to the whitelist

```bash
curl -s -X POST \
  "https://api.torrust-tracker-demo.com/api/v1/whitelist/0000000000000000000000000000000000000001?token=<ADMIN_TOKEN>"
```

Expected response:

```json
{ "status": "ok" }
```

### 2. Verify the row in the database

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  cd /opt/torrust
  MYSQL_PWD='<MYSQL_PASSWORD>'
  sudo docker compose exec -e MYSQL_PWD=\"\$MYSQL_PWD\" mysql \
    mysql -u torrust torrust_tracker \
    -e 'SELECT * FROM whitelist;'
"
```

Expected output:

```text
id  info_hash
1   0000000000000000000000000000000000000001
```

### 3. Clean up

```bash
curl -s -X DELETE \
  "https://api.torrust-tracker-demo.com/api/v1/whitelist/0000000000000000000000000000000000000001?token=<ADMIN_TOKEN>"
```

Expected response:

```json
{ "status": "ok" }
```

## Results (2026-03-04)

| Check                         | Result  |
| ----------------------------- | ------- |
| Tables present (4 tables)     | ✅ Pass |
| Whitelist write via API       | ✅ Pass |
| Row visible in DB immediately | ✅ Pass |
| Whitelist delete via API      | ✅ Pass |
| Row removed from DB           | ✅ Pass |

The tracker→MySQL connection is working correctly. Writes are synchronous —
rows appear in the database immediately after the API call completes.
