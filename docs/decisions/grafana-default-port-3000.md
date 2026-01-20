# Decision: Use Grafana's Default Port 3000 Instead of 3100

## Status

Accepted

## Date

2026-01-20

## Context

The Grafana service configuration was originally copied from the [Torrust Demo project](https://github.com/torrust/torrust-demo), which uses port 3100 on the host to avoid conflicts with other services commonly using port 3000 (like Node.js development servers that typically run on port 3000).

In the Torrust Demo environment, this port offset was necessary because:

- The demo runs multiple services that might conflict with development tools
- Users often have Node.js applications running on port 3000
- The demo is frequently run on developer machines with other services active

However, in the Torrust Tracker Deployer context:

- Deployments target dedicated VM instances, not developer machines
- No other services in our stack use port 3000
- Users expect Grafana to be on its default port
- The port offset (3100 vs 3000) causes confusion when consulting Grafana documentation

## Decision

Change the Grafana host port from `3100` to `3000`, matching Grafana's internal default port.

**Before:**

```yaml
grafana:
  ports:
    - "3100:3000" # Host:Container
```

**After:**

```yaml
grafana:
  ports:
    - "3000:3000" # Host:Container (using Grafana's default port)
```

## Rationale

1. **Simplicity**: Using `3000:3000` is more intuitive than `3100:3000` - host and container ports match
2. **No Conflict**: In our deployment context (dedicated VMs), there's no service using port 3000
3. **Documentation Alignment**: Grafana's official documentation uses port 3000; matching this reduces confusion
4. **User Expectations**: Users familiar with Grafana expect it on port 3000
5. **Reduced Cognitive Load**: One less port mapping to remember

## Consequences

### Positive

- More intuitive port configuration (same port inside and outside container)
- Better alignment with Grafana's official documentation
- Simpler mental model for users
- Consistent with how most Grafana installations are configured

### Negative

- **Breaking Change for Existing Deployments**: Users with existing deployments using port 3100 will need to update their bookmarks, scripts, and firewall rules
- **Migration Required**: Existing environments need to be re-released to apply the new port configuration

### Migration Path

For existing deployments:

1. Run `release <environment>` to regenerate docker-compose files with new port
2. Run `run <environment>` to restart services with new configuration
3. Update any bookmarks, scripts, or monitoring that reference port 3100

## Alternatives Considered

### Keep Port 3100

**Pros**: No breaking change for existing users

**Cons**: Perpetuates unnecessary complexity inherited from a different project context

**Decision**: Rejected - the benefits of using the default port outweigh the one-time migration cost

### Make Port Configurable

**Pros**: Maximum flexibility

**Cons**: Adds configuration complexity for a setting rarely needed; violates "convention over configuration" principle

**Decision**: Rejected - not worth the added complexity; users with special needs can fork/modify

## Related

- [Issue #275](https://github.com/torrust/torrust-tracker-deployer/issues/275) - Implementation tracking
- [ADR: Grafana Integration Pattern](grafana-integration-pattern.md) - Original Grafana integration decision
- [Torrust Demo](https://github.com/torrust/torrust-demo) - Original source of the 3100 port choice
