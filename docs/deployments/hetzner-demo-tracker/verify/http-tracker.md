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

## 5. Using the Torrust Tracker Client

The [Torrust Tracker](https://github.com/torrust/torrust-tracker) project ships
a reference `http_tracker_client` binary located in
[`console/tracker-client`](https://github.com/torrust/torrust-tracker/tree/develop/console/tracker-client).
It sends a full BEP 3 HTTP announce request and displays the JSON-decoded
response.

### HTTP Tracker 1

```bash
# From the torrust-tracker console/tracker-client directory
cargo run --bin http_tracker_client announce \
  https://http1.torrust-tracker-demo.com \
  9c38422213e30bff212b30c360d26f9a02136422
```

Output:

```json
{
  "complete": 3,
  "incomplete": 0,
  "interval": 300,
  "min interval": 300,
  "peers": [
    { "ip": "::ffff:2.137.92.24", "port": 34094 },
    { "ip": "::ffff:2.137.92.24", "port": 48887 }
  ]
}
```

### HTTP Tracker 2

```bash
cargo run --bin http_tracker_client announce \
  https://http2.torrust-tracker-demo.com \
  9c38422213e30bff212b30c360d26f9a02136422
```

Output:

```json
{
  "complete": 3,
  "incomplete": 0,
  "interval": 300,
  "min interval": 300,
  "peers": [
    { "ip": "::ffff:2.137.92.24", "port": 34094 },
    { "ip": "::ffff:2.137.92.24", "port": 48887 }
  ]
}
```

Both trackers returned the same response — they share the same backend database.
The IP `::ffff:2.137.92.24` is the IPv4-mapped IPv6 form of the local machine's
public IP (`2.137.92.24`), confirming the peer was registered correctly from the
client's perspective. The two ports (`34094` and `48887`) correspond to peers
registered from previous HTTP and UDP announces during this verification session.

## Results

| Check                               | Result | Notes                                          |
| ----------------------------------- | ------ | ---------------------------------------------- |
| HTTP Tracker 1 connectivity         | ✅     | HTTP 200 with bencoded response                |
| HTTP Tracker 1 TLS cert             | ✅     | Let's Encrypt, valid until Jun 2, 2026         |
| HTTP Tracker 2 connectivity         | ✅     | HTTP 200 with bencoded response                |
| Health check                        | ✅     | See [health-check.md](health-check.md)         |
| Torrust client announce (tracker 1) | ✅     | `interval=300`, `complete=3`, 2 peers returned |
| Torrust client announce (tracker 2) | ✅     | `interval=300`, `complete=3`, 2 peers returned |
