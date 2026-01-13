# Evaluate Caddy as TLS Proxy for HTTPS Termination

**Issue**: #270
**Parent Epic**: #1 - Roadmap (Item 6: Add HTTPS support)
**Predecessor**: #234 - Pingoo evaluation (CLOSED - Not Adopted)
**Related**:

- [Caddy Official Website](https://caddyserver.com/)
- [Caddy GitHub Repository](https://github.com/caddyserver/caddy)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Pingoo Evaluation Results](../research/pingoo-tls-proxy-evaluation/)
- [Pingoo WebSocket Issue](https://github.com/pingooio/pingoo/issues/23)

## Overview

This issue is a **research and experimentation task** to evaluate [Caddy](https://caddyserver.com/) as a simpler alternative to nginx+certbot for adding HTTPS termination to deployed Torrust Tracker environments.

This evaluation follows the **Pingoo evaluation** (#234), which was not adopted due to lack of WebSocket support required for Grafana Live functionality.

## Why Caddy?

From the Pingoo evaluation conclusion and [Caddy's feature documentation](https://caddyserver.com/features):

| Aspect                  | Pingoo            | Caddy                                           | nginx+certbot              |
| ----------------------- | ----------------- | ----------------------------------------------- | -------------------------- |
| **WebSocket Support**   | ❌ Not supported  | ✅ Full support (streaming, full duplex)        | ✅ Full support            |
| **ACME/Let's Encrypt**  | ✅ Automatic      | ✅ Automatic (HTTP-01, TLS-ALPN-01, DNS-01)     | ⚠️ Manual setup            |
| **TLS Versions**        | TLS 1.3 only      | TLS 1.2 + TLS 1.3 (configurable)                | TLS 1.2+                   |
| **Certificate Renewal** | ✅ Automatic      | ✅ Automatic (with intelligent error handling)  | ⚠️ Cronjob required        |
| **Configuration**       | Simple YAML       | Simple Caddyfile (also: JSON, YAML, TOML, etc.) | Complex nginx.conf         |
| **Hot Reload**          | ❓ Unknown        | ✅ Zero-downtime graceful reloads               | ⚠️ Requires reload command |
| **Maturity**            | New (2024)        | Mature (since ~2015)                            | Mature                     |
| **Post-Quantum Crypto** | ✅ X25519MLKEM768 | ❌ Not yet                                      | ❌ Not yet                 |
| **Prometheus Metrics**  | ❓ Unknown        | ✅ Built-in `/metrics` endpoint                 | ⚠️ Requires module         |
| **Binary Size**         | ~4MB              | ~40MB (static, no dependencies)                 | ~1MB (nginx only)          |
| **Language**            | Rust              | Go (memory-safe)                                | C                          |
| **Community**           | Small             | Large, well-established                         | Very large                 |

## Problem Statement

### Why Not Pingoo?

The Pingoo evaluation (#234) showed that:

1. ✅ Experiments 1-3 (Hello World, Tracker API, HTTP Tracker) all **succeeded**
2. ❌ Experiment 4 (Grafana WebSocket) **failed** - Pingoo strips `Upgrade` header

Root cause from Pingoo source (`http_proxy_service.rs`):

```rust
let dominated_headers = &[
    "host",
    "upgrade",  // ← WebSocket upgrade stripped!
    "connection",
    ...
];
```

### Why Not nginx+certbot?

The current approach has operational complexity:

1. **Manual initial setup**: Must run certbot manually to generate first certificate
2. **Manual renewal**: Requires bash script/cronjob for certificate renewal
3. **Verbose configuration**: ~200+ lines of nginx.conf for SSL, headers, locations
4. **Special handling for WebSockets**: Grafana Live requires specific HTTP upgrade configuration
5. **Multiple domain certificates**: Each subdomain needs separate certificate management

### Proposed Solution: Caddy

Caddy promises:

1. **Automatic HTTPS** by default - no configuration needed for TLS
2. **WebSocket support** out of the box
3. **Simple configuration** via Caddyfile
4. **Certificate renewal** fully automatic with ACME
5. **Mature and stable** - used in production since 2015

## Domain Strategy

Using the same **subdomain-based routing** as Pingoo evaluation:

| Service          | Domain                        | Internal Port | Protocol          |
| ---------------- | ----------------------------- | ------------- | ----------------- |
| Tracker REST API | `api.torrust-tracker.com`     | 1212          | HTTPS             |
| HTTP Tracker 1   | `http1.torrust-tracker.com`   | 7070          | HTTPS             |
| UDP Tracker 1    | `udp1.torrust-tracker.com`    | 6969          | UDP (no TLS)      |
| Grafana UI       | `grafana.torrust-tracker.com` | 3000          | HTTPS + WebSocket |

## Goals

### Primary Goals

- [ ] **Verify WebSocket Support**: Confirm Grafana Live works through Caddy (critical requirement that Pingoo failed)
- [ ] **Validate Automatic HTTPS**: Test certificate generation and renewal
- [ ] **Compare Configuration**: Document simplicity vs nginx+certbot
- [ ] **Evaluate for Production**: Determine if Caddy can replace nginx+certbot

### Secondary Goals

- Compare performance characteristics with Pingoo and nginx
- Test certificate storage and persistence across restarts
- Evaluate Caddy's health check and monitoring capabilities
- Document any limitations or edge cases

### Non-Goals

- Full implementation of HTTPS in the deployer (separate issue)
- Deciding the final proxy architecture (one proxy vs per-service)
- Production deployment configuration

## Implementation Plan

This evaluation simplifies the approach compared to Pingoo evaluation - we'll run just **one comprehensive experiment** with the full production stack to validate all requirements at once.

### Phase 1: Environment Preparation

**Duration**: 30 minutes

Reusing infrastructure from Pingoo evaluation:

- [ ] Verify Hetzner server is still available (or provision new one)
- [ ] Verify DNS records still point to server IP
- [ ] Clean up any remaining Pingoo experiments
- [ ] Ensure ports 80 and 443 are open

**Server Details** (from Pingoo evaluation):

- **Domain**: `torrust-tracker.com` (with subdomains: api, http1, grafana)
- **Server**: Hetzner ccx23, Ubuntu 24.04
- **IP**: 46.224.206.37

**Server Directory Structure:**

```text
/opt/torrust/                    # Production deployment (deployed via deployer)
├── docker-compose.yml           # Current production stack (no TLS proxy)
├── .env                         # Environment variables
└── storage/
    ├── tracker/
    │   ├── etc/                 # tracker.toml configuration
    │   ├── lib/                 # Database (tracker.db for SQLite)
    │   └── log/                 # Tracker logs
    ├── prometheus/
    │   └── etc/                 # prometheus.yml configuration
    └── grafana/
        └── provisioning/        # Grafana datasource configuration

/root/experiments/               # Pingoo evaluation experiments (issue #234)
├── experiment-1/                # Hello World test
├── experiment-2/                # Tracker API test
├── experiment-3/                # HTTP Tracker test
└── experiment-4/                # Full stack (Pingoo - WebSocket failed)
```

**Current Production Stack** (no HTTPS - direct port exposure):

- Tracker: `http://46.224.206.37:1212` (API), `http://46.224.206.37:7070` (HTTP), `udp://46.224.206.37:6969` (UDP)
- Prometheus: `http://localhost:9090` (localhost only)
- Grafana: `http://46.224.206.37:3100`

**Target with Caddy** (HTTPS termination):

- Tracker API: `https://api.torrust-tracker.com` → `http://tracker:1212`
- HTTP Tracker: `https://http1.torrust-tracker.com` → `http://tracker:7070`
- Grafana: `https://grafana.torrust-tracker.com` → `http://grafana:3000`
- UDP Tracker: `udp://udp1.torrust-tracker.com:6969` (no change - UDP has no TLS)

### Phase 2: Experiment - Full Stack with Caddy

**Duration**: 2-3 hours

**Objective**: Deploy complete production stack with Caddy as TLS proxy and verify ALL requirements:

- ✅ Automatic certificate generation
- ✅ Tracker API via HTTPS
- ✅ HTTP Tracker via HTTPS
- ✅ **Grafana WebSocket** (CRITICAL - Pingoo failed this)

**Configuration Files:**

#### Caddyfile

```caddyfile
# Caddyfile for Torrust Tracker full stack
# Based on Caddy official docs and Docker Hub examples

# Global options
{
    # Email for Let's Encrypt notifications (optional but recommended)
    email admin@torrust.com

    # Enable admin API for metrics and management
    admin 0.0.0.0:2019
}

# Tracker REST API
api.torrust-tracker.com {
    reverse_proxy tracker:1212
}

# HTTP Tracker (BitTorrent announce/scrape)
http1.torrust-tracker.com {
    reverse_proxy tracker:7070
}

# Grafana UI with WebSocket support
grafana.torrust-tracker.com {
    # WebSocket connections work automatically - no special config needed!
    # Caddy automatically handles Connection: Upgrade headers
    reverse_proxy grafana:3000
}
```

#### docker-compose.yml

```yaml
# Based on production config: /opt/torrust/docker-compose.yml + Caddy TLS proxy
services:
  caddy:
    image: caddy:2.10
    container_name: caddy
    tty: true
    restart: unless-stopped
    ports:
      - "80:80" # HTTP (ACME HTTP-01 challenge)
      - "443:443" # HTTPS
      - "443:443/udp" # HTTP/3 (QUIC)
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data # TLS certificates (MUST persist!)
      - caddy_config:/config
    networks:
      - metrics_network # Access tracker API/HTTP
      - visualization_network # Access Grafana
    logging:
      options:
        max-size: "10m"
        max-file: "10"

  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    tty: true
    restart: unless-stopped
    environment:
      - USER_ID=1000
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=${TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER}
      - TORRUST_TRACKER_CONFIG_TOML_PATH=${TORRUST_TRACKER_CONFIG_TOML_PATH}
      - TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN}
    networks:
      - metrics_network
    ports:
      - 6969:6969/udp
      - 7070:7070
      - 1212:1212
    volumes:
      - ./storage/tracker/lib:/var/lib/torrust/tracker:Z
      - ./storage/tracker/log:/var/log/torrust/tracker:Z
      - ./storage/tracker/etc:/etc/torrust/tracker:Z
    logging:
      options:
        max-size: "10m"
        max-file: "10"

  prometheus:
    image: prom/prometheus:v3.5.0
    container_name: prometheus
    tty: true
    restart: unless-stopped
    networks:
      - metrics_network
      - visualization_network
    ports:
      - "127.0.0.1:9090:9090"
    volumes:
      - ./storage/prometheus/etc:/etc/prometheus:Z
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:9090/-/healthy"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s
    logging:
      options:
        max-size: "10m"
        max-file: "10"
    depends_on:
      - tracker

  grafana:
    image: grafana/grafana:12.3.1
    container_name: grafana
    tty: true
    restart: unless-stopped
    networks:
      - visualization_network
    ports:
      - "3100:3000"
    environment:
      - GF_SECURITY_ADMIN_USER=${GF_SECURITY_ADMIN_USER}
      - GF_SECURITY_ADMIN_PASSWORD=${GF_SECURITY_ADMIN_PASSWORD}
      - GF_SERVER_ROOT_URL=https://grafana.torrust-tracker.com
      - GF_LIVE_ALLOWED_ORIGINS=https://grafana.torrust-tracker.com
    volumes:
      - grafana_data:/var/lib/grafana
      - ./storage/grafana/provisioning:/etc/grafana/provisioning:ro
    healthcheck:
      test:
        ["CMD", "wget", "--spider", "-q", "http://localhost:3000/api/health"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 30s
    logging:
      options:
        max-size: "10m"
        max-file: "10"
    depends_on:
      prometheus:
        condition: service_healthy

networks:
  metrics_network:
    driver: bridge
  visualization_network:
    driver: bridge

volumes:
  caddy_data: # TLS certificates - DO NOT DELETE!
  caddy_config:
  grafana_data:
    driver: local
```

#### .env File

```env
# .env - Environment variables (matches production at /opt/torrust/.env)

# Tracker Configuration
TORRUST_TRACKER_CONFIG_TOML_PATH='/etc/torrust/tracker/tracker.toml'
TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER='sqlite3'
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN='/YGzPOA3rEu17IxEnXTXL0ckMClkL8F0hZmnMmBsZXI='

# Grafana Configuration
GF_SECURITY_ADMIN_USER='admin'
# cspell:disable-next-line
GF_SECURITY_ADMIN_PASSWORD='puoWa0S7FF/IhnlIig2ilU8V'
```

#### prometheus.yml

```yaml
# prometheus.yml - Prometheus configuration (matches production)
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"
    params:
      token: ["/YGzPOA3rEu17IxEnXTXL0ckMClkL8F0hZmnMmBsZXI="]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:1212"]

  - job_name: "tracker_metrics"
    metrics_path: "/api/v1/metrics"
    params:
      token: ["/YGzPOA3rEu17IxEnXTXL0ckMClkL8F0hZmnMmBsZXI="]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:1212"]
```

**Deployment Commands:**

```bash
# Create experiment directory on server
ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "mkdir -p /root/experiments/caddy-full-stack"

# Copy all configuration files
scp -i ~/.ssh/torrust_tracker_rsa Caddyfile root@46.224.206.37:/root/experiments/caddy-full-stack/
scp -i ~/.ssh/torrust_tracker_rsa docker-compose.yml root@46.224.206.37:/root/experiments/caddy-full-stack/
scp -i ~/.ssh/torrust_tracker_rsa .env root@46.224.206.37:/root/experiments/caddy-full-stack/
scp -i ~/.ssh/torrust_tracker_rsa tracker.toml root@46.224.206.37:/root/experiments/caddy-full-stack/
scp -i ~/.ssh/torrust_tracker_rsa prometheus.yml root@46.224.206.37:/root/experiments/caddy-full-stack/

# Deploy the stack
ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "cd /root/experiments/caddy-full-stack && docker compose up -d"

# Watch Caddy logs for certificate generation
ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "docker logs -f caddy"
```

**Testing Procedure:**

1. **Certificate Generation** (~30 seconds expected):

   ```bash
   # Wait for Caddy to generate certificates for all 3 domains
   # Check Caddy logs for ACME challenge completion
   ```

2. **Test Tracker API**:

   ```bash
   curl -s -o /dev/null -w '%{http_code}' https://api.torrust-tracker.com/api/health_check
   # Expected: 200
   ```

3. **Test HTTP Tracker**:

   ```bash
   curl -s -o /dev/null -w '%{http_code}' 'https://http1.torrust-tracker.com/announce?...'
   # Expected: 200 with bencoded response
   ```

4. **Test Grafana HTTP**:

   ```bash
   curl -s -o /dev/null -w '%{http_code}' https://grafana.torrust-tracker.com/
   # Expected: 200 or 302 (redirect to login)
   ```

5. **Test Grafana WebSocket (CRITICAL)**:

   - Open `https://grafana.torrust-tracker.com` in browser
   - Login with admin credentials
   - Navigate to a dashboard with live data
   - Open DevTools → Network → WS filter
   - Verify WebSocket connection to `/api/live/ws` is established
   - Check that `Connection: Upgrade` header is present
   - Verify live metrics update in real-time
   - Monitor for 5 minutes - no disconnections

6. **Check Caddy Metrics** (bonus):

   ```bash
   curl -s https://grafana.torrust-tracker.com:2019/metrics | grep caddy
   # Should show Caddy Prometheus metrics
   ```

**Success Criteria:**

- [ ] All 3 domains get valid Let's Encrypt certificates automatically
- [ ] Tracker API endpoints work via HTTPS
- [ ] HTTP Tracker announce/scrape work via HTTPS
- [ ] Grafana UI accessible via HTTPS
- [ ] **WebSocket connection established and stable** (critical - Pingoo failed this)
- [ ] Grafana Live real-time updates work
- [ ] No certificate errors in browser
- [ ] Certificate generation takes < 60 seconds total

### Phase 3: Documentation and Decision

**Duration**: 1-2 hours

- [ ] Create research directory: `docs/research/caddy-tls-proxy-evaluation/`
- [ ] Document experiment results in `experiment-full-stack.md`
- [ ] Write conclusion with recommendation in `conclusion.md`
- [ ] Compare with Pingoo evaluation results
- [ ] If adopting: Create ADR in `docs/decisions/`
- [ ] Propose next steps for HTTPS implementation

## Acceptance Criteria

**Quality Checks**:

- [ ] All configuration deployed and tested
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Experiment Criteria**:

- [ ] All services accessible via HTTPS with valid certificates
- [ ] Tracker API and HTTP tracker work correctly
- [ ] **Grafana WebSocket connections work** (must verify `Upgrade` header preserved)
- [ ] Grafana Live real-time updates functional
- [ ] No disconnections over 5-minute monitoring period

**Documentation Criteria**:

- [ ] Research directory: `docs/research/caddy-tls-proxy-evaluation/`
- [ ] Experiment documented with full results
- [ ] Comparison with Pingoo evaluation
- [ ] Clear recommendation: adopt Caddy or evaluate alternatives
- [ ] ADR if adopting Caddy

## Open Questions

1. **Certificate Storage**: How does Caddy persist certificates? (Documentation suggests configurable storage: filesystem, Redis, Postgres, etc.)
2. **Performance**: How does Caddy compare to Pingoo's ~7 second certificate generation?
3. **TLS Configuration**: Can we enforce TLS 1.3 only if desired? (Verified: yes, TLS versions are configurable)
4. **Reload**: Does Caddy support hot config reloads? (Verified: yes, zero-downtime graceful reloads via API or CLI)
5. **Health Checks**: Does Caddy have built-in health endpoints? (Verified: yes, active and passive health checks)
6. **Monitoring**: What metrics does Caddy expose for Prometheus? (Verified: built-in `/metrics` endpoint)
7. **Load Balancing**: What policies are available? (Verified: random, round-robin, least connections, IP hash, etc.)
8. **ACME Challenges**: Which challenge types work best for our setup? (Options: HTTP-01, TLS-ALPN-01, DNS-01)

## Research Output

```text
docs/research/caddy-tls-proxy-evaluation/
├── README.md                      # Overview and summary
├── experiment-full-stack.md       # Complete experiment results
└── conclusion.md                  # Final recommendation
```

## References

- [Caddy Official Documentation](https://caddyserver.com/docs/)
- [Caddy Reverse Proxy](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy)
- [Caddy Automatic HTTPS](https://caddyserver.com/docs/automatic-https)
- [Caddy Docker Image](https://hub.docker.com/_/caddy)
- [Pingoo Evaluation (Not Adopted)](../research/pingoo-tls-proxy-evaluation/)
- [Grafana Live WebSocket Requirements](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/)

---

**Created**: 2026-01-13
**Last Updated**: 2026-01-13
**Status**: Planning
