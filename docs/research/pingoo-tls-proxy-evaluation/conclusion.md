# Pingoo TLS Proxy Evaluation - Conclusion

**Status**: ✅ EVALUATION COMPLETE
**Last Updated**: 2026-01-12

## Final Decision

**Use hybrid architecture:** Pingoo for Tracker services, nginx for Grafana.

Pingoo provides excellent TLS termination for HTTP-based services but **does not support WebSocket connections**, which are required for Grafana Live. The hybrid approach maximizes Pingoo's simplicity benefits while maintaining full Grafana functionality.

## Architecture Diagram

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Public Internet                          │
└─────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
         ┌──────────────────┐        ┌───────────────────┐
         │     Pingoo       │        │  nginx+certbot    │
         │   (port 443)     │        │   (port 3443)     │
         │                  │        │                   │
         │ api.example.com  │        │grafana.example.com│
         │http1.example.com │        │                   │
         └────────┬─────────┘        └────────┬──────────┘
                  │                           │
                  ▼                           ▼
         ┌──────────────────┐        ┌───────────────────┐
         │  Tracker API     │        │     Grafana       │
         │  HTTP Tracker    │        │   (WebSocket)     │
         └──────────────────┘        └───────────────────┘
```

## Decision Rationale

### Why Pingoo?

| Aspect                    | Pingoo                       | nginx+certbot                             |
| ------------------------- | ---------------------------- | ----------------------------------------- |
| Configuration complexity  | ~10 lines YAML               | ~50+ lines (nginx config + certbot setup) |
| Email required            | ❌ No                        | ✅ Yes (or explicit opt-out)              |
| TLS version               | 1.3 only (modern)            | 1.2 and 1.3                               |
| Post-quantum cryptography | ✅ Built-in (X25519MLKEM768) | ❌ No                                     |
| Certificate auto-renewal  | ✅ Built-in                  | ✅ Via cron/systemd timer                 |
| Expiration notifications  | ❌ No                        | ✅ Via email                              |
| Single binary             | ✅ Yes                       | ❌ Multiple components                    |
| Docker-native             | ✅ Yes                       | ⚠️ Requires orchestration                 |

### Key Advantages

1. **Dramatically simpler configuration** - Just specify domains in YAML, no separate
   certbot commands or nginx virtual host configs

2. **Modern security by default** - TLS 1.3 only with post-quantum key exchange,
   no legacy protocol support to misconfigure

3. **Zero-touch certificate management** - No email setup, no cron jobs, no renewal
   scripts to maintain

4. **Better fit for container deployments** - Single container handles both TLS
   termination and reverse proxying

### Trade-offs Accepted

1. **No expiration email notifications** - Must implement own monitoring or rely on
   Pingoo's automatic renewal

2. **TLS 1.3 only** - Very old clients (pre-2018) won't connect. This is acceptable
   as modern BitTorrent clients all support TLS 1.3

3. **Newer project** - Less battle-tested than nginx+certbot, but actively maintained
   and well-documented

## Pending Verification

### Certificate Renewal

Certificate renewal cannot be tested during this evaluation (certificates are valid
for 90 days). Pingoo claims automatic renewal - this should work based on the ACME
implementation, but should be verified after deployment.

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

### Potential Future Solutions

1. **Pingoo WebSocket support** - The Pingoo team may add WebSocket support
2. **TCP+TLS mode** - Could use raw TCP proxying (loses HTTP routing)
3. **Feature request** - Could file an issue requesting WebSocket support

## Files to Backup (for Disaster Recovery)

When implementing backup procedures (Roadmap Task 7), include these Pingoo files:

| File         | Purpose                                             | Location                    |
| ------------ | --------------------------------------------------- | --------------------------- |
| `acme.json`  | ACME account credentials (private key + account ID) | `/etc/pingoo/tls/acme.json` |
| `*.key`      | Certificate private keys                            | `/etc/pingoo/tls/`          |
| `*.pem`      | Certificate chains                                  | `/etc/pingoo/tls/`          |
| `pingoo.yml` | Pingoo configuration                                | `/etc/pingoo/pingoo.yml`    |

**Note:** The `acme.json` file contains the ACME account private key. Losing this file
means you'll need to re-register with Let's Encrypt (not a major issue, but rate limits
apply to new registrations).

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
5. 🔲 File issue with Pingoo project requesting WebSocket support
6. 🔲 Update deployment templates with hybrid architecture
7. 🔲 Document migration path from pure nginx+certbot
8. 🔲 Implement Pingoo templates in deployer codebase

## References

- [Pingoo Documentation](https://pingoo.io/docs)
- [Issue #234 - Evaluate Pingoo](https://github.com/torrust/torrust-tracker-deployer/issues/234)
- [Issue Specification](../../issues/234-evaluate-pingoo-for-https-termination.md)
