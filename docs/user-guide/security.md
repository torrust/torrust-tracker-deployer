# Security Guide

This guide covers security considerations and best practices when deploying Torrust Tracker using the deployer.

## Overview

Security is a critical aspect of production deployments. The Torrust Tracker Deployer implements several security measures automatically during the deployment process, with additional considerations for production environments.

## Firewall Configuration

### Automatic Firewall Setup

**CRITICAL**: The `configure` command automatically configures UFW (Uncomplicated Firewall) on virtual machines to protect internal services from unauthorized external access.

During the `configure` step, the deployer:

1. **Installs UFW** - Ensures the firewall is available
2. **Sets restrictive policies** - Denies all incoming traffic by default
3. **Allows SSH access** - Preserves SSH connectivity (configured port)
4. **Allows tracker services** - Opens only necessary tracker ports:
   - UDP tracker ports (configured in environment)
   - HTTP tracker ports (configured in environment)
   - HTTP API port (configured in environment)
5. **Enables the firewall** - Activates rules to protect the system

### Why Firewall Configuration Matters

The Docker Compose configuration (`templates/docker-compose/docker-compose.yml.tera`) exposes several service ports that should **NOT** be publicly accessible:

**Exposed Ports in Docker Compose**:

```yaml
services:
  # Tracker - Public ports (UDP/HTTP tracker, HTTP API)
  tracker:
    ports:
      - "6969:6969/udp" # ✅ Public - UDP tracker
      - "7070:7070" # ✅ Public - HTTP tracker
      - "1212:1212" # ✅ Public - HTTP API

  # Prometheus - INTERNAL ONLY
  prometheus:
    ports:
      - "9090:9090" # ⚠️ INTERNAL - Metrics UI

  # MySQL - INTERNAL ONLY
  mysql:
    ports:
      - "3306:3306" # ⚠️ INTERNAL - Database
```

**Without firewall protection**, services like Prometheus (port 9090) and MySQL (port 3306) would be accessible from the internet, potentially exposing:

- **Prometheus** - Internal metrics, performance data, system topology
- **MySQL** - Database access (even with authentication, this is a security risk)

**With firewall protection** (UFW configured by `configure` command):

- ✅ **Tracker ports** - Accessible externally (UDP tracker, HTTP tracker, HTTP API)
- 🔒 **Prometheus port** - Blocked from external access
- 🔒 **MySQL port** - Blocked from external access
- ✅ **SSH access** - Preserved for administration

### E2E Testing vs Production

**E2E Testing (Docker Containers)**:

- Uses Docker containers instead of VMs for faster test execution
- Firewall **NOT** configured inside containers (containers provide isolation)
- Services exposed for testing purposes
- ⚠️ **NOT suitable for production use**

**Production Deployments (Virtual Machines)**:

- Uses real VMs (LXD, cloud providers)
- Firewall **automatically configured** by `configure` command
- Only tracker services exposed externally
- ✅ **Production-ready security posture**

### Firewall Rules Applied

The deployer configures these firewall rules during the `configure` step:

```bash
# SSH Access (required for management)
ufw allow <ssh-port>/tcp

# UDP Tracker Ports (configured in environment)
ufw allow <udp-port>/udp

# HTTP Tracker Ports (configured in environment)
ufw allow <http-port>/tcp

# HTTP API Port (configured in environment)
ufw allow <api-port>/tcp

# Default policies
ufw default deny incoming   # Block everything else
ufw default allow outgoing  # Allow outbound connections
```

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
