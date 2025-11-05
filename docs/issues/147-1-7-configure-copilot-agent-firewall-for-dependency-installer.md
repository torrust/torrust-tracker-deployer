# Configure Copilot Agent Firewall for Dependency Installer

**Issue**: [#147](https://github.com/torrust/torrust-tracker-deployer/issues/147)
**Parent Epic**: [#112 - Refactor and Improve E2E Test Execution](https://github.com/torrust/torrust-tracker-deployer/issues/112)
**Related**: [#146 - Update Pre-Commit Script for GitHub Runner-Compatible E2E Tests](https://github.com/torrust/torrust-tracker-deployer/issues/146)

## Overview

Configure GitHub Copilot agent's firewall to allow network access to domains required by the dependency installer binaries. The Copilot agent environment has a restricted firewall that blocks access to external domains by default. This task involves identifying all required domains and configuring repository settings to whitelist them.

## Problem Statement

When GitHub Copilot agent attempts to install dependencies using the `dependency-installer` binary, network requests are blocked by the agent's firewall:

```bash
$ cargo run -p torrust-dependency-installer --bin dependency-installer -- install --dependency opentofu
2025-11-05T19:46:23.668278Z ERROR torrust_dependency_installer::app: Command failed error=Install command failed: Failed to install specific dependency: Installation failed: Failed to install dependency 'opentofu': Failed to download installer: curl: (6) Could not resolve host: get.opentofu.org
```

This prevents the agent from:

- Installing OpenTofu via the installer script
- Running pre-commit checks that depend on installed tools
- Executing E2E tests that require infrastructure dependencies

## Goals

- [ ] Identify all domains required by dependency installers
- [ ] Configure Copilot agent firewall allowlist in repository settings
- [ ] Document firewall configuration for future maintainers
- [ ] Verify that dependency installation works in Copilot agent environment

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (configuration, not code changes)
**Module Path**: N/A (repository settings configuration)
**Pattern**: Repository Configuration

### Configuration Requirements

- [ ] Repository settings must be configured by repository admin
- [ ] Firewall rules should be minimal (only required domains)
- [ ] Configuration should be documented for reproducibility

### Anti-Patterns to Avoid

- ❌ Disabling the firewall entirely (increases security risks)
- ❌ Whitelisting overly broad domains
- ❌ Not documenting why each domain is needed

## Specifications

### Required Domains Analysis

Based on analysis of `packages/dependency-installer/src/installer/` modules:

#### OpenTofu Installer

**File**: `packages/dependency-installer/src/installer/opentofu.rs`

```rust
// Downloads installer script from:
"https://get.opentofu.org/install-opentofu.sh"
```

**Required Domain**: `opentofu.org`

- **Why**: Downloads OpenTofu installer script and packages
- **Subdomain Coverage**: Using `opentofu.org` allows both `get.opentofu.org` and any other subdomains the installer script may use for package downloads

#### Ansible Installer

**File**: `packages/dependency-installer/src/installer/ansible.rs`

```rust
// Uses system package manager:
sudo apt-get install -y ansible
```

**Required Domain**: None (covered by recommended allowlist)

- **Why**: Ubuntu package repositories are included in the default "recommended allowlist"

#### cargo-machete Installer

**File**: `packages/dependency-installer/src/installer/cargo_machete.rs`

```rust
// Uses Rust package registry:
cargo install cargo-machete
```

**Required Domain**: None (covered by recommended allowlist)

- **Why**: Rust package registry (crates.io) is included in the default "recommended allowlist"

#### LXD Installer

**File**: `packages/dependency-installer/src/installer/lxd.rs`

```rust
// Uses snap package manager:
sudo snap install lxd
```

**Required Domain**: None (covered by recommended allowlist)

- **Why**: Snap store is included in the default "recommended allowlist"

### Firewall Configuration Summary

**Domains to Whitelist**:

1. `opentofu.org` - Required for OpenTofu installation

**Domains Already Covered**:

- Ubuntu/Debian package repositories (apt)
- Rust package registry (crates.io)
- Snap store

## Implementation Plan

### Phase 1: Repository Settings Configuration (15-30 minutes)

**Prerequisites**:

- Repository admin access required
- Must be logged into GitHub

**Steps**:

- [ ] Navigate to repository settings: `https://github.com/torrust/torrust-tracker-deployer/settings`
- [ ] In the "Code & automation" section, click **Copilot** → **coding agent**
- [ ] Verify **Enable firewall** is toggled ON
- [ ] Verify **Recommended allowlist** is toggled ON (default)
- [ ] Click **Custom allowlist**
- [ ] Add domain: `opentofu.org`
  - This allows traffic to `opentofu.org` and all subdomains (e.g., `get.opentofu.org`)
- [ ] Click **Add Rule**
- [ ] Click **Save changes**

### Phase 2: Documentation (15-30 minutes)

- [ ] Create new document: `docs/contributing/copilot-agent-firewall.md`
- [ ] Document configured domains and their purposes
- [ ] Document configuration steps for future reference
- [ ] Link to GitHub documentation on firewall customization
- [ ] Update related documentation:
  - [ ] Add reference in `docs/contributing/roadmap-issues.md` if relevant
  - [ ] Add reference in `packages/dependency-installer/README.md`

### Phase 3: Verification (15-30 minutes)

- [ ] Trigger a Copilot agent workflow that uses dependency-installer
- [ ] Verify OpenTofu installation succeeds
- [ ] Check for any new firewall warnings in agent logs
- [ ] Update documentation if additional domains are needed

**Total Estimated Time**: 45 minutes - 1.5 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Configuration Checks**:

- [ ] Repository firewall settings show `opentofu.org` in custom allowlist
- [ ] Recommended allowlist remains enabled
- [ ] Firewall remains enabled (not disabled)

**Documentation Checks**:

- [ ] New document `docs/contributing/copilot-agent-firewall.md` exists
- [ ] Document includes all configured domains with rationale
- [ ] Document includes step-by-step configuration instructions
- [ ] Links to official GitHub documentation included

**Verification Checks**:

- [ ] Copilot agent can successfully run: `cargo run --bin dependency-installer install --dependency opentofu`
- [ ] No firewall warnings appear for configured domains
- [ ] Pre-commit checks pass after configuration

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh` (for documentation changes only)

## Related Documentation

### GitHub Documentation

- [GitHub Docs: Customizing the agent firewall](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-firewall)
- [GitHub Docs: Preinstalling tools in Copilot's environment](https://docs.github.com/en/copilot/customizing-copilot/customizing-the-development-environment-for-copilot-coding-agent#preinstalling-tools-in-copilots-environment)

### Project Documentation

- [Dependency Installer Package](../../packages/dependency-installer/README.md)
- [E2E Testing Guide](../e2e-testing.md)
- [Issue #146 - Update Pre-Commit Script](./146-1-6-update-precommit-script-for-github-runner-compatible-e2e-tests.md)

### Firewall Documentation

The GitHub Copilot agent firewall has the following characteristics:

- **Default Policy**: Blocks all external network access except GitHub hosts
- **Recommended Allowlist**: Pre-configured list of common package repositories, container registries, and certificate authorities
- **Custom Allowlist**: Repository-specific additions for domains not covered by recommended list
- **Domain vs URL Rules**:
  - **Domain** (e.g., `opentofu.org`): Allows traffic to domain and all subdomains
  - **URL** (e.g., `https://get.opentofu.org/installer/`): Only allows specified scheme, host, and path

### Limitations

From GitHub documentation:

- Only applies to processes started by the agent via its Bash tool
- Does not apply to Model Context Protocol (MCP) servers
- Does not apply to processes started in configured Copilot setup steps
- Sophisticated attacks may bypass the firewall
- Only operates within GitHub Actions appliance environment

## Notes

### Why This Issue Cannot Be Implemented by Copilot Agent

This issue requires **repository admin access** to modify repository settings. GitHub Copilot agents do not have permission to:

- Access repository settings pages
- Modify firewall configuration
- Change Copilot agent settings

Therefore, this must be implemented manually by a repository administrator (user with admin role).

### Security Considerations

The recommended approach is to:

1. ✅ Keep firewall enabled
2. ✅ Keep recommended allowlist enabled
3. ✅ Only add specific domains needed (minimal whitelist)
4. ❌ Avoid disabling the firewall entirely

This balances functionality with security, minimizing data exfiltration risks while allowing necessary tool installations.

### Future Maintenance

When adding new dependency installers:

1. Check if the installer downloads from external hosts
2. Test in Copilot agent environment first
3. If network access is blocked, update firewall configuration
4. Document the new domain in `docs/contributing/copilot-agent-firewall.md`
5. Update this issue specification with new domains
