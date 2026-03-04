# HTTP Tracker Verification

**Status**: ✅ Verified (2026-03-04)

## Endpoints

| Domain         | URL                                               |
| -------------- | ------------------------------------------------- |
| HTTP Tracker 1 | `https://http1.torrust-tracker-demo.com/announce` |
| HTTP Tracker 2 | `https://http2.torrust-tracker-demo.com/announce` |

## 1. Basic Connectivity (scrape request)

The simplest check — a scrape request with no info hashes returns a valid
bencoded response.

```bash
curl -v "https://http1.torrust-tracker-demo.com/announce?info_hash=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&peer_id=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&port=6881&uploaded=0&downloaded=0&left=0&event=started&compact=1"
```

Expected: HTTP 200 with a bencoded response body starting with `d` (a bencoded
dictionary). The response will contain `interval` and `peers` fields.

## 2. TLS Certificate Check

Confirm the TLS certificate is valid and issued for the correct domain.

```bash
curl -v --head https://http1.torrust-tracker-demo.com/announce 2>&1 | grep -E "subject|issuer|expire|SSL"
```

Expected: Certificate issued by Let's Encrypt for `http1.torrust-tracker-demo.com`,
not expired.

## 3. Second Endpoint

Repeat the connectivity check for the second HTTP tracker:

```bash
curl -sv "https://http2.torrust-tracker-demo.com/announce?info_hash=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&peer_id=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&port=6881&uploaded=0&downloaded=0&left=0&event=started&compact=1" 2>&1 | head -20
```

## 4. Health Check

The tracker exposes a health check API bound to the container's loopback
interface (`127.0.0.1:1313`). It is not accessible from the host directly.
See [health-check.md](health-check.md) for the verification procedure and
actual output.

## Results

| Check                       | Result | Notes                                  |
| --------------------------- | ------ | -------------------------------------- |
| HTTP Tracker 1 connectivity | ✅     | HTTP 200 with bencoded response        |
| HTTP Tracker 1 TLS cert     | ✅     | Let's Encrypt, valid until Jun 2, 2026 |
| HTTP Tracker 2 connectivity | ✅     | HTTP 200 with bencoded response        |
| Health check                | ✅     | See [health-check.md](health-check.md) |
