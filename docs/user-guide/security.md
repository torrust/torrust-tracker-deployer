# Security Guide

This guide covers security considerations and best practices when deploying Torrust Tracker using the deployer.

## Overview

Security is a critical aspect of production deployments. The Torrust Tracker Deployer implements several security measures automatically during the deployment process, with additional considerations for production environments.

## Firewall Configuration

### Layered Security Approach

**CRITICAL**: The deployer uses a **layered security approach** combining UFW firewall and Docker networking to protect your deployment. Understanding how these layers work together is essential for secure deployments.

#### Security Architecture

The deployer implements security at two levels:

1. **Instance-Level Security (UFW)** - Protects the VM itself

   - Denies all incoming traffic by default
   - Allows only SSH access for administration
   - **Does NOT control Docker container ports** (Docker bypasses UFW)

2. **Service-Level Security (Docker)** - Controls service exposure
   - Public services have explicit port bindings (Tracker, Grafana)
   - Internal services have NO port bindings (MySQL)
   - Localhost-only services bind to `127.0.0.1` (Prometheus)
   - Docker network segmentation isolates service communication

#### Why UFW Cannot Protect Docker Ports

**Important**: Docker manipulates iptables directly and **bypasses UFW rules** for published container ports. This is documented behavior (see [Docker documentation](https://docs.docker.com/engine/network/packet-filtering-firewalls/)).

```yaml
# This port binding BYPASSES UFW firewall rules
services:
  mysql:
    ports:
      - "3306:3306" # ⚠️ PUBLICLY ACCESSIBLE despite UFW rules!
```

Docker routes container traffic in the NAT table, meaning packets are diverted before reaching the INPUT and OUTPUT chains that UFW uses. Therefore:

- ✅ UFW protects the VM and SSH access
- ❌ UFW **does not** protect Docker-published ports
- ✅ Docker port bindings control service exposure

### Automatic Firewall Setup

During the `configure` command, the deployer:

1. **Installs UFW** - Ensures the firewall is available
2. **Sets restrictive policies** - Denies all incoming traffic by default
3. **Allows SSH access** - Preserves SSH connectivity (configured port)
4. **Enables the firewall** - Activates rules to protect SSH access

**Note**: UFW only controls SSH access. Application ports are controlled by Docker port bindings in the docker-compose configuration.

###Service Exposure Strategy

### Service Exposure Strategy

The Docker Compose configuration (`templates/docker-compose/docker-compose.yml.tera`) controls which services are accessible from the internet through **explicit port bindings**:

**Service Exposure Levels**:

```yaml
services:
  # ✅ PUBLIC SERVICES - Explicit port bindings
  tracker:
    ports:
      - "6969:6969/udp" # Public - UDP tracker
      - "7070:7070" # Public - HTTP tracker
      - "1212:1212" # Public - REST API

  grafana:
    ports:
      - "3100:3000" # Public - Monitoring UI (authenticated)

  # 🔒 LOCALHOST-ONLY SERVICES - Bound to 127.0.0.1
  prometheus:
    ports:
      - "127.0.0.1:9090:9090" # Accessible only from VM host

  # 🔒 INTERNAL-ONLY SERVICES - No port bindings
  mysql:
    # No ports section - completely internal
    # Accessed via Docker network: mysql:3306
```

**Security Properties**:

- **Public Services** - Have `ports:` section binding to `0.0.0.0` (accessible externally)
- **Localhost Services** - Bind to `127.0.0.1` (accessible only from VM host via SSH)
- **Internal Services** - No port bindings (accessible only via Docker internal networks)

### Network Segmentation

The deployer implements **three isolated Docker networks** for defense-in-depth security:

```yaml
networks:
  database_network: # Tracker ↔ MySQL only
  metrics_network: # Tracker ↔ Prometheus only
  visualization_network: # Prometheus ↔ Grafana only

services:
  tracker:
    networks:
      - database_network # Can access MySQL
      - metrics_network # Can be scraped by Prometheus

  mysql:
    networks:
      - database_network # Isolated - only Tracker can access

  prometheus:
    networks:
      - metrics_network # Can scrape Tracker metrics
      - visualization_network # Can be queried by Grafana

  grafana:
    networks:
      - visualization_network # Can query Prometheus only
```

**Security Benefits**:

1. **Reduced Attack Surface**: MySQL accessible from 1 service (Tracker) instead of 3 services
2. **Lateral Movement Prevention**: Compromised Grafana cannot access MySQL or Tracker
3. **Principle of Least Privilege**: Services can only communicate where necessary
4. **Compliance**: Aligns with PCI DSS, NIST 800-53, CIS Docker Benchmark

### Security Comparison

**Without proper configuration** ⚠️:

```yaml
# INSECURE - All services on one network with public port bindings
services:
  mysql:
    ports:
      - "3306:3306" # ⚠️ MySQL publicly accessible!
    networks:
      - backend_network
```

- ❌ Internal services exposed to internet
- ❌ All services can communicate (no segmentation)
- ❌ Docker bypasses UFW firewall rules
- ❌ High attack surface

**With deployer configuration** ✅:

```yaml
# SECURE - Network segmentation + no public port bindings
services:
  mysql:
    # No ports section - internal only
    networks:
      - database_network # Only Tracker can access
```

- ✅ Internal services not publicly accessible
- ✅ Network segmentation limits lateral movement
- ✅ UFW protects SSH access
- ✅ Reduced attack surface

### E2E Testing vs Production

**E2E Testing (Docker Containers)**:

- Uses Docker containers for faster test execution
- Firewall **not configured** inside containers (container isolation sufficient)
- Services may be exposed for testing purposes
- ⚠️ **NOT production-grade security**

**Production Deployments (Virtual Machines)**:

- Uses real VMs (LXD, cloud providers)
- UFW **configured automatically** for SSH protection
- Docker port bindings control service exposure
- Network segmentation isolates services
- ✅ **Production-ready security**

### Firewall Rules Applied

The deployer configures these UFW firewall rules during `configure`:

```bash
# SSH Access (required for administration)
ufw allow <ssh-port>/tcp

# Default policies
ufw default deny incoming   # Block all incoming traffic
ufw default allow outgoing  # Allow outbound connections
ufw enable                  # Activate firewall
```

**Note**: Application ports (Tracker, Grafana, Prometheus, MySQL) are **not** managed by UFW. They are controlled by Docker port bindings in the docker-compose.yml configuration.

### Security Best Practices

**DO**:

- ✅ Use the deployer's default docker-compose template (network segmentation included)
- ✅ Review port bindings before deploying (`build/{env}/docker-compose/docker-compose.yml`)
- ✅ Keep internal services without port bindings (MySQL)
- ✅ Use `127.0.0.1` bindings for localhost-only access (Prometheus)
- ✅ Apply security updates to the VM regularly
- ✅ Use strong SSH keys and disable password authentication
- ✅ Monitor logs for suspicious activity

**DON'T**:

- ❌ Add port bindings to internal services (e.g., `3306:3306` for MySQL)
- ❌ Disable UFW firewall on production VMs
- ❌ Remove network segmentation from docker-compose.yml
- ❌ Assume UFW protects Docker-published ports
- ❌ Expose Prometheus/MySQL publicly
- ❌ Use default passwords for services

### Verifying Firewall Configuration

After running the `configure` command, verify firewall rules:

```bash
# SSH into your VM
INSTANCE_IP=$(cat data/<env-name>/environment.json | jq -r '.Configured.context.runtime_outputs.instance_ip')
ssh -i <private-key> <username>@$INSTANCE_IP

# Check UFW status
sudo ufw status numbered

# Expected output shows:
# - SSH port allowed
# - Tracker ports allowed (UDP/HTTP/API)
# - Default deny incoming policy
# - All other ports blocked
```

**Example output**:

```text
Status: active

     To                         Action      From
     --                         ------      ----
[ 1] 22/tcp                     ALLOW IN    Anywhere
[ 2] 6969/udp                   ALLOW IN    Anywhere
[ 3] 7070/tcp                   ALLOW IN    Anywhere
[ 4] 1212/tcp                   ALLOW IN    Anywhere
```

Note that ports 9090 (Prometheus) and 3306 (MySQL) are **not** in this list, meaning they are blocked from external access.

## SSH Security

### SSH Key Authentication

The deployer requires SSH key-based authentication for VM access:

**Best Practices**:

1. **Use strong SSH keys** - Generate RSA keys with at least 4096 bits:

   ```bash
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/torrust_deploy
   ```

2. **Protect private keys** - Set restrictive permissions:

   ```bash
   chmod 600 ~/.ssh/torrust_deploy
   ```

3. **Use dedicated keys** - Don't reuse personal SSH keys for deployments

4. **Rotate keys regularly** - Update SSH keys periodically

### SSH Port Configuration

The default SSH port (22) is commonly targeted by automated attacks. Consider using a custom port:

```json
{
  "ssh_credentials": {
    "port": 2222 // Custom SSH port
  }
}
```

**Trade-offs**:

- ✅ Reduces automated attack attempts
- ✅ Adds minimal security through obscurity
- ⚠️ Must remember custom port for manual access
- ⚠️ Not a substitute for strong authentication

## Docker Security Considerations

### Container Isolation

Services run in isolated Docker containers with:

- **Network isolation** - Backend network for inter-container communication
- **Volume mounts** - Limited filesystem access with `:Z` SELinux labels
- **Resource limits** - Logging limits prevent disk exhaustion
- **Restart policies** - Automatic recovery from failures

### Image Security

**Current Images**:

- `torrust/tracker:develop` - Torrust Tracker (development tag)
- `prom/prometheus:v3.0.1` - Prometheus (pinned version)
- `mysql:8.0` - MySQL (major version pinned)

**Recommendations**:

1. **Pin specific versions** - Use exact version tags in production
2. **Scan images regularly** - Check for known vulnerabilities
3. **Update periodically** - Apply security patches
4. **Use official images** - Prefer official/verified images

### Environment Variables

Sensitive configuration is managed via `.env` files on the VM:

**Best Practices**:

1. **Strong passwords** - Use complex, randomly generated passwords
2. **Unique credentials** - Different passwords per environment
3. **Secure storage** - Never commit `.env` files to version control
4. **Rotation policy** - Update passwords periodically

**Example** (DO NOT use these values):

```bash
# Bad - Weak passwords
MYSQL_ROOT_PASSWORD=password123
MYSQL_PASSWORD=tracker

# Good - Strong, unique passwords
MYSQL_ROOT_PASSWORD=7k#mP9$vL2@qX5nR8jW
MYSQL_PASSWORD=xF4!hT6@dN9$sK2mQ7wE
```

## Network Security

### Service Exposure

The deployer follows the principle of least exposure:

**Public Services** (accessible externally):

- UDP Tracker - Required for BitTorrent protocol
- HTTP Tracker - Required for HTTP-based tracker operations
- HTTP API - Required for tracker management and metrics

**Internal Services** (blocked by firewall):

- Prometheus UI - Metrics collection (internal monitoring only)
- MySQL Database - Data storage (internal access only)

### Internal Communication

Services communicate via Docker's `backend_network`:

- Container-to-container communication allowed
- Isolated from host network by default
- DNS resolution via container names (e.g., `tracker`, `mysql`, `prometheus`)

## Production Security Checklist

Before deploying to production, verify:

### Infrastructure Security

- [ ] **Virtual machines used** (not Docker containers for testing)
- [ ] **Firewall configured** (`configure` command completed successfully)
- [ ] **SSH key authentication** (password authentication disabled)
- [ ] **Custom SSH port** (optional but recommended)
- [ ] **Firewall rules verified** (`ufw status` shows expected rules)

### Credential Security

- [ ] **Strong SSH keys** (4096-bit RSA minimum)
- [ ] **Strong database passwords** (randomly generated, complex)
- [ ] **Unique API tokens** (per environment, rotated regularly)
- [ ] **No credentials in git** (`.env` files gitignored)
- [ ] **Secure key storage** (restricted permissions on private keys)

### Application Security

- [ ] **Pinned image versions** (not using `latest` or `develop` tags)
- [ ] **Image scanning enabled** (vulnerability checks in CI/CD)
- [ ] **Logging configured** (audit trail and debugging)
- [ ] **Resource limits set** (prevent resource exhaustion)
- [ ] **Regular updates scheduled** (security patches applied)

### Monitoring Security

- [ ] **Prometheus UI not exposed** (firewall blocks port 9090)
- [ ] **Database not exposed** (firewall blocks port 3306)
- [ ] **Access logs reviewed** (regular security audits)
- [ ] **Metrics monitored** (unusual patterns detected)

## Security Incident Response

If you suspect a security breach:

1. **Isolate the system** - Disable network access if necessary
2. **Check logs** - Review `data/logs/log.txt` and container logs
3. **Review firewall rules** - Verify UFW configuration hasn't changed
4. **Rotate credentials** - Update all passwords and keys immediately
5. **Update software** - Apply latest security patches
6. **Report vulnerabilities** - Contact maintainers for Torrust Tracker issues

## Future Security Enhancements

Planned improvements for future releases:

- **TLS/SSL support** - HTTPS for HTTP tracker and API
- **Certificate management** - Automated Let's Encrypt integration
- **Rate limiting** - Protection against abuse
- **Fail2ban integration** - Automated IP blocking for failed attempts
- **Security scanning** - Automated vulnerability detection in CI/CD
- **Audit logging** - Detailed access logs for compliance

## Additional Resources

### Related Documentation

- **[User Guide](README.md)** - Main deployment guide
- **[Configuration Guide](configuration/)** - Environment configuration details
- **[Services Guide](services/)** - Service-specific security considerations

### External Resources

- **[UFW Documentation](https://help.ubuntu.com/community/UFW)** - Firewall configuration
- **[Docker Security Best Practices](https://docs.docker.com/engine/security/)** - Container security
- **[SSH Hardening Guide](https://www.ssh.com/academy/ssh/security)** - SSH security best practices
- **[OWASP Top 10](https://owasp.org/www-project-top-ten/)** - Web application security risks

## Questions or Concerns?

Security is an ongoing process. If you have questions or discover security issues:

- **Security Issues** - Report privately to maintainers (do not open public issues)
- **General Questions** - [GitHub Discussions](https://github.com/torrust/torrust-tracker-deployer/discussions)
- **Feature Requests** - [GitHub Issues](https://github.com/torrust/torrust-tracker-deployer/issues)

Stay secure! 🔒
