# Tracker API Verification

**Status**: ⏳ Not yet verified

## Endpoint

`https://api.torrust-tracker-demo.com/api/v1`

## Authentication

All API endpoints require the admin token as a query parameter or header.

Admin token: see `envs/torrust-tracker-demo.json` → `tracker.http_api.admin_token`

```bash
# Set token for reuse in the commands below
TOKEN="thmbSikMOIzdJXLT0EMRrx9uyiio4wMeVA75x99cRyM="
```

## 1. TLS Certificate Check

```bash
curl -v --head "https://api.torrust-tracker-demo.com/api/v1/stats?token=$TOKEN" 2>&1 | grep -E "subject|issuer|SSL|HTTP"
```

Expected: valid Let's Encrypt certificate, HTTP 200.

## 2. Tracker Statistics

Fetch global tracker statistics (total torrents, peers, etc.).

```bash
TOKEN="thmbSikMOIzdJXLT0EMRrx9uyiio4wMeVA75x99cRyM="
curl -s "https://api.torrust-tracker-demo.com/api/v1/stats?token=$TOKEN" | python3 -m json.tool
```

Expected response structure:

```json
{
  "torrents": 0,
  "seeders": 0,
  "completed": 0,
  "leechers": 0,
  "tcp4_connections_handled": 0,
  "tcp4_announces_handled": 0,
  "tcp4_scrapes_handled": 0,
  "udp4_connections_handled": 0,
  "udp4_announces_handled": 0,
  "udp4_scrapes_handled": 0,
  "udp6_connections_handled": 0,
  "udp6_announces_handled": 0,
  "udp6_scrapes_handled": 0
}
```

All counters will be zero on a fresh deployment with no activity.

## 3. List Torrents

```bash
TOKEN="thmbSikMOIzdJXLT0EMRrx9uyiio4wMeVA75x99cRyM="
curl -s "https://api.torrust-tracker-demo.com/api/v1/torrents?token=$TOKEN&limit=10&offset=0" | python3 -m json.tool
```

Expected: empty list `[]` on a fresh deployment.

## 4. Add and Remove a Test Torrent (whitelist mode only)

> **Note**: This deployment runs in **public mode** (`private = false`), so
> whitelisting is not enforced. Adding a torrent via the API is still useful
> to confirm write access works.

```bash
TOKEN="thmbSikMOIzdJXLT0EMRrx9uyiio4wMeVA75x99cRyM="
INFO_HASH="0000000000000000000000000000000000000001"

# Add
curl -s -X POST "https://api.torrust-tracker-demo.com/api/v1/torrent/$INFO_HASH?token=$TOKEN"

# Verify it appears
curl -s "https://api.torrust-tracker-demo.com/api/v1/torrents?token=$TOKEN" | python3 -m json.tool

# Remove
curl -s -X DELETE "https://api.torrust-tracker-demo.com/api/v1/torrent/$INFO_HASH?token=$TOKEN"
```

## 5. Invalid Token Returns 401

Confirm authentication is enforced.

```bash
curl -s -o /dev/null -w "%{http_code}" "https://api.torrust-tracker-demo.com/api/v1/stats?token=invalid"
```

Expected: `401`

## Results

| Check                 | Result | Notes |
| --------------------- | ------ | ----- |
| TLS certificate valid | ⏳     |       |
| GET /api/v1/stats     | ⏳     |       |
| GET /api/v1/torrents  | ⏳     |       |
| POST/DELETE torrent   | ⏳     |       |
| Invalid token → 401   | ⏳     |       |
