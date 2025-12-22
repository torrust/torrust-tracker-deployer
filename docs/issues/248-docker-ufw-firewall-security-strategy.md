# Docker and UFW Firewall Security Strategy

**Issue**: [#248](https://github.com/torrust/torrust-tracker-deployer/issues/248)  
**Type**: Architecture / Security  
**Priority**: CRITICAL  
**Related**:

- [#246 - Grafana slice](./246-grafana-slice-release-run-commands.md) (where this was discovered)
- [torrust-demo#72 - Docker bypassing systemd-resolved](https://github.com/torrust/torrust-demo/issues/72)

## Overview

During implementation of issue #246 (Grafana slice), we re-encountered a **known but forgotten issue: Docker bypasses UFW firewall rules**. This behavior was previously addressed in the Torrust Tracker Live Demo by using Digital Ocean's cloud firewall (which Docker cannot bypass), but that solution was deliberately avoided in the deployer to maintain provider portability.

The deployer chose UFW for firewall management to avoid cloud-provider-specific features and enable easy migration between providers. However, during development, we configured UFW expecting it to block Docker-exposed ports, forgetting that Docker manipulates iptables directly and bypasses UFW rules entirely.

The ports were intentionally exposed via Docker (e.g., `0.0.0.0:9090:9090`) to facilitate service testing via SSH. However, this exposed internal services publicly. The correct approach is to use localhost binding (e.g., `127.0.0.1:9090:9090`) for services that should only be accessible from the host, not externally.

This task involves creating a comprehensive, documented security strategy that correctly addresses Docker-UFW interaction while maintaining provider-agnostic deployment capabilities.

## Goals

- [ ] Research Docker networking security model and UFW interaction
- [ ] Design comprehensive firewall security strategy
- [ ] Create ADR documenting the architectural decision
- [ ] Implement security strategy across all templates
- [ ] Add validation/testing to prevent future security misconfigurations

## 🏗️ Architecture Requirements

**DDD Layer**: Multiple (Domain, Infrastructure, Application)  
**Module Path**:

- `templates/ansible/configure-firewall.yml` (Infrastructure templates)
- `templates/docker-compose/` (Infrastructure templates)
- Security validation in application layer (future)

**Pattern**: Infrastructure as Code (IaC) templates, Architectural Decision Record

### Module Structure Requirements

- [ ] Templates follow project template architecture (see [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md))
- [ ] Security configurations are explicit and well-documented
- [ ] Validation mechanisms prevent accidental exposure

### Architectural Constraints

- [ ] Maintain provider-agnostic deployment approach
- [ ] Use layered security approach (instance-level + service-level)
- [ ] Follow Infrastructure as Software principles
- [ ] Error handling for security misconfigurations must be explicit and actionable

### Anti-Patterns to Avoid

- ❌ Relying solely on UFW when Docker is present
- ❌ Implicit security assumptions without validation
- ❌ Provider-specific firewall dependencies (maintain portability)
- ❌ Silent security failures

## Problem Statement

### Current Architecture Assumption (INVALID)

The original deployment strategy assumed:

1. Use UFW firewall to secure the entire VM instance
2. Only open specific ports that should be publicly accessible
3. Avoid provider-specific firewalls to maintain provider-agnostic deployment
4. Default deny all incoming traffic except explicitly allowed services

**This assumption is INVALID** because Docker manipulates iptables directly, bypassing UFW rules.

### Discovered Security Issue

**Scenario**: Prometheus service configured in docker-compose with port binding:

```yaml
prometheus:
  ports:
    - "9090:9090" # Binds to 0.0.0.0:9090
```

**Expected Behavior**:

- UFW default policy: deny incoming
- Port 9090 NOT explicitly allowed in UFW
- Port 9090 should be inaccessible from external network

**Actual Behavior**:

- Prometheus UI accessible at `http://<vm-ip>:9090` from external network
- UFW rules completely bypassed
- Security breach - internal service exposed publicly

**Root Cause**: Docker creates iptables rules that take precedence over UFW rules when publishing ports with `0.0.0.0:<port>:<container-port>` binding.

### Where This Was Discovered

**File**: `templates/docker-compose/docker-compose.yml.tera`  
**Commit**: Security fix applied in commit 8323def  
**Issue**: #246 - Grafana slice implementation

**Evidence**:

```bash
# UFW status shows port 9090 NOT allowed
$ sudo ufw status | grep 9090
# (no output - port not in UFW rules)

# But Prometheus is accessible externally
$ curl http://10.140.190.35:9090
HTTP/1.1 405 Method Not Allowed  # Accessible!
```

**Manual testing documentation**: [docs/e2e-testing/manual/grafana-testing-results.md](../e2e-testing/manual/grafana-testing-results.md)

## Specifications

### Proposed Security Strategy

Use a **layered security approach** combining UFW and Docker networking:

#### Layer 1: UFW Firewall (Instance-Level Protection)

- **Purpose**: Secure the entire VM instance
- **Configuration**: Deny all incoming traffic except SSH
- **Responsibility**: Prevent unauthorized access to the instance itself

```yaml
# templates/ansible/configure-firewall.yml
- Set default policy: deny incoming
- Allow only SSH port (22 or custom)
- Do NOT allow application ports (tracker, grafana, etc.)
```

#### Layer 2: Docker Port Bindings (Service-Level Exposure)

- **Purpose**: Selectively expose services to external network
- **Configuration**: Only bind ports for public-facing services
- **Responsibility**: Control which services are accessible from outside

```yaml
# templates/docker-compose/docker-compose.yml.tera

# Public services - port binding
tracker:
  ports:
    - "6969:6969/udp" # UDP tracker
    - "7070:7070" # HTTP tracker
    - "1212:1212" # REST API

grafana:
  ports:
    - "3100:3000" # Public UI (custom port)

# Internal services - NO port binding
prometheus:
  # No ports section - internal only
  # Accessed via Docker network: http://prometheus:9090

mysql:
  # No ports section - internal only
  # Accessed via Docker network: mysql:3306
```

**Note:** When HTTPS support is added (roadmap task 6), the architecture will change:

- A reverse proxy will be introduced to handle HTTPS/TLS certificates
- HTTP tracker (7070) and REST API (1212) will route through the proxy
- UDP tracker (6969) will remain directly exposed (cannot be proxied through HTTPS)
- Only the proxy's HTTPS port(s) will be publicly exposed for HTTP services

#### Layer 3: Docker Internal Networks (Inter-Service Communication)

- **Purpose**: Allow services to communicate securely within Docker
- **Configuration**: Use Docker network names for service discovery
- **Responsibility**: Internal service communication without external exposure

```yaml
# Real example from build/manual-test-grafana/docker-compose/docker-compose.yml

networks:
  backend_network: {}

services:
  tracker:
    networks:
      - backend_network
    ports:
      - "6969:6969/udp" # Public: UDP tracker
      - "7070:7070" # Public: HTTP tracker
      - "1212:1212" # Public: REST API

  prometheus:
    networks:
      - backend_network
    ports:
      - "127.0.0.1:9090:9090" # Localhost only - accessible only from host
    # Prometheus scrapes metrics from tracker REST API: http://tracker:1212

  grafana:
    networks:
      - backend_network
    ports:
      - "3100:3000" # Public: Grafana UI
    # Grafana reads from Prometheus: http://prometheus:9090

  mysql:
    networks:
      - backend_network
    # No ports section - internal only
    # Tracker connects via: mysql:3306
```

**Key observations**:

- Services discover each other using service names (e.g., `tracker:1212`, `prometheus:9090`)
- Prometheus is bound to localhost only (`127.0.0.1:9090:9090`) - accessible from host but not external network
- MySQL has no port binding - completely internal
- All services share `backend_network` for internal communication

### Key Principle

**UFW secures the instance, Docker secures the services:**

- UFW closes everything except SSH (instance-level security)
- Docker port bindings control external service exposure (service-level security)
- Docker networks enable internal service communication (no external exposure)

### Benefits and Drawbacks

**Benefits**:

1. ✅ Provider-agnostic - Works on any VM provider without provider-specific firewall integration
2. ✅ Layered security - Multiple security boundaries
3. ✅ Explicit exposure - Port bindings make it clear what's public vs internal
4. ✅ Simple configuration - No need for UFW rules per service
5. ✅ Docker-native - Leverages Docker's built-in networking and security

**Drawbacks**:

1. ⚠️ UFW not controlling application ports - Relies on correct docker-compose configuration
2. ⚠️ Human error risk - Mistakenly adding port binding exposes service immediately
3. ⚠️ No defense-in-depth for Docker - If docker-compose misconfigured, service exposed
4. ⚠️ Trust in Docker networking - Assumes Docker network isolation is secure

### Technical Questions to Investigate

1. **Docker Network Isolation**: How secure is Docker's internal network isolation? Can containers on different networks communicate?

2. **Port Binding Risk**: What happens if a developer accidentally adds a port binding to an internal service? Is there any safeguard?

3. **iptables Priority**: Can we configure UFW to take precedence over Docker's iptables rules? (Likely not without breaking Docker)

4. **Alternative Solutions**:

   - Could we use `127.0.0.1:<host-port>:<container-port>` bindings and nginx/reverse-proxy?
   - Should we integrate with provider-specific firewalls despite complexity?
   - Can we use Docker's built-in firewall features (docker-proxy, etc.)?

5. **Testing Strategy**: How do we automatically verify no unintended ports are exposed during E2E tests?

### Security Questions

1. **Threat Model**: What attack vectors exist with this approach?

   - Misconfigured docker-compose exposing internal services
   - Docker daemon compromise
   - Container escape vulnerabilities

2. **Compliance**: Does this approach meet security best practices for production deployments?

3. **Monitoring**: How do we detect if internal services become accidentally exposed?

4. **Recovery**: If a service is exposed, what's the remediation process?

## Implementation Plan

### Phase 1: Research and Analysis (estimated: 2-3 hours)

- [ ] **Review prior work**: Examine how this was handled in the Torrust Tracker Live Demo project
- [ ] **Review Docker official documentation**: Read [Docker Packet filtering and firewalls](https://docs.docker.com/engine/network/packet-filtering-firewalls/) - especially the "Docker and ufw" section which explicitly documents the incompatibility and explains how Docker routes container traffic in the NAT table, bypassing ufw's INPUT/OUTPUT chains
- [ ] Study Docker networking security model and isolation guarantees
- [ ] Review Docker iptables integration and UFW interaction mechanisms
- [ ] Research how other projects handle this (Kubernetes, Docker Swarm, Compose-based deployments)
- [ ] Analyze the torrust-demo#72 issue for related lessons learned
- [ ] Review security best practices for Docker production deployments
- [ ] Investigate alternative firewall strategies and their trade-offs
- [ ] Document threat model for proposed strategy
- [ ] Analyze attack vectors and security boundaries
- [ ] Compare with provider-specific firewall integration complexity
- [ ] Evaluate trade-offs: simplicity vs security vs portability

### Phase 2: Design and Documentation (estimated: 2-3 hours)

- [ ] Create comprehensive ADR for firewall security strategy in `docs/decisions/`
- [ ] Define explicit rules for which services should have port bindings
- [ ] Document operational procedures (monitoring, incident response)
- [ ] Design validation/linting strategy for docker-compose security
- [ ] Create security testing strategy for E2E tests
- [ ] Update architecture documentation with security patterns

### Phase 3: Template Implementation (estimated: 3-4 hours)

- [ ] **Remove obsolete firewall configuration**: Delete `templates/ansible/configure-tracker-firewall.yml` - no longer needed since Docker bypasses UFW
- [ ] **Remove tracker firewall step**: Delete or refactor `src/application/steps/system/configure_tracker_firewall.rs` - tracker ports don't need UFW rules
- [ ] **Remove playbook registration**: Remove `configure-tracker-firewall.yml` from `src/infrastructure/templating/ansible/template/renderer/project_generator.rs`
- [ ] Update `templates/ansible/configure-firewall.yml` to clarify it only manages SSH access (not application ports)
- [ ] Review and update all `templates/docker-compose/*.yml.tera` files
- [ ] Remove unnecessary port bindings from internal services
- [ ] Add explicit comments documenting public vs internal services
- [ ] Ensure consistent network configuration across all services
- [ ] Validate all existing docker-compose configurations

### Phase 4: Validation and Testing (estimated: 2-3 hours)

- [ ] Design and implement E2E security tests
- [ ] Add test cases to verify port exposure protection
- [ ] Test that internal services are NOT accessible externally
- [ ] Test that public services ARE accessible with correct ports
- [ ] Add validation logic to detect misconfigured port bindings (future work)
- [ ] Document testing procedures in `docs/e2e-testing/`

### Phase 5: Documentation and Review (estimated: 1-2 hours)

- [ ] **Review and update user security guide**: Review `docs/user-guide/security.md` and verify it aligns with the new Docker/UFW security strategy - update any outdated assumptions about UFW protecting Docker ports
- [ ] Update user guide with security strategy explanation
- [ ] Document deployment security best practices
- [ ] Add warnings about Docker port binding risks
- [ ] Create troubleshooting guide for firewall issues
- [ ] Review all documentation for accuracy and completeness
- [ ] Security audit of final implementation

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Research and Analysis**:

- [ ] Docker networking security model documented
- [ ] UFW-Docker interaction thoroughly understood
- [ ] Threat model documented with attack vectors
- [ ] Alternative approaches evaluated and compared
- [ ] Security trade-offs clearly documented

**Design and Documentation**:

- [ ] ADR created in `docs/decisions/` following ADR template
- [ ] Security strategy explicitly documented with examples
- [ ] Clear rules for public vs internal service configuration
- [ ] Operational procedures documented (monitoring, incident response)

**Implementation**:

- [ ] All docker-compose templates updated with security strategy
- [ ] UFW firewall configuration updated as needed
- [ ] Internal services have NO port bindings
- [ ] Public services have EXPLICIT port bindings with comments
- [ ] All services use Docker networks for inter-service communication

**Testing**:

- [ ] E2E tests verify internal services are NOT externally accessible
- [ ] E2E tests verify public services ARE externally accessible
- [ ] Test documentation updated with security test cases
- [ ] All existing E2E tests pass

**Documentation**:

- [ ] User guide updated with security strategy
- [ ] Architecture documentation reflects security patterns
- [ ] Troubleshooting guide includes firewall issues
- [ ] Comments in templates explain security decisions

## Related Documentation

### Internal Documentation

- [Manual Grafana Testing Results](../e2e-testing/manual/grafana-testing-results.md) - Where security issue was discovered
- [Issue #246 - Grafana Slice](./246-grafana-slice-release-run-commands.md) - Implementation that revealed the issue
- [Firewall Ansible Playbook](../../templates/ansible/configure-firewall.yml) - Current UFW configuration
- [Codebase Architecture](../codebase-architecture.md) - DDD layer placement
- [ADR Template](../decisions/README.md) - For creating the security strategy ADR
- [E2E Testing Guide](../e2e-testing/README.md) - For security test implementation

### External References

- [torrust-demo#72 - Docker bypassing systemd-resolved](https://github.com/torrust/torrust-demo/issues/72) - Related Docker bypass issue
- **[Docker Documentation: Packet filtering and firewalls](https://docs.docker.com/engine/network/packet-filtering-firewalls/)** - **ESSENTIAL READING**: Official Docker documentation explaining Docker-UFW incompatibility. The "Docker and ufw" section states: "Docker and ufw use firewall rules in ways that make them incompatible with each other. When you publish a container's ports using Docker, traffic gets diverted before it goes through the ufw firewall settings. Docker routes container traffic in the NAT table, which means packets are diverted before reaching the INPUT and OUTPUT chains that ufw uses."
- [Docker with iptables](https://docs.docker.com/engine/network/firewall-iptables/) - Technical details on Docker's iptables integration
- [Docker with nftables](https://docs.docker.com/engine/network/firewall-nftables/) - Alternative firewall backend
- [UFW and Docker GitHub Discussion](https://github.com/docker/for-linux/issues/690) - Known interactions and issues
- [UFW-Docker Community Solution](https://github.com/chaifeng/ufw-docker) - Community approaches to the problem
- [Docker and Firewall Security Flaw Analysis](https://www.techrepublic.com/article/how-to-fix-the-docker-and-ufw-security-flaw/)

## Notes

### Context

- **Historical Note**: This Docker/UFW firewall interaction was previously known and addressed in the Torrust Tracker Live Demo project, but was forgotten when starting the deployer project
- This issue was re-discovered during real-world manual E2E testing of the Grafana slice feature
- The immediate fix for Prometheus (removing port binding) was applied in commit 8323def as part of #246
- That fix was a tactical solution; this issue addresses the strategic solution
- This affects not just this project but potentially all Torrust projects using Docker
- The Tracker Live Demo implementation likely contains solutions/patterns that can be referenced

### Scope

- This issue focuses on the deployer project's firewall strategy
- It does NOT include integration with provider-specific firewalls (AWS Security Groups, Hetzner Cloud Firewall, etc.)
- Future work may revisit provider-specific firewall integration if needed

### Priority Justification

**CRITICAL Priority** because:

1. **Security vulnerability** - Internal services can be accidentally exposed
2. **Silent failure** - UFW shows correct configuration but doesn't protect
3. **False sense of security** - Developers may assume UFW is protecting them
4. **Production impact** - Affects all deployments using Docker
5. **Architecture foundation** - Firewall strategy is fundamental to security

### Implementation Notes

- **Immediate fix already applied**: Prometheus port binding removed in #246 (commit 8323def)
- **This issue scope**: Create comprehensive strategy and apply it consistently across all services
- **AI Assistant suitability**: This work is well-suited for AI assistant implementation with human supervision
- **No provider lock-in**: Solution must remain provider-agnostic

---

**Created**: 2025-12-22  
**Discovered During**: Issue #246 - Grafana slice implementation  
**Process**: Research → Analysis → Design (ADR) → Implementation → Testing → Documentation
