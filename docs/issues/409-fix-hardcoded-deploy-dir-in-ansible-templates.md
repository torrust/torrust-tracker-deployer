# Fix Hardcoded Deployment Directory in Ansible Templates

**Issue**: #409
**Parent Epic**: None
**Related**: #405 - Deploy Hetzner Demo Tracker and Document the Process

## Overview

Several Ansible playbook templates under `templates/ansible/` hardcode the deployment directory as `/opt/torrust` instead of using the `{{ deploy_dir }}` variable sourced from `variables.yml`. This causes deployment failures when a user configures a custom deployment directory other than the default `/opt/torrust`.

The correct pattern — used by `create-grafana-storage.yml`, `create-mysql-storage.yml`, and `deploy-grafana-provisioning.yml` — is to load `variables.yml` via `vars_files` and reference `{{ deploy_dir }}` in all paths.

This bug was discovered while deploying the Torrust Tracker demo to the Hetzner provider (issue #405).

## Goals

- [ ] Replace all hardcoded `/opt/torrust` path occurrences in `templates/ansible/` task definitions with `{{ deploy_dir }}`
- [ ] Ensure all affected playbooks load `variables.yml` via `vars_files` where not already done
- [ ] Standardize the variable name (`deploy_dir`) across all playbooks — `deploy-compose-files.yml` currently uses a different name (`remote_deploy_dir`)
- [ ] Verify the fix works for both the default value `/opt/torrust` and a custom deployment directory

## Specifications

### Affected Templates

The following 10 templates need to be updated:

#### 1. Templates with fully hardcoded paths (no variable usage at all)

These templates use `/opt/torrust` directly in task `path:`, `dest:`, and loop items. None of them load `variables.yml`.

| Template                                          | Hardcoded occurrences              |
| ------------------------------------------------- | ---------------------------------- |
| `templates/ansible/create-tracker-storage.yml`    | 3 (loop items)                     |
| `templates/ansible/create-prometheus-storage.yml` | 1 (loop item)                      |
| `templates/ansible/create-backup-storage.yml`     | 2 (`path:` params)                 |
| `templates/ansible/deploy-backup-config.yml`      | 4 (`dest:` and `path:` params)     |
| `templates/ansible/deploy-tracker-config.yml`     | 2 (`dest:` and `path:` params)     |
| `templates/ansible/deploy-prometheus-config.yml`  | 2 (`dest:` and `path:` params)     |
| `templates/ansible/init-tracker-database.yml`     | 2 (`path:` params)                 |
| `templates/ansible/deploy-caddy-config.yml`       | 6 (loop items + `dest:` + `path:`) |

#### 2. Templates using a variable inline (not from `variables.yml`)

These templates define the deployment directory as an inline playbook variable, bypassing the user-configured value from `variables.yml`.

| Template                                     | Issue                                                                                                             |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `templates/ansible/run-compose-services.yml` | Defines `deploy_dir: /opt/torrust` inline — does not load from `variables.yml`                                    |
| `templates/ansible/deploy-compose-files.yml` | Defines `remote_deploy_dir: /opt/torrust` inline — different variable name AND does not load from `variables.yml` |

#### 3. Templates already correct (reference)

These correctly use `vars_files: variables.yml` and `{{ deploy_dir }}`:

- `templates/ansible/create-grafana-storage.yml` ✓
- `templates/ansible/create-mysql-storage.yml` ✓
- `templates/ansible/deploy-grafana-provisioning.yml` ✓

### Required Fix Pattern

Each affected playbook must be updated to follow the correct pattern:

```yaml
- name: <playbook name>
  hosts: all
  become: true
  vars_files:
    - variables.yml

  tasks:
    - name: <task name>
      ansible.builtin.file:
        path: "{{ deploy_dir }}/storage/..."
```

### Variable Naming

The variable must consistently be named `deploy_dir` across all playbooks, matching the name defined in `variables.yml.tera`:

```yaml
deploy_dir: /opt/torrust
```

The `deploy-compose-files.yml` template must be updated to rename `remote_deploy_dir` to `deploy_dir` for consistency.

## Implementation Plan

### Phase 1: Fix fully hardcoded templates

- [ ] `create-tracker-storage.yml`: Add `vars_files: variables.yml` and replace all 3 hardcoded loop items with `{{ deploy_dir }}/storage/...`
- [ ] `create-prometheus-storage.yml`: Add `vars_files: variables.yml` and replace the hardcoded loop item
- [ ] `create-backup-storage.yml`: Add `vars_files: variables.yml` and replace 2 hardcoded `path:` values
- [ ] `deploy-backup-config.yml`: Add `vars_files: variables.yml` and replace all 4 hardcoded `dest:`/`path:` values
- [ ] `deploy-tracker-config.yml`: Add `vars_files: variables.yml` and replace 2 hardcoded `dest:`/`path:` values
- [ ] `deploy-prometheus-config.yml`: Add `vars_files: variables.yml` and replace 2 hardcoded `dest:`/`path:` values
- [ ] `init-tracker-database.yml`: Add `vars_files: variables.yml` and replace 2 hardcoded `path:` values
- [ ] `deploy-caddy-config.yml`: Add `vars_files: variables.yml` and replace all 6 hardcoded occurrences

### Phase 2: Fix inline-variable templates

- [ ] `run-compose-services.yml`: Remove inline `deploy_dir: /opt/torrust` from `vars` and add `vars_files: variables.yml` instead
- [ ] `deploy-compose-files.yml`: Remove inline `remote_deploy_dir: /opt/torrust` from `vars`, add `vars_files: variables.yml`, and rename all `remote_deploy_dir` references to `deploy_dir`

### Phase 3: Update comments in affected templates

- [ ] Update inline comments in all modified templates to reference `{{ deploy_dir }}` instead of `/opt/torrust` as the example path (where applicable)
- [ ] Run linters: `cargo run --bin linter all`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] No playbook task in `templates/ansible/` uses a hardcoded `/opt/torrust` path in `path:`, `dest:`, or loop items
- [ ] All playbooks that reference the deployment directory load `variables.yml` via `vars_files`
- [ ] The variable name `deploy_dir` is used consistently (no `remote_deploy_dir` or other aliases)
- [ ] Playbooks that previously had inline `vars:` blocks for the deploy directory no longer define it inline
- [ ] The default behavior (with `deploy_dir: /opt/torrust`) is unchanged

## Related Documentation

- [docs/issues/409-fix-hardcoded-deploy-dir-in-ansible-templates.md](409-fix-hardcoded-deploy-dir-in-ansible-templates.md) — this specification
- [templates/ansible/variables.yml.tera](../../templates/ansible/variables.yml.tera) — defines `deploy_dir`
- [docs/issues/405-deploy-hetzner-demo-tracker-and-document-process.md](405-deploy-hetzner-demo-tracker-and-document-process.md) — parent issue where this bug was discovered
