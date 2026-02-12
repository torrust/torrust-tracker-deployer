# Environment Configuration Questionnaire

This structured questionnaire helps AI agents and users gather all required information to create a valid environment configuration for torrust-tracker-deployer. Follow the decision tree to collect configuration details systematically.

## Basic Information

### 1. Environment Name

**Question**: What name do you want for this environment?

**Constraints**:

- Pattern: Lowercase letters (a-z), numbers (0-9), and hyphens (-) only
- Length: 3-50 characters
- Cannot start or end with hyphens
- Cannot start with numbers
- Used for resource naming and organization

**Examples**:

- `dev` - Development environment
- `production` - Production deployment
- `staging-01` - First staging environment
- `test-mysql` - Testing with MySQL database

**Field**: `environment.name`

### 2. Environment Description (Optional)

**Question**: How would you describe this environment's purpose? (2-3 sentences recommended)

**Purpose**: Helps document:

- Use case: What this environment is designed for
- Key decisions: Why certain values were chosen
- Context: When this environment is appropriate

**Examples**:

- "Minimal development setup with SQLite and UDP/HTTP trackers. No HTTPS or monitoring. Ideal for local testing."
- "Production-ready deployment with MySQL, full monitoring stack (Prometheus + Grafana), and HTTPS for all services. Includes daily backups with 7-day retention."
- "UDP-only tracker optimized for high-performance scenarios. No HTTP endpoints to minimize overhead."

**Field**: `environment.description`

## Infrastructure Provider

### 3. Provider Selection

**Question**: Which infrastructure provider do you want to use?

**Options**:

- **LXD** (local development, testing, on-premises)
  - Pros: Fast provisioning, no cloud costs, local control
  - Cons: Requires LXD installation, limited to local/network resources
  - Best for: Development, testing, on-premises deployments

- **Hetzner** (cloud production)
  - Pros: Public IP, cloud infrastructure, scalable
  - Cons: Costs per hour, requires API token
  - Best for: Production deployments, public-facing services

**Field**: `provider.provider`

#### 3a. If LXD Selected

**Question**: What LXD profile name should be used?

**Constraints**:

- Pattern: Lowercase letters, numbers, and hyphens
- Typically prefixed with `torrust-profile-` for organization
- Must not conflict with existing LXD profiles

**Example**: `torrust-profile-dev` for environment named `dev`

**Field**: `provider.profile_name`

**Default Behavior**: Uses default LXD bridge network automatically

#### 3b. If Hetzner Selected

**Questions**:

1. **API Token**: What is your Hetzner API token?
   - **Security**: Use a read-write token with server creation permissions
   - **Field**: `provider.api_token`

2. **Server Location**: Which Hetzner datacenter location?
   - Options: `nbg1` (Nuremberg), `fsn1` (Falkenstein), `hel1` (Helsinki), `ash` (Ashburn, VA)
   - Default: `nbg1`
   - **Field**: `provider.location`

3. **Server Type**: Which server size?
   - Options: `cx22` (2 vCPU, 4 GB RAM), `cx32` (4 vCPU, 8 GB RAM), `cx42` (8 vCPU, 16 GB RAM)
   - Default: `cx22`
   - **Field**: `provider.server_type`

4. **Image**: Which operating system image?
   - Default: `ubuntu-24.04` (Ubuntu 24.04 LTS)
   - **Field**: `provider.image`

## Database Configuration

### 4. Database Type

**Question**: What database do you want to use?

**Options**:

- **SQLite** (file-based)
  - Pros: Simpler setup, no credentials, single-file storage
  - Cons: Lower concurrent performance for high loads
  - Best for: Small deployments, testing, development
  - **Driver**: `sqlite3`

- **MySQL** (server-based)
  - Pros: Better for high-load, production deployments, concurrent access
  - Cons: Requires credentials, separate container
  - Best for: Production, high-traffic trackers
  - **Driver**: `mysql`

**Field**: `tracker.core.database.driver`

#### 4a. If SQLite Selected

**Question**: What database filename?

