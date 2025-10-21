# Refactor Ansible Templates to Variables Pattern

**Issue**: #19
**Parent Epic**: #16 - Finish ConfigureCommand - System Security Configuration
**Depends On**: #18 - Configure UFW Firewall
**Related**: [Parent Epic](./16-epic-finish-configure-command-system-security.md), [Template System Architecture](../technical/template-system-architecture.md)

## Overview

Refactor Ansible templates to use a centralized variables pattern similar to OpenTofu's `variables.tfvars.tera` approach. This consolidates multiple Tera templates into a single variables file, reducing complexity and establishing a consistent pattern for future Ansible template additions.

After implementing the security updates and firewall configuration, we now have 2 Tera templates (`inventory.yml.tera` and `configure-firewall.yml.tera`). This refactoring consolidates them into a single variables-based approach that will simplify future service additions.

## Goals

- [ ] **Centralized Variables**: Create single `variables.yml.tera` for all Ansible variables
- [ ] **Template Consolidation**: Reduce from 2 Tera templates to 1 Tera template
- [ ] **Consistent Pattern**: Match OpenTofu's elegant variables approach
- [ ] **Future-Proofing**: Establish pattern for easy addition of new services
- [ ] **Reduced Complexity**: Minimize Rust template handling boilerplate
- [ ] **Maintain Functionality**: Ensure all existing functionality continues to work

## Specifications

### Current State Analysis

**Before Refactoring**:

- `templates/ansible/inventory.yml.tera` (variables: ansible_host, ansible_port, ansible_ssh_private_key_file)
- `templates/ansible/configure-firewall.yml.tera` (variables: ssh_port)
- Static templates: `install-docker.yml`, `install-docker-compose.yml`, `configure-security-updates.yml`, etc.

**After Refactoring**:

- `templates/ansible/variables.yml.tera` (all variables centralized)
- `templates/ansible/inventory.yml` (static, references variables.yml)
- `templates/ansible/configure-firewall.yml` (static, uses vars_files)
- All other templates remain static

### New Centralized Variables Template

Create `templates/ansible/variables.yml.tera`:

```yaml
---
# Centralized Ansible Variables
# This file contains all dynamic variables used across Ansible playbooks.
# It follows the same pattern as OpenTofu's variables.tfvars.tera for consistency.

# Connection Configuration
ansible_host: { { ansible_host } }
ansible_port: { { ansible_port } }
ansible_ssh_private_key_file: { { ansible_ssh_private_key_file } }

# System Configuration
ssh_port: { { ssh_port } }
# Future service variables can be added here:
# mysql_port: {{ mysql_port }}
# tracker_port: {{ tracker_port }}
# prometheus_port: {{ prometheus_port }}
# grafana_port: {{ grafana_port }}
```

### Refactored Inventory Template

Convert `templates/ansible/inventory.yml.tera` → `templates/ansible/inventory.yml`:

```yaml
# Ansible Inventory File (YAML format)
# This file defines the hosts that Ansible will manage and how to connect to them.
# Variables are loaded from variables.yml for centralized management.

all:
  hosts:
    torrust-tracker-vm:
      # Variables loaded from variables.yml via vars_files
      ansible_host: "{{ ansible_host }}"
      ansible_port: "{{ ansible_port }}"
      ansible_user: torrust
      ansible_connection: ssh
      ansible_ssh_private_key_file: "{{ ansible_ssh_private_key_file }}"
      ansible_ssh_common_args: "-o StrictHostKeyChecking=no"

  vars:
    ansible_python_interpreter: /usr/bin/python3
```

### Refactored Firewall Template

Convert `templates/ansible/configure-firewall.yml.tera` → `templates/ansible/configure-firewall.yml`:

