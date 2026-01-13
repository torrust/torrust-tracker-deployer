# Pingoo TLS Proxy Evaluation - Conclusion

**Status**: ✅ EVALUATION COMPLETE - NOT ADOPTING
**Last Updated**: 2026-01-13

## Final Decision

**Not adopting Pingoo.** We will evaluate Caddy as an alternative instead.

Pingoo provides excellent TLS termination for HTTP-based services but **does not support
WebSocket connections**, which are required for Grafana Live. Rather than adopting a
hybrid architecture (Pingoo + another proxy), we have decided to evaluate Caddy, which
can handle both the Tracker and Grafana with a single proxy.

## Why Not Adopt Pingoo?

### The Hybrid Architecture Problem

Initially, we considered using Pingoo for the Tracker and nginx/Caddy for Grafana:

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Public Internet                          │
└─────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
         ┌──────────────────┐        ┌───────────────────┐
         │     Pingoo       │        │  nginx or Caddy   │
         │   (port 443)     │        │   (port 3443)     │
         └────────┬─────────┘        └────────┬──────────┘
                  │                           │
                  ▼                           ▼
         ┌──────────────────┐        ┌───────────────────┐
         │  Tracker API     │        │     Grafana       │
         │  HTTP Tracker    │        │   (WebSocket)     │
         └──────────────────┘        └───────────────────┘
```

However, this approach has significant drawbacks for our use case:

1. **Operational complexity** - Two different TLS proxies means two different
   configurations, two sets of certificates to manage, and two potential points of
   failure.

2. **Overkill for simple setups** - For a typical Torrust deployment (Tracker +
   Grafana), maintaining two proxies adds unnecessary complexity without clear benefits.

3. **Better alternatives exist** - Caddy offers similar benefits to Pingoo (automatic
   HTTPS, simple configuration) while also supporting WebSocket natively.

### Caddy as Alternative

[Caddy](https://caddyserver.com/) is a more mature alternative that can replace
nginx+certbot entirely:

| Feature              | Pingoo            | Caddy          |
| -------------------- | ----------------- | -------------- |
| Automatic HTTPS      | ✅ Yes            | ✅ Yes         |
| Simple config        | ✅ ~10 lines      | ✅ ~5-10 lines |
| WebSocket support    | ❌ No             | ✅ Native      |
| Post-quantum crypto  | ✅ X25519MLKEM768 | ❌ No          |
| TLS versions         | 1.3 only          | 1.2 and 1.3    |
| Language             | Rust              | Go             |
| Maturity             | Newer             | Very mature    |
| Single proxy for all | ❌ Needs hybrid   | ✅ Yes         |

### Performance Considerations

One concern about Caddy (Go) vs Pingoo (Rust) is performance. However:

- **Proxy is not the bottleneck** - Based on running a Torrust tracker demo for a couple
  of years, the TLS proxy is unlikely to be the performance bottleneck.

- **UDP tracker dominates** - Most BitTorrent clients prefer the UDP tracker protocol,
  which doesn't go through the HTTP/HTTPS proxy at all.

- **Stability over raw speed** - For a production deployment, Caddy's maturity and
  stability may be more valuable than Pingoo's potential performance advantages.

### What We Lose by Not Using Pingoo

1. **Post-quantum cryptography** - Pingoo's X25519MLKEM768 key exchange provides
   protection against future quantum computers. Caddy doesn't have this yet.

2. **TLS 1.3-only enforcement** - Pingoo only supports TLS 1.3, which simplifies
   security configuration. Caddy supports both 1.2 and 1.3.

3. **Rust implementation** - Pingoo's Rust codebase may offer better memory safety
   and performance characteristics.

These are trade-offs we accept in favor of a simpler, single-proxy architecture.

## Open Issue on Pingoo Repository

We filed [pingooio/pingoo#23](https://github.com/pingooio/pingoo/issues/23) to:

1. Confirm that WebSocket proxying is not currently supported
2. Discuss potential solutions or workarounds
3. Request consideration for adding WebSocket support

If Pingoo adds WebSocket support in the future, it could be reconsidered for Torrust
deployments.

## WebSocket Limitation - Root Cause

Pingoo's HTTP proxy explicitly removes the `Upgrade` header, which is required for
WebSocket protocol upgrades. From the source code:

```rust
// https://github.com/pingooio/pingoo/blob/main/pingoo/services/http_proxy_service.rs
const HOP_HEADERS: &[&str] = &[
    "Connection",
    // ... other headers ...
    "Upgrade",  // This breaks WebSocket!
];
```

This means any service requiring WebSocket connections cannot use Pingoo's `http_proxy`.
This is a fundamental limitation, not a configuration issue.

## Experiment Results Summary

| Experiment             | Status      | Result                                       |
| ---------------------- | ----------- | -------------------------------------------- |
| 1. Hello World         | ✅ Complete | SUCCESS - Certificate auto-generated         |
| 2. Tracker API         | ✅ Complete | SUCCESS - API endpoints work via HTTPS       |
| 3. HTTP Tracker        | ✅ Complete | SUCCESS - BitTorrent announce/scrape working |
| 4. Grafana (WebSocket) | ⚠️ Partial  | HTTP works, WebSocket FAILS                  |

## Key Findings from Experiments

### Experiment 1: Hello World

- ✅ Automatic Let's Encrypt certificate generation works
- ✅ No email or manual steps required
- ✅ TLS 1.3 with post-quantum key exchange (X25519MLKEM768)
- ✅ ECDSA certificate from Let's Encrypt E8 intermediate
- ✅ Certificate stored with domain-named files for easy identification
- ✅ ACME account persisted for future renewals

### Experiment 2: Tracker API

- ✅ JSON API responses proxied correctly
- ✅ Health check endpoints work
- ✅ No issues with TLS 1.3 for API clients

### Experiment 3: HTTP Tracker

- ✅ BitTorrent `announce` endpoint works via HTTPS
- ✅ BitTorrent `scrape` endpoint works via HTTPS
- ✅ Binary bencoded responses handled correctly

### Experiment 4: Grafana (WebSocket)

- ✅ HTTP dashboard access works
- ✅ Login and navigation work
- ❌ **WebSocket fails** - `Upgrade` header stripped by Pingoo
- ❌ Grafana Live (real-time streaming) does not work

## Next Steps

1. ✅ ~~Complete Experiment 1 (Hello World)~~ - Certificate auto-generation verified
2. ✅ ~~Complete Experiment 2 (Tracker API)~~ - JSON API proxying verified
3. ✅ ~~Complete Experiment 3 (HTTP Tracker)~~ - BitTorrent protocol verified
4. ✅ ~~Complete Experiment 4 (Grafana)~~ - WebSocket limitation discovered
5. ✅ ~~File issue with Pingoo project~~ - [pingooio/pingoo#23](https://github.com/pingooio/pingoo/issues/23)
6. ✅ ~~Decision made~~ - Not adopting Pingoo, will evaluate Caddy instead
7. 🔲 Open new issue to evaluate Caddy as nginx+certbot replacement
8. 🔲 Run Caddy experiments following similar methodology

## References

- [Pingoo Documentation](https://pingoo.io/docs)
- [Pingoo WebSocket Issue](https://github.com/pingooio/pingoo/issues/23) - Our issue asking about WebSocket support
- [Issue #234 - Evaluate Pingoo](https://github.com/torrust/torrust-tracker-deployer/issues/234)
- [Issue Specification](../../issues/234-evaluate-pingoo-for-https-termination.md)
- [Caddy Server](https://caddyserver.com/) - Alternative to evaluate next
