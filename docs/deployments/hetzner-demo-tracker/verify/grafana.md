# Grafana Verification

**Status**: ✅ Verified (2026-03-04)

## Endpoint

`https://grafana.torrust-tracker-demo.com`

## Credentials

- **Username**: `admin`
- **Password**: see `envs/torrust-tracker-demo.json` → `grafana.admin_password`

## 1. TLS Certificate and Login Page

Open in a browser or check with curl:

```bash
curl -sv --head "https://grafana.torrust-tracker-demo.com" 2>&1 | grep -E "HTTP|subject|issuer"
```

Expected: HTTP 302 redirect to `/login` (or direct 200), valid Let's Encrypt
certificate for `grafana.torrust-tracker-demo.com`.

## 2. API Login Check

Verify the credentials work via the Grafana HTTP API.

> **Note**: use `-u` flag — URL-embedded credentials (`admin:pass@host`) will
> fail if the password contains `/`, as does this deployment's password.

```bash
GRAFANA_PASS="<GRAFANA_ADMIN_PASSWORD>"
curl -s -u "admin:${GRAFANA_PASS}" "https://grafana.torrust-tracker-demo.com/api/user" | python3 -m json.tool
```

Expected response:

```json
{
  "id": 1,
  "email": "admin@localhost",
  "name": "admin",
  "login": "admin",
  "role": "Admin",
  ...
}
```

## 3. Data Source — Prometheus Connected

Confirm Grafana is receiving metrics from Prometheus.

```bash
GRAFANA_PASS="<GRAFANA_ADMIN_PASSWORD>"
curl -s -u "admin:${GRAFANA_PASS}" "https://grafana.torrust-tracker-demo.com/api/datasources" | python3 -m json.tool
```

Expected: a data source named `Prometheus` with `"type": "prometheus"` and
`"url": "http://prometheus:9090"`.

## 4. Dashboards Provisioned

Confirm the pre-provisioned dashboards are present.

```bash
GRAFANA_PASS="<GRAFANA_ADMIN_PASSWORD>"
curl -s -u "admin:${GRAFANA_PASS}" "https://grafana.torrust-tracker-demo.com/api/dashboards/home" | python3 -m json.tool
```

For a full list of dashboards:

```bash
curl -s -u "admin:${GRAFANA_PASS}" "https://grafana.torrust-tracker-demo.com/api/search?type=dash-db" | python3 -m json.tool
```

Expected: one or more dashboards from the `torrust` folder as configured in
`templates/grafana/provisioning/dashboards/`.

## 5. Browser Verification

Navigate to `https://grafana.torrust-tracker-demo.com` in a browser, log in with
admin credentials, and confirm:

- Dashboards load without errors
- Prometheus data source shows a green "Data source connected" status
  (Settings → Data sources → Prometheus → Test)
- The tracker metrics dashboard shows panels (values may be zero on a fresh
  deployment with no traffic)

## Results

| Check                          | Result | Notes                                                        |
| ------------------------------ | ------ | ------------------------------------------------------------ |
| TLS certificate valid          | ✅     | Let's Encrypt, valid until Jun 2, 2026                       |
| Login page reachable           | ✅     | HTTP 302 redirect to `/login`                                |
| API login with credentials     | ✅     | Returns admin user details                                   |
| Prometheus data source present | ✅     | `http://prometheus:9090`, default, `readOnly: true`          |
| Dashboards provisioned         | ✅     | 2 dashboards in "Torrust Tracker" folder (metrics and stats) |
| Browser login and dashboard    | ⏳     | Manual browser check not yet performed                       |