```yaml
---
# Configure UFW Firewall with Safe SSH Access
# Variables are loaded from variables.yml for centralized management.

- name: Configure UFW firewall safely
  hosts: torrust_servers
  become: yes
  gather_facts: yes
  vars_files:
    - variables.yml

  tasks:
    - name: Install UFW
      ansible.builtin.apt:
        name: ufw
        state: present
        update_cache: yes
      tags: [security, firewall, packages]

    - name: Reset UFW to clean state
      community.general.ufw:
        state: reset
      tags: [security, firewall, reset]

    - name: Set UFW default policies
      community.general.ufw:
        default: "{{ item.policy }}"
        direction: "{{ item.direction }}"
      loop:
        - { policy: deny, direction: incoming }
        - { policy: allow, direction: outgoing }
      tags: [security, firewall, policy]

    # CRITICAL: Allow SSH BEFORE enabling firewall
    - name: Allow SSH access on configured port
      community.general.ufw:
        rule: allow
        port: "{{ ssh_port }}"
        proto: tcp
        comment: "SSH access (port {{ ssh_port }})"
      tags: [security, firewall, ssh]

    - name: Allow SSH service by name (additional safety)
      community.general.ufw:
        rule: allow
        name: ssh
        comment: "SSH service (standard)"
      tags: [security, firewall, ssh]

    - name: Enable UFW firewall
      community.general.ufw:
        state: enabled
      tags: [security, firewall, enable]

    - name: Verify UFW status and SSH access
      ansible.builtin.shell:
        cmd: "ufw status | grep -E '{{ ssh_port }}/tcp.*ALLOW'"
      register: ssh_port_check
      changed_when: false
      failed_when: ssh_port_check.rc != 0
      tags: [security, firewall, verification]

    - name: Display firewall configuration complete
      ansible.builtin.debug:
        msg:
          - "UFW firewall configured successfully"
          - "SSH access preserved on port {{ ssh_port }}"
          - "Default policy: deny incoming, allow outgoing"
      tags: [security, firewall]
```

### Template Rendering Updates

Update Ansible template rendering logic to:

1. **Render variables.yml.tera** → `build/{env}/ansible/variables.yml`
2. **Copy static templates** as-is to `build/{env}/ansible/`
3. **Remove** rendering of `inventory.yml.tera` and `configure-firewall.yml.tera`

### Ansible Execution Updates

Update Ansible execution to ensure `variables.yml` is available:

```rust
// In AnsibleClient implementation
impl AnsibleClient {
    pub fn run_playbook(&self, playbook_name: &str) -> Result<(), CommandError> {
        // Ensure variables.yml exists in the same directory as playbooks
        let variables_path = self.build_dir.join("variables.yml");
        if !variables_path.exists() {
            return Err(CommandError::ConfigurationError {
                message: "variables.yml not found - template rendering may have failed".to_string()
            });
        }

        // Execute playbook (vars_files automatically loads variables.yml)
        // ... existing execution logic
    }
}
```

## Implementation Plan

This task should be implemented as a **single PR** with the following subtasks:

### Subtask 1: Create Variables Template (0.5 days)

- [ ] **Create variables template**: Implement `templates/ansible/variables.yml.tera` with all current variables
- [ ] **Template integration**: Ensure variables template is properly embedded and rendered
- [ ] **Variable extraction**: Identify all variables from existing Tera templates
- [ ] **Test variables rendering**: Verify variables.yml is properly generated

### Subtask 2: Refactor Inventory Template (0.5 days)

- [ ] **Convert to static**: Change `inventory.yml.tera` → `inventory.yml`
- [ ] **Add vars_files reference**: Update inventory to use variables from variables.yml
- [ ] **Test inventory**: Verify inventory still works with centralized variables
- [ ] **Update template handling**: Remove inventory.yml.tera from Tera processing

### Subtask 3: Refactor Firewall Template (0.5 days)

- [ ] **Convert to static**: Change `configure-firewall.yml.tera` → `configure-firewall.yml`
- [ ] **Add vars_files**: Update playbook to load variables from variables.yml
- [ ] **Update variable references**: Ensure all `{{ ssh_port }}` references work with vars_files
- [ ] **Test firewall playbook**: Verify firewall configuration still works

