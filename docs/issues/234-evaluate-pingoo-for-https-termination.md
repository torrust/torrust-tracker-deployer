# Evaluate Pingoo as TLS Proxy for HTTPS Termination

**Issue**: #234
**Parent Epic**: #1 - Roadmap (Item 6: Add HTTPS support)
**Related**:

- [Pingoo Website](https://pingoo.io/)
- [Pingoo GitHub](https://github.com/pingooio/pingoo)
- [Pingoo Documentation](https://pingoo.io/docs/getting-started)
- [Current nginx.conf in Torrust Demo](https://github.com/torrust/torrust-demo/blob/main/share/container/default/config/nginx.conf)
- [Split Torrust Demo into Index and Tracker](https://github.com/torrust/torrust-demo/issues/79)

## Overview

This issue is a **research and experimentation task** to evaluate [Pingoo](https://pingoo.io/) as a simpler alternative to nginx+certbot for adding HTTPS termination to deployed Torrust Tracker environments.

The goal is to determine if Pingoo can replace the complex nginx+certbot setup currently used in the Torrust Demo, specifically for:

1. Automatic TLS certificate generation (Let's Encrypt)
2. Automatic certificate renewal
3. HTTPS termination for multiple services

## Problem Statement

### Current Implementation (Torrust Demo)

> **Note**: The current [Torrust Demo](https://github.com/torrust/torrust-demo) at `torrust-demo.com` includes
> both the **Index** and the **Tracker** in a single deployment:
>
> - `https://index.torrust-demo.com/` - Index UI and API
> - `https://tracker.torrust-demo.com/api/v1/stats` - Tracker API
> - `https://grafana.torrust-demo.com/` - Grafana UI
>
> However, **this deployer is focused only on the Tracker**. There is a plan to split the demo into two
> separate independently deployable services (see [torrust-demo#79](https://github.com/torrust/torrust-demo/issues/79)).
> The nginx configuration referenced below includes Index-related settings that are not relevant to this deployer.

The current approach uses **nginx + certbot** with significant complexity:

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Current Setup (nginx + certbot)              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐     ┌───────────────────────────────────────┐  │
│  │   Certbot   │────▶│  Manual initial certificate generation│  │
│  │  Container  │     │  (must run once manually)             │  │
│  └─────────────┘     └───────────────────────────────────────┘  │
│         │                                                       │
│         │ Cronjob for renewal                                   │
│         ▼                                                       │
│  ┌─────────────┐     ┌───────────────────────────────────────┐  │
│  │    Nginx    │────▶│  Complex configuration with:          │  │
│  │   Proxy     │     │  - Port 80 for ACME challenge         │  │
│  └─────────────┘     │  - Port 443 for HTTPS services        │  │
│         │            │  - SSL certificate paths              │  │
│         │            │  - Manual header configuration        │  │
│         │            │  - WebSocket upgrade for Grafana      │  │
│         ▼            └───────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Services (Tracker-only scope for this deployer):        │   │
│  │  - Tracker API (port 1212) → https://tracker.domain/api/ │   │
│  │  - HTTP Tracker (port 7070) → https://tracker.domain/    │   │
│  │  - Grafana UI (port 3000) → https://grafana.domain/      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Problems with current approach:**

1. **Complex initial setup**: Must manually run certbot to generate first certificate
2. **Manual renewal**: Requires a bash script/cronjob for certificate renewal
3. **Verbose configuration**: ~200+ lines of nginx.conf for SSL, headers, locations
4. **Special handling for WebSockets**: Grafana Live requires specific HTTP upgrade configuration
5. **Multiple domain certificates**: Each subdomain needs separate certificate management

### Proposed Solution (Pingoo)

Pingoo promises automatic certificate management with minimal configuration:

```yaml
# Potential Pingoo configuration
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains:
      [
        "api.torrust-tracker.com",
        "http1.torrust-tracker.com",
        "grafana.torrust-tracker.com",
      ]

services:
  tracker_api:
    route: host == "api.torrust-tracker.com"
    http_proxy: ["http://tracker:1212"]

  tracker_http:
    route: host == "http1.torrust-tracker.com"
    http_proxy: ["http://tracker:7070"]

  grafana:
    route: host == "grafana.torrust-tracker.com"
    http_proxy: ["http://grafana:3000"]
```

## Domain Strategy

### Why Subdomains per Service?

The Torrust Demo originally used a single domain (`tracker.torrust-demo.com`) with path-based routing:

- `https://tracker.torrust-demo.com/api/` → Tracker API (port 1212)
- `https://tracker.torrust-demo.com/` → HTTP Tracker (port 7070)

**Problem**: This approach doesn't scale to multiple tracker instances. You can only map port 443 once per subdomain.

### Planned Domain Structure

For these experiments and future production use, we'll use **subdomain-based routing**:

| Service          | Domain                        | Internal Port | Protocol     |
| ---------------- | ----------------------------- | ------------- | ------------ |
| Tracker REST API | `api.torrust-tracker.com`     | 1212          | HTTPS        |
| HTTP Tracker 1   | `http1.torrust-tracker.com`   | 7070          | HTTPS        |
| UDP Tracker 1    | `udp1.torrust-tracker.com`    | 6969          | UDP (no TLS) |
| Grafana UI       | `grafana.torrust-tracker.com` | 3000          | HTTPS        |

**Benefits**:

- Each service has its own subdomain → independent TLS certificates
- Supports multiple tracker instances (`http1`, `http2`, etc.)
- Clear separation of concerns
- Easier firewall rules and monitoring

**Note**: UDP trackers don't use HTTPS - they use the UDP protocol directly on their designated port.

## Goals

### Primary Goals

- [ ] **Learn**: Understand how Pingoo handles automatic TLS certificate management
- [ ] **Experiment**: Test Pingoo with real services in the existing Hetzner environment
- [ ] **Validate**: Determine if Pingoo supports all required features (WebSockets, routing)
- [ ] **Document**: Create decision documentation for future HTTPS implementation

### Secondary Goals (Nice-to-Have)

- Compare configuration complexity between Pingoo and nginx+certbot
- Measure certificate renewal reliability
- Evaluate Pingoo's Docker service discovery feature

### Non-Goals

- Full implementation of HTTPS in the deployer (separate issue)
- Deciding the final proxy architecture (one proxy vs per-service proxies)
- Production deployment configuration

## Research Findings

### Pingoo Key Features

Based on [Pingoo documentation](https://pingoo.io/docs/tls):

| Feature                      | Status           | Notes                            |
| ---------------------------- | ---------------- | -------------------------------- |
| **ACME/Let's Encrypt**       | ✅ Supported     | Automatic certificate generation |
| **Auto Renewal**             | ✅ Automatic     | Managed via `acme.json`          |
| **TLS Termination**          | ✅ Supported     | TLS 1.3 only (security focused)  |
| **HTTP Proxy**               | ✅ Supported     | With load balancing              |
| **Docker Service Discovery** | ✅ Supported     | Via `pingoo.service` label       |
| **Wildcard Certificates**    | ❌ Not supported | With ACME                        |
| **WebSocket Support**        | ❓ Unknown       | **Needs testing**                |

### Pingoo ACME Challenge Requirements

From documentation:

> Pingoo currently only supports the [tls-alpn-01](https://letsencrypt.org/docs/challenge-types/#tls-alpn-01) challenge. It means that one of your TLS listeners must be publicly accessible on the port `443`.

**Important**: This differs from nginx+certbot which uses HTTP-01 challenge (port 80). Pingoo requires port 443 to be publicly accessible.

### Current nginx Configuration Analysis

The [current nginx.conf](https://github.com/torrust/torrust-demo/blob/main/share/container/default/config/nginx.conf) shows:

**Active HTTP servers (port 80):**

```nginx
# index.torrust-demo.com - Proxies to index:3001 and index-gui:3000
# tracker.torrust-demo.com - Proxies to tracker:1212 (API) and tracker:7070 (HTTP)
# grafana.torrust-demo.com - Proxies to grafana:3000
# All include /.well-known/acme-challenge for Let's Encrypt HTTP-01 challenge
```

**HTTPS configuration (commented out but available):**

- Separate SSL certificates per subdomain
- Manual SSL configuration (protocols, ciphers, dhparam)
- WebSocket upgrade handling for Grafana:

```nginx
# This is required to proxy Grafana Live WebSocket connections.
map $http_upgrade $connection_upgrade {
  default upgrade;
  '' close;
}

upstream grafana {
  server grafana:3000;
}

location /api/live/ {
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
    proxy_set_header Host $host;
    proxy_pass http://grafana;
}
```

## Implementation Plan

### Phase 1: Environment Preparation

**Duration**: 1-2 hours

- [ ] Verify existing Hetzner environment (`docker-hetzner-test`) is running
- [ ] Document current services and their internal ports
- [ ] Obtain/configure domain names pointing to the Hetzner server IP
- [ ] Ensure ports 443 and 80 are open on the server firewall

### Phase 2: Experiment 1 - Minimal HTTPS Setup (Static Web Server)

**Duration**: 1-2 hours

**Objective**: Test Pingoo certificate generation in isolation with a minimal setup

This experiment uses an **independent docker-compose configuration** with only:

- Pingoo (TLS proxy)
- A simple nginx container serving static "Hello World" content

This isolates the certificate generation testing from any tracker-specific complexity.

- [ ] Create a new directory for the experiment (e.g., `experiments/pingoo-hello-world/`)
- [ ] Create minimal `docker-compose.yml` with Pingoo + nginx static server
- [ ] Create `pingoo.yml` configuration for a single domain
- [ ] Create simple `index.html` with "Hello World" content
- [ ] Deploy to the Hetzner server
- [ ] Verify automatic certificate generation
- [ ] Test HTTPS access to the static page
- [ ] Document results and any issues

**Directory Structure:**

```text
experiments/pingoo-hello-world/
├── docker-compose.yml
├── pingoo/
│   └── pingoo.yml
└── www/
    └── index.html
```

**docker-compose.yml:**

```yaml
services:
  pingoo:
    image: pingooio/pingoo:latest
    ports:
      - "443:443"
    volumes:
      - ./pingoo:/etc/pingoo
    networks:
      - test-network
    depends_on:
      - webserver

  webserver:
    image: nginx:alpine
    volumes:
      - ./www:/usr/share/nginx/html:ro
    networks:
      - test-network
    # No exposed ports - only accessible via Pingoo

networks:
  test-network:
    driver: bridge
```

**pingoo/pingoo.yml:**

```yaml
# pingoo.yml - Experiment 1: Minimal Hello World
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains: ["test.torrust-tracker.com"]

services:
  static:
    http_proxy: ["http://webserver:80"]
```

**www/index.html:**

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Pingoo Test</title>
  </head>
  <body>
    <h1>Hello World!</h1>
    <p>If you see this page via HTTPS, Pingoo certificate generation works!</p>
    <p>Certificate info: Check browser padlock for details.</p>
  </body>
</html>
```

**Success Criteria:**

- [ ] `https://test.torrust-tracker.com` shows the Hello World page
- [ ] Browser shows valid Let's Encrypt certificate
- [ ] No manual certificate generation required

### Phase 3: Experiment 2 - Tracker API HTTPS

**Duration**: 2-4 hours

**Objective**: Add Pingoo to the existing tracker stack to serve Tracker API via HTTPS

- [ ] Create `pingoo.yml` configuration for Tracker API
- [ ] Add Pingoo service to the existing tracker docker-compose stack
- [ ] Configure domain: `api.torrust-tracker.com`
- [ ] Test HTTPS access to Tracker API endpoints
- [ ] Verify automatic certificate generation
- [ ] Document results and any issues

**Expected Configuration:**

```yaml
# pingoo.yml - Experiment 2
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains: ["api.torrust-tracker.com"]

services:
  tracker_api:
    http_proxy: ["http://tracker:1212"]
```

**Docker Compose Addition:**

```yaml
# docker-compose.yml addition
pingoo:
  image: pingooio/pingoo:latest
  ports:
    - "443:443"
  volumes:
    - ./pingoo:/etc/pingoo
  networks:
    - tracker-network
```

### Phase 4: Experiment 3 - HTTP Tracker HTTPS

**Duration**: 2-4 hours

**Objective**: Extend configuration to serve HTTP Tracker via HTTPS

- [ ] Update `pingoo.yml` to add HTTP tracker routing
- [ ] Configure domain: `http1.torrust-tracker.com`
- [ ] Test HTTPS access to announce/scrape endpoints
- [ ] Verify BitTorrent clients can use HTTPS tracker URL
- [ ] Document results and any issues

**Expected Configuration Update:**

```yaml
# pingoo.yml - Experiment 3
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains: ["api.torrust-tracker.com", "http1.torrust-tracker.com"]

services:
  tracker_api:
    route: http_request.host == "api.torrust-tracker.com"
    http_proxy: ["http://tracker:1212"]

  tracker_http:
    route: http_request.host == "http1.torrust-tracker.com"
    http_proxy: ["http://tracker:7070"]
```

### Phase 5: Experiment 4 - Grafana UI HTTPS (WebSocket Test)

**Duration**: 3-5 hours

**Objective**: Test Pingoo's WebSocket support for Grafana Live

- [ ] Update `pingoo.yml` to add Grafana routing
- [ ] Configure domain: `grafana.torrust-tracker.com`
- [ ] Test basic Grafana UI access via HTTPS
- [ ] **Critical**: Test Grafana Live WebSocket functionality
- [ ] Monitor for connection issues or dropped connections
- [ ] Document WebSocket support findings

**Expected Configuration Update:**

```yaml
# pingoo.yml - Experiment 4
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains:
      [
        "api.torrust-tracker.com",
        "http1.torrust-tracker.com",
        "grafana.torrust-tracker.com",
      ]

services:
  tracker_api:
    route: http_request.host == "api.torrust-tracker.com"
    http_proxy: ["http://tracker:1212"]

  tracker_http:
    route: http_request.host == "http1.torrust-tracker.com"
    http_proxy: ["http://tracker:7070"]

  grafana:
    route: http_request.host == "grafana.torrust-tracker.com"
    http_proxy: ["http://grafana:3000"]
```

**WebSocket Test Procedure:**

1. Open Grafana dashboard with real-time metrics
2. Navigate to a dashboard using Grafana Live (real-time updates)
3. Open browser DevTools → Network → WS filter
4. Verify WebSocket connection is established
5. Monitor for ~5 minutes for stability
6. Check for any reconnection attempts or errors

### Phase 6: Documentation and Decision

**Duration**: 2-3 hours

- [ ] Create research directory: `docs/research/pingoo-tls-proxy-evaluation/`
- [ ] Document each experiment's results in separate files
- [ ] Write conclusion with recommendation
- [ ] If adopting Pingoo: Create ADR in `docs/decisions/`
- [ ] Document any limitations or workarounds found
- [ ] Propose next steps for HTTPS implementation issue

## Acceptance Criteria

**Quality Checks**:

- [ ] All experiments completed and documented
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh` (if code changes are made)

**Experiment 1 Criteria (Minimal Hello World)**:

- [ ] Static page accessible via HTTPS
- [ ] Certificate automatically generated by Pingoo (no manual steps)
- [ ] Browser shows valid Let's Encrypt certificate
- [ ] Independent docker-compose setup works without tracker stack

**Experiment 2 Criteria (Tracker API)**:

- [ ] Tracker API accessible via HTTPS
- [ ] Certificate automatically generated by Pingoo
- [ ] API endpoints respond correctly (`/api/v1/stats`, etc.)

**Experiment 3 Criteria (HTTP Tracker)**:

- [ ] HTTP Tracker accessible via HTTPS
- [ ] Announce endpoint works (`/announce`)
- [ ] Scrape endpoint works (`/scrape`)

**Experiment 4 Criteria (Grafana)**:

- [ ] Grafana UI accessible via HTTPS
- [ ] Login and dashboard navigation works
- [ ] **WebSocket test**: Grafana Live functionality verified (or documented as not working)

**Documentation Criteria**:

- [ ] Research directory created: `docs/research/pingoo-tls-proxy-evaluation/`
- [ ] Each experiment documented with results
- [ ] Clear recommendation: proceed with Pingoo, stick with nginx+certbot, or evaluate alternatives
- [ ] Known limitations documented
- [ ] ADR created if decision is made to adopt Pingoo

## Open Questions

1. **WebSocket Support**: Does Pingoo automatically handle HTTP Upgrade headers for WebSocket connections?
2. **Certificate Storage**: How does Pingoo persist certificates across container restarts?
3. **Multiple Domains**: Can a single Pingoo instance handle multiple domains efficiently?
4. **TLS 1.2 Support**: Pingoo only supports TLS 1.3 - is this a concern for older clients?
5. **Port 443 Requirement**: The tls-alpn-01 challenge requires port 443 - does this conflict with existing services?

## Research Output

Since this is an **evaluation/research task** (not yet a decision), findings will be stored in:

```text
docs/research/pingoo-tls-proxy-evaluation/
├── README.md                      # Overview and summary of findings
├── experiment-1-hello-world.md    # Results from minimal static server test
├── experiment-2-tracker-api.md    # Results from Tracker API HTTPS test
├── experiment-3-http-tracker.md   # Results from HTTP Tracker HTTPS test
├── experiment-4-grafana.md        # Results from Grafana WebSocket test
└── conclusion.md                  # Final recommendation and next steps
```

**Progression of documentation:**

1. **During research** → `docs/research/pingoo-tls-proxy-evaluation/`
2. **If we decide to adopt Pingoo** → Create ADR in `docs/decisions/`
3. **If we decide NOT to adopt** → Document reasoning in `conclusion.md` and close issue

## Test Environment

**Existing Hetzner Environment**: `docker-hetzner-test`

Configuration file: `envs/docker-hetzner-test.json`

```text
Environment: docker-hetzner-test
Provider: Hetzner (ccx23 server, Ubuntu 24.04, nbg1 location)
Services:
  - Tracker (UDP: 6969, HTTP: 7070, API: 1212)
  - Prometheus (scrape interval: 15s)
  - Grafana (admin/admin credentials configured)
```

## Future Considerations

After completing this research, a **separate implementation issue** will be created to address:

1. **Architecture Decision**: Single proxy vs. per-service proxies
2. **Service Enable/Disable**: How to handle optional services (e.g., Grafana disabled)
3. **Configuration Integration**: How to template proxy configuration based on environment settings
4. **Fallback Strategy**: What to do if Pingoo doesn't meet requirements (nginx+certbot, Caddy, Traefik)

## References

- [Pingoo Official Documentation](https://pingoo.io/docs/getting-started)
- [Pingoo TLS/HTTPS Configuration](https://pingoo.io/docs/tls)
- [Pingoo Services Configuration](https://pingoo.io/docs/services)
- [Let's Encrypt Challenge Types](https://letsencrypt.org/docs/challenge-types/)
- [Current Torrust Demo nginx.conf](https://github.com/torrust/torrust-demo/blob/main/share/container/default/config/nginx.conf)
- [Grafana WebSocket Requirements](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/)

---

**Created**: 2026-01-10
**Last Updated**: 2026-01-10
**Status**: Planning
