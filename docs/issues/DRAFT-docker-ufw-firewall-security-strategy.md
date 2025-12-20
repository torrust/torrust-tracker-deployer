# DRAFT: Docker and UFW Firewall Security Strategy

**Status**: DRAFT - Needs Analysis  
**Priority**: CRITICAL - Security Issue  
**Issue Type**: Architecture / Security  
**Related Issues**:

- [#246 - Grafana slice](./246-grafana-slice-release-run-commands.md) (where this was discovered)
- [torrust-demo#72 - Docker bypassing systemd-resolved](https://github.com/torrust/torrust-demo/issues/72)

## Problem Statement

During implementation of issue #246 (Grafana slice), we discovered that **Docker bypasses UFW firewall rules**, exposing services even when UFW is configured with "deny incoming" default policy.

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

## Original Security Strategy

The deployment was designed to:

1. **Use UFW exclusively** for firewall management (provider-agnostic)
2. **Avoid provider-specific firewalls** (AWS Security Groups, Hetzner Cloud Firewall, etc.)
3. **Maintain portability** across different hosting providers
4. **Simple configuration** - single firewall mechanism (UFW)

**Rationale**: Integrating with multiple provider-specific firewalls would significantly increase complexity and make deployment harder across different providers.

**NOTE**: No ADR was created for this decision initially, but it was the working assumption.

## Potential Solution (Needs Validation)

### Proposed Strategy

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
    - "8080:8080" # Public API
    - "6969:6969/udp" # Public tracker

grafana:
  ports:
    - "3100:3000" # Public UI

# Internal services - NO port binding
prometheus:
  # No ports section - internal only
  # Accessed via Docker network: http://prometheus:9090

mysql:
  # No ports section - internal only
  # Accessed via Docker network: mysql:3306
```

#### Layer 3: Docker Internal Networks (Inter-Service Communication)

- **Purpose**: Allow services to communicate securely within Docker
- **Configuration**: Use Docker network names for service discovery
- **Responsibility**: Internal service communication without external exposure

```yaml
networks:
  backend_network: {}

services:
  grafana:
    networks:
      - backend_network
    # Connects to Prometheus via: http://prometheus:9090

  prometheus:
    networks:
      - backend_network
    # Connects to Tracker via: http://tracker:8080
```

### Key Principle

UFW secures the instance, Docker secures the services:

- UFW closes everything except SSH (instance-level security)
- Docker port bindings control external service exposure (service-level security)
- Docker networks enable internal service communication (no external exposure)

### Benefits

1. ✅ **Provider-agnostic** - Works on any VM provider without provider-specific firewall integration
2. ✅ **Layered security** - Multiple security boundaries
3. ✅ **Explicit exposure** - Port bindings make it clear what's public vs internal
4. ✅ **Simple configuration** - No need for UFW rules per service
5. ✅ **Docker-native** - Leverages Docker's built-in networking and security

### Drawbacks

1. ⚠️ **UFW not controlling application ports** - Relies on correct docker-compose configuration
2. ⚠️ **Human error risk** - Mistakenly adding port binding exposes service immediately
3. ⚠️ **No defense-in-depth for Docker** - If docker-compose misconfigured, service exposed
4. ⚠️ **Trust in Docker networking** - Assumes Docker network isolation is secure

## Questions to Investigate

### Technical Questions

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

### Implementation Questions

1. **Migration**: How do we update existing deployments to this strategy?

2. **Documentation**: What warnings/guidance do we provide to prevent misconfigurations?

3. **Validation**: Can we add linting/validation to detect port bindings on internal services?

4. **Testing**: How do we test the security posture in E2E tests?

## Required Actions

### 1. Research Phase

- [ ] Study Docker networking security model
- [ ] Review Docker iptables integration and UFW interaction
- [ ] Research how other projects handle this (Kubernetes, Docker Swarm, etc.)
- [ ] Analyze the torrust-demo#72 issue for related lessons learned
- [ ] Review security best practices for Docker deployments
- [ ] Investigate alternative firewall strategies

### 2. Analysis Phase

- [ ] Document threat model for proposed strategy
- [ ] Analyze attack vectors and security boundaries
- [ ] Compare with provider-specific firewall integration complexity
- [ ] Evaluate trade-offs: simplicity vs security vs portability
- [ ] Define clear security requirements

### 3. Design Phase

- [ ] Create comprehensive ADR for firewall security strategy
- [ ] Define explicit rules for which services get port bindings
- [ ] Design validation/linting for docker-compose security
- [ ] Create security testing strategy for E2E tests
- [ ] Document operational procedures (monitoring, incident response)

### 4. Implementation Phase

- [ ] Update all docker-compose templates with security principles
- [ ] Remove unnecessary port bindings (like Prometheus 9090)
- [ ] Add validation to prevent accidental exposures
- [ ] Implement E2E security tests
- [ ] Update documentation and user guides

### 5. Review Phase

- [ ] Security audit of implementation
- [ ] Penetration testing
- [ ] Documentation review
- [ ] Team review and sign-off

## Immediate Actions (Already Taken)

As part of issue #246 implementation:

✅ **Security fix applied** (commit 8323def):

- Removed Prometheus port binding (`9090:9090`)
- Added comments explaining internal-only services
- Updated tests to verify port NOT exposed
- Documented security issue in manual testing results

✅ **Documentation**:

- Recorded security issue discovery in [manual testing results](../e2e-testing/manual/grafana-testing-results.md)
- Explained Docker bypassing UFW in commit messages
- Created this draft issue specification

## Related Documentation

### Internal Documentation

- [Manual Grafana Testing Results](../e2e-testing/manual/grafana-testing-results.md) - Where security issue was discovered
- [Issue #246 - Grafana Slice](./246-grafana-slice-release-run-commands.md) - Implementation that revealed the issue
- [Firewall Ansible Playbook](../../templates/ansible/configure-firewall.yml) - Current UFW configuration

### External References

- [torrust-demo#72 - Docker bypassing systemd-resolved](https://github.com/torrust/torrust-demo/issues/72) - Related Docker bypass issue
- Docker Documentation: [Packet filtering and firewalls](https://docs.docker.com/network/packet-filtering-firewalls/)
- UFW and Docker: [Known interactions and issues](https://github.com/docker/for-linux/issues/690)

### Similar Problems in the Wild

- [UFW and Docker: The Problem](https://github.com/chaifeng/ufw-docker) - Community solutions
- [Docker and Firewall Issues](https://www.techrepublic.com/article/how-to-fix-the-docker-and-ufw-security-flaw/)

## Priority Justification

**CRITICAL Priority** because:

1. **Security vulnerability** - Internal services can be accidentally exposed
2. **Silent failure** - UFW shows correct configuration but doesn't protect
3. **False sense of security** - Developers may assume UFW is protecting them
4. **Production impact** - Affects all deployments using Docker
5. **Architecture foundation** - Firewall strategy is fundamental to security

**Why DRAFT**:

- Requires thorough analysis before making architectural decisions
- Need to validate proposed solution against security requirements
- Must consider all alternatives and trade-offs
- ADR required for such a fundamental decision

## Next Steps

1. **Schedule analysis session** - Dedicate time to research and analyze
2. **Consult security resources** - Review Docker security best practices
3. **Draft ADR** - Create comprehensive architectural decision record
4. **Team review** - Get feedback on proposed strategy
5. **Implement and test** - Apply solution across codebase
6. **Document** - Update all relevant documentation

## Notes

- This issue was discovered during real-world manual E2E testing
- The fix for Prometheus (removing port binding) is a band-aid, not a complete solution
- We need a coherent, documented strategy for all current and future services
- This affects not just this project but potentially all Torrust projects using Docker

## Open Questions for Discussion

1. Should we reconsider provider-specific firewall integration despite complexity?
2. Is Docker network isolation sufficient for production security?
3. What's the acceptable level of risk for accidental service exposure?
4. Should we implement automated security scanning for port bindings?
5. How do other similar projects (deployment tools for containerized apps) handle this?

---

**Created**: 2025-12-19  
**Discovered During**: Issue #246 - Grafana slice implementation  
**Needs**: Research → Analysis → ADR → Implementation
