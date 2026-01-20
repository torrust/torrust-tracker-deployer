# Security: MySQL port 3306 is publicly exposed in Docker Compose configuration

**Issue**: #277
**Parent Epic**: None (standalone security fix)
**Related**: [docs/decisions/docker-ufw-firewall-security-strategy.md](../decisions/docker-ufw-firewall-security-strategy.md)

## Overview

The MySQL service in the Docker Compose template exposes port 3306 publicly to all network interfaces, allowing external connections to the database. This is a security risk as it enables potential brute-force attacks on MySQL credentials and exposes the database to unauthorized access.

## Goals

- [ ] Remove public exposure of MySQL port 3306
- [ ] Maintain MySQL healthcheck functionality
- [ ] Ensure tracker can still connect to MySQL via internal Docker network

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (templates)
**Module Path**: `templates/docker-compose/`
**Pattern**: Template modification

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Template changes only - no Rust code changes required
- [ ] Must maintain compatibility with existing environment configurations

### Anti-Patterns to Avoid

- ❌ Exposing internal services to public network
- ❌ Breaking healthcheck functionality
- ❌ Breaking inter-container communication

## Specifications

### Current State (Problematic)

The MySQL service in `templates/docker-compose/docker-compose.yml.tera` has:

```yaml
mysql:
  # ...
  ports:
    - "3306:3306"
```

This exposes MySQL to the entire network. Anyone with network access to the VM can connect to the database using the credentials.

### Verified Issue

The issue was verified by:

1. Creating a test environment with MySQL configuration (`envs/manual-mysql-test.json`)
2. Deploying the environment
3. Successfully connecting to MySQL from outside the VM:

```bash
mysql -h 10.140.190.23 -P 3306 -u tracker_user -p tracker_password -e "SELECT 1;"
+-----------------+
| connection_test |
+-----------------+
|               1 |
+-----------------+
```

### Desired State (Secure)

Remove the `ports` section entirely. The MySQL healthcheck uses `mysqladmin ping -h localhost` which works without port exposure because it runs inside the container.

Unlike Prometheus (which binds to localhost for host validation via `curl http://localhost:9090`), MySQL only needs to be accessible by the tracker container via Docker's internal `database_network`, not from the host.

## Implementation Plan

### Phase 1: Fix Template (5 minutes)

- [ ] Task 1.1: Remove `ports: - "3306:3306"` from MySQL service in `docker-compose.yml.tera`
- [ ] Task 1.2: Add security comment explaining why port is not exposed

### Phase 2: Verification (15 minutes)

- [ ] Task 2.1: Deploy test environment with MySQL
- [ ] Task 2.2: Verify MySQL is NOT accessible from outside the VM (`nc -zv <ip> 3306` should fail)
- [ ] Task 2.3: Verify healthcheck still works (container becomes healthy)
- [ ] Task 2.4: Verify tracker can still connect to MySQL via internal network

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Security Criteria**:

- [ ] MySQL port 3306 is not accessible from outside the VM
- [ ] MySQL healthcheck still passes (container reaches healthy state)
- [ ] Tracker can still connect to MySQL via Docker internal network
- [ ] E2E tests pass

## Related Documentation

- [Docker UFW Firewall Security Strategy](../decisions/docker-ufw-firewall-security-strategy.md)
- [Docker Network Segmentation Analysis](../analysis/security/docker-network-segmentation-analysis.md)
- [User Guide: Security](../user-guide/security.md)

## Notes

The fix is straightforward - simply remove the `ports` section from MySQL service. The healthcheck will continue to work because:

1. Docker healthchecks run inside the container
2. `mysqladmin ping -h localhost` connects to MySQL within the container's network namespace
3. No external port binding is required for container-internal commands