### Subtask 4: Update Template Rendering System (0.5 days)

- [ ] **Update template renderer**: Modify Ansible renderer to handle new pattern
- [ ] **Remove old Tera templates**: Stop rendering inventory.yml.tera and configure-firewall.yml.tera
- [ ] **Add variables rendering**: Ensure variables.yml.tera is processed
- [ ] **Update error handling**: Add checks for variables.yml existence

### Subtask 5: Integration Testing and Validation (0.5 days)

- [ ] **E2E test validation**: Run full E2E tests to ensure refactoring doesn't break functionality
- [ ] **Manual verification**: Test both inventory connection and firewall configuration
- [ ] **Template verification**: Confirm only variables.yml.tera requires Tera processing
- [ ] **Documentation updates**: Update any relevant documentation about template patterns

## Acceptance Criteria

- [ ] **Single Variables Template**: Only `variables.yml.tera` requires Tera processing
- [ ] **Static Templates**: `inventory.yml` and `configure-firewall.yml` are static files
- [ ] **Functionality Preserved**: All existing Ansible functionality continues to work
- [ ] **Variables Centralized**: All dynamic values are defined in variables.yml
- [ ] **Pattern Established**: Clear pattern for adding future service variables
- [ ] **Tests Pass**: All existing tests continue to pass
- [ ] **E2E Validation**: Full deployment workflow works with refactored templates
- [ ] **Reduced Complexity**: Fewer Tera templates to maintain and debug

## Related Documentation

- [Template System Architecture](../technical/template-system-architecture.md)
- [OpenTofu Variables Pattern](../../templates/tofu/lxd/variables.tfvars.tera)
- [Ansible Variables Documentation](https://docs.ansible.com/ansible/latest/playbook_guide/playbooks_variables.html)
- [Ansible vars_files Documentation](https://docs.ansible.com/ansible/latest/playbook_guide/playbooks_variables.html#defining-variables-in-files)
- [Parent Epic](./16-epic-finish-configure-command-system-security.md)

## Benefits

### **Architectural Consistency**

- Matches OpenTofu's successful `variables.tfvars.tera` pattern
- Consistent approach to variable management across infrastructure tools
- Single source of truth for environment-specific values

### **Reduced Complexity**

- Only 1 Tera template instead of multiple
- Less Rust boilerplate for template handling
- Simpler debugging and maintenance

### **Future-Proofing**

- Easy pattern for adding new services (just add variables, write static playbook)
- Scalable approach for the full roadmap implementation
- Clear separation of concerns (variables vs. logic)

### **Developer Experience**

- Easier to understand variable flow
- Centralized variable management
- Reduced cognitive overhead when adding new features

## Migration Strategy

### **Risk Mitigation**

- Incremental approach (one template at a time)
- Extensive testing at each step
- Preserve existing functionality throughout migration
- Clear rollback path if issues arise

### **Validation Strategy**

- Test each refactored template individually
- Run full E2E tests after each subtask
- Manual verification of critical functionality (SSH access, firewall rules)
- Compare before/after behavior for consistency

## Notes

### **Timing Rationale**

This refactoring is perfectly timed because:

1. **Clear Justification**: We now have 2 Tera templates to consolidate
2. **Proven Pattern**: OpenTofu variables approach has been successful
3. **Future Value**: Establishes pattern before adding more services in roadmap 3.2+
4. **Natural Evolution**: Organic growth from 1→2→1 templates shows the pattern emerging

### **Implementation Order**

This should be the **final issue** in Epic 3.1 because:

- Security updates and firewall provide the business value
- Refactoring is architectural improvement/debt reduction
- Creates clean foundation for future Epic 3.2 work
- Demonstrates the pattern with real complexity

### **Success Metrics**

- Template count: 2 Tera templates → 1 Tera template ✅
- Functionality: All existing features continue to work ✅
- Maintainability: Easier to add future service variables ✅
- Consistency: Matches OpenTofu architectural pattern ✅
