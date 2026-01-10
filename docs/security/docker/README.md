# Docker Image Security Scanning Guide

This document explains how to perform security scans on Docker images used in the deployer.

## Purpose

Regular security scanning ensures that Docker images used in production deployments are free from known vulnerabilities. This documentation provides:

- Instructions for running security scans
- Configuration guidelines
- Best practices for vulnerability management

## Automated Scanning

For ongoing security monitoring, see [Issue #250: Implement periodic security vulnerability scanning workflow](https://github.com/torrust/torrust-tracker-deployer/issues/250).

The automated workflow will:

- Run Trivy scans on CI/CD pipeline
- Generate security reports
- Alert on new vulnerabilities
- Track vulnerability trends over time

## Manual Scanning with Trivy

### Installation

```bash
# macOS
brew install trivy

# Linux (Debian/Ubuntu)
sudo apt-get install trivy

# Or use Docker
docker run --rm aquasec/trivy:latest image <image-name>
```

### Scan Configuration

**Recommended Scan Command**:

```bash
trivy image --severity HIGH,CRITICAL <image-name>
```

**Severity Levels**:

- `CRITICAL`: Exploitable vulnerabilities with severe impact
- `HIGH`: Significant vulnerabilities requiring attention
- `MEDIUM`: Moderate vulnerabilities (optional to include)
- `LOW`: Minor vulnerabilities (typically noise)

### Example Scans

```bash
# Scan the deployer image
trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:latest

# Scan with all severities for full report
trivy image torrust/tracker-deployer:latest

# Scan and output as JSON
trivy image --format json --output report.json torrust/tracker-deployer:latest

# Scan specific image version
trivy image --severity HIGH,CRITICAL prom/prometheus:v3.5.0
```

## Trivy Warning Messages Explained

### Common Warnings (Not Security Issues)

**"OS is not detected"** (Prometheus):

- Expected for minimal scratch images
- Application binary has zero vulnerabilities
- No OS packages to scan

**"Alpine/Oracle Linux no longer supported"**:

- Cosmetic warning from Trivy's detection heuristics
- Official images are actively maintained by vendors
- Zero vulnerabilities confirm images are secure

## When to Act

**If HIGH/CRITICAL vulnerabilities appear**:

1. Review vulnerability details in Trivy output
2. Check if vendor has released patched image
3. Update image version in `templates/docker-compose/docker-compose.yml.tera`
4. Re-run security scan to verify fix
5. Update scan documentation with new results

## Security Best Practices

### Image Selection

- ✅ Use official vendor images (prom, grafana, mysql, torrust)
- ✅ Pin to specific versions (not `latest` tags in production)
- ✅ Prefer LTS versions for production stability
- ✅ Verify support EOL dates before deployment

### Regular Scanning

- 🔄 Scan images before deployment
- 🔄 Re-scan periodically (monthly recommended)
- 🔄 Monitor vendor security advisories
- 🔄 Update images when patches available

### Documentation

- 📝 Record scan dates and results in [scans/](scans/)
- 📝 Document update rationale
- 📝 Track support lifecycle dates
- 📝 Maintain historical scan records

## Historical Scan Results

See the [scans/](scans/) directory for historical security scan results:

- [Torrust Tracker Deployer](scans/torrust-tracker-deployer.md)
- [Prometheus](scans/prometheus.md)
- [Grafana](scans/grafana.md)
- [MySQL](scans/mysql.md)

## References

- [Trivy Documentation](https://aquasecurity.github.io/trivy/)
- [Issue #250: Automated Security Scanning](https://github.com/torrust/torrust-tracker-deployer/issues/250)
- [Issue #253: Docker Image Updates](https://github.com/torrust/torrust-tracker-deployer/issues/253)
