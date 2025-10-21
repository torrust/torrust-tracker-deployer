# Finish ConfigureCommand - System Security Configuration

**Issue**: #16
**Parent Epic**: N/A (This is the main epic for roadmap task 3.1)
**Roadmap**: Task 3.1 - Finish ConfigureCommand
**Related**: [Roadmap](../roadmap.md), [ConfigureCommand](../../src/application/commands/configure.rs)

## Overview

This Epic completes the `ConfigureCommand` implementation by adding system security configuration capabilities. After establishing the foundational architecture and Docker installation in previous work, this Epic focuses specifically on **system-level security hardening** through three key areas:

1. **Automatic Security Updates** - Configure `unattended-upgrades` for automated security patch management
2. **UFW Firewall Configuration** - Implement network-level security with safe SSH access preservation
3. **Template Architecture Refinement** - Consolidate Ansible templates into a centralized variables pattern

This Epic provides **immediate production value** by securing deployed instances with industry-standard security practices, while also establishing a clean architectural foundation for future service additions.

## Goals

- [ ] **Security Updates**: Configure automatic security updates with scheduled reboots
- [ ] **Firewall Protection**: Setup UFW firewall with safe SSH access rules
- [ ] **Incremental Delivery**: Implement in phases for faster value delivery and easier review
- [ ] **Domain Integration**: Extend `ConfigureStep` enum and error handling
- [ ] **Ansible Integration**: Create new Ansible playbooks for system configuration
- [ ] **Production Ready**: Ensure configurations match production security standards

## Architecture Impact

### Domain Changes Required

The `ConfigureStep` enum in `src/domain/environment/state/configure_failed.rs` needs to be extended:

```rust
/// Steps in the configure workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureStep {
    /// Installing Docker
    InstallDocker,
    /// Installing Docker Compose
    InstallDockerCompose,
    /// Configuring automatic security updates
    ConfigureSecurityUpdates,
    /// Configuring UFW firewall
    ConfigureFirewall,
}
```

### New Application Steps

Two new step implementations required:

- `src/application/steps/system/configure_security_updates.rs`
- `src/application/steps/system/configure_firewall.rs`

### New Ansible Playbooks

Two new Ansible playbooks required:

- `templates/ansible/configure-security-updates.yml` (static)
- `templates/ansible/configure-firewall.yml.tera` (dynamic - requires SSH port resolution)

## Implementation Strategy

### Phase 1: Automatic Security Updates (Child Task #17)

**Estimated Time**: 2-3 days

Lower risk implementation that configures unattended-upgrades for automatic security patching:

- Install and configure unattended-upgrades package
- Enable automatic reboots at 2:00 AM for security updates
- Configure update notifications and logging

### Phase 2: UFW Firewall Configuration (Child Task #18)

**Estimated Time**: 2-3 days

Higher risk implementation requiring careful SSH access handling:

- Reset UFW to clean state
- Configure restrictive default policies (deny incoming, allow outgoing)
- **CRITICALLY**: Allow SSH access BEFORE enabling firewall
- Use configurable SSH port from `user_inputs.ssh_port`

### Phase 3: Template Architecture Refinement (Child Task #19)

**Estimated Time**: 1-2 days

Architectural cleanup and consistency improvement:

- Consolidate 2 Tera templates into 1 centralized variables pattern
- Create `variables.yml.tera` matching OpenTofu's approach
- Convert `inventory.yml.tera` and `configure-firewall.yml.tera` to static files
- Establish consistent pattern for future service additions

## Sub-Tasks

This Epic is broken down into **3 Sub-Tasks** that can be implemented incrementally:

### Task #17: Configure Automatic Security Updates (Lower Risk)

**Type**: Implementation  
**Priority**: High  
**Risk**: Low  
**Estimated Effort**: 1-2 days

Configure automatic security updates using `unattended-upgrades` to ensure critical security patches are applied automatically.

- **Rationale**: Lower risk implementation without networking/firewall concerns
- **Technical Approach**: Static Ansible playbook (no Tera template needed)
- **Testing Strategy**: Verify package installation and configuration validation
- **Business Value**: Automated security patch management

See: [Configure Automatic Security Updates](./17-configure-automatic-security-updates.md)

### Task #18: Configure UFW Firewall (Higher Risk)

