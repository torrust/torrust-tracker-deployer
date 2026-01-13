# Caddy TLS Proxy Evaluation

**Issue**: [#270](https://github.com/torrust/torrust-tracker-deployer/issues/270)  
**Date**: January 13, 2026  
**Status**: ✅ **COMPLETE - RECOMMEND ADOPTION**

## Overview

This evaluation tested Caddy v2.10 as a TLS termination proxy for the Torrust Tracker stack. The primary success criterion was WebSocket support, which was the critical failure point for Pingoo (issue #234).

**Result**: ✅ Caddy successfully passes all tests, including WebSocket support.

## Documentation

- **[Experiment Report](./experiment-full-stack.md)** - Complete deployment procedure, test results, and technical details
- **[Conclusion](./conclusion.md)** - Evaluation summary, comparison with alternatives, and recommendation
- **[Configuration Files](./experiment-files/)** - All configuration files used in the experiment (secrets redacted)

## Quick Summary

### ✅ Successes

1. **WebSocket Support** - Works perfectly (Pingoo failed this)
2. **Automatic HTTPS** - Let's Encrypt certificates in ~3-4 seconds
3. **Simple Configuration** - 21 lines (Caddyfile) vs complex nginx+certbot
4. **All Endpoints Working** - Tracker API, HTTP Tracker, Grafana
5. **Built-in HTTP/3** - QUIC support out of the box
6. **Automatic Renewal** - Zero operational overhead

### WebSocket Test (Critical)

```text
URL: wss://grafana.torrust-tracker.com/api/live/ws
Status: 101 Switching Protocols
Result: ✅ Dashboard live updates working perfectly
```

### Recommendation

**✅ ADOPT CADDY** as the TLS termination proxy for Torrust Tracker deployments.

## Next Steps

1. Create ADR documenting adoption decision
2. Update deployer templates with Caddy configuration
3. Document in user guide
4. Migrate production deployments

## References

- [Caddy Official Website](https://caddyserver.com/)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Issue #270](https://github.com/torrust/torrust-tracker-deployer/issues/270)
- [Pingoo Evaluation](../pingoo-tls-proxy-evaluation/) (not adopted)