- Default: `tracker.db`
- **Field**: `tracker.core.database.database_name`

#### 4b. If MySQL Selected

**Questions**:

1. **Database Host**: What MySQL server hostname?
   - Default: `mysql` (Docker Compose service name)
   - **Field**: `tracker.core.database.host`

2. **Database Port**: What MySQL server port?
   - Default: `3306`
   - **Field**: `tracker.core.database.port`

3. **Database Name**: What database name?
   - Default: `tracker`
   - **Field**: `tracker.core.database.database_name`

4. **Database Username**: What database user?
   - Example: `tracker_user`
   - **Field**: `tracker.core.database.username`

5. **Database Password**: What database password?
   - **Security**: Use a strong password for production
   - **Field**: `tracker.core.database.password`

## Tracker Configuration

### 5. Tracker Privacy Mode

**Question**: Do you want to run a private tracker?

**Options**:

- **Public** (`false`): Anyone can announce/scrape torrents
- **Private** (`true`): Only authorized users can participate

**Default**: `false` (public)

**Field**: `tracker.core.private`

### 6. UDP Trackers

**Question**: How many UDP tracker instances do you want?

**Options**:

- 0 = No UDP trackers
- 1+ = One or more UDP tracker instances on different ports

**Common scenarios**:

- 0 instances: HTTP-only tracker
- 1 instance: Standard UDP tracker
- 2+ instances: Load distribution, different ports for different swarms

#### 6a. For EACH UDP Tracker

**Questions**:

1. **Binding Address**: What IP:port should the UDP tracker bind to?
   - Format: `{ip}:{port}`
   - Default IP: `0.0.0.0` (all interfaces)
   - Default port: `6969`
   - **Constraint**: Port must be unique across all services
   - **Field**: `tracker.udp_trackers[].bind_address`

2. **Custom Domain** (Optional): Do you want a custom domain for this UDP tracker?
   - **Note**: UDP does not support HTTPS/TLS
   - If yes: What domain? (e.g., `udp.example.com`)
   - **Field**: `tracker.udp_trackers[].domain`

### 7. HTTP Trackers

**Question**: How many HTTP tracker instances do you want?

**Options**:

- 0 = No HTTP trackers
- 1+ = One or more HTTP tracker instances

**Common scenarios**:

- 0 instances: UDP-only tracker
- 1 instance: Standard HTTP tracker
- 2+ instances: Multiple endpoints for different use cases

#### 7a. For EACH HTTP Tracker

**Questions**:

1. **Binding Address**: What IP:port should the HTTP tracker bind to?
   - Format: `{ip}:{port}`
   - Default IP: `0.0.0.0` (all interfaces)
   - Default port: `7070`
   - **Constraint**: Port must be unique across all services
   - **Field**: `tracker.http_trackers[].bind_address`

2. **Custom Domain** (Optional): Do you want a custom domain for this HTTP tracker?
   - If yes: What domain? (e.g., `tracker.example.com`)
   - **Field**: `tracker.http_trackers[].domain`

3. **HTTPS/TLS** (requires domain): Do you want to enable HTTPS for this HTTP tracker?
   - **Requires**: Domain must be configured
   - If yes: Set `use_tls_proxy: true`
   - **Field**: `tracker.http_trackers[].use_tls_proxy`

### 8. HTTP API

**Question**: Do you want to enable the HTTP API?

**Purpose**: Provides management endpoints for tracker administration

#### 8a. If HTTP API Enabled

**Questions**:

1. **Binding Address**: What IP:port should the HTTP API bind to?
   - Format: `{ip}:{port}`
   - Default: `0.0.0.0:1212`
   - **Constraint**: Port must be unique
   - **Field**: `tracker.http_api.bind_address`

2. **Admin Token**: What admin access token?
   - Example: `MyAccessToken` -**Security**: Use a strong, unique token for production
   - **Field**: `tracker.http_api.admin_token`

