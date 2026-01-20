# Change Grafana Default Port from 3100 to 3000

**Issue**: #275
**Parent Epic**: None (standalone task)
**Related**: [ADR: Grafana Integration Pattern](../decisions/grafana-integration-pattern.md)

## Overview

Change the Grafana host port from 3100 to 3000 in the Docker Compose template and all related documentation.

The Grafana service was originally copied from the [Torrust Demo project](https://github.com/torrust/torrust-demo), which uses port 3100 to avoid conflicts with other services using port 3000 (like Node.js dev servers). In this deployer configuration, we don't have that conflict, so we should use the default Grafana port (3000) for simplicity and to align with common expectations.

## Goals

- [ ] Use Grafana's default port (3000) instead of custom port 3100
- [ ] Update all documentation references from port 3100 to 3000
- [ ] Update Rust source code port references
- [ ] Create an ADR documenting this decision

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (template) + Application + Presentation
**Module Path**: Multiple locations
**Pattern**: Configuration change

### Module Structure Requirements

- [ ] Update docker-compose template
- [ ] Update Rust source code port constants/references
- [ ] Update documentation

### Architectural Constraints

- [ ] No functional changes beyond port number update
- [ ] All references must be consistent across codebase

### Anti-Patterns to Avoid

- ❌ Leaving inconsistent port references in documentation
- ❌ Missing any hardcoded port references

## Specifications

### Files to Update

#### Template (Primary Change)

- `templates/docker-compose/docker-compose.yml.tera` - Change `"3100:3000"` to `"3000:3000"`

#### Rust Source Code

- `src/application/command_handlers/show/info/grafana.rs`
- `src/infrastructure/remote_actions/validators/grafana.rs`
- `src/presentation/views/commands/show/environment_info/grafana.rs`
- `src/testing/e2e/tasks/run_run_validation.rs`

#### Active Documentation (Update these)

- `docs/decisions/grafana-integration-pattern.md`
- `docs/decisions/docker-ufw-firewall-security-strategy.md`
- `docs/analysis/security/docker-network-segmentation-analysis.md`
- `docs/e2e-testing/manual/grafana-verification.md`
- `docs/user-guide/README.md`
- `docs/user-guide/security.md`
- `docs/user-guide/quick-start/docker.md`
- `docs/user-guide/services/grafana.md`
- `docs/user-guide/services/https.md`

#### Archived/Experimental Documentation (Do NOT update)

- `docs/research/caddy-tls-proxy-evaluation/production-deployment.md` - Historical research
- `docs/research/caddy-tls-proxy-evaluation/experiment-files/docker-compose.yml` - Historical experiment
- `experiments/caddy-full-stack/docker-compose.yml` - Experimental setup

## Implementation Plan

### Phase 1: Template and Source Code (15 min)

- [ ] Task 1.1: Update `templates/docker-compose/docker-compose.yml.tera` port from 3100 to 3000
- [ ] Task 1.2: Update all Rust source files with port references
- [ ] Task 1.3: Run pre-commit checks to verify no compilation/test failures

### Phase 2: Documentation Updates (15 min)

- [ ] Task 2.1: Update all active documentation files listed above
- [ ] Task 2.2: Run linters to verify documentation formatting

### Phase 3: ADR and Finalization (10 min)

- [ ] Task 3.1: Create ADR documenting the port change decision
- [ ] Task 3.2: Run full pre-commit checks
- [ ] Task 3.3: Commit all changes
- [ ] Task 3.4: Push to remote main branch

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Docker Compose template uses port 3000:3000 for Grafana
- [ ] All Rust source code references updated to port 3000
- [ ] All active documentation references updated to port 3000
- [ ] No grep results for "3100" related to Grafana in active code/docs
- [ ] ADR created explaining the rationale for the change
- [ ] E2E tests pass with the new port

## Related Documentation

- [ADR: Grafana Integration Pattern](../decisions/grafana-integration-pattern.md)
- [Torrust Demo](https://github.com/torrust/torrust-demo) - Original source of Grafana configuration
- [Grafana Docker Documentation](https://grafana.com/docs/grafana/latest/setup-grafana/installation/docker/)

## Notes

- The change from 3100 to 3000 is purely cosmetic/simplification - both ports work identically
- Port 3000 is Grafana's default internal port, so using 3000:3000 is more intuitive
- Archived/experimental documentation is intentionally NOT updated to preserve historical accuracy
