# GitHub Copilot Agent Firewall Configuration

This document describes the firewall configuration for GitHub Copilot coding agent in this repository and provides guidance for future maintenance.

## Overview

The GitHub Copilot coding agent operates in a restricted environment with a firewall that blocks external network access by default. This configuration is necessary to allow the agent to install project dependencies using the [dependency-installer](../../packages/dependency-installer/README.md) tool.

## Current Configuration

### Firewall Status

- **Firewall**: ✅ Enabled (recommended for security)
- **Recommended Allowlist**: ✅ Enabled (pre-configured common repositories)
- **Custom Allowlist**: ✅ Configured (project-specific domains)

### Custom Allowlist Domains

The following domains have been added to the custom allowlist:

#### opentofu.org

- **Purpose**: OpenTofu installation
- **Used By**: `packages/dependency-installer/src/installer/opentofu.rs`
- **Rationale**: Downloads OpenTofu installer script from `get.opentofu.org` and installation packages
- **Subdomain Coverage**: Allows traffic to all subdomains (e.g., `get.opentofu.org`)
- **Date Added**: November 5, 2025
- **Added By**: Repository administrator

### Domains Covered by Recommended Allowlist

The following dependencies are automatically allowed through GitHub's recommended allowlist and do **not** require custom configuration:

#### Package Repositories

- **Ubuntu/Debian APT Repositories**: Used by Ansible installer (`apt-get install ansible`)
- **Rust Package Registry (crates.io)**: Used by cargo-machete installer (`cargo install cargo-machete`)
- **Snap Store**: Used by LXD installer (`snap install lxd`)

These are included in GitHub's default recommended allowlist which covers common package repositories, container registries, and certificate authorities.

## Configuration Steps

### Prerequisites

- Repository admin access required
- Must be logged into GitHub

### Step-by-Step Instructions

1. Navigate to repository settings:

   ```text
   https://github.com/torrust/torrust-tracker-deployer/settings
   ```

2. In the sidebar under "Code & automation", click:
   - **Copilot** → **coding agent**

3. Verify firewall settings:
   - ✅ Ensure **Enable firewall** is toggled ON
   - ✅ Ensure **Recommended allowlist** is toggled ON

4. Configure custom allowlist:
   - Click **Custom allowlist**
   - In the "Add domain" field, enter: `opentofu.org`
   - Click **Add Rule**
   - Click **Save changes**

5. Verify configuration:
   - The custom allowlist should now show `opentofu.org`
   - Firewall and recommended allowlist should remain enabled

## Domain vs URL Rules

When configuring the custom allowlist, you can add either domains or specific URLs:

- **Domain** (e.g., `opentofu.org`):
  - ✅ Allows traffic to the domain **and all subdomains**
  - ✅ Recommended for most cases
  - Example: `opentofu.org` allows both `get.opentofu.org` and `packages.opentofu.org`

- **URL** (e.g., `https://get.opentofu.org/installer/`):
  - ⚠️ Only allows specified scheme, host, and path
  - ⚠️ More restrictive, harder to maintain
  - Use only when you need to restrict to specific paths

**Recommendation**: Use domain rules for flexibility and easier maintenance.

## Security Considerations

### Best Practices

1. ✅ **Keep firewall enabled** - Protects against data exfiltration
2. ✅ **Keep recommended allowlist enabled** - Covers common package repositories
3. ✅ **Use minimal custom allowlist** - Only add domains that are absolutely necessary
4. ✅ **Document each domain** - Explain why each domain is needed
5. ❌ **Never disable the firewall** - Increases security risks significantly

### Limitations

From GitHub documentation, the Copilot agent firewall has the following limitations:

- **Scope**: Only applies to processes started by the agent via its Bash tool
- **Not Applied To**:
  - Model Context Protocol (MCP) servers
  - Processes started in configured Copilot setup steps
- **Security Note**: Sophisticated attacks may bypass the firewall
- **Environment**: Only operates within GitHub Actions appliance environment

For more details, see [GitHub's official documentation](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall).

## Testing Firewall Configuration

After adding a domain to the allowlist, verify that the dependency installer can access it:

```bash
# Test OpenTofu installation
cargo run --bin dependency-installer install --dependency opentofu

# Expected result: Installation should succeed without DNS resolution errors
# Before configuration: "curl: (6) Could not resolve host: get.opentofu.org"
# After configuration: Installation completes successfully
```

## Future Maintenance

### Adding New Domains

When adding new dependency installers that require external network access:

1. **Test First**: Run the installer in the Copilot agent environment
2. **Check for Errors**: Look for DNS resolution or connection failures
3. **Identify Domain**: Determine which domain needs to be whitelisted
4. **Add to Allowlist**: Follow the configuration steps above
5. **Update Documentation**: Add the new domain to this document with:
   - Purpose and rationale
   - Which installer uses it
   - Date added and who added it
6. **Test Again**: Verify the installer now works
7. **Update Issue Spec**: Update [issue #147 specification](../issues/147-1-7-configure-copilot-agent-firewall-for-dependency-installer.md) if needed

### Troubleshooting Common Issues

#### DNS Resolution Errors

```bash
curl: (6) Could not resolve host: example.com
```

**Solution**: Add `example.com` to the custom allowlist.

#### Connection Refused Errors

```bash
curl: (7) Failed to connect to example.com port 443: Connection refused
```

**Possible Causes**:

- Domain not in allowlist (add it)
- Service is down (check service status)
- Wrong port/protocol (verify URL)

#### Subdomain Access Issues

If you added `example.com` but `api.example.com` is still blocked:

**Solution**: Domain rules should cover subdomains. Verify:

- Domain was added correctly (not as a URL)
- Changes were saved
- Try again after a few minutes (changes may take time to propagate)

## Related Documentation

### GitHub Documentation

- [Customizing the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall)
- [Preinstalling tools in Copilot's environment](https://docs.github.com/en/copilot/customizing-copilot/customizing-the-development-environment-for-copilot-coding-agent#preinstalling-tools-in-copilots-environment)

### Project Documentation

- [Dependency Installer Package](../../packages/dependency-installer/README.md)
- [E2E Testing Guide](../e2e-testing.md)
- [Issue #147 Specification](../issues/147-1-7-configure-copilot-agent-firewall-for-dependency-installer.md)
- [Issue #146 - Update Pre-Commit Script](../issues/146-1-6-update-precommit-script-for-github-runner-compatible-e2e-tests.md)

## History

### Configuration Changes

| Date | Change | Added By | Rationale |
|------|--------|----------|-----------|
| 2025-11-05 | Added `opentofu.org` | Repository administrator | Enable OpenTofu installation for dependency-installer tool |

### Documentation Changes

| Date | Change | Author |
|------|--------|--------|
| 2025-11-05 | Initial documentation | GitHub Copilot Agent |

## Notes

- This configuration was created as part of [Issue #147](https://github.com/torrust/torrust-tracker-deployer/issues/147)
- Parent epic: [Issue #112 - Refactor and Improve E2E Test Execution](https://github.com/torrust/torrust-tracker-deployer/issues/112)
- Repository settings modifications require admin access and cannot be performed by Copilot agent
