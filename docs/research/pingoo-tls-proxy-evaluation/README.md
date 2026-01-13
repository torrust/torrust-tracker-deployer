# Pingoo TLS Proxy Evaluation

**Issue**: [#234](https://github.com/torrust/torrust-tracker-deployer/issues/234)
**Specification**: [docs/issues/234-evaluate-pingoo-for-https-termination.md](../../issues/234-evaluate-pingoo-for-https-termination.md)
**Status**: ✅ CLOSED - Not Adopting
**Started**: 2026-01-12
**Completed**: 2026-01-13

## Overview

This research evaluated [Pingoo](https://pingoo.io/) as a potential replacement for nginx+certbot
for automatic HTTPS/TLS termination in Torrust Tracker deployments.

**Outcome**: Pingoo works excellently for the Tracker (API + HTTP Tracker) but does not
support WebSocket connections required for Grafana Live. We have decided **not to adopt
Pingoo** at this time and will instead evaluate **Caddy** as a simpler alternative that
supports both HTTP proxying and WebSocket.

## Test Environment

- **Server**: Hetzner ccx23, Ubuntu 24.04, nbg1 location
- **IP**: 46.224.206.37
- **Domain**: torrust-tracker.com (with subdomains)

### Subdomains

| Subdomain                     | Purpose                    | DNS Status    |
| ----------------------------- | -------------------------- | ------------- |
| `test.torrust-tracker.com`    | Experiment 1: Hello World  | ✅ Propagated |
| `api.torrust-tracker.com`     | Experiment 2: Tracker API  | ✅ Propagated |
| `http1.torrust-tracker.com`   | Experiment 3: HTTP Tracker | ✅ Propagated |
| `grafana.torrust-tracker.com` | Experiment 4: Grafana UI   | ✅ Propagated |

## Phases

### Phase 1: Environment Preparation

See [phase-1-environment-preparation.md](phase-1-environment-preparation.md) for:

- DNS configuration and propagation verification
- Server accessibility checks
- Prerequisites for running experiments

**Status**: ✅ Complete

### Phase 2: Experiments

| Experiment             | Document                                                     | Status               |
| ---------------------- | ------------------------------------------------------------ | -------------------- |
| 1. Minimal Hello World | [experiment-1-hello-world.md](experiment-1-hello-world.md)   | ✅ Complete          |
| 2. Tracker API HTTPS   | [experiment-2-tracker-api.md](experiment-2-tracker-api.md)   | ✅ Complete          |
| 3. HTTP Tracker HTTPS  | [experiment-3-http-tracker.md](experiment-3-http-tracker.md) | ✅ Complete          |
| 4. Grafana WebSocket   | [experiment-4-grafana.md](experiment-4-grafana.md)           | ⚠️ Partial (WS fail) |

## Key Questions to Answer

1. Does Pingoo automatically generate Let's Encrypt certificates? **✅ YES**
2. Does certificate renewal work without manual intervention? **⏳ Cannot test (90-day validity)**
3. Does Pingoo support WebSocket connections (needed for Grafana Live)? **❌ NO**
4. How does configuration complexity compare to nginx+certbot? **Much simpler (~10 lines vs ~50+)**
5. Are there any issues with TLS 1.3-only support? **✅ No issues detected**

## Findings Summary

### ✅ Successful Tests

- **Automatic certificate generation** - Pingoo obtained Let's Encrypt certs without manual steps
- **TLS 1.3 with post-quantum cryptography** - Uses X25519MLKEM768 key exchange
- **Minimal configuration** - Only ~10 lines of YAML needed
- **No email required** - Unlike certbot, no email setup needed
- **Tracker API proxying** - Health checks and API endpoints work perfectly
- **HTTP Tracker proxying** - BitTorrent announce/scrape work via HTTPS

### ❌ Failed Test

- **WebSocket support** - Pingoo strips the `Upgrade` header, breaking WebSocket connections
  - Root cause: `Upgrade` header treated as hop-by-hop header in `http_proxy_service.rs`
  - Impact: Grafana Live (real-time streaming) does not work
  - Workaround: Use nginx for services requiring WebSocket

### ⏳ Pending Verification

- **Certificate renewal** - Cannot test yet (cert valid for 90 days)

## Final Decision

**Not adopting Pingoo** - We will evaluate Caddy instead.

### Why Not Adopt Pingoo?

While Pingoo works excellently for the Tracker, the WebSocket limitation means we would
need a **hybrid architecture** (Pingoo for Tracker + another proxy for Grafana). This
adds unnecessary complexity for our simple deployment setup:

1. **Two proxies is overkill** - For a simple Tracker + Grafana setup, maintaining two
   different TLS proxies adds operational complexity without significant benefit.

2. **Caddy as alternative** - [Caddy](https://caddyserver.com/) offers similar benefits
   to Pingoo (automatic HTTPS, simple config) while also supporting WebSocket natively.
   It can handle both Tracker and Grafana with a single proxy.

3. **Maturity considerations** - Caddy is more mature and battle-tested than Pingoo.
   While Pingoo has attractive features (post-quantum crypto, Rust implementation),
   Caddy's stability may be more valuable for production deployments.

4. **Performance is not the bottleneck** - One concern about Caddy (written in Go) vs
   Pingoo (written in Rust) is performance. However, based on running a tracker demo
   for a couple of years, the proxy is unlikely to be the bottleneck. Most users prefer
   the UDP tracker which doesn't use the proxy at all.

### Open Issue on Pingoo

We filed [pingooio/pingoo#23](https://github.com/pingooio/pingoo/issues/23) to confirm
the WebSocket limitation and discuss potential solutions. If Pingoo adds WebSocket
support in the future, it could be reconsidered.

### Next Steps

A new research issue will be opened to evaluate **Caddy** as an alternative to
nginx+certbot, following the same experimental approach used here.

See [conclusion.md](conclusion.md) for full rationale.

## Timeline

- **2026-01-12**: Research started, Experiments 1-4 completed
- **2026-01-12**: WebSocket limitation discovered, issue filed on Pingoo repo
- **2026-01-13**: Decision made not to adopt Pingoo, research closed
