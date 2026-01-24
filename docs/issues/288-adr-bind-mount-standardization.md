# [ADR] Bind Mount Standardization for Docker Compose

**Issue**: [#288](https://github.com/torrust/torrust-tracker-deployer/issues/288)
**Parent Epic**: [#287](https://github.com/torrust/torrust-tracker-deployer/issues/287) - Docker Compose Topology Domain Model Refactoring
**Related**: [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md)

## Overview

This task creates an Architectural Decision Record (ADR) documenting the decision to use bind mounts exclusively for all Docker Compose volume mounts, replacing the current mix of named volumes and bind mounts.

This ADR must be created **before** implementing the bind mount changes (Phase 0) to establish the rationale and ensure alignment.

## Goals

- [ ] Document the architectural decision to standardize on bind mounts
- [ ] Capture all 9 reasons for the decision
- [ ] Document alternatives considered and why they were rejected
- [ ] Register the ADR in the decisions index

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (documentation only)
**Module Path**: `docs/decisions/`
**Pattern**: Architectural Decision Record (ADR)

### Module Structure Requirements

- [x] Follow ADR conventions (see [docs/decisions/README.md](../decisions/README.md))
- [x] Use ADR template structure (Title, Status, Context, Decision, Consequences)
- [x] Register in ADR index

### Architectural Constraints

- [x] ADR must be created before implementation begins
- [x] ADR must reference the refactoring plan
- [x] ADR must document all alternatives considered

### Anti-Patterns to Avoid

- ❌ Implementing changes before documenting the decision
- ❌ Incomplete rationale (must include all 9 reasons)
- ❌ Missing alternatives section

## Specifications

### ADR File Location

`docs/decisions/bind-mount-standardization.md`

### Reasons to Document (all 9 must be included)

1. **Observability**
   - Users can see exactly where persistent data is stored
   - No need to search `/var/lib/docker/volumes/` for hidden data
   - File system tools (ls, du, find) work directly on data

2. **Backup Simplicity**
   - Single command backup: `cp -r ./storage/ backup/` or `rsync -av ./storage/ backup/`
   - No Docker-specific tooling required (no `docker volume` commands)
   - Standard backup tools and scripts work without modification
   - Incremental backups are straightforward

3. **Restore Simplicity**
   - Restore by copying files back to `./storage/`
   - No need to recreate Docker volumes before restore
   - Can restore to different machines without Docker volume migration

4. **Consistency**
   - Same pattern for all services (Tracker, Caddy, MySQL, Grafana, Prometheus)
   - Predictable directory structure: `./storage/{service}/{type}/`
   - Eliminates cognitive overhead of mixed volume types

5. **Portability**
   - Data directory can be moved between hosts by copying
   - No Docker volume export/import dance
   - Works with any container runtime that supports bind mounts

6. **Debugging & Troubleshooting**
   - Direct file inspection without entering containers
   - Easy to check file permissions, ownership, disk usage
   - Can modify config files directly for debugging
   - Log files accessible without `docker logs`

7. **Development Experience**
   - Easy to reset state by deleting directories
   - Can pre-populate data for testing scenarios
   - IDE file watchers can observe changes

8. **Deployment Architecture Simplification**
   - Eliminates top-level `volumes:` section in docker-compose.yml
   - No volume derivation logic needed (which volumes are required?)
   - Ansible only needs to create directories, not manage Docker volumes

9. **Security Visibility**
   - File permissions are visible and controllable
   - SELinux labels can be applied consistently (`:Z` suffix)
   - No hidden data in Docker-managed locations

### Alternatives to Document

1. **Named Volumes Only**: Rejected because data is hidden, backup requires Docker commands
2. **Mixed Approach**: Rejected because inconsistency creates confusion and maintenance burden
3. **Docker Volume Plugins**: Rejected as overkill for single-VM deployments

### Consequences to Document

**Positive:**

- Simplified backup and restore procedures
- Better observability of persistent data
- Consistent storage pattern across all services
- Easier debugging and troubleshooting

**Negative:**

- Requires explicit directory creation with correct permissions before container start
- Must manage SELinux labels manually (`:Z` suffix)
- Slightly more complex Ansible playbooks for permission management

## Implementation Plan

### Phase 1: Create ADR Document

- [ ] Task 1.1: Create `docs/decisions/bind-mount-standardization.md`
- [ ] Task 1.2: Write Context section with all 9 reasons
- [ ] Task 1.3: Write Decision section stating the choice
- [ ] Task 1.4: Write Alternatives section with 3 rejected options
- [ ] Task 1.5: Write Consequences section (positive and negative)
- [ ] Task 1.6: Reference the refactoring plan

### Phase 2: Register and Verify

- [ ] Task 2.1: Add entry to ADR index in `docs/decisions/README.md`
- [ ] Task 2.2: Run linters: `cargo run --bin linter all`
- [ ] Task 2.3: Verify links work correctly

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] ADR file exists at `docs/decisions/bind-mount-standardization.md`
- [ ] All 9 reasons are documented in Context section
- [ ] All 3 alternatives are documented with rejection rationale
- [ ] Consequences section includes both positive and negative impacts
- [ ] ADR references the refactoring plan
- [ ] ADR is registered in `docs/decisions/README.md` index
- [ ] All markdown linting passes

## Related Documentation

- [docs/decisions/README.md](../decisions/README.md) - ADR conventions and template
- [docs/refactors/plans/docker-compose-topology-domain-model.md](../refactors/plans/docker-compose-topology-domain-model.md) - Refactoring plan
- [docs/decisions/grafana-integration-pattern.md](../decisions/grafana-integration-pattern.md) - ADR to be superseded (recommends named volumes)

## Notes

- This is a **documentation-only** task - no code changes
- This ADR supersedes the volume recommendations in `grafana-integration-pattern.md`
- The ADR establishes the foundation for Phase 0 implementation
- Estimated effort: ~1 hour