**Type**: Implementation  
**Priority**: High  
**Risk**: Medium  
**Estimated Effort**: 2-3 days

Configure UFW firewall with proper SSH access preservation to secure network access while preventing SSH lockout.

- **Rationale**: Higher risk due to potential SSH lockout scenarios
- **Technical Approach**: Tera template required for SSH port resolution
- **Testing Strategy**: Comprehensive E2E testing with SSH connectivity validation
- **Business Value**: Network security hardening

See: [Configure UFW Firewall](./18-configure-ufw-firewall.md)

### Task #19: Refactor Ansible Templates to Variables Pattern

**Type**: Refactoring  
**Priority**: Medium  
**Risk**: Low  
**Estimated Effort**: 1-2 days

Refactor Ansible templates from multiple Tera files to centralized variables pattern, matching OpenTofu's elegant approach.

- **Rationale**: With 2 Tera templates now in use, establish consistent variables pattern
- **Technical Approach**: Create single `variables.yml.tera`, convert other templates to static
- **Testing Strategy**: Ensure no functionality regression through comprehensive E2E testing
- **Business Value**: Reduced complexity, easier maintenance, consistent architecture

See: [Refactor Ansible Templates to Variables Pattern](./19-refactor-ansible-templates-variables-pattern.md)

## Acceptance Criteria

- [ ] **Security Updates**: Instances automatically install security updates and reboot when needed
- [ ] **Firewall Active**: UFW firewall is enabled with restrictive default policies
- [ ] **SSH Access Maintained**: SSH access continues to work on configured port
- [ ] **Domain Integration**: New steps properly integrated into `ConfigureStep` enum
- [ ] **Template Consistency**: Ansible templates use centralized variables pattern matching OpenTofu
- [ ] **Reduced Complexity**: Only one Tera template (`variables.yml.tera`) needs variable processing
- [ ] **Error Handling**: Comprehensive error handling with actionable messages
- [ ] **Tests Pass**: All existing tests continue to pass
- [ ] **E2E Validation**: Full E2E tests validate the new configurations
- [ ] **Backward Compatibility**: Existing functionality remains unchanged

## Related Documentation

- [ConfigureCommand Implementation](../../src/application/commands/configure.rs)
- [Configure Automatic Security Updates](./17-configure-automatic-security-updates.md)
- [Configure UFW Firewall](./18-configure-ufw-firewall.md)
- [Refactor Ansible Templates to Variables Pattern](./19-refactor-ansible-templates-variables-pattern.md)
- [Original Bash PoC Cloud-init](https://github.com/torrust/torrust-tracker-deploy-bash-poc/blob/main/infrastructure/cloud-init/user-data.yaml.tpl)
- [Development Principles](../development-principles.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Roadmap](../roadmap.md)

## Technical Notes

### SSH Port Configuration

The SSH port is configurable via `UserInputs.ssh_port` but currently defaults to 22:

- Configured in `src/domain/environment/user_inputs.rs`
- Hardcoded to 22 in `src/bin/e2e_tests_full.rs` (line 145)
- Ansible templates already support variables (`ansible_port`)
- Future customization possible by changing the default and SSH daemon configuration

### Original PoC Reference

The bash PoC implemented these features in cloud-init:

**Security Updates:**

```bash
# Configure automatic security updates
- apt-get install -y unattended-upgrades
- echo 'Unattended-Upgrade::Automatic-Reboot "true";' >> /etc/apt/apt.conf.d/50unattended-upgrades
- echo 'Unattended-Upgrade::Automatic-Reboot-Time "02:00";' >> /etc/apt/apt.conf.d/50unattended-upgrades
- systemctl enable unattended-upgrades
- systemctl start unattended-upgrades
```

**UFW Firewall:**

```bash
# CRITICAL: Configure UFW firewall SAFELY (allow SSH BEFORE enabling)
- ufw --force reset
- ufw default deny incoming
- ufw default allow outgoing
- ufw allow ssh
- ufw allow 22/tcp
- ufw --force enable
```

### Implementation Order Rationale

1. **Security Updates First**: Lower risk, immediate security value, easier to test
2. **Firewall Second**: Higher risk due to potential SSH lockout, requires careful validation

This approach follows lean principles by delivering security value incrementally and reducing implementation risk through smaller, focused changes.
