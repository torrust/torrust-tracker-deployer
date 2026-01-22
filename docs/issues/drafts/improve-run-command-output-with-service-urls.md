# Draft: Improve Run Command Output with Service URLs

**Status**: Draft (not yet created on GitHub)
**Related Roadmap Section**: 10. Improve usability (UX)

## Summary

Enhance the `run` command output to display service URLs immediately after services start, plus a hint about the `show` command for full details.

## Context

Currently, the `run` command output is minimal:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 22.9s)
✅ Run command completed for 'lxd-local-example'
```

In contrast, the `show` command provides rich information about service URLs, connection details, and DNS instructions.

**Problem**: After running `run`, users want to immediately access their services but must run `show` separately to find the URLs.

## Proposed Output

After `run` succeeds, show the most actionable information (service URLs) plus a hint:

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 22.9s)
✅ Run command completed for 'lxd-local-example'

Services are now accessible:
  Tracker (UDP):  udp://10.140.190.188:6969/announce
  Tracker (HTTP): http://10.140.190.188:7070/announce
  API:            http://10.140.190.188:1212/api
  Grafana:        http://10.140.190.188:3000/

Tip: Run 'torrust-tracker-deployer show lxd-local-example' for full details.
```

For TLS-enabled environments:

```text
✅ Run command completed for 'lxd-local-https-example'

Services are now accessible:
  Tracker (UDP):  udp://udp.tracker.local:6969/announce
  Tracker (HTTP): https://http.tracker.local/announce
  API:            https://api.tracker.local/api
  Grafana:        https://grafana.tracker.local/

Note: HTTPS services require DNS configuration. See 'show' command for details.

Tip: Run 'torrust-tracker-deployer show lxd-local-https-example' for full details.
```

## Rationale

1. **Actionable** - User immediately knows where to access services
2. **Not overwhelming** - Doesn't duplicate the full `show` output (omits SSH details, internal ports, etc.)
3. **Educational** - Teaches users about the `show` command
4. **Follows project principles** - "Actionability: The system must always tell users how to continue"

The `run` command is the moment users want to _use_ the services, so showing URLs immediately is high value.

## Implementation Notes

- Reuse URL generation logic from `show` command
- Show subset of information: only service URLs
- Include DNS note for TLS environments
- Always include the `Tip` about `show` command

## Reference Outputs

See captured command outputs for current behavior:

- Without TLS: [lxd-local-example.md](../reference/command-outputs/lxd-local-example.md)
- With TLS: [lxd-local-https-example.md](../reference/command-outputs/lxd-local-https-example.md)

## Open Questions

1. Should we include the Health Check URL in the output?
2. Should Prometheus be mentioned (currently internal only)?
3. Should the output format match `show` exactly, or be more compact?
