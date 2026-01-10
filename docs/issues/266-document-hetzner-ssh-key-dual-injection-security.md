# Document Hetzner SSH Key Dual Injection and Root Access Security

**Issue**: [#266](https://github.com/torrust/torrust-tracker-deployer/issues/266)
**Parent Epic**: None (standalone documentation task)
**Related**:

- [Hetzner Provider Documentation](../user-guide/providers/hetzner.md)
- [LXD Provider Documentation](../user-guide/providers/lxd.md)
- [SSH Keys Guide](../tech-stack/ssh-keys.md)

## Overview

When deploying to Hetzner Cloud, the deployer configures SSH key access through two independent mechanisms:

1. **OpenTofu `hcloud_ssh_key` resource**: Registers the SSH public key in Hetzner's account-level key registry and attaches it to the server, enabling root SSH access
2. **cloud-init `ssh_authorized_keys`**: Injects the same SSH public key into the application user's (`torrust`) authorized_keys file

This results in **both root and application user having SSH access** after deployment. While this behavior is intentional (provides cloud-init debugging capability), users should be aware of this security implication and can optionally disable root SSH access after successful deployment.

This issue tracks documenting this behavior across the codebase and providing guidance for users who want stricter security.

## Goals

- [ ] Document the SSH key dual injection behavior
- [ ] Explain the security implications
- [ ] Provide instructions for disabling root SSH access post-deployment
- [ ] Create a decision record explaining the architectural rationale
- [ ] Document why LXD provider doesn't have this behavior

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (documentation only)
**Module Path**: N/A
**Pattern**: Documentation and Decision Record

### Architectural Context

The Hetzner provider uses a dual SSH key injection pattern:

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Hetzner Cloud                                │
├─────────────────────────────────────────────────────────────────┤
│  1. OpenTofu creates hcloud_ssh_key resource                    │
│     └─ Key appears in Hetzner Console → Security → SSH Keys     │
│                                                                 │
│  2. OpenTofu creates server with ssh_keys = [hcloud_ssh_key.id] │
│     └─ Hetzner injects key into root's ~/.ssh/authorized_keys   │
│                                                                 │
│  3. cloud-init runs user-data script                            │
│     └─ Creates 'torrust' user with same key in authorized_keys  │
└─────────────────────────────────────────────────────────────────┘
```

The LXD provider only uses cloud-init (mechanism 3), making the behavior provider-specific.

## Specifications

### Why Both Mechanisms Exist

| Mechanism                        | Purpose                        | When It Runs                         |
| -------------------------------- | ------------------------------ | ------------------------------------ |
| OpenTofu `hcloud_ssh_key`        | Emergency/debug access as root | During server creation (before boot) |
| cloud-init `ssh_authorized_keys` | Application user access        | After first boot                     |

**Primary reason for keeping OpenTofu SSH key**: If cloud-init fails (syntax error, network issue, script error), the server would be completely inaccessible without root SSH access. The OpenTofu SSH key provides a recovery path.

### Security Implications

**Risks of root SSH access**:

- Root has unrestricted system access
- Compromised SSH key grants full system control
- Violates principle of least privilege

**Mitigations in place**:

- Application runs as non-root user (`torrust`)
- User has passwordless sudo for administrative tasks
- SSH key is the same for both users (no additional exposure)

### How to Disable Root SSH Access

Users who want stricter security can disable root SSH access after verifying deployment succeeded:

#### Option 1: Remove root's authorized_keys

```bash
ssh torrust@<server-ip> "sudo rm /root/.ssh/authorized_keys"
```

#### Option 2: Disable root login via SSH config

```bash
ssh torrust@<server-ip> "sudo sed -i 's/#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config && sudo systemctl restart sshd"
```

#### Option 3: Delete SSH key from Hetzner Console

1. Go to Hetzner Cloud Console → Security → SSH Keys
2. Find the key named `torrust-tracker-vm-<environment>-ssh-key`
3. Delete it (note: this only affects future servers, not the current one)

### Why LXD Provider Is Different

The LXD provider doesn't create a provider-level SSH key resource because:

1. **Local access**: LXD runs locally, so `lxc exec` provides direct console access without SSH
2. **No account-level keys**: LXD doesn't have a concept of account-level SSH key registry
3. **Simpler model**: cloud-init is sufficient for all access needs

## Implementation Plan

### Phase 1: Decision Record (30 min)

- [ ] Task 1.1: Create ADR `docs/decisions/hetzner-ssh-key-dual-injection.md` documenting the architectural decision
- [ ] Task 1.2: Update ADR index in `docs/decisions/README.md`

### Phase 2: Security Documentation (30 min)

- [ ] Task 2.1: Create `docs/security/ssh-root-access-hetzner.md` explaining the security implications
- [ ] Task 2.2: Include step-by-step instructions for disabling root access
- [ ] Task 2.3: Document the debugging use case that justifies keeping it

### Phase 3: Provider Documentation Updates (30 min)

- [ ] Task 3.1: Update `docs/user-guide/providers/hetzner.md` with a new "SSH Key Behavior" section
- [ ] Task 3.2: Update `docs/user-guide/providers/lxd.md` explaining why it doesn't have this behavior
- [ ] Task 3.3: Add cross-references between provider docs and security doc

### Phase 4: Template Comments (15 min)

- [ ] Task 4.1: Add explanatory comment to `templates/tofu/hetzner/main.tf` at the `hcloud_ssh_key` resource

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] ADR created with proper status, date, and all sections filled
- [ ] ADR index updated in README.md
- [ ] Security document explains both the risk and the mitigation
- [ ] Security document includes working commands to disable root access
- [ ] Hetzner provider docs reference the security document
- [ ] LXD provider docs explain why it's different
- [ ] Template has clear comment explaining the SSH key resource purpose
- [ ] All links between documents are valid
- [ ] No spelling errors (cspell passes)

## Related Documentation

- [Hetzner Cloud SSH Keys Documentation](https://docs.hetzner.com/cloud/servers/getting-started/connecting-to-a-server/)
- [cloud-init User Data Documentation](https://cloudinit.readthedocs.io/en/latest/reference/examples.html#including-users-and-groups)
- [OpenSSH Security Best Practices](https://www.openssh.com/security.html)

## Notes

- This is a documentation-only change; no code modifications required
- Future enhancement: Consider making root SSH access configurable via environment configuration
- The same SSH key is used for both mechanisms, so there's no additional key exposure risk
- This pattern is common in cloud deployments where cloud-init reliability cannot be guaranteed
