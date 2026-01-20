# Decision: Docker and UFW Firewall Security Strategy

## Status

Accepted

## Date

2025-12-22

## Context

During the implementation of the Grafana slice feature (issue #246), we re-encountered a known but previously forgotten security issue: **Docker bypasses UFW firewall rules entirely**, rendering UFW ineffective for controlling access to Docker-exposed ports.

### Background

This issue was previously known and addressed in the Torrust Tracker Live Demo deployment by using Digital Ocean's cloud firewall, which operates at the network level and cannot be bypassed by Docker. However, for the deployer project, we deliberately chose UFW for firewall management to:

1. Maintain provider-agnostic deployment capabilities
2. Enable easy migration between cloud providers
3. Avoid dependency on cloud-provider-specific features
4. Simplify deployment architecture

During development, we configured UFW expecting it to block Docker-exposed ports, forgetting that Docker manipulates iptables directly and completely bypasses UFW's INPUT/OUTPUT chains.

### The Problem

**Docker's iptables Manipulation**: When Docker publishes container ports using `0.0.0.0:<host-port>:<container-port>` binding, it creates iptables rules in the NAT table that take precedence over UFW rules. Traffic is diverted before it reaches UFW's INPUT and OUTPUT chains.

**Real-world Example**: Prometheus service configured with port binding `9090:9090`:

- UFW status shows port 9090 NOT allowed
- UFW default policy: deny incoming
- **Result**: Prometheus UI accessible at `http://<vm-ip>:9090` from external network
- **Security breach**: Internal service exposed publicly despite UFW configuration

This creates a **false sense of security** where operators believe UFW is protecting services, but Docker is exposing them publicly.

### Official Docker Documentation

The Docker official documentation explicitly states this incompatibility:

> "Docker and ufw use firewall rules in ways that make them incompatible with each other. When you publish a container's ports using Docker, traffic gets diverted before it goes through the ufw firewall settings. Docker routes container traffic in the NAT table, which means packets are diverted before reaching the INPUT and OUTPUT chains that ufw uses."
>
> — [Docker Documentation: Packet filtering and firewalls](https://docs.docker.com/engine/network/packet-filtering-firewalls/)

## Decision

We adopt a **layered security approach** that acknowledges and works with Docker's networking behavior rather than fighting against it. Security is enforced through three complementary layers:

### Layer 1: UFW Firewall (Instance-Level Protection)

**Purpose**: Secure the entire VM instance from unauthorized access

**Configuration**:

- Default policy: deny incoming
- Allow only SSH port (22 or custom)
- **Do NOT** allow application ports (tracker, grafana, prometheus, etc.)

**Responsibility**: Prevent unauthorized SSH access and instance-level attacks

```yaml
# templates/ansible/configure-firewall.yml
- Set default policy: deny incoming
- Allow only SSH port
- No application port rules (Docker handles this)
```

**Rationale**: Since UFW cannot control Docker-exposed ports, we simplify UFW configuration to its actual effective scope: non-Docker traffic, primarily SSH access.

### Layer 2: Docker Port Bindings (Service-Level Exposure)

**Purpose**: Selectively expose services to the external network

**Configuration**: Only bind ports for services that should be publicly accessible

**Responsibility**: Control which services are accessible from outside the VM

```yaml
# Public services - explicit port binding to 0.0.0.0
tracker:
  ports:
    - "6969:6969/udp" # UDP tracker - public
    - "7070:7070" # HTTP tracker - public
    - "1212:1212" # REST API - public

grafana:
  ports:
    - "3000:3000" # Grafana UI - public

# Host-accessible services - bind to localhost only
prometheus:
  ports:
    - "127.0.0.1:9090:9090" # Localhost only - NOT external
  # Accessible from host for debugging: curl http://localhost:9090
  # Other services access via Docker network: http://prometheus:9090

# Internal-only services - no port binding
mysql:
  # No ports section - completely internal
  # Only accessible via Docker network: mysql:3306
```

**Binding Patterns**:

1. **Public services**: `"<port>:<port>"` or `"0.0.0.0:<port>:<port>"` - accessible externally
2. **Host-only services**: `"127.0.0.1:<port>:<port>"` - accessible from host for debugging, not external
3. **Internal services**: No ports section - Docker network only

### Layer 3: Docker Internal Networks (Inter-Service Communication)

**Purpose**: Enable secure communication between services without external exposure

**Configuration**: All services share a Docker network and use service names for discovery

**Responsibility**: Internal service-to-service communication

```yaml
networks:
  backend_network: {}

services:
  tracker:
    networks:
      - backend_network
    # Provides metrics API at http://tracker:1212 for Prometheus

  prometheus:
    networks:
      - backend_network
    # Scrapes tracker metrics via Docker network: http://tracker:1212
    # Grafana accesses via: http://prometheus:9090

  grafana:
    networks:
      - backend_network
    # Reads from Prometheus via: http://prometheus:9090

  mysql:
    networks:
      - backend_network
    # Tracker connects via: mysql:3306
```

**Key Principle**: Services discover each other using Docker service names (e.g., `tracker:1212`, `prometheus:9090`, `mysql:3306`). No port exposure needed for internal communication.

### Security Model Summary

**UFW secures the instance, Docker secures the services:**

1. **Instance-Level (UFW)**: Deny all incoming except SSH
2. **Service-Level (Docker)**: Explicit port bindings control public exposure
3. **Internal Communication (Docker Networks)**: Service discovery without exposure

### Removal of Obsolete Configuration

As part of this strategy, we remove obsolete files and code that assumed UFW could control Docker ports:

- **Delete**: `templates/ansible/configure-tracker-firewall.yml` - no longer needed
- **Delete/Refactor**: `src/application/steps/system/configure_tracker_firewall.rs` - tracker ports don't need UFW rules
- **Update**: `templates/ansible/configure-firewall.yml` - clarify it only manages SSH access
- **Update**: All `docker-compose/*.yml.tera` templates - explicit comments on public vs internal services

## Consequences

### Benefits

1. ✅ **Provider-Agnostic**: Works on any VM provider without cloud-specific firewall integration (Hetzner, Digital Ocean, AWS, Linode, etc.)
2. ✅ **Layered Security**: Multiple security boundaries (instance + service levels)
3. ✅ **Explicit Exposure**: Port bindings make it immediately clear what's public vs internal
4. ✅ **Simple Configuration**: No need for complex UFW rules per service
5. ✅ **Docker-Native**: Leverages Docker's built-in networking and security features
6. ✅ **Debugging-Friendly**: Localhost bindings (e.g., `127.0.0.1:9090:9090`) allow SSH access for debugging without public exposure
7. ✅ **Prevents Forgotten Mistakes**: No expectation that UFW protects Docker ports - eliminates false sense of security

### Drawbacks

1. ⚠️ **UFW Not Controlling Application Ports**: Relies entirely on correct docker-compose configuration
2. ⚠️ **Human Error Risk**: Mistakenly adding `0.0.0.0` port binding exposes service immediately
3. ⚠️ **No Defense-in-Depth for Docker**: If docker-compose misconfigured, service is exposed (no UFW safety net)
4. ⚠️ **Trust in Docker Networking**: Assumes Docker network isolation is secure (generally true, but adds dependency)
5. ⚠️ **Configuration Complexity**: Developers must understand three binding patterns (public, localhost, internal)

### Mitigation Strategies

To address the drawbacks:

1. **Code Review**: All docker-compose changes must be reviewed for correct port bindings
2. **E2E Testing**: Automated tests verify internal services are NOT externally accessible
3. **Documentation**: Clear guidelines on public vs internal service configuration
4. **Template Comments**: Explicit comments in docker-compose templates explaining binding patterns
5. **Future Work**: Consider automated validation/linting for docker-compose security (flagging `0.0.0.0` bindings on internal services)

## Alternatives Considered

### Alternative 1: Provider-Specific Cloud Firewalls

**Description**: Use cloud provider firewalls (AWS Security Groups, Hetzner Cloud Firewall, Digital Ocean Firewall, etc.) which operate at the network level and cannot be bypassed by Docker.

**Pros**:

- ✅ Defense-in-depth - firewall external to the VM
- ✅ Cannot be bypassed by Docker
- ✅ Already used successfully in Torrust Tracker Live Demo

**Cons**:

- ❌ Provider lock-in - different configuration for each provider
- ❌ Increased deployment complexity - must integrate with multiple provider APIs
- ❌ Harder to migrate between providers
- ❌ Requires provider-specific credentials and permissions
- ❌ Different security models per provider

**Decision**: Rejected. Violates the principle of provider-agnostic deployment, which is a core requirement for the deployer project.

### Alternative 2: UFW-Docker Integration Scripts

**Description**: Use community solutions like [ufw-docker](https://github.com/chaifeng/ufw-docker) that modify UFW configuration to control Docker ports.

**Pros**:

- ✅ Maintains UFW as single firewall interface
- ✅ Provides UFW control over Docker ports

**Cons**:

- ❌ Third-party dependency - not officially supported by Docker
- ❌ Brittle - breaks with Docker or UFW updates
- ❌ Adds complexity - modifies both UFW and Docker configuration
- ❌ Maintenance burden - must track upstream changes
- ❌ Harder to debug when issues occur

**Decision**: Rejected. Adds complexity and maintenance burden for questionable benefits. The layered approach is simpler and more maintainable.

### Alternative 3: Disable Docker's iptables Manipulation

**Description**: Set `iptables: false` in Docker daemon configuration to prevent Docker from modifying iptables, allowing UFW to control all ports.

**Pros**:

- ✅ UFW controls all ports, including Docker

**Cons**:

- ❌ **Breaks Docker networking** - containers cannot access external network
- ❌ No container-to-container communication across networks
- ❌ No NAT/masquerading for containers
- ❌ Requires manual iptables rules to restore Docker functionality
- ❌ Extremely complex and error-prone

**Decision**: Rejected. The Docker documentation explicitly warns this is "not appropriate for most users" and "will likely break container networking."

### Alternative 4: All Services Localhost-Only + Reverse Proxy

**Description**: Bind all services to localhost (`127.0.0.1:<port>:<port>`) and use an nginx reverse proxy for public access.

**Pros**:

- ✅ No direct public exposure of any application service
- ✅ Single public entry point (reverse proxy)
- ✅ Can add HTTPS/TLS termination
- ✅ Centralized access control

**Cons**:

- ❌ Cannot proxy UDP traffic (BitTorrent UDP tracker)
- ❌ Adds complexity - nginx configuration and management
- ❌ Additional component to maintain and monitor
- ❌ Performance overhead for proxying
- ❌ Overkill for current requirements

**Decision**: Deferred to future work. When HTTPS support is added (roadmap task 6), we will introduce a reverse proxy for HTTP services. UDP tracker will remain directly exposed. For now, the simpler approach suffices.

## Research Findings

This section addresses the technical questions raised during investigation and provides Docker-official documentation references for each finding.

### 1. Docker Network Isolation Security

**Question**: How secure is Docker's internal network isolation? Can containers on different networks communicate?

**Finding**: Docker provides strong network isolation between user-defined bridge networks:

- **User-defined networks provide better isolation**: "All containers without a `--network` specified, are attached to the default bridge network. This can be a risk, as unrelated stacks/services/containers are then able to communicate. Using a user-defined network provides a scoped network in which only containers attached to that network are able to communicate." ([Docker Bridge Driver Documentation](https://docs.docker.com/engine/network/drivers/bridge/#differences-between-user-defined-bridges-and-the-default-bridge))

- **Containers on different networks CANNOT communicate**: By default, containers on separate user-defined networks cannot reach each other unless explicitly connected to both networks. This was verified in Docker's official examples where `alpine1` (on `alpine-net`) cannot ping `alpine3` (on `bridge` network) by IP or name.

- **Port exposure within networks**: "Containers connected to the same user-defined bridge network effectively expose all ports to each other. For a port to be accessible to containers or non-Docker hosts on different networks, that port must be published using the `-p` or `--publish` flag." ([Docker Bridge Driver Documentation](https://docs.docker.com/engine/network/drivers/bridge/#differences-between-user-defined-bridges-and-the-default-bridge))

- **Connection limit**: Bridge networks become unstable with 1000+ containers due to Linux kernel limitations ([moby/moby#44973](https://github.com/moby/moby/issues/44973#issuecomment-1543747718)).

**Security Implication**: Docker network isolation is robust for our use case (small deployments with <10 containers). Internal services (Prometheus, MySQL) on a user-defined network cannot be accessed from outside the network unless ports are explicitly published.

### 2. Port Binding Risk and Safeguards

**Question**: What happens if a developer accidentally adds a port binding to an internal service? Is there any safeguard?

**Finding**: **No automated safeguards exist** - Docker immediately exposes the port publicly:

- **Publishing is insecure by default**: "Publishing container ports is insecure by default. Meaning, when you publish a container's ports it becomes available not only to the Docker host, but to the outside world as well." ([Docker Port Publishing Documentation](https://docs.docker.com/engine/network/port-publishing/#publishing-ports))

- **Immediate exposure**: When `-p 8080:80` is added to docker-compose, the service becomes publicly accessible on all host interfaces (`0.0.0.0` and `[::]`) without any confirmation or validation.

- **No built-in validation**: Docker has no mechanism to prevent or warn about accidental port exposure. The only protection is code review and testing.

**Mitigation Strategies Implemented**:

1. **Code review**: All docker-compose templates will have explicit comments marking services as PUBLIC or INTERNAL
2. **E2E security tests**: Automated tests will verify internal services are NOT externally accessible
3. **Documentation**: Clear warnings in templates about the risks of port bindings
4. **Localhost binding for debugging**: Use `127.0.0.1:9090:9090` for services that should only be accessible from the host

**Localhost binding protection**: "If you include the localhost IP address (`127.0.0.1`, or `::1`) with the publish flag, only the Docker host can access the published container port." ([Docker Port Publishing Documentation](https://docs.docker.com/engine/network/port-publishing/#publishing-ports))

**Historical Note**: In Docker versions <28.0.0, localhost bindings had a vulnerability where hosts on the same L2 segment could access them ([moby/moby#45610](https://github.com/moby/moby/issues/45610)). This was fixed in Docker 28.0.0.

### 3. iptables Priority - Can UFW Take Precedence?

**Question**: Can we configure UFW to take precedence over Docker's iptables rules?

**Answer**: **No, not without breaking Docker networking entirely**.

**Official Docker Statement**: "Docker and ufw use firewall rules in ways that make them incompatible with each other. When you publish a container's ports using Docker, traffic gets diverted before it goes through the ufw firewall settings. Docker routes container traffic in the NAT table, which means packets are diverted before reaching the INPUT and OUTPUT chains that ufw uses." ([Docker Packet Filtering Documentation](https://docs.docker.com/engine/network/packet-filtering-firewalls/))

**Why UFW can't control Docker ports**:

- Docker modifies iptables NAT table BEFORE packets reach UFW's INPUT/OUTPUT chains
- NAT table rules take precedence in the Linux netfilter packet processing order
- UFW operates on FILTER table (INPUT/OUTPUT chains), which processes packets AFTER NAT

**Alternative attempted (rejected)**: Setting `iptables: false` in Docker daemon configuration:

- **Consequence**: "This **breaks Docker networking** - containers cannot access external network, no container-to-container communication across networks, no NAT/masquerading for containers"
- **Docker's warning**: Docker documentation explicitly warns this is "not appropriate for most users" and "will likely break container networking"

**Conclusion**: We must accept that Docker bypasses UFW. The solution is to control service exposure through Docker port bindings, not through UFW rules for application ports.

### 4. Alternative Solutions Evaluation

#### 4a. Localhost Bindings with Reverse Proxy

**Question**: Could we use `127.0.0.1:<host-port>:<container-port>` bindings and nginx/reverse-proxy?

**Answer**: **Partially viable, but deferred to future work**.

**Pros**:

- ✅ No direct public exposure of application services
- ✅ Single public entry point (reverse proxy)
- ✅ Can add HTTPS/TLS termination
- ✅ Centralized access control
- ✅ Localhost binding is secure: "only the Docker host can access the published container port" ([Docker Port Publishing](https://docs.docker.com/engine/network/port-publishing/#publishing-ports))

**Cons**:

- ❌ **Cannot proxy UDP traffic** - BitTorrent UDP tracker (port 6969) cannot be proxied through HTTP/HTTPS
- ❌ Adds complexity - nginx configuration and management
- ❌ Additional component to maintain and monitor
- ❌ Performance overhead for proxying
- ❌ Overkill for current requirements

**Decision**: Deferred until HTTPS support is added (roadmap task 6). At that point, HTTP services (tracker HTTP API, Grafana) will route through a reverse proxy, but UDP tracker will remain directly exposed.

#### 4b. Provider-Specific Firewalls

**Question**: Should we integrate with provider-specific firewalls despite complexity?

**Answer**: **No - violates core portability requirement**.

Provider-specific firewalls (AWS Security Groups, Hetzner Cloud Firewall, Digital Ocean Firewall) operate at the network level BEFORE traffic reaches the VM, so Docker cannot bypass them. However:

**Cons**:

- ❌ **Provider lock-in** - different configuration for each provider
- ❌ Increased deployment complexity - must integrate with multiple provider APIs
- ❌ Harder to migrate between providers
- ❌ Requires provider-specific credentials and permissions
- ❌ Different security models per provider

**Precedent**: The Torrust Tracker Live Demo used Digital Ocean's firewall successfully, but this violated the deployer's portability goal.

**Decision**: Rejected. Maintain provider-agnostic deployment approach using UFW + Docker port bindings.

#### 4c. Docker Built-in Firewall Features

**Question**: Can we use Docker's built-in firewall features (docker-proxy, etc.)?

**Answer**: **Already using them - Docker's bridge network isolation**.

Docker's built-in security features we rely on:

1. **Bridge network isolation**: Containers on different networks cannot communicate unless explicitly connected
2. **Port publishing control**: Only explicitly published ports are accessible from outside the host
3. **Gateway modes**: Control NAT behavior and direct routing ([Docker Gateway Modes](https://docs.docker.com/engine/network/port-publishing/#gateway-modes))

**Additional Docker feature (not used)**: `--internal` network flag creates networks with no external access, but this is too restrictive for our needs (containers need internet access for package downloads).

**Conclusion**: We are already leveraging Docker's isolation features. The chosen strategy (layered security with UFW + Docker port bindings + Docker networks) is the correct use of Docker's native security mechanisms.

### 5. Testing Strategy for Port Exposure Verification

**Question**: How do we automatically verify no unintended ports are exposed during E2E tests?

**Answer**: **Multi-layer verification approach**.

**Test Strategy**:

1. **External Port Scanning**:
   - Use `nmap` or `netstat` from external host to scan VM's public IP
   - Verify ONLY expected ports are open (SSH, tracker UDP/HTTP, Grafana)
   - Verify internal service ports (Prometheus 9090, MySQL 3306) are NOT accessible

2. **Docker Network Inspection**:
   - Query `docker port <container>` to list published ports
   - Parse `docker inspect` output to verify port bindings match specification
   - Fail if any internal service has unexpected `HostPort` mapping

3. **Service Accessibility Tests**:
   - **From external host**: Verify public services respond (tracker, Grafana)
   - **From external host**: Verify internal services timeout/refuse (Prometheus, MySQL)
   - **From Docker host**: Verify localhost bindings work (`curl 127.0.0.1:9090` succeeds)
   - **From Docker network**: Verify internal service discovery works (`curl http://prometheus:9090` from Grafana container)

4. **UFW Configuration Verification**:
   - Verify UFW default policy is `deny incoming`
   - Verify ONLY SSH port is allowed in UFW rules
   - Document that application ports should NOT appear in `ufw status`

**Test Implementation Plan** (Phase 4):

```rust
// Example test structure
#[test]
fn it_should_block_external_access_to_prometheus_when_deployed() {
    // 1. Deploy environment with Prometheus
    // 2. Get VM public IP
    // 3. Attempt to connect to http://<vm-ip>:9090 from external host
    // 4. Assert connection timeout or refused
    // 5. Verify from VM host: curl 127.0.0.1:9090 succeeds
}

#[test]
fn it_should_allow_external_access_to_grafana_when_deployed() {
    // 1. Deploy environment with Grafana
    // 2. Get VM public IP and configured Grafana port
    // 3. Connect to http://<vm-ip>:3000
    // 4. Assert successful connection and Grafana login page
}

#[test]
fn it_should_verify_no_unexpected_ports_are_published() {
    // 1. Deploy environment
    // 2. Run: docker ps --format '{{.Names}}: {{.Ports}}'
    // 3. Parse output and extract all published ports
    // 4. Compare against whitelist of expected public ports
    // 5. Fail if any internal service has HostPort mapping
}
```

**Automated CI/CD Integration**:

- E2E security tests run on every PR that modifies docker-compose templates
- Tests use LXD VMs in GitHub Actions runners
- Security test failures block PR merges

## Security Analysis

This section addresses critical security questions about the chosen approach.

### 1. Threat Model: Attack Vectors

#### Attack Vector 1: Misconfigured docker-compose Exposing Internal Services

- **Risk**: Developer accidentally adds port binding to internal service (e.g., `ports: - "3306:3306"` to MySQL)
- **Impact**: 🔴 CRITICAL - Internal service immediately exposed to internet
- **Likelihood**: MEDIUM - Human error during template modification
- **Mitigation**:
  - ✅ Code review process - all docker-compose changes require review
  - ✅ Automated E2E security tests - verify no unintended port exposure
  - ✅ Template comments - clearly mark services as PUBLIC or INTERNAL
  - ✅ Pre-commit linting - could add custom rule to detect risky patterns
  - ✅ CI/CD blocking - security test failures prevent merge

#### Attack Vector 2: Docker Daemon Compromise

- **Risk**: Attacker gains root access to Docker host, compromises Docker daemon
- **Impact**: 🔴 CRITICAL - Complete system compromise, all containers accessible
- **Likelihood**: LOW - Requires root-level host compromise
- **Mitigation**:
  - ✅ UFW blocks all incoming except SSH - reduces attack surface
  - ✅ SSH key authentication only - no password authentication
  - ✅ Docker daemon not exposed externally - listens on unix socket only
  - ✅ Regular security updates - keep Docker Engine patched
  - ✅ Host hardening - minimal installed packages, fail2ban for SSH protection
  - ⚠️ NOT MITIGATED: If host compromised, game over (unavoidable)

#### Attack Vector 3: Container Escape Vulnerabilities

- **Risk**: CVE in Docker allows container escape (e.g., runc vulnerability)
- **Impact**: 🔴 CRITICAL - Container can escape to host, gain root access
- **Likelihood**: LOW - Rare but documented (CVE-2019-5736, etc.)
- **Mitigation**:
  - ✅ Use latest Docker images - apply security patches promptly
  - ✅ Run containers as non-root user - `USER_ID=1000` in tracker
  - ✅ Drop capabilities - `cap_drop: ALL` where possible
  - ✅ Read-only root filesystem - limits post-escape damage
  - ✅ Security monitoring - detect unusual container behavior
  - ⚠️ NOT FULLY MITIGATED: Zero-day exploits cannot be prevented

#### Attack Vector 4: Compromised Public Service (Grafana/Tracker)

- **Risk**: CVE in Grafana or Tracker allows remote code execution
- **Impact**: 🟡 MEDIUM to 🔴 HIGH - Depends on network segmentation
- **Likelihood**: MEDIUM - Public services are attack targets
- **Mitigation**:
  - ✅ Network segmentation - limits lateral movement (see network analysis)
  - ✅ Regular updates - patch vulnerable images promptly
  - ✅ Minimal privileges - non-root users, dropped capabilities
  - ✅ Monitoring and alerting - detect suspicious activity
  - ✅ Credential rotation - minimize credential exposure window

#### Attack Vector 5: Dependency Vulnerabilities

- **Risk**: Vulnerable library in base image or application dependencies
- **Impact**: Varies by vulnerability (RCE, DoS, info disclosure)
- **Likelihood**: HIGH - Common in complex dependency chains
- **Mitigation**:
  - ✅ Docker Scout scanning - detect CVEs in images
  - ✅ Automated dependency updates - Dependabot/Renovate
  - ✅ Minimal base images - reduce attack surface (alpine, distroless)
  - ✅ Regular security audits - review dependency chains

**Residual Risks** (cannot be fully mitigated):

- Zero-day exploits in Docker, kernel, or dependencies
- Social engineering to obtain credentials or .env files
- Physical access to infrastructure
- Insider threats with legitimate access

### 2. Compliance: Security Best Practices

**Does this approach meet production security standards?**

**✅ Meets Best Practices**:

1. **Defense in Depth**:
   - UFW firewall (host level)
   - Docker port bindings (service level)
   - Docker network segmentation (lateral movement prevention)
   - Application authentication (Grafana, Tracker API)

2. **Principle of Least Privilege**:
   - Services only have network access to what they need
   - Non-root container users
   - Minimal capabilities

3. **Security by Default**:
   - Internal services not exposed (MySQL, Prometheus)
   - Public services explicitly configured
   - Localhost-only bindings for debugging services

4. **Auditability**:
   - Explicit port bindings documented
   - Network topology documented in templates
   - Security decisions documented in ADR

5. **Operational Security**:
   - Credentials in environment variables (upgradeable to Docker secrets)
   - Secrets not in code/templates
   - Monitoring and logging enabled

**⚠️ Areas for Improvement**:

1. **Secrets Management**: Environment variables → Docker Secrets or HashiCorp Vault
2. **TLS/HTTPS**: Future roadmap item (task 6) - reverse proxy with TLS termination
3. **Network Policies**: Could add iptables rules for additional granularity
4. **Intrusion Detection**: Could add OSSEC, Wazuh, or similar HIDS
5. **Backup Encryption**: Database backups should be encrypted at rest

**Compliance Frameworks**:

- ✅ OWASP Docker Security Cheat Sheet - majority of recommendations followed
- ✅ CIS Docker Benchmark - baseline security practices met
- ⚠️ PCI-DSS / SOC 2 - would require additional controls (secrets management, encryption, audit logging)
- ⚠️ HIPAA / GDPR - would require additional controls (data encryption, access logging, right to deletion)

**Conclusion**: Suitable for production use in non-regulated environments. Additional controls needed for compliance-heavy industries.

### 3. Monitoring: Detecting Accidental Exposure

**How do we detect if internal services become accidentally exposed?**

**Detection Mechanisms**:

1. **Automated E2E Security Tests** (implemented in Phase 4):

   ```rust
   #[test]
   fn it_should_detect_unexpected_port_exposure() {
       // Run nmap scan from external host
       // Parse open ports
       // Assert only expected ports open (22, 6969, 7070, 1212, 3000)
       // Fail if MySQL (3306) or Prometheus (9090) detected
   }
   ```

2. **Docker Port Inspection** (script for manual/automated use):

   ```bash
   #!/bin/bash
   # scripts/check-port-exposure.sh

   EXPECTED_PORTS="22,6969,7070,1212,3000"
   FORBIDDEN_PORTS="3306,9090"

   # Check docker ps for published ports
   PUBLISHED=$(docker ps --format '{{.Ports}}' | grep -oE '\d+:\d+' | cut -d: -f1 | sort -u)

   for port in $FORBIDDEN_PORTS; do
     if echo "$PUBLISHED" | grep -q "^${port}$"; then
       echo "ERROR: Forbidden port $port is exposed!"
       exit 1
     fi
   done
   ```

3. **UFW Status Monitoring**:

   ```bash
   # Verify UFW only allows SSH
   sudo ufw status numbered | grep -v "22" | grep "ALLOW"
   # Should return empty - only SSH allowed
   ```

4. **Network Scanning (External)**:

   ```bash
   # From external machine, scan VM
   nmap -p- <vm-ip>

   # Expected open ports:
   # 22 (SSH)
   # 6969 (UDP tracker)
   # 7070 (HTTP tracker)
   # 1212 (REST API)
   # 3000 (Grafana)

   # Unexpected = security issue
   ```

5. **Prometheus Metrics** (future enhancement):

   ```yaml
   # Alert on unexpected port bindings
   - alert: UnexpectedPortExposed
     expr: node_network_receive_bytes_total{port="3306"} > 0
     annotations:
       summary: "MySQL port is receiving external traffic"
   ```

6. **Docker Events Monitoring**:

   ```bash
   # Monitor for container starts with unexpected port configs
   docker events --filter 'event=start' --format '{{.Actor.Attributes.name}}: {{.Actor.Attributes.image}}'
   ```

**Monitoring Frequency**:

- **CI/CD**: On every PR that modifies docker-compose templates
- **Production**: Daily automated security scan (cron job)
- **Real-time**: Docker events monitoring (if implemented)
- **Manual**: Weekly security audit checklist

**Alerting Channels**:

- CI/CD: GitHub Actions failure notification
- Production: Email/Slack alerts for security scan failures
- Real-time: Prometheus Alertmanager (if monitoring implemented)

### 4. Recovery: Remediation Process

**If a service is accidentally exposed, what's the remediation process?**

**Incident Response Workflow**:

**Step 1: Detection and Verification** (minutes)

```bash
# Verify the exposure
nmap -p- <vm-ip>

# Check docker-compose configuration
docker ps --format 'table {{.Names}}\t{{.Ports}}'

# Identify the misconfigured service
grep -r "3306:3306" build/*/docker-compose/
```

**Step 2: Immediate Mitigation** (minutes)

```bash
# Option A: Stop the exposed container
docker stop <container-name>

# Option B: Reconfigure port binding to localhost
# Edit docker-compose.yml: "3306:3306" → "127.0.0.1:3306:3306"
docker-compose up -d <container-name>

# Option C: Add temporary UFW rule (not recommended - doesn't fix root cause)
sudo ufw deny 3306
```

**Step 3: Root Cause Analysis** (hours)

- Review git history: When was the misconfiguration introduced?
- Identify who made the change and why
- Determine if exposure was exploited (check access logs)
- Document timeline and impact

**Step 4: Fix and Deploy** (hours)

```bash
# Fix the template
vim templates/docker-compose/docker-compose.yml.tera
# Remove port binding or change to localhost

# Regenerate templates
cargo run -- create templates <environment-name>

# Deploy fix
cargo run -- release <environment-name>
cargo run -- run <environment-name>

# Verify fix
nmap -p- <vm-ip>  # Should NOT show the previously exposed port
```

**Step 5: Security Assessment** (days)

- Review access logs for unauthorized access attempts
- Check database logs for suspicious queries
- Rotate credentials if exposure confirmed
- Perform security audit of all services

**Step 6: Prevention** (days)

- Add automated test to detect this specific exposure
- Update code review checklist
- Improve template comments/warnings
- Conduct team security training

**Rollback Procedure** (if deployment broken):

```bash
# Revert to previous working state
git revert <bad-commit-hash>

# Or restore from backup
cargo run -- create templates <environment-name>
# (using old environment.json from backup)

cargo run -- release <environment-name>
cargo run -- run <environment-name>
```

**Credential Rotation** (if compromise suspected):

```bash
# Generate new passwords
NEW_MYSQL_PASSWORD=$(openssl rand -base64 32)
NEW_ADMIN_TOKEN=$(openssl rand -base64 32)

# Update .env file
vim .env
# MYSQL_ROOT_PASSWORD=<new-password>
# TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=<new-token>

# Restart services with new credentials
docker-compose down
docker-compose up -d
```

**Post-Incident Actions**:

1. Update incident response documentation
2. Add regression test for this scenario
3. Conduct post-mortem meeting
4. Document lessons learned
5. Update monitoring/alerting rules

**Communication Plan**:

- Internal: Notify development team immediately
- Users: If exposure confirmed and data accessed, notify users per disclosure policy
- Stakeholders: Report incident to security team/management

## Implementation Phases

### Phase 1: Research and Analysis ✅ Complete

- Reviewed Docker official documentation
- Analyzed UFW-Docker interaction
- Documented threat model and alternatives
- Created this ADR

### Phase 2: Template Implementation (This ADR)

- Remove obsolete tracker firewall configuration files
- Update docker-compose templates with security comments
- Ensure correct port binding patterns across all services

### Phase 3: Testing

- Add E2E security tests to verify internal services NOT externally accessible
- Verify public services ARE externally accessible
- Document testing procedures

### Phase 4: Documentation

- Update `docs/user-guide/security.md` with new strategy
- Add warnings about Docker port binding risks
- Document localhost binding pattern for debugging

## Related Decisions

- None yet - this is the foundational security decision

## References

### Official Documentation

- **[Docker: Packet filtering and firewalls](https://docs.docker.com/engine/network/packet-filtering-firewalls/)** - Essential reading on Docker-UFW incompatibility
- [Docker with iptables](https://docs.docker.com/engine/network/firewall-iptables/) - Technical details on Docker's iptables integration
- [Docker with nftables](https://docs.docker.com/engine/network/firewall-nftables/) - Alternative firewall backend

### Related Issues

- [Issue #246 - Grafana slice](https://github.com/torrust/torrust-tracker-deployer/issues/246) - Where this issue was re-discovered
- [Issue #248 - Docker UFW Firewall Security Strategy](https://github.com/torrust/torrust-tracker-deployer/issues/248) - Implementation tracking issue
- [torrust-demo#72 - Docker bypassing systemd-resolved](https://github.com/torrust/torrust-demo/issues/72) - Related Docker bypass issue

### Community Resources

- [UFW and Docker GitHub Discussion](https://github.com/docker/for-linux/issues/690) - Known interactions and issues
- [UFW-Docker Community Solution](https://github.com/chaifeng/ufw-docker) - Third-party integration approach
- [TechRepublic: Docker and Firewall Security Flaw](https://www.techrepublic.com/article/how-to-fix-the-docker-and-ufw-security-flaw/)

### Internal Documentation

- [Manual Grafana Testing Results](../e2e-testing/manual/grafana-testing-results.md) - Evidence of security issue
- [Issue Specification](../issues/248-docker-ufw-firewall-security-strategy.md) - Detailed implementation plan
