# Draft: Add DNS Resolution Check to Test Command

**Status**: Draft (not yet created on GitHub)
**Related Roadmap Section**: 10. Improve usability (UX)

## Summary

Add an optional DNS resolution check to the `test` command that verifies configured domains resolve to the expected instance IP.

## Context

The `test` command validates infrastructure health by making actual requests to services. However, these tests use the instance IP directly (resolved internally) rather than relying on external DNS resolution. This is intentional - it decouples infrastructure tests from user's DNS configuration.

**Current behavior:**

- Tests resolve domains internally using the known instance IP
- Tests pass regardless of external DNS configuration
- User may not realize their DNS isn't configured correctly

**Problem:** A user could have all tests pass but still be unable to access services via domain names because their DNS isn't configured.

## Proposed Behavior

Add a DNS resolution check as an advisory warning (not a failure):

```text
⏳ [3/3] Testing infrastructure...
⏳   ✓ Cloud-init completed
⏳   ✓ Docker running
⏳   ✓ Docker Compose running
⏳   ✓ HTTP tracker responding
⏳   ✓ API responding
⚠️   DNS check: http.tracker.local does not resolve (expected: 10.140.190.254)
⚠️   DNS check: api.tracker.local does not resolve (expected: 10.140.190.254)
⏳   ✓ Infrastructure tests passed (with warnings)
✅ Infrastructure validation completed successfully for 'lxd-local-https-example'
```

When DNS is correctly configured:

```text
⏳   ✓ DNS check: http.tracker.local → 10.140.190.254 ✓
⏳   ✓ DNS check: api.tracker.local → 10.140.190.254 ✓
```

## Design Decisions

1. **Warning, not failure** - DNS issues shouldn't fail infrastructure tests because:
   - DNS propagation can take time
   - Local `.local` domains use `/etc/hosts`, not real DNS
   - User may intentionally be testing without DNS

2. **Separate from service tests** - This is a distinct check from "is the service running":
   - Service tests: verify the application works (using internal IP resolution)
   - DNS check: verify external access will work (using system DNS)

3. **Only when domains configured** - Skip the check entirely if no domains are defined

## Implementation Notes

- Use system DNS resolution (not internal resolution)
- Check each configured domain (HTTP trackers, API, health check, Grafana)
- Compare resolved IP with expected instance IP
- Report as warnings, not errors
- Include expected IP in warning message for troubleshooting

## Considerations

- **Local domains (.local)**: These typically use mDNS or `/etc/hosts`, not DNS servers
- **DNS propagation**: New records can take minutes to hours to propagate
- **Multiple IPs**: Some domains may legitimately resolve to different IPs (load balancers, CDNs)

## Reference Outputs

See captured command outputs:

- Without TLS: [lxd-local-example.md](../reference/command-outputs/lxd-local-example.md)
- With TLS: [lxd-local-https-example.md](../reference/command-outputs/lxd-local-https-example.md)

## Open Questions

1. Should there be a flag to make DNS checks strict (fail on mismatch)?
2. Should we check from the deployer machine or from the target instance?
3. How to handle `.local` domains (mDNS vs /etc/hosts)?
