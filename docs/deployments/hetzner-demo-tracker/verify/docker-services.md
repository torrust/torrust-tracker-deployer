# Docker Services Verification

**Status**: ✅ Verified (2026-03-04)

## How to Check

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 "
  cd /opt/torrust
  sudo docker compose ps
  sudo docker compose logs --tail=30 tracker
  sudo docker compose logs --tail=20 mysql
  sudo docker compose logs --tail=20 caddy
  sudo docker compose logs --tail=20 prometheus
  sudo docker compose logs --tail=20 grafana
"
```

## 1. Container Health Status

All containers must report `(healthy)`. Docker evaluates each service's
health check before setting this status.

Expected:

```text
NAME         IMAGE                     SERVICE      STATUS
caddy        caddy:2.10                caddy        Up X hours (healthy)
grafana      grafana/grafana:12.3.1    grafana      Up X hours (healthy)
mysql        mysql:8.4                 mysql        Up X hours (healthy)
prometheus   prom/prometheus:v3.5.0    prometheus   Up X hours (healthy)
tracker      torrust/tracker:develop   tracker      Up X hours (healthy)
```

### Actual Output (2026-03-04)

| Container  | Image                     | Status     |
| ---------- | ------------------------- | ---------- |
| caddy      | `caddy:2.10`              | ✅ healthy |
| grafana    | `grafana/grafana:12.3.1`  | ✅ healthy |
| mysql      | `mysql:8.4`               | ✅ healthy |
| prometheus | `prom/prometheus:v3.5.0`  | ✅ healthy |
| tracker    | `torrust/tracker:develop` | ✅ healthy |

## 2. Service Logs

### Tracker

✅ **Clean** — INFO level only. Logs show periodic health check polling and
Prometheus metrics scrapes, all returning `200 OK`. No warnings or errors.

### MySQL

⚠️ **Expected warnings at startup** — three cosmetic warnings that appear on
every MySQL 8.4 container start:

- `Unable to load '/usr/share/zoneinfo/zone.tab' as time zone` — MySQL 8.4
  cosmetic warning; timezone data not installed in the container image. Does
  not affect operation.
- `CA certificate ca.pem is self signed` — default self-signed cert for
  encrypted connections; not used in this deployment.
- `Insecure configuration for --pid-file` — the `/var/run/mysqld` path is
  accessible to all OS users inside the container; harmless in Docker context.

No errors. Database initialization log shows `torrust_tracker` database and
`torrust` user were created successfully.

### Caddy

⚠️ **Two categories of expected noise** — no application errors:

1. **Transient DNS errors at startup** (`ERROR: dial tcp: lookup grafana on
127.0.0.11:53: server misbehaving`): Docker's internal DNS resolver
   was not ready when Caddy first tried to resolve `grafana`. These appear
   only during the first seconds after `docker compose up` and self-resolve.
   Not present in steady-state operation.

2. **WARN: aborting with incomplete response**: External bots and scanners
   (probing for `/wp-login.php`, `/wp-admin/`, `/administrator/`) dropped
   TCP connections before Caddy finished sending the response. No legitimate
   traffic is affected.

### Prometheus

✅ **Clean** — INFO level only. Startup, WAL replay, and scrape configuration
loaded successfully. No warnings or errors.

### Grafana

✅ **Clean** — INFO level only. The only notable entries:

- Periodic cleanup jobs and plugin update checks — expected background tasks.
- `404` responses for `/api/dashboards/uid/*/public-dashboards` — these are
  expected because public dashboards are not configured. Grafana logs these
  at INFO (not ERROR), and they do not affect dashboard functionality.

## Results

| Check                  | Result | Notes                                               |
| ---------------------- | ------ | --------------------------------------------------- |
| All containers healthy | ✅     | All 5 report `(healthy)` status                     |
| Tracker logs clean     | ✅     | INFO only                                           |
| MySQL logs clean       | ⚠️     | 3 cosmetic startup warnings — expected, harmless    |
| Caddy logs clean       | ⚠️     | Transient DNS + bot scan WARNs — expected, harmless |
| Prometheus logs clean  | ✅     | INFO only                                           |
| Grafana logs clean     | ✅     | INFO only; 404 for unconfigured public dashboards   |