3. **Custom Domain** (Optional): Do you want a custom domain for the HTTP API?
   - If yes: What domain? (e.g., `api.example.com`)
   - **Field**: `tracker.http_api.domain`

4. **HTTPS/TLS** (requires domain): Do you want to enable HTTPS for the HTTP API?
   - **Requires**: Domain must be configured
   - If yes: Set `use_tls_proxy: true`
   - **Field**: `tracker.http_api.use_tls_proxy`

### 9. Health Check API

**Question**: Do you want to expose the Health Check API?

**Purpose**: Provides health monitoring endpoints

#### 9a. If Health Check API Enabled

**Questions**:

1. **Binding Address**: What IP:port should the Health Check API bind to?
   - Default: `127.0.0.1:1313` (localhost only for security)
   - **Field**: `tracker.health_check_api.bind_address`

2. **Custom Domain** (Optional): Do you want a custom domain for the Health Check API?
   - If yes: What domain? (e.g., `health.example.com`)
   - **Field**: `tracker.health_check_api.domain`

3. **HTTPS/TLS** (requires domain): Do you want to enable HTTPS for the Health Check API?
   - **Requires**: Domain must be configured
   - **Field**: `tracker.health_check_api.use_tls_proxy`

## HTTPS/TLS Configuration

### 10. HTTPS Requirement Check

**Automatic Check**: Are there any services with domains configured for HTTPS?

Services that can use HTTPS:

- HTTP trackers with `use_tls_proxy: true`
- HTTP API with `use_tls_proxy: true`
- Health Check API with `use_tls_proxy: true`
- Grafana with `use_tls_proxy: true`

#### 10a. If ANY Service Uses HTTPS

**Required Configuration**:

1. **Admin Email**: What email address for Let's Encrypt certificate notifications?
   - Format: Valid email address
   - Used for: Certificate expiration notices, account recovery
   - **Field**: `https.admin_email`

2. **Certificate Environment**: Which Let's Encrypt environment?
   - **Staging** (`use_staging: true`): Safe for testing, no rate limits
     - Certificates are not trusted by browsers (for testing only)
     - Use for: Development, LXD environments, testing HTTPS flow
   - **Production** (`use_staging: false` or omit): Real trusted certificates
     - Certificates trusted by all browsers
     - Rate limits apply (50 certificates per domain per week)
     - Use for: Production deployments on Hetzner
   - **Field**: `https.use_staging`

**Recommendation**: Always use staging certificates for LXD/local testing to avoid hitting production rate limits.

## Monitoring (Optional)

### 11. Prometheus

**Question**: Do you want to enable Prometheus metrics collection?

**Purpose**: Collects metrics from the tracker for monitoring and alerting

#### 11a. If Prometheus Enabled

**Questions**:

1. **Scrape Interval**: How often should Prometheus scrape metrics (in seconds)?
   - Default: `15` seconds
   - Range: 5-300 seconds
   - **Field**: `prometheus.scrape_interval_in_secs`

2. **Custom Domain** (Optional): Do you want a custom domain for Prometheus?
   - If yes: What domain? (e.g., `prometheus.example.com`)
   - **Note**: Prometheus domain configuration is not currently supported in schema

3. **HTTPS/TLS** (Optional): Do you want to enable HTTPS for Prometheus?
   - **Note**: Prometheus HTTPS configuration is not currently supported in schema

### 12. Grafana

**Question**: Do you want to enable Grafana dashboards?

**Requirements**: Prometheus must be enabled (Grafana depends on Prometheus as data source)

#### 12a. If Grafana Enabled

**Questions**:

1. **Admin User**: What admin username for Grafana?
   - Default: `admin`
   - **Field**: `grafana.admin_user`

2. **Admin Password**: What admin password for Grafana?
   - **Security**: Use a strong password for production
   - **Field**: `grafana.admin_password`

3. **Custom Domain** (Optional): Do you want a custom domain for Grafana?
   - If yes: What domain? (e.g., `grafana.example.com`)
   - **Field**: `grafana.domain`

