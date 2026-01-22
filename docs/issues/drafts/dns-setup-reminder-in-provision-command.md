# Draft: Add DNS Setup Reminder in Provision Command

**Status**: Draft (not yet created on GitHub)
**Related Roadmap Section**: 10. Improve usability (UX)

## Summary

Add a reminder message in the `provision` command output to inform users they need to configure DNS to point their domains to the newly assigned server IP.

## Context

When the `provision` command completes, the user receives a new IP address. If they have configured domains for any services, they need to update their DNS records to point to this IP. Currently, this information is only shown in the `show` command output when TLS proxy is enabled.

**Current behavior:**

- `show` command displays DNS setup info only when `use_tls_proxy: true`
- `provision` command shows the IP but no DNS reminder

**Proposed behavior:**

- `provision` command shows a DNS reminder when ANY service has a domain configured (with or without TLS)
- The reminder is proactive, appearing when the IP is first assigned

## Why This Matters

- The IP is new at `provision` time - user hasn't had time to configure DNS yet
- It's a proactive reminder, not a troubleshooting message
- Prevents confusion when services fail due to DNS misconfiguration
- Domains are optional, so the reminder should only appear when domains are actually configured

## Proposed Output

When at least one domain is configured in the environment:

```text
✅ Environment 'my-environment' provisioned successfully

Instance Connection Details:
  IP Address:        XX.XX.XX.XX
  ...

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: XX.XX.XX.XX

  Configured domains:
    - http.tracker.example.com
    - api.tracker.example.com
    - grafana.example.com
```

## Implementation Notes

- Check all services for domain configuration (HTTP trackers, API, health check, Grafana)
- Collect all unique domains
- Only show the reminder if at least one domain is configured
- Message should be informative but not alarming (it's expected that DNS isn't set up yet)

## Reference Outputs

See captured command outputs for comparison:

- Without TLS: [lxd-local-example.md](../reference/command-outputs/lxd-local-example.md)
- With TLS: [lxd-local-https-example.md](../reference/command-outputs/lxd-local-https-example.md)

## Open Questions

1. Should we also list the domains that need to be configured?
2. Should the message appear in other commands too (e.g., `show`)?
3. Should we differentiate between TLS and non-TLS domain requirements?
