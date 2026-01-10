# Decision: Hetzner SSH Key Dual Injection Pattern

## Status

Accepted

## Date

2026-01-10

## Context

When deploying to Hetzner Cloud, the deployer needs to configure SSH access for the provisioned server. There are two primary mechanisms available:

1. **Hetzner Provider SSH Key Resource**: The `hcloud_ssh_key` OpenTofu resource registers an SSH public key in Hetzner's account-level key registry. When a server is created with `ssh_keys = [hcloud_ssh_key.id]`, Hetzner automatically injects this key into the root user's `~/.ssh/authorized_keys` during server creation (before the OS boots).

2. **cloud-init SSH Key Injection**: The cloud-init `user-data` configuration can create a non-root user (e.g., `torrust`) with SSH authorized keys via the `ssh_authorized_keys` directive. This runs after the first boot.

The question arose: Do we need both mechanisms, or is cloud-init sufficient?

### The Problem with cloud-init Only

If cloud-init fails (syntax error, network issue, script error, timeout), the server becomes completely inaccessible:

- No SSH access to any user
- Cannot debug what went wrong
- Only option is to destroy and recreate the server
- Root cause may be unclear without access to logs

### Debugging Scenario

During development of this deployer, cloud-init issues were common:

- YAML syntax errors in cloud-init configuration
- Package installation failures due to network issues
- User creation failures
- Script execution errors

Having root SSH access (injected before cloud-init runs) provided a crucial debugging path.

## Decision

**Use both SSH key injection mechanisms for Hetzner deployments:**

1. **OpenTofu `hcloud_ssh_key` resource** with server `ssh_keys` reference → provides root SSH access as a fallback/debugging mechanism
2. **cloud-init `ssh_authorized_keys`** → provides application user SSH access for normal operations

The same SSH key is used for both mechanisms, so there is no additional key exposure.

### Why Not Make It Configurable?

While a configuration option (e.g., `enable_root_ssh_fallback`) was considered, we opted to always enable it because:

- The security risk is minimal (same key, user can disable post-deployment)
- The debugging benefit is significant
- Complexity of additional configuration outweighs the benefit
- Users who want stricter security can disable root access after deployment

## Consequences

### Positive

- **Debugging capability**: Can SSH as root to diagnose cloud-init failures
- **Recovery path**: Server is never completely inaccessible after creation
- **Same key**: No additional key exposure since the same key is used
- **Visibility**: SSH key appears in Hetzner Console for management

### Negative

- **Root access enabled by default**: Violates principle of least privilege
- **Provider-specific behavior**: LXD provider doesn't have this (uses `lxc exec` instead)
- **Manual cleanup needed**: Users wanting strict security must disable root access manually
- **Key in Hetzner registry**: SSH key visible in Hetzner Console Security section

### Neutral

- **Documentation required**: Users need to understand this behavior
- **Post-deployment hardening**: Security-conscious users have a clear path to disable

## Alternatives Considered

### Alternative 1: cloud-init Only

Remove the `hcloud_ssh_key` resource and rely solely on cloud-init.

**Rejected because**: Loses debugging capability. Failed cloud-init means inaccessible server.

### Alternative 2: Configurable via Environment Config

Add a configuration option like `enable_root_ssh_fallback: bool`.

**Rejected because**: Added complexity for minimal benefit. Users can disable manually.

### Alternative 3: Disable by Default, Enable for Debug

Only create the Hetzner SSH key when an environment variable or flag is set.

**Rejected because**: Most users would benefit from the fallback, and forgetting to enable it when needed causes frustration.

## Related Decisions

- [Cloud-Init SSH Port Configuration with Reboot](./cloud-init-ssh-port-reboot.md) - Related cloud-init configuration pattern
- [Configuration Directories as Secrets](./configuration-directories-as-secrets.md) - Security considerations for configuration

## References

- [Hetzner Cloud SSH Keys Documentation](https://docs.hetzner.com/cloud/servers/getting-started/connecting-to-a-server/)
- [cloud-init User Data Documentation](https://cloudinit.readthedocs.io/en/latest/reference/examples.html#including-users-and-groups)
- [OpenTofu Hetzner Provider - hcloud_ssh_key](https://registry.terraform.io/providers/hetznercloud/hcloud/latest/docs/resources/ssh_key)
- [Security Doc: SSH Root Access on Hetzner](../security/ssh-root-access-hetzner.md)
