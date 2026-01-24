# Draft Issue: Add Explicit Rule for Atomic Ansible Playbooks

## Problem

During implementation of issue #292, the contributor (AI agent) initially added storage creation tasks directly into existing playbooks (`deploy-compose-files.yml` and `deploy-grafana-provisioning.yml`), violating the Single Responsibility Principle.

The AGENTS.md has 20+ rules, but the atomic playbook principle was implicit rather than explicit. Rule #8 explains _how_ to register playbooks but doesn't state that each playbook should do exactly one thing.

### Root Causes

1. **Rule Density**: Too many rules to remember, easy to miss implicit conventions
2. **Pattern Recognition Failure**: Contributor saw existing playbooks and assumed adding tasks was acceptable
3. **Implicit vs Explicit**: The architecture principle wasn't stated as a clear rule

## Example of the Mistake

### Incorrect Approach (What Was Done Initially)

```yaml
# deploy-compose-files.yml - WRONG: Adding unrelated task
- name: Deploy Docker Compose files
  hosts: all
  become: true
  vars_files:
    - variables.yml

  tasks:
    - name: Create MySQL storage directory # WRONG - Different responsibility!
      ansible.builtin.file:
        path: "{{ deploy_dir }}/storage/mysql/data"
        state: directory
        owner: "999"
        group: "999"
      when: mysql_enabled | default(false)

    - name: Copy docker-compose files # Original task
      # ...
```

### Correct Approach (After User Correction)

```yaml
# create-mysql-storage.yml - RIGHT: Atomic playbook with single responsibility
- name: Create MySQL storage directory
  hosts: all
  become: true
  vars_files:
    - variables.yml

  tasks:
    - name: Create MySQL data directory with correct ownership
      ansible.builtin.file:
        path: "{{ deploy_dir }}/storage/mysql/data"
        state: directory
        mode: "0755"
        owner: "999"
        group: "999"
      when: mysql_enabled | default(false)
```

With conditional execution in Rust:

```rust
// In release handler - conditional check happens in Rust, not Ansible
fn create_mysql_storage(environment: &Environment<Releasing>, ...) {
    // Check if MySQL is configured (via tracker database driver)
    if !environment.context().user_inputs.tracker().uses_mysql() {
        info!("MySQL not configured - skipping storage creation");
        return Ok(());
    }

    // Run the atomic playbook
    CreateMysqlStorageStep::new(ansible_client).execute()
}
```

## Proposed Solution

### 1. Update Rule #8 in AGENTS.md

Change from:

```markdown
8. **When adding new Ansible playbooks**: Read `docs/contributing/templates/tera.md`...
```

To:

```markdown
8. **When adding new Ansible playbooks**: Read `docs/contributing/templates/ansible.md` for the complete guide.
   - **CRITICAL: One playbook = One responsibility** (Single Responsibility Principle)
   - Each playbook should perform exactly ONE conceptual operation
   - Conditional logic for service enablement belongs in Rust code, NOT in playbooks with `when:` clauses
   - Static playbooks must be registered in `copy_static_templates()`
```

### 2. Add "Red Flags" Section to Ansible Guide

In `docs/contributing/templates/ansible.md`:

```markdown
## Red Flags (Stop and Reconsider)

If you find yourself doing any of these, STOP and reconsider:

- **Adding tasks to an existing playbook** → Create a new atomic playbook instead
- **Using `when:` for service enablement** → Move the condition to Rust code
- **Playbook names with "and"** (e.g., "create-storage-and-deploy-config.yml") → Split into separate playbooks
- **Multiple unrelated `ansible.builtin.file` tasks** → Each belongs in its own playbook

**Correct pattern**: New feature = New atomic playbook + New Rust step with conditional check
```

### 3. Create ADR for Atomic Playbook Architecture

Create `docs/decisions/atomic-ansible-playbooks.md` explaining:

- Why we use atomic playbooks (testability, composability, clear failure points)
- How conditional execution works (Rust checks service enablement, then calls atomic playbook)
- The three-level architecture (Command → Step → Ansible playbook)

### 4. Pre-Implementation Checklist

Add to ansible guide:

```markdown
## Before Adding Ansible Functionality Checklist

- [ ] Am I adding tasks to an existing playbook? → **Create new atomic playbook**
- [ ] Does my playbook do more than one conceptual thing? → **Split it**
- [ ] Am I using `when:` for "should this service run"? → **Move to Rust conditional**
- [ ] Have I created a corresponding Rust step file?
- [ ] Have I registered the playbook in `copy_static_templates()`?
```

## Files to Modify

1. `AGENTS.md` - Update Rule #8
2. `docs/contributing/templates/ansible.md` - Add Red Flags section and checklist
3. Create `docs/decisions/atomic-ansible-playbooks.md` - ADR

## Related

- Issue #292 where this problem was discovered
- Existing pattern: `create-prometheus-storage.yml`, `create-tracker-storage.yml`