4. **HTTPS/TLS** (requires domain): Do you want to enable HTTPS for Grafana?
   - **Requires**: Domain must be configured
   - If yes: Set `use_tls_proxy: true`
   - **Field**: `grafana.use_tls_proxy`

## Backup Configuration (Optional)

### 13. Backups

**Question**: Do you want to enable automated backups?

**Purpose**: Automated backups of tracker database and persistent data

#### 13a. If Backups Enabled

**Questions**:

1. **Backup Schedule**: What cron schedule for backups?
   - Format: 5-field cron expression (minute hour day month weekday)
   - Default: `0 3 * * *` (3:00 AM daily)
   - Examples:
     - `0 3 * * *` - 3:00 AM daily
     - `0 */6 * * *` - Every 6 hours
     - `0 0 * * 0` - Midnight every Sunday
   - **Field**: `backup.schedule`

2. **Retention Period**: How many days to retain backups before automatic deletion?
   - Default: `7` days
   - Must be greater than 0
   - **Field**: `backup.retention_days`

## SSH Access

### 14. SSH Credentials

**Required for all deployments**: SSH credentials to access the provisioned instance

**Questions**:

1. **Private Key Path**: What is the absolute path to your SSH private key file?
   - Example: `/home/user/.ssh/id_rsa`
   - **Constraint**: File must exist and be readable
   - **Field**: `ssh_credentials.private_key_path`

2. **Public Key Path**: What is the absolute path to your SSH public key file?
   - Example: `/home/user/.ssh/id_rsa.pub`
   - **Constraint**: File must exist and be readable
   - **Field**: `ssh_credentials.public_key_path`

3. **SSH Username**: What username to use for SSH access?
   - Default: `torrust`
   - **Field**: `ssh_credentials.username`

4. **SSH Port**: What SSH port?
   - Default: `22`
   - **Field**: `ssh_credentials.port`

## Validation Rules

After gathering all information, the configuration must satisfy these validation rules:

**Cross-Service Validation**:

1. **Unique Ports**: All binding addresses must use unique ports
   - UDP trackers, HTTP trackers, HTTP API, Health Check API must not conflict

2. **HTTPS Dependencies**: If any service has `use_tls_proxy: true`, then:
   - That service must have a `domain` configured
   - The `https` section must be present with `admin_email`

3. **Grafana Dependencies**: If Grafana is enabled, then:
   - Prometheus must also be enabled

4. **Domain Format**: All domains must be valid DNS names (e.g., `example.com`, `subdomain.example.com`)

5. **SSH Keys**: Both private and public key files must exist at the specified paths

## Output Format

Once all information is gathered, generate a JSON configuration file following the schema at [`schemas/environment-config.json`](../../schemas/environment-config.json).

**Validation Command**:

```bash
cargo run -- validate --env-file <path-to-config.json>
```

**Template Generation** (for manual editing):

```bash
cargo run -- create template --provider {lxd|hetzner} --output <path-to-template.json>
```

## Common Configuration Patterns

**Minimal Development (LXD + SQLite)**:

- Provider: LXD
- Database: SQLite
- Trackers: 1 UDP + 1 HTTP (no HTTPS)
- No monitoring, no backups

**Production (Hetzner + MySQL + HTTPS + Monitoring)**:

- Provider: Hetzner
- Database: MySQL with credentials
- Trackers: Multiple UDP + HTTP with HTTPS
- Monitoring: Prometheus + Grafana with HTTPS
- Backups: Daily with 7-day retention
- HTTPS: Production certificates

**Development Full-Stack (LXD + All Features)**:

- Provider: LXD
- Database: MySQL
- Trackers: Multiple with HTTPS staging certificates
- Monitoring: Full stack
- Backups: Enabled
- HTTPS: Staging certificates (safe for testing)

## References

- [Environment Configuration Schema](../../schemas/environment-config.json)
- [Create Command Documentation](../user-guide/commands/create.md)
- [Configuration DTO Documentation](../../src/application/command_handlers/create/config/README.md)
- [Example Configurations](./examples/) (after Phase 3-4 implementation)
