---
name: add-ansible-playbook
description: Guide for adding Ansible playbooks to this project. Covers the atomic playbook rule (one playbook = one responsibility), the correct pattern for conditional enablement (in Rust commands/steps, NOT Ansible when: clauses), registering static playbooks in copy_static_templates(), the centralized variables.yml.tera pattern, and a checklist before adding. Use when creating new Ansible playbooks, adding Ansible tasks, or modifying the Ansible infrastructure. Triggers on "ansible playbook", "new playbook", "ansible task", "playbook", "add ansible", "copy_static_templates", or "infrastructure playbook".
metadata:
  author: torrust
  version: "1.0"
---

# Adding Ansible Playbooks

## Atomic Playbook Rule

**One playbook = one responsibility.** Never bundle multiple unrelated tasks.

```yaml
# ✅ Good: single responsibility
# install-docker.yml — only installs Docker

# ❌ Bad: multiple concerns
# install-docker-and-configure-firewall.yml
```

Red flags:

- Playbook name contains "and"
- Multiple unrelated `ansible.builtin.file` / copy tasks bundled together

## Conditional Enablement Belongs in Rust

Use `when:` **only for Ansible host facts** (OS detection, etc.).
**Never use `when:` to decide if a feature/service is enabled.**

```yaml
# ✅ Good: host fact check
- when: ansible_os_family == "Debian"

# ❌ Bad: feature gating — this belongs in the Rust step
- when: enable_monitoring == true
```

The Rust command/step decides which playbooks to run. New feature → new atomic playbook + new Rust step that conditionally calls it.

## Registration in copy_static_templates()

Every new static playbook (`.yml`, not `.tera`) must be registered:

```text
src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs
```

In the `copy_static_templates()` function. Without this, the file is never copied to the build directory and Ansible won't find it.

## Centralized Variables Pattern

- `variables.yml.tera` — all runtime variables (rendered once by Tera)
- `inventory.yml.tera` — connection details
- Static playbooks load variables via `vars_files: [variables.yml]`

When adding a new playbook that needs a runtime value, add the variable to `variables.yml.tera`, not to the playbook itself.

```yaml
# my-new-playbook.yml (static)
---
- name: Configure X
  hosts: all
  vars_files:
    - variables.yml # ← always load centralized variables
  tasks:
    - name: Use variable
      ansible.builtin.template:
        src: "{{ my_variable }}" # ← from variables.yml
```

## Pre-Submit Checklist

- [ ] Does one thing only (no "and" in the name)
- [ ] No `when:` for feature/service enablement
- [ ] `vars_files: [variables.yml]` included
- [ ] Registered in `copy_static_templates()`
- [ ] Corresponding Rust step added that calls this playbook

## Reference

Guide: [`docs/contributing/templates/ansible.md`](../../docs/contributing/templates/ansible.md)
ADR: [`docs/decisions/atomic-ansible-playbooks.md`](../../docs/decisions/atomic-ansible-playbooks.md)
