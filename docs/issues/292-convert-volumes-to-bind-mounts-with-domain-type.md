# [Refactor] Phase 0: Convert volumes to bind mounts with domain type

**Issue**: #292
**Parent Epic**: #287 - Docker Compose Topology Domain Model Refactoring
**Related**:

- [Refactoring Plan: Phase 0](../refactors/plans/docker-compose-topology-domain-model.md#phase-0-bind-mount-standardization-foundation)
- [ADR: Bind Mount Standardization](../decisions/bind-mount-standardization.md)
- Supersedes volume decisions in [ADR: Grafana Integration Pattern](../decisions/grafana-integration-pattern.md)

## Overview

This task converts all Docker named volumes to bind mounts and introduces a domain type (`BindMount`, `MountOption`) to represent volume mounts in a type-safe manner. This is a foundational change that simplifies the architecture by eliminating top-level volume declarations and provides observability into all persistent data via the `./storage/` directory.

## Goals

- [ ] Convert all named volumes to bind mounts under `./storage/{service}/`
- [ ] Remove top-level `volumes:` section from docker-compose template
- [ ] Create domain types (`BindMount`, `MountOption`) for type-safe mount representation
- [ ] Create Ansible playbooks to set correct directory ownership for Grafana and MySQL
- [ ] Update ADR status for grafana-integration-pattern.md (partially superseded)

## 🏗️ Architecture Requirements

**DDD Layer**: Domain (new types) + Infrastructure (template updates + Ansible playbooks)
**Module Path**:

- New: `src/domain/deployment/topology/volume.rs`
- Existing: `templates/docker-compose/docker-compose.yml.tera`
- New: `templates/ansible/playbooks/create-grafana-storage.yml`
- New: `templates/ansible/playbooks/create-mysql-storage.yml`

**Pattern**: Value Object (BindMount, MountOption)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Domain types go in `src/domain/deployment/topology/` (new module)
- [ ] Template changes stay in infrastructure layer
- [ ] Ansible playbooks are static files (no `.tera` extension)

### Architectural Constraints

- [ ] Domain types must be serializable for template context
- [ ] `MountOption` enum should be exhaustive (ReadOnly, SELinux)
- [ ] New Ansible playbooks must be registered in `copy_static_templates` method

### Anti-Patterns to Avoid

- ❌ Putting infrastructure logic in domain layer
- ❌ Forgetting to register static playbooks (causes "playbook not found" error)
- ❌ Using mixed volume types (named + bind mount)

## Specifications

### Current Named Volumes to Convert

| Service | Current Named Volume            | New Bind Mount                            | Host Path                |
| ------- | ------------------------------- | ----------------------------------------- | ------------------------ |
| Caddy   | `caddy_data:/data`              | `./storage/caddy/data:/data`              | `./storage/caddy/data`   |
| Caddy   | `caddy_config_vol:/config`      | `./storage/caddy/config:/config`          | `./storage/caddy/config` |
| Grafana | `grafana_data:/var/lib/grafana` | `./storage/grafana/data:/var/lib/grafana` | `./storage/grafana/data` |
| MySQL   | `mysql_data:/var/lib/mysql`     | `./storage/mysql/data:/var/lib/mysql`     | `./storage/mysql/data`   |

### Directory Ownership Requirements

| Service | Container User        | Required Host Ownership |
| ------- | --------------------- | ----------------------- |
| Grafana | `472:472` (grafana)   | `472:472`               |
| MySQL   | `999:999` (mysql)     | `999:999`               |
| Caddy   | `root`                | `ansible_user` (OK)     |
| Tracker | `1000:1000` (USER_ID) | `ansible_user` (OK)     |

### Domain Types

```rust
// src/domain/deployment/topology/volume.rs

/// Mount options for Docker bind mounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MountOption {
    /// Read-only mount (`:ro`) - used for config files
    ReadOnly,
    /// SELinux private relabeling (`:Z`) - used for writable data
    SELinux,
}

/// A Docker bind mount from host to container
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BindMount {
    host_path: String,
    container_path: String,
    option: Option<MountOption>,
}
```

### Ansible Playbooks

**create-grafana-storage.yml**:

```yaml
- name: Create Grafana data directory
  ansible.builtin.file:
    path: /opt/torrust/storage/grafana/data
    state: directory
    mode: "0755"
    owner: "472"
    group: "472"
  when: grafana_enabled | default(false)
```

**create-mysql-storage.yml**:

```yaml
- name: Create MySQL data directory
  ansible.builtin.file:
    path: /opt/torrust/storage/mysql/data
    state: directory
    mode: "0755"
    owner: "999"
    group: "999"
  when: mysql_enabled | default(false)
```

## Implementation Plan

### Phase 1: Template Changes (estimated: 1-2 hours)

- [ ] Update Caddy volumes in template: `caddy_data` → `./storage/caddy/data:/data`
- [ ] Update Caddy volumes in template: `caddy_config_vol` → `./storage/caddy/config:/config`
- [ ] Update Grafana volumes in template: `grafana_data` → `./storage/grafana/data:/var/lib/grafana`
- [ ] Update MySQL volumes in template: `mysql_data` → `./storage/mysql/data:/var/lib/mysql`
- [ ] Remove top-level `volumes:` section from template

### Phase 2: Ansible Playbooks (estimated: 1-2 hours)

- [ ] Create `templates/ansible/playbooks/create-grafana-storage.yml` (static file)
- [ ] Create `templates/ansible/playbooks/create-mysql-storage.yml` (static file)
- [ ] Register playbooks in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs` (`copy_static_templates` method)
- [ ] Update deployment step orchestration to call new playbooks before starting containers
- [ ] Update `templates/ansible/variables.yml.tera` if needed

### Phase 3: Domain Types (estimated: 2-3 hours)

- [ ] Create `src/domain/deployment/topology/mod.rs` module
- [ ] Create `src/domain/deployment/topology/volume.rs` with `MountOption` and `BindMount`
- [ ] Add unit tests for `MountOption::as_str()` and `BindMount` formatting
- [ ] Integrate domain types with template context (future use)

### Phase 4: Documentation & Cleanup (estimated: 30 min)

- [ ] Update `docs/decisions/grafana-integration-pattern.md` status to "Partially Superseded"
- [ ] Add reference to `bind-mount-standardization.md` ADR
- [ ] Update refactoring plan progress tracking

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Template Changes**:

- [ ] No named volumes remain in docker-compose template
- [ ] Top-level `volumes:` section is removed
- [ ] All bind mounts use `./storage/{service}/` pattern

**Ansible Playbooks**:

- [ ] Grafana storage playbook creates directory with owner `472:472`
- [ ] MySQL storage playbook creates directory with owner `999:999`
- [ ] Playbooks are registered in `copy_static_templates`

**Domain Types**:

- [ ] `MountOption` enum has `ReadOnly` and `SELinux` variants
- [ ] `BindMount` struct has `host_path`, `container_path`, `option` fields
- [ ] Unit tests pass for domain types

**E2E Verification**:

- [ ] E2E tests pass (infrastructure lifecycle + deployment workflow)
- [ ] Services start correctly with bind mounts
- [ ] TLS certificates persist across container restarts (Caddy)
- [ ] Grafana dashboards persist
- [ ] MySQL data persists

**Documentation**:

- [ ] `grafana-integration-pattern.md` status updated

## Related Documentation

- [Refactoring Plan: Docker Compose Topology Domain Model](../refactors/plans/docker-compose-topology-domain-model.md)
- [ADR: Bind Mount Standardization](../decisions/bind-mount-standardization.md)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Ansible Templates Guide](../contributing/templates/tera.md)
- [Module Organization](../contributing/module-organization.md)

## Notes

- The domain types (`BindMount`, `MountOption`) are created in this phase but may not be fully integrated into the template context until Phase 2 (Network Domain Types) when service configs are refactored
- This phase eliminates the need for volume derivation logic that would have been required in Phase 2
- Existing playbooks (`create-tracker-storage.yml`, `create-prometheus-storage.yml`, `deploy-caddy-config.yml`) already handle their respective directories correctly
