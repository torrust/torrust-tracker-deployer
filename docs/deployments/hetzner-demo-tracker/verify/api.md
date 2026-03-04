# Tracker API Verification

**Status**: ✅ Verified (2026-03-04)

## Endpoint

`https://api.torrust-tracker-demo.com/api/v1`

## Authentication

All API endpoints require the admin token as a query parameter or header.

Admin token: see `envs/torrust-tracker-demo.json` → `tracker.http_api.admin_token`

```bash
# Set token for reuse in the commands below
TOKEN="<ADMIN_TOKEN>"
```

## 1. TLS Certificate Check

```bash
curl -v --head "https://api.torrust-tracker-demo.com/api/v1/stats?token=$TOKEN" 2>&1 | grep -E "subject|issuer|SSL|HTTP"
```

Expected: valid Let's Encrypt certificate, HTTP 200.

## 2. Tracker Statistics

Fetch global tracker statistics (total torrents, peers, etc.).

```bash
TOKEN="<ADMIN_TOKEN>"
curl -s "https://api.torrust-tracker-demo.com/api/v1/stats?token=$TOKEN" | python3 -m json.tool
```

Expected response structure (counters may be non-zero if there has been any
network activity since deployment):

```json
{
  "torrents": 0,
  "seeders": 0,
  "completed": 0,
  "leechers": 0,
  "tcp4_connections_handled": 0,
  "tcp4_announces_handled": 0,
  "tcp4_scrapes_handled": 0,
  "tcp6_connections_handled": 0,
  "tcp6_announces_handled": 0,
  "tcp6_scrapes_handled": 0,
  "udp_requests_aborted": 0,
  "udp_requests_banned": 0,
  "udp_banned_ips_total": 0,
  "udp_avg_connect_processing_time_ns": 0,
  "udp_avg_announce_processing_time_ns": 0,
  "udp_avg_scrape_processing_time_ns": 0,
  "udp4_requests": 0,
  "udp4_connections_handled": 0,
  "udp4_announces_handled": 0,
  "udp4_scrapes_handled": 0,
  "udp4_responses": 0,
  "udp4_errors_handled": 0,
  "udp6_requests": 0,
  "udp6_connections_handled": 0,
  "udp6_announces_handled": 0,
  "udp6_scrapes_handled": 0,
  "udp6_responses": 0,
  "udp6_errors_handled": 0
}
```

## 3. List Torrents

```bash
TOKEN="<ADMIN_TOKEN>"
curl -s "https://api.torrust-tracker-demo.com/api/v1/torrents?token=$TOKEN&limit=10&offset=0" | python3 -m json.tool
```

Expected: empty list `[]` on a fresh deployment.

## 4. Add and Remove a Test Torrent (whitelist mode only)

> **Note**: This deployment runs in **public mode** (`private = false`), so
> whitelisting is not enforced. Adding a torrent via the API is still useful
> to confirm write access works.

```bash
TOKEN="<ADMIN_TOKEN>"
INFO_HASH="0000000000000000000000000000000000000001"

# Add
curl -s -X POST "https://api.torrust-tracker-demo.com/api/v1/torrent/$INFO_HASH?token=$TOKEN"

# Verify it appears
curl -s "https://api.torrust-tracker-demo.com/api/v1/torrents?token=$TOKEN" | python3 -m json.tool

# Remove
curl -s -X DELETE "https://api.torrust-tracker-demo.com/api/v1/torrent/$INFO_HASH?token=$TOKEN"
```

## 5. Invalid Token Rejected

Confirm authentication is enforced.

```bash
curl -s -o /dev/null -w "%{http_code}" "https://api.torrust-tracker-demo.com/api/v1/stats?token=invalid"
```

Expected: `401` (a `500` is currently returned — see [Bug 4](#bug-4-invalid-token-returns-500-instead-of-401))

## Results

| Check              | Result | Notes                                                     |
| ------------------ | ------ | --------------------------------------------------------- |
| TLS certificate    | ✅     | Let's Encrypt, valid until Jun 2, 2026                    |
| Tracker statistics | ✅     | Non-zero UDP6 counters from network activity after deploy |
| List torrents      | ✅     | Empty list on fresh deployment                            |
| Add/remove torrent | ✅     | Both return empty body on success                         |
| Invalid token      | ⚠️     | Returns HTTP `500` instead of `401` — see Bug 4 below     |

## Bug 4: Invalid Token Returns 500 Instead of 401

When sending an invalid token, the API returns:

- **HTTP status**: `500`
- **Body**: `Unhandled rejection: Err { reason: "token not valid" }`

A `401 Unauthorized` would be the correct HTTP status for an authentication
failure. This is a bug in the tracker API error handling.
| --------------------- | ------ | ----- |
| TLS certificate valid | ⏳ | |
| GET /api/v1/stats | ⏳ | |
| GET /api/v1/torrents | ⏳ | |
| POST/DELETE torrent | ⏳ | |
| Invalid token → 401 | ⏳ | |
