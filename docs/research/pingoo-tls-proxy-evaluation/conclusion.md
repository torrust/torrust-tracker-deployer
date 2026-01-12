# Pingoo TLS Proxy Evaluation - Conclusion

**Status**: Decision Pending WebSocket Verification
**Last Updated**: 2026-01-12

## Preliminary Decision

**Switch to Pingoo** as the primary TLS proxy for Torrust Tracker deployments.

Pingoo offers significant advantages in simplicity and modern security features that
make it the preferred choice over nginx+certbot for automatic HTTPS/TLS termination.

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

### WebSocket Support (Experiment 4)

Grafana Live uses WebSocket connections for real-time dashboard updates. We need to
verify that Pingoo correctly proxies WebSocket connections.

**Possible outcomes:**

1. **WebSocket works** → Use Pingoo for all services (Tracker API, HTTP Tracker, Grafana)
2. **WebSocket doesn't work** → Hybrid approach (see below)

### Fallback Strategy

If Pingoo doesn't support WebSocket for Grafana:

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Public Internet                          │
└─────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
         ┌──────────────────┐        ┌──────────────────┐
         │     Pingoo       │        │  nginx+certbot   │
         │   (port 443)     │        │   (port 3443)    │
         │                  │        │                  │
         │ api.example.com  │        │grafana.example.com│
         │http1.example.com │        │                  │
         └────────┬─────────┘        └────────┬─────────┘
                  │                           │
                  ▼                           ▼
         ┌──────────────────┐        ┌──────────────────┐
         │  Tracker API     │        │     Grafana      │
         │  HTTP Tracker    │        │   (WebSocket)    │
         └──────────────────┘        └──────────────────┘
```

**Benefits of hybrid approach:**

- Users who don't need Grafana get the simpler Pingoo-only setup
- Grafana users get WebSocket support via nginx
- Can migrate Grafana to Pingoo when WebSocket support is added

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

| Experiment             | Status      | Result                                   |
| ---------------------- | ----------- | ---------------------------------------- |
| 1. Hello World         | ✅ Complete | SUCCESS - Certificate auto-generated     |
| 2. Tracker API         | ⏳ Pending  | -                                        |
| 3. HTTP Tracker        | ⏳ Pending  | -                                        |
| 4. Grafana (WebSocket) | ⏳ Pending  | CRITICAL - Determines final architecture |

## Key Findings from Experiments

### Experiment 1: Hello World

- ✅ Automatic Let's Encrypt certificate generation works
- ✅ No email or manual steps required
- ✅ TLS 1.3 with post-quantum key exchange (X25519MLKEM768)
- ✅ ECDSA certificate from Let's Encrypt E8 intermediate
- ✅ Certificate stored with domain-named files for easy identification
- ✅ ACME account persisted for future renewals

## Next Steps

1. Complete Experiment 2 (Tracker API) - Verify JSON API proxying
2. Complete Experiment 3 (HTTP Tracker) - Verify announce/scrape endpoints
3. Complete Experiment 4 (Grafana) - **Critical** WebSocket verification
4. Finalize architecture decision based on Experiment 4 results
5. Update deployment templates to use Pingoo
6. Document migration path from nginx+certbot (if applicable)

## References

- [Pingoo Documentation](https://pingoo.io/docs)
- [Issue #234 - Evaluate Pingoo](https://github.com/torrust/torrust-tracker-deployer/issues/234)
- [Issue Specification](../../issues/234-evaluate-pingoo-for-https-termination.md)
