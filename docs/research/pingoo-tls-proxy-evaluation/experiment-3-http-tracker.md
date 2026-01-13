# Experiment 3: HTTP Tracker with HTTPS

**Status**: ✅ Complete
**Started**: 2026-01-12
**Completed**: 2026-01-12
**Domain**: `http1.torrust-tracker.com`

## Objective

Test Pingoo with the Torrust HTTP Tracker to verify:

- HTTPS termination for BitTorrent announce/scrape endpoints
- Correct proxying of tracker protocol requests
- Certificate generation for `http1.torrust-tracker.com`

## Pre-requisites

- [x] Experiment 2 completed successfully
- [x] DNS propagated for `http1.torrust-tracker.com` → 46.224.206.37
- [x] Port 443 available (Experiment 2 stopped)

## Setup

### Files Created

```text
/root/experiments/experiment-3/
├── docker-compose.yml
├── pingoo/
│   └── pingoo.yml
└── storage/
    └── tracker/
        ├── etc/
        │   └── tracker.toml
        ├── lib/
        └── log/
```

### docker-compose.yml

```yaml
# Experiment 3: HTTP Tracker with Pingoo TLS termination
# Tests announce/scrape endpoints via HTTPS

services:
  pingoo:
    image: pingooio/pingoo:latest
    container_name: pingoo
    restart: unless-stopped
    ports:
      - "443:443"
    volumes:
      - ./pingoo:/etc/pingoo
    networks:
      - tracker-network
    depends_on:
      - tracker

  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    tty: true
    restart: unless-stopped
    environment:
      - USER_ID=1000
      - TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=sqlite3
    networks:
      - tracker-network
    volumes:
      - ./storage/tracker/lib:/var/lib/torrust/tracker:Z
      - ./storage/tracker/log:/var/log/torrust/tracker:Z
      - ./storage/tracker/etc:/etc/torrust/tracker:Z
    logging:
      options:
        max-size: "10m"
        max-file: "10"

networks:
  tracker-network:
    driver: bridge
```

### pingoo/pingoo.yml

```yaml
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains: ["http1.torrust-tracker.com"]

services:
  http-tracker:
    http_proxy: ["http://tracker:7070"]
```

### storage/tracker/etc/tracker.toml

Same as Experiment 2 - production-like tracker configuration.

## Deployment Steps

1. Stop Experiment 2: `cd /root/experiments/experiment-2 && docker compose down`
2. Create the experiment directory structure
3. Copy the configuration files
4. Run `docker compose up -d`
5. Check Pingoo logs for certificate generation
6. Test tracker endpoints via HTTPS

## Results

### Deployment Log

```text
$ ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 \
    "cd /root/experiments/experiment-3 && docker compose up -d"

 Network experiment-3_tracker-network  Creating
 Network experiment-3_tracker-network  Created
 Container tracker  Creating
 Container tracker  Created
 Container pingoo  Creating
 Container pingoo  Created
 Container tracker  Starting
 Container tracker  Started
 Container pingoo  Starting
 Container pingoo  Started
```

### Certificate Generation

```text
$ docker logs pingoo

{"timestamp":"2026-01-12T17:19:49.432914Z","level":"INFO","message":"configuration successfully loaded from /etc/pingoo/pingoo.yml","services":1,"listeners":1}
{"timestamp":"2026-01-12T17:19:49.433578Z","level":"INFO","message":"docker socket (/var/run/docker.sock) not found. Docker service discovery disabled."}
{"timestamp":"2026-01-12T17:19:50.428199Z","level":"INFO","message":"Starting listener https on https://0.0.0.0:443","listener":"https"}
{"timestamp":"2026-01-12T17:19:57.329901Z","level":"INFO","message":"tls: ACME TLS certificate successfully saved","domain":"http1.torrust-tracker.com"}
```

Certificate issued in ~7 seconds.

### TLS Details

```text
$ curl -v https://http1.torrust-tracker.com/health_check 2>&1 | grep -E "(SSL|subject|issuer|expire)"

* SSL connection using TLSv1.3 / TLS_AES_256_GCM_SHA384 / X25519MLKEM768 / id-ecPublicKey
*  subject: CN=http1.torrust-tracker.com
*  expire date: Apr 12 16:21:24 2026 GMT
*  subjectAltName: host "http1.torrust-tracker.com" matched cert's "http1.torrust-tracker.com"
*  issuer: C=US; O=Let's Encrypt; CN=E8
*  SSL certificate verify ok.
```

### Tracker Protocol Tests

#### Health Check

```text
$ curl -s https://http1.torrust-tracker.com/health_check
{"status":"Ok"}
```

#### Announce Endpoint

```text
$ curl -s 'https://http1.torrust-tracker.com/announce?info_hash=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&peer_id=-qB0000-000000000000&port=6881&uploaded=0&downloaded=0&left=0&compact=1'

d8:completei1e10:incompletei0e8:intervali300e12:min intervali300e5:peers0:6:peers60:e
```

Valid bencoded response showing:

- `complete`: 1 (seeder count)
- `incomplete`: 0 (leecher count)
- `interval`: 300 seconds
- `min interval`: 300 seconds
- `peers`: empty (no other peers for this test torrent)

#### Scrape Endpoint

```text
$ curl -s 'https://http1.torrust-tracker.com/scrape?info_hash=01234567890123456789'

d5:filesd20:01234567890123456789d8:completei0e10:downloadedi0e10:incompletei0eeee
```

Valid bencoded scrape response showing torrent statistics.

## Success Criteria

- [x] `https://http1.torrust-tracker.com/health_check` returns OK
- [x] Valid Let's Encrypt certificate for `http1.torrust-tracker.com`
- [x] Announce endpoint returns valid bencoded response
- [x] Scrape endpoint returns valid bencoded response
- [x] BitTorrent protocol works correctly via HTTPS

## Issues Encountered

None.

## Observations

1. **BitTorrent Protocol Works**: Both announce and scrape endpoints work correctly
   through Pingoo. The binary bencoded responses are proxied without corruption.

2. **Same Performance**: Certificate generation (~7 seconds) and TLS configuration
   (TLS 1.3, X25519MLKEM768) are consistent with previous experiments.

3. **Back to E8 CA**: This certificate was issued by Let's Encrypt E8 (same as
   Experiment 1), confirming the CA rotation is normal.

4. **Transparent Binary Proxying**: Pingoo correctly handles the non-JSON tracker
   responses (bencoded binary format) without any issues.

## Conclusion

**Experiment 3 is SUCCESSFUL.** Pingoo successfully:

- Generated a valid Let's Encrypt certificate for `http1.torrust-tracker.com`
- Proxied BitTorrent tracker protocol requests correctly
- Handled binary bencoded responses without corruption
- Required minimal configuration (10 lines)

This validates that Pingoo can serve as the TLS proxy for the HTTP Tracker in production.
The BitTorrent protocol works identically whether accessed directly or through Pingoo.

**Next**: Proceed to Experiment 4 to test Pingoo with Grafana (WebSocket support - CRITICAL).
