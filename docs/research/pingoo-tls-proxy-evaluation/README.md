# Pingoo TLS Proxy Evaluation

**Issue**: [#234](https://github.com/torrust/torrust-tracker-deployer/issues/234)
**Specification**: [docs/issues/234-evaluate-pingoo-for-https-termination.md](../../issues/234-evaluate-pingoo-for-https-termination.md)
**Status**: In Progress
**Started**: 2026-01-12

## Overview

This research evaluates [Pingoo](https://pingoo.io/) as a potential replacement for nginx+certbot
for automatic HTTPS/TLS termination in Torrust Tracker deployments.

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

| Experiment             | Document                                                     | Status      |
| ---------------------- | ------------------------------------------------------------ | ----------- |
| 1. Minimal Hello World | [experiment-1-hello-world.md](experiment-1-hello-world.md)   | ✅ Complete |
| 2. Tracker API HTTPS   | [experiment-2-tracker-api.md](experiment-2-tracker-api.md)   | ✅ Complete |
| 3. HTTP Tracker HTTPS  | [experiment-3-http-tracker.md](experiment-3-http-tracker.md) | ✅ Complete |
| 4. Grafana WebSocket   | [experiment-4-grafana.md](experiment-4-grafana.md)           | Not started |

## Key Questions to Answer

1. Does Pingoo automatically generate Let's Encrypt certificates?
2. Does certificate renewal work without manual intervention?
3. Does Pingoo support WebSocket connections (needed for Grafana Live)?
4. How does configuration complexity compare to nginx+certbot?
5. Are there any issues with TLS 1.3-only support?

## Preliminary Findings (from Experiment 1)

- ✅ **Automatic certificate generation works** - Pingoo obtained Let's Encrypt cert without manual steps
- ✅ **TLS 1.3 with post-quantum cryptography** - Uses X25519MLKEM768 key exchange
- ✅ **Minimal configuration** - Only 10 lines of YAML needed
- ✅ **No email required** - Unlike certbot, no email setup needed
- ⏳ **Certificate renewal** - Cannot test yet (cert valid for 90 days)
- ⏳ **WebSocket support** - Will test in Experiment 4

## Preliminary Decision

**Switch to Pingoo** - See [conclusion.md](conclusion.md) for full rationale.

Key factors:

- Dramatically simpler configuration (~10 lines vs ~50+ for nginx+certbot)
- Modern security defaults (TLS 1.3, post-quantum crypto)
- Zero-touch certificate management

**Pending**: WebSocket verification for Grafana (Experiment 4). If WebSocket doesn't
work, a hybrid approach (Pingoo for Tracker, nginx for Grafana) is planned.

## Timeline

- **2026-01-12**: Research started, Experiment 1 completed ✅
- **TBD**: Experiments 2-4 completed
- **TBD**: Final decision after WebSocket verification
