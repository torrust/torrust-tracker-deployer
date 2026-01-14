# Add HTTPS Support with Caddy for All HTTP Services

**Issue**: [#272](https://github.com/torrust/torrust-tracker-deployer/issues/272)
**Parent Epic**: #1 - Roadmap (Section 6: Add HTTPS support)
**Related**: [#270 - Evaluate Caddy for HTTPS Termination](https://github.com/torrust/torrust-tracker-deployer/issues/270) (Research Complete)

## Overview

Implement official HTTPS support for all HTTP services (Tracker API, HTTP Tracker, Grafana) using Caddy v2.10 as a TLS termination proxy. This task integrates the proven Caddy configuration from production deployment (`/opt/torrust/`) into the deployer's Tera templates, enabling automated HTTPS setup for all new deployments.

**Background**: Issue #270 successfully evaluated Caddy v2.10, proving it works in production with:

- ✅ Automatic HTTPS with Let's Encrypt certificates
- ✅ WebSocket support for Grafana (Pingoo's failure point)
- ✅ HTTP/3 (QUIC) support
- ✅ Simple configuration (21-line Caddyfile)

Production deployment at `/opt/torrust/` on Hetzner server (46.224.206.37) serves as the reference implementation.

## Goals

- [x] Integrate Caddy into deployer Tera templates
- [x] Support HTTPS for all HTTP services (Tracker API, HTTP Tracker, Grafana)
- [x] Enable automatic Let's Encrypt certificate management
- [x] Add HTTPS configuration to environment schema
- [ ] Implement security scanning for Caddy in CI/CD
- [ ] Document HTTPS setup in user guide
- [ ] Add E2E tests for HTTPS functionality

## 🏗️ Architecture Requirements

**DDD Layers**: Multiple (templates in Infrastructure, config in Application, CLI in Presentation)

**Affected Modules**:

- `templates/caddy/` - New Caddy templates
- `templates/docker-compose/` - Docker Compose templates (add Caddy service)
- `src/application/command_handlers/create/config/` - Environment configuration DTOs
- `src/infrastructure/external_tools/ansible/template/` - Template rendering
- Documentation in `docs/user-guide/`

**Pattern**: Template generation + Configuration extension

### Module Structure Requirements

- [ ] Follow existing template generation patterns (see `templates/prometheus/`, `templates/grafana/`)
- [ ] Use Tera templating for dynamic configuration (see [docs/contributing/templates/tera.md](../contributing/templates/tera.md))
- [ ] Register static templates in ProjectGenerator (see [docs/contributing/templates/template-system-architecture.md](../contributing/templates/template-system-architecture.md))
- [ ] Extend environment configuration schema properly (see [docs/decisions/configuration-dto-layer-placement.md](../decisions/configuration-dto-layer-placement.md))

### Architectural Constraints

- [ ] Templates must follow project conventions: config files in `storage/<service>/etc/`
- [ ] Configuration DTOs must validate constraints (domain names, email format, port numbers)
- [ ] Secrets (admin email) must use secrecy crate wrappers (see [docs/contributing/secret-handling.md](../contributing/secret-handling.md))
- [ ] Error messages must be user-friendly and actionable (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Hardcoding domain names or admin emails in templates
- ❌ Mixing template generation logic with business logic
- ❌ Using plain `String` for sensitive data (email addresses used in certificates)
- ❌ Not registering static templates in ProjectGenerator (will cause "file not found" errors)

## Specifications

### 1. Reference Production Configuration

**Location**: `/opt/torrust/` on Hetzner server (46.224.206.37)

**Files to Template**:

- `storage/caddy/etc/Caddyfile` → `templates/caddy/Caddyfile.tera`
- `docker-compose.yml` (Caddy service block) → `templates/docker-compose/docker-compose.yml.tera`
- `.env` (Caddy-related variables) → Already templated, may need additions

**Directory Structure** (production):

```text
/opt/torrust/
├── .env
├── docker-compose.yml
├── prometheus.yml
└── storage/
    ├── caddy/
    │   └── etc/
    │       └── Caddyfile
    ├── grafana/
    ├── prometheus/
    └── tracker/
```

### 2. Caddyfile Template

**File**: `templates/caddy/Caddyfile.tera`

**Requirements**:

- Support independent HTTPS configuration for each service
- Support multiple HTTP trackers with individual domains
- Only generate blocks for services with TLS enabled
- Each HTTP tracker maps to its configured port
- Follow Torrust Tracker convention: if `tls` section exists in service config, HTTPS is enabled

**Template Variables** (pre-processed in Rust Context):

Following the [Context Data Preparation Pattern](../contributing/templates/template-system-architecture.md#-context-data-preparation-pattern), all data is pre-processed in Rust before being passed to the template. The template receives ready-to-use values:

- `{{ admin_email }}` - Admin email for Let's Encrypt notifications
- `{{ use_staging }}` - Boolean for Let's Encrypt staging environment
- `{{ http_api_service }}` - Optional service object with `domain` and `port` (only present if TLS configured)
- `{{ http_tracker_services }}` - Array of service objects, each with `domain` and `port` (only TLS-enabled trackers)
- `{{ grafana_service }}` - Optional service object with `domain` and `port` (only present if TLS configured)

**Example**:

```caddyfile
# Caddyfile for Torrust Tracker - Automatic HTTPS with Let's Encrypt
# IMPORTANT: Caddy requires TABS for indentation, not spaces.

# Global options
{
    # Email for Let's Encrypt notifications
    email {{ admin_email }}

    {% if use_staging %}
    # Use Let's Encrypt staging environment (for testing, avoids rate limits)
    # WARNING: Staging certificates will show browser warnings (not trusted)
    acme_ca https://acme-staging-v02.api.letsencrypt.org/directory
    {% endif %}
}

{% if http_api_service %}
# Tracker REST API
{{ http_api_service.domain }} {
    reverse_proxy tracker:{{ http_api_service.port }}
}
{% endif %}

{% for service in http_tracker_services %}
# HTTP Tracker
{{ service.domain }} {
    reverse_proxy tracker:{{ service.port }}
}
{% endfor %}

{% if grafana_service %}
# Grafana UI with WebSocket support
{{ grafana_service.domain }} {
    reverse_proxy grafana:{{ grafana_service.port }}
}
{% endif %}
```

**Note**: Port extraction and TLS filtering happens in Rust when building `CaddyContext`, not in the template. See [Context Data Preparation Pattern](../contributing/templates/template-system-architecture.md#-context-data-preparation-pattern).

**Configuration Example** (user input):

```json
{
  "https": {
    "admin_email": "admin@example.com"
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "tls": {
        "domain": "api.torrust-tracker.com"
      }
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "tls": {
          "domain": "http1.torrust-tracker.com"
        }
      },
      {
        "bind_address": "0.0.0.0:7071",
        "tls": {
          "domain": "http2.torrust-tracker.com"
        }
      },
      {
        "bind_address": "0.0.0.0:7072"
        // No tls section - uses HTTP only
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin",
    "tls": {
      "domain": "grafana.torrust-tracker.com"
    }
  }
}
```

In this example:

- ✅ Tracker API uses HTTPS (api.torrust-tracker.com) - port from bind_address (1212)
- ✅ HTTP Tracker 1 uses HTTPS (http1.torrust-tracker.com) - port from bind_address (7070)
- ✅ HTTP Tracker 2 uses HTTPS (http2.torrust-tracker.com) - port from bind_address (7071)
- ❌ HTTP Tracker 3 uses HTTP only (no tls section) - port from bind_address (7072)
- ✅ Grafana uses HTTPS (grafana.torrust-tracker.com) - hardcoded port 3000

### 3. Docker Compose Template Updates

**File**: `templates/docker-compose/docker-compose.yml.tera`

**Add Caddy Service** (conditional on any service having TLS configured):

```yaml
{% if needs_caddy %}
  caddy:
    image: caddy:2.10
    container_name: caddy
    tty: true
    restart: unless-stopped
    ports:
      - "80:80"       # HTTP (ACME HTTP-01 challenge)
      - "443:443"     # HTTPS
      - "443:443/udp" # HTTP/3 (QUIC)
    volumes:
      - ./storage/caddy/etc/Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data     # TLS certificates (MUST persist!)
      - caddy_config:/config
    networks:
      - metrics_network
      - visualization_network
    healthcheck:
      test: ["CMD", "caddy", "validate", "--config", "/etc/caddy/Caddyfile"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s
    logging:
      options:
        max-size: "10m"
        max-file: "10"
{% endif %}
```

**Add Volumes** (at bottom of file):

```yaml
volumes:
  {% if needs_caddy %}
  caddy_data:
  caddy_config:
  {% endif %}
```

**Notes**:

- Caddy service is deployed if any service has a `tls` section configured
- Service name is `caddy` (not `proxy`) to match naming convention of other services (mysql, grafana, prometheus)
- The `needs_caddy` variable is derived from checking if any service (tracker.api, tracker.http_trackers[], grafana) has TLS enabled

### 4. Environment Configuration Schema

**Note**: The JSON schema (`schemas/environment-config.json`) is automatically generated from the Rust configuration DTOs. After implementing the Rust types in Phase 2, the schema will be regenerated in Phase 7 using:

```bash
cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
```

**See**: [schemas/README.md](../../schemas/README.md) for schema generation documentation.

**HTTPS Configuration Structure** (to be implemented in Rust DTOs):

The configuration follows **Torrust Tracker conventions**: if a `tls` section exists in a service configuration, HTTPS is enabled for that service.

**Common HTTPS Configuration** (top-level):

- `https.admin_email` (string, email format, required if any service has TLS) - Admin email for Let's Encrypt notifications
- `https.use_staging` (boolean, optional, defaults to false) - Use Let's Encrypt staging environment for testing (avoids rate limits, certificates show browser warnings)

**Service-Level TLS Configuration** (within each service):

- `tracker.http_api.tls` (object, optional) - TLS configuration for Tracker API
  - `domain` (string, domain format) - Domain for Tracker API
  - Port extracted from `tracker.http_api.bind_address`
- `tracker.http_trackers[].tls` (object, optional) - TLS configuration per HTTP tracker
  - `domain` (string, domain format) - Domain for this HTTP tracker
  - Port extracted from each tracker's `bind_address`
- `grafana.tls` (object, optional) - TLS configuration for Grafana
  - `domain` (string, domain format) - Domain for Grafana
  - Port: hardcoded 3000 (matches docker-compose template)

**Validation Requirements**:

- If any service has `tls` section, `https.admin_email` is required
- If `https.admin_email` provided, at least one service must have `tls` configured
- Email must be valid format
- Domain names must follow DNS naming conventions
- When tracker has `tls`, it may need additional proxy configuration (e.g., trust proxy headers for original client IP)

**Design Decision - Validation Strategy**:

When `https.admin_email` is provided but no services have `tls` configured, the system will **fail with a clear validation error** (rather than silently ignoring the config or deploying an unused Caddy service). This approach:

- ✅ **Prevents surprises**: User gets immediate feedback about configuration mismatch
- ✅ **Avoids debugging confusion**: No silent skipping that might leave users wondering why Caddy isn't deployed
- ✅ **Follows lean philosophy**: Doesn't waste resources deploying unused services
- ✅ **Reduces security surface**: Avoids running unnecessary containers that could become attack vectors
- ✅ **Clear intent**: Configuration explicitly states what the user wants, errors indicate mismatches

**Configuration Examples**:

**Example 1**: HTTPS for all services (production Let's Encrypt)

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": false // Optional, defaults to false (production)
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "tls": {
        "domain": "api.torrust-tracker.com"
      }
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "tls": {
          "domain": "http1.torrust-tracker.com"
        }
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin",
    "tls": {
      "domain": "grafana.torrust-tracker.com"
    }
  }
}
```

**Example 2**: HTTPS only for Tracker API with staging environment (for testing)

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": true // Use Let's Encrypt staging (testing only)
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "tls": {
        "domain": "api.torrust-tracker.com"
      }
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
        // No tls section - uses HTTP only
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin"
    // No tls section - uses HTTP only
  }
}
```

**Example 3**: Multiple HTTP trackers, some with HTTPS, some without

```json
{
  "https": {
    "admin_email": "admin@example.com"
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
      // No tls section - uses HTTP only
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "tls": {
          "domain": "http1.torrust-tracker.com"
        }
      },
      {
        "bind_address": "0.0.0.0:7071",
        "tls": {
          "domain": "http2.torrust-tracker.com"
        }
      },
      {
        "bind_address": "0.0.0.0:7072"
        // No tls section - uses HTTP only
      }
    ]
  }
}
```

In Example 3: http1 and http2 use HTTPS, http3 uses HTTP only. Tracker API uses HTTP only.

**Example 4**: No HTTPS (omit configuration entirely)

```json
{
  // No https section - all services use HTTP
}
```

### 5. Configuration DTO Updates

**Architecture**: Service-based TLS configuration (each service has optional `tls` field)

**File**: `src/application/command_handlers/create/config/https.rs` (new file)

```rust
use serde::{Deserialize, Serialize};

/// Common HTTPS configuration (top-level)
/// Only contains configuration shared across all TLS-enabled services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsConfig {
    /// Admin email for Let's Encrypt certificate notifications
    /// Required if any service has TLS configured
    pub admin_email: String, // TODO: Use AdminEmail wrapper type (secrecy crate)

    /// Use Let's Encrypt staging environment (for testing)
    /// When true, uses https://acme-staging-v02.api.letsencrypt.org/directory
    /// When false or omitted, uses production API (https://acme-v02.api.letsencrypt.org/directory)
    ///
    /// Staging certificates will show browser warnings (not trusted by browsers)
    /// but allow testing the HTTPS flow without hitting rate limits:
    /// - Production: 50 certs/week per domain, 5 duplicates/week
    /// - Staging: Much higher limits for testing
    #[serde(default)] // Defaults to false (production)
    pub use_staging: bool,
}

/// Service-specific TLS configuration
/// Embedded in each service that supports HTTPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Domain name for this service
    pub domain: String, // TODO: Use Domain wrapper type
}
```

**Update**: Existing service DTOs to include optional `tls` field

**File**: `src/application/command_handlers/create/config/tracker.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpApiSection {
    pub bind_address: String,
    pub admin_token: String,

    /// Optional TLS configuration for HTTPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTrackerSection {
    pub bind_address: String,

    /// Optional TLS configuration for HTTPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}
```

**File**: `src/application/command_handlers/create/config/grafana.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaSection {
    pub admin_user: String,
    pub admin_password: String,

    /// Optional TLS configuration for HTTPS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}
```

**File**: `src/application/command_handlers/create/config/environment_creation_config.rs`

```rust
pub struct EnvironmentCreationConfig {
    // ... existing fields ...

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub https: Option<HttpsConfig>,
}

impl EnvironmentCreationConfig {
    /// Check if any service has TLS configured
    fn has_any_tls_configured(&self) -> bool {
        // Check tracker API
        if self.tracker.http_api.tls.is_some() {
            return true;
        }

        // Check HTTP trackers
        for tracker in &self.tracker.http_trackers {
            if tracker.tls.is_some() {
                return true;
            }
        }

        // Check Grafana
        if self.grafana.tls.is_some() {
            return true;
        }

        false
    }

    pub fn validate_https_config(&self) -> Result<(), String> {
        let has_tls = self.has_any_tls_configured();
        let has_https_section = self.https.is_some();

        // If any service has TLS, admin_email is required
        if has_tls && !has_https_section {
            return Err(
                "TLS configured for one or more services but 'https' section not provided. \
                 Add 'https' section with 'admin_email' for Let's Encrypt certificate notifications."
                    .to_string()
            );
        }

        // If admin_email provided, at least one service must have TLS configured
        // Design decision: Fail explicitly rather than silently ignoring or deploying unused Caddy.
        // This prevents confusion and follows lean principles (don't deploy what you don't use).
        if has_https_section && !has_tls {
            return Err(
                "Admin email provided but no services have TLS configured. \
                 Add 'tls' section to at least one service (tracker.http_api, tracker.http_trackers[], or grafana). \
                 Remove the 'https' section entirely if you don't want HTTPS."
                    .to_string()
            );
        }

        // TODO: Validate domain names format in each service's TlsConfig
        // TODO: Add email format validation for HttpsConfig.admin_email

        Ok(())
    }
}
```

**Note**: HTTPS configuration is entirely optional. If omitted, all services use HTTP only.

### 6. Security Workflow Updates

**File**: `.github/workflows/docker-security-scan.yml`

**Add Caddy to Third-Party Images Matrix**:

```yaml
matrix:
  image:
    - torrust/tracker:develop
    - mysql:8.0
    - grafana/grafana:11.4.0
    - prom/prometheus:v3.0.1
    - caddy:2.10 # NEW
```

**Add SARIF Upload Step for Caddy**:

```yaml
- name: Upload third-party caddy SARIF
  if: always()
  uses: github/codeql-action/upload-sarif@v4
  with:
    sarif_file: sarif-third-party-caddy-2.10-${{ github.run_id }}/trivy.sarif
    category: docker-third-party-caddy-2.10
  continue-on-error: true
```

### 7. Documentation Updates

**File**: `docs/user-guide/https-setup.md` (new file)

Create comprehensive guide covering:

- Prerequisites (domain names, DNS configuration)
- Environment configuration example
- Let's Encrypt certificate process
- Troubleshooting common issues
- Certificate renewal (automatic)
- Domain verification requirements

**File**: `docs/user-guide/README.md`

Add link to HTTPS setup guide.

## Implementation Plan

### Phase 1: Template Creation (3-4 hours)

- [x] Create `templates/caddy/Caddyfile.tera` based on production configuration
- [x] Create `docs/contributing/templates/caddy.md` documenting template variables (per project convention: no README in templates/)
- [x] Update `templates/docker-compose/docker-compose.yml.tera` with Caddy service block
- [x] Register Caddyfile in `CaddyProjectGenerator` (`src/infrastructure/templating/caddy/`)
- [x] Test template rendering with sample data (14 unit tests in `CaddyProjectGenerator` and `CaddyfileRenderer`)

### Phase 2: Configuration DTOs (3-4 hours)

- [x] Create `src/application/command_handlers/create/config/https.rs` with DTOs
  - [x] `HttpsSection` struct with `admin_email` and `use_staging` fields
    - [x] `admin_email: String` (required if TLS configured)
    - [x] `use_staging: bool` (optional, defaults to false for production)
  - [x] `TlsSection` struct with only `domain` field (service-specific)
- [x] Update existing service DTOs to include optional `tls` field:
  - [x] `HttpApiSection` in `tracker.rs` - add `tls: Option<TlsSection>`
  - [x] `HttpTrackerSection` in `tracker.rs` - add `tls: Option<TlsSection>`
  - [x] `GrafanaSection` in `grafana.rs` - add `tls: Option<TlsSection>`
- [x] Update `EnvironmentCreationConfig` to include optional `HttpsSection`
- [x] Add validation logic:
  - [x] `has_any_tls_configured()` - check if any service has `tls` section
  - [x] If any service has TLS, `https` section with `admin_email` is required
  - [x] If `https.admin_email` provided, at least one service must have TLS configured
  - [x] Email format validation for `HttpsSection.admin_email` (using `email_address` crate via `Email` type in `src/shared/email.rs`)
  - [x] Domain name format validation in each service's `TlsSection` (using `DomainName` type in `src/shared/domain_name.rs`)
- [x] Add proper type wrappers for validation (`Email`, `DomainName` in `src/shared/`) - Note: DTOs remain as `String` primitives for JSON serialization, domain types used for validation during boundary crossing
- [x] Create unit tests for all validation scenarios

### Phase 3: Template Rendering Integration (3-4 hours)

- [x] Create `CaddyProjectGenerator` following Project Generator pattern
- [x] Create `CaddyContext` with pre-processed data (following [Context Data Preparation Pattern](../contributing/templates/template-system-architecture.md#-context-data-preparation-pattern)):
  - [x] `admin_email: String` - extracted from config
  - [x] `use_staging: bool` - extracted from config
  - [x] `http_api_service: Option<CaddyService>` - only if TLS configured, with pre-extracted port
  - [x] `http_tracker_services: Vec<CaddyService>` - only TLS-enabled trackers, with pre-extracted ports
  - [x] `grafana_service: Option<CaddyService>` - only if TLS configured, with pre-extracted port
- [x] Create `CaddyService` struct with `domain: String` and `port: u16`
- [x] Implement port extraction in Rust (from `SocketAddr`) when building context
- [x] Handle conditional rendering in templates:
  - [x] `needs_caddy` variable checks if any service list is non-empty
  - [x] Only include Caddy service in docker-compose if `needs_caddy` is true
  - [x] `{% if http_api_service %}` for API service block in Caddyfile
  - [x] `{% for service in http_tracker_services %}` for tracker iteration in Caddyfile
  - [x] `{% if grafana_service %}` for Grafana service block in Caddyfile
- [x] Update `ReleaseCommand` to include Caddy template generation:
  - [x] Add `RenderCaddyTemplates` step to `ReleaseStep` enum
  - [x] Add `DeployCaddyConfigToRemote` step to `ReleaseStep` enum
  - [x] Create `RenderCaddyTemplatesStep` for template rendering
  - [x] Create `DeployCaddyConfigStep` for Ansible deployment
  - [x] Create Ansible playbook `deploy-caddy-config.yml`
  - [x] Register playbook in `copy_static_templates` method
  - [x] Integrate `CaddyContext` into Docker Compose template rendering
  - [x] Add error variant `CaddyConfigDeployment` with help text
- [x] Test template generation with various scenarios:
  - [x] All services HTTPS
  - [x] Only Tracker API HTTPS
  - [x] Multiple HTTP trackers, mixed HTTPS/HTTP
  - [x] No HTTPS (Caddy not deployed)

### Phase 4: Security Workflow Updates (1 hour)

- [ ] Add `caddy:2.10` to security scan workflow matrix
- [ ] Add SARIF upload step for Caddy scan results
- [ ] Update `docs/security/docker/scans/README.md` with Caddy entry
- [ ] Run security scan locally to verify configuration
- [ ] Document vulnerability assessment (reference [docs/research/caddy-tls-proxy-evaluation/security-scan.md](../research/caddy-tls-proxy-evaluation/security-scan.md))

### Phase 5: Documentation (4-5 hours)

- [ ] Create `docs/user-guide/https-setup.md` with complete setup guide:
  - [ ] Prerequisites (domain names, DNS configuration)
  - [ ] **Configuration patterns**: All services, single service, multiple trackers
  - [ ] **Selective HTTPS**: How to enable HTTPS for some services but not others
  - [ ] **Multiple HTTP trackers**: Configuration examples with mixed HTTPS/HTTP
  - [ ] Environment configuration examples for each pattern
  - [ ] Let's Encrypt certificate process
  - [ ] **Let's Encrypt staging environment**: Document `use_staging: true` for testing (avoids rate limits)
  - [ ] **Rate limits**: Document Let's Encrypt limits (50 certs/week, 5 duplicates/week)
  - [ ] **Staging certificates warning**: Browser warnings expected (not trusted), only for testing
  - [ ] Troubleshooting common issues
  - [ ] Certificate renewal (automatic)
  - [ ] Domain verification requirements
- [ ] Update `docs/user-guide/README.md` with HTTPS guide link
- [ ] Update `docs/user-guide/configuration.md` with HTTPS config examples:
  - [ ] Example: HTTPS only for API (sensitive token)
  - [ ] Example: Multiple trackers, selective HTTPS
  - [ ] Example: VPN deployment without HTTPS
- [ ] Create example environment files in `envs/` demonstrating patterns
- [ ] Add troubleshooting section for common certificate issues
- [ ] Document Let's Encrypt rate limits and best practices

### Phase 6: E2E Testing (5-6 hours)

**Automated E2E Tests**:

- [ ] Create E2E test environment configs with various HTTPS patterns:
  - [ ] All services HTTPS
  - [ ] Only Tracker API HTTPS
  - [ ] Multiple HTTP trackers with selective HTTPS
  - [ ] No HTTPS (baseline)
- [ ] Add test for Caddyfile template generation:
  - [ ] Verify conditional service blocks
  - [ ] Verify HTTP tracker iteration
  - [ ] Verify port mapping correctness
- [ ] Add test for docker-compose.yml:
  - [ ] Caddy service included when HTTPS configured
  - [ ] Caddy service excluded when no HTTPS
- [ ] Add test for certificate acquisition (mock or staging Let's Encrypt)
- [ ] Add test for HTTPS endpoint accessibility (each service independently)
- [ ] Add test for WebSocket connectivity through Caddy
- [ ] Add test for mixed HTTP/HTTPS deployment (some services HTTPS, some HTTP)
- [ ] Update existing E2E tests to handle optional HTTPS configuration

**Manual E2E Test** (reproduce production locally):

- [x] Create manual test environment config in `envs/`:
  - [x] Base on production config (`envs/docker-hetzner-test.json`)
  - [x] Replace Hetzner provider with LXD provider
  - [x] Add TLS configuration matching production (all services HTTPS)
  - [x] Use test domains (e.g., `api.tracker.local`)
- [x] Run full deployment workflow locally:

  ```bash
  cargo run -- create environment --env-file envs/manual-https-test.json
  cargo run -- provision manual-https-test
  cargo run -- configure manual-https-test
  cargo run -- release manual-https-test
  cargo run -- run manual-https-test
  ```

- [x] Verify rendered templates in `build/manual-https-test/`:
  - [x] Check `caddy/Caddyfile` contains all service blocks with correct domains
  - [x] Check `docker-compose/docker-compose.yml` includes Caddy service
  - [x] Verify port extraction from bind_address (e.g., 0.0.0.0:7070 → 7070)
  - [x] Confirm Caddy volumes (caddy_data, caddy_config) are present
- [x] Verify Caddyfile deployed to server at `/opt/torrust/storage/caddy/etc/Caddyfile`
- [x] Verify Caddy container running and healthy
- [x] Verify Caddy logs show successful certificate acquisition (local CA for `.local` domains)
- [x] Verify HTTPS endpoints accessible via curl:
  - [x] `https://api.tracker.local` - Tracker API responds (HTTP/2 500, expected - auth required)
  - [x] `https://grafana.tracker.local` - Grafana redirects to `/login` (HTTP/2 302)
  - [x] `https://http1.tracker.local` - HTTP Tracker responds (HTTP/2 404, expected for GET)
  - [x] `https://http2.tracker.local` - HTTP Tracker responds (HTTP/2 404, expected for GET)
- [x] Verify HTTP→HTTPS redirect works (HTTP 308 Permanent Redirect)
- [x] Verify `via: 1.1 Caddy` header present in responses
- [x] Verify HTTP/2 and HTTP/3 enabled (`alt-svc: h3=":443"` header)
- [x] Verify port filtering (TLS ports NOT exposed, non-TLS ports exposed)
- [ ] Compare with production templates to ensure consistency
- [ ] Document manual test procedure in `docs/e2e-testing/manual-https-testing.md`

**Manual Test Results** (2026-01-14):

| Test                            | Status  | Notes                                      |
| ------------------------------- | ------- | ------------------------------------------ |
| Caddyfile template rendering    | ✅ Pass | Clean output, no formatting warnings       |
| Caddy service in docker-compose | ✅ Pass | Ports 80, 443, 443/udp exposed             |
| Caddyfile deployment to server  | ✅ Pass | `/opt/torrust/storage/caddy/etc/Caddyfile` |
| Caddy container health          | ✅ Pass | Running, healthy                           |
| Certificate acquisition         | ✅ Pass | Local CA used for `.local` domains         |
| HTTPS API endpoint              | ✅ Pass | HTTP/2 500 (auth required)                 |
| HTTPS Grafana endpoint          | ✅ Pass | HTTP/2 302 redirect to /login              |
| HTTPS HTTP Tracker 1            | ✅ Pass | HTTP/2 404 (expected for GET)              |
| HTTPS HTTP Tracker 2            | ✅ Pass | HTTP/2 404 (expected for GET)              |
| HTTP→HTTPS redirect             | ✅ Pass | 308 Permanent Redirect                     |
| HTTP/2 enabled                  | ✅ Pass | Confirmed in response                      |
| HTTP/3 available                | ✅ Pass | `alt-svc: h3=":443"` header                |
| TLS port filtering              | ✅ Pass | TLS ports hidden, non-TLS ports exposed    |

**Local DNS Setup** (for testing):

Add to `/etc/hosts` (replace IP with your LXD VM IP):

```text
10.140.190.58   api.tracker.local
10.140.190.58   http1.tracker.local
10.140.190.58   http2.tracker.local
10.140.190.58   grafana.tracker.local
```

**Certificate Behavior**:

| Domain Type                                | Certificate Source         | Trust Level                    |
| ------------------------------------------ | -------------------------- | ------------------------------ |
| Real domains (e.g., `tracker.example.com`) | Let's Encrypt (or staging) | Browser trusted                |
| Local domains (e.g., `*.tracker.local`)    | Caddy's Local CA           | Self-signed (browser warnings) |
| Unreachable domains / No internet          | Caddy's Local CA           | Self-signed                    |

**Manual E2E Test Procedure**:

This section documents the step-by-step procedure for running manual E2E tests with HTTPS support.

**1. Setup the environment configuration file**:

Create an environment configuration file (e.g., `envs/manual-https-test.json`) with the desired HTTPS settings. See `envs/manual-https-test.json` for a complete example.

**2. Run the deployment workflow**:

```bash
# Destroy any existing environment with the same name
cargo run -- destroy manual-https-test

# Clean up local build artifacts (if needed)
rm -rf data/manual-https-test build/manual-https-test

# Create the environment
cargo run -- create environment --env-file envs/manual-https-test.json

# Provision infrastructure (creates LXD VM)
cargo run -- provision manual-https-test

# Configure the environment (install Docker, etc.)
cargo run -- configure manual-https-test

# Release application (render templates and deploy to VM)
cargo run -- release manual-https-test

# Run the application (start Docker Compose services)
cargo run -- run manual-https-test
```

**3. Verify local build artifacts**:

Check the generated templates in `build/<env-name>/`:

```bash
# Verify docker-compose.yml has correct port exposure
cat build/manual-https-test/docker-compose/docker-compose.yml

# Verify Caddyfile has all TLS services configured
cat build/manual-https-test/caddy/Caddyfile
```

**4. Verify deployment on the VM**:

Get the VM IP from the provision output or from environment data:

```bash
# Check environment state for VM IP
cat data/manual-https-test/environment.json | jq '.context.provisioned_context.instance_ip'
```

SSH into the VM to verify services:

```bash
# Check running containers (use your SSH key path)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP> "docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'"

# Check Caddyfile inside the Caddy container
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP> "docker exec caddy cat /etc/caddy/Caddyfile"

# Check container logs if needed
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP> "docker logs caddy --tail 50"
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP> "docker logs tracker --tail 50"
```

**5. Key file locations on the VM**:

| File               | Location on VM                         | Location in Container   |
| ------------------ | -------------------------------------- | ----------------------- |
| Caddyfile          | N/A (bind mount)                       | `/etc/caddy/Caddyfile`  |
| docker-compose.yml | `/home/torrust/app/docker-compose.yml` | N/A                     |
| Tracker config     | Bind mount from host                   | `/etc/torrust/tracker/` |
| Caddy certificates | Docker volume `caddy_data`             | `/data/`                |

**6. Port exposure verification**:

For mixed TLS/non-TLS configurations, verify correct port exposure:

- TLS-enabled services (API, HTTP trackers with TLS, Grafana with TLS) should NOT have ports exposed directly
- Non-TLS services (UDP trackers, HTTP trackers without TLS) should have ports exposed
- Caddy ports (80, 443, 443/udp) should always be exposed when HTTPS is configured

Example verification with `docker ps`:

```text
# Expected output for mixed TLS config (7070, 7071 have TLS, 7072 doesn't):
tracker   6969/udp, 7072/tcp   # 7070, 7071 NOT exposed (Caddy handles them)
caddy     80/tcp, 443/tcp, 443/udp  # Entry point for HTTPS
```

### Phase 7: Schema Generation (30 minutes)

- [ ] Regenerate JSON schema from Rust DTOs:

  ```bash
  cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
  ```

- [ ] Verify schema includes HTTPS configuration section
- [ ] Verify schema validation rules match Rust DTO constraints
- [ ] Test schema with example HTTPS-enabled environment file
- [ ] Commit updated schema file

### Phase 8: Create ADR (1 hour)

- [ ] Create `docs/decisions/caddy-for-tls-termination.md`
- [ ] Document decision rationale (reference #270 evaluation)
- [ ] Document alternatives considered (Pingoo, nginx+certbot)
- [ ] Document implementation approach
- [ ] Document risks and mitigations

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Template Implementation**:

- [ ] Caddyfile template renders correctly with all variables
- [ ] Docker Compose template includes Caddy service when HTTPS enabled
- [ ] Docker Compose template excludes Caddy service when HTTPS disabled
- [ ] Templates follow project conventions (config in `storage/<service>/etc/`)
- [ ] All static templates registered in ProjectGenerator

**Configuration**:

- [ ] Environment config schema includes HTTPS section with flexible structure
- [ ] Configuration DTOs support optional HTTPS per service
- [ ] Configuration DTOs support multiple HTTP trackers with individual HTTPS control
- [ ] Configuration DTOs properly validate email format
- [ ] Configuration DTOs properly validate domain name format
- [ ] Configuration DTOs validate HTTP tracker port/name consistency
- [ ] HTTPS configuration is entirely optional (default: no HTTPS)
- [ ] When any endpoint configured, admin_email is required
- [ ] Admin email uses secrecy crate wrapper (not plain String)

**Security**:

- [ ] Caddy added to security scan workflow
- [ ] Security scan passes (vulnerabilities documented as acceptable)
- [ ] Admin email not exposed in logs or error messages
- [ ] Certificate data stored in Docker volumes (persisted)

**Documentation**:

- [ ] HTTPS setup guide created with complete examples
- [ ] User guide index updated with HTTPS documentation link
- [ ] Configuration examples include HTTPS scenarios
- [ ] Troubleshooting section covers common certificate issues
- [ ] ADR created documenting Caddy adoption decision

**Testing**:

- [ ] E2E tests pass with all services HTTPS configuration
- [ ] E2E tests pass with selective HTTPS (only some services)
- [ ] E2E tests pass with multiple HTTP trackers (mixed HTTPS/HTTP)
- [ ] E2E tests pass with no HTTPS configuration
- [ ] Template rendering tests pass for all HTTPS scenarios:
  - [ ] All services HTTPS
  - [ ] Single service HTTPS
  - [ ] Multiple trackers with selective HTTPS
  - [ ] No HTTPS (Caddy not deployed)
- [ ] Configuration validation tests pass for valid and invalid inputs:
  - [ ] Valid: admin_email with at least one endpoint
  - [ ] Invalid: admin_email without endpoints
  - [ ] Invalid: endpoints without admin_email
  - [ ] Valid: no HTTPS configuration at all
- [ ] WebSocket connectivity tested through Caddy proxy

**Production Verification**:

- [ ] Test deployment with all services HTTPS enabled
- [ ] Test deployment with only Tracker API HTTPS (verify API token security)
- [ ] Test deployment with multiple HTTP trackers, selective HTTPS
- [ ] Test deployment with no HTTPS (baseline HTTP-only)
- [ ] Verify Let's Encrypt certificates obtained automatically for each domain
- [ ] Verify all configured HTTPS domains accessible via HTTPS
- [ ] Verify non-HTTPS services remain accessible via HTTP
- [ ] Verify Grafana WebSocket connections work through Caddy
- [ ] Verify HTTP→HTTPS redirect works correctly for HTTPS-enabled services

## Related Documentation

**Evaluation and Research**:

- [Issue #270 - Evaluate Caddy for HTTPS Termination](https://github.com/torrust/torrust-tracker-deployer/issues/270)
- [docs/issues/270-evaluate-caddy-for-https-termination.md](./270-evaluate-caddy-for-https-termination.md)
- [docs/research/caddy-tls-proxy-evaluation/README.md](../research/caddy-tls-proxy-evaluation/README.md)
- [docs/research/caddy-tls-proxy-evaluation/security-scan.md](../research/caddy-tls-proxy-evaluation/security-scan.md)
- [docs/research/caddy-tls-proxy-evaluation/production-deployment.md](../research/caddy-tls-proxy-evaluation/production-deployment.md)

**Template System**:

- [docs/contributing/templates/tera.md](../contributing/templates/tera.md)
- [docs/contributing/templates/template-system-architecture.md](../contributing/templates/template-system-architecture.md)

**Configuration**:

- [schemas/README.md](../../schemas/README.md) - Schema generation documentation
- [schemas/environment-config.json](../../schemas/environment-config.json)
- [docs/decisions/configuration-dto-layer-placement.md](../decisions/configuration-dto-layer-placement.md)
- [src/application/command_handlers/create/config/README.md](../../src/application/command_handlers/create/config/README.md)

**Security**:

- [docs/contributing/secret-handling.md](../contributing/secret-handling.md)
- [docs/security/docker/scans/README.md](../security/docker/scans/README.md)
- [.github/workflows/docker-security-scan.yml](../../.github/workflows/docker-security-scan.yml)

**Architecture**:

- [docs/codebase-architecture.md](../codebase-architecture.md)
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md)
- [docs/contributing/error-handling.md](../contributing/error-handling.md)

**External Resources**:

- [Caddy Official Documentation](https://caddyserver.com/docs/)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Tera Template Documentation](https://keats.github.io/tera/docs/)

## Notes

### Production Reference

The production deployment at `/opt/torrust/` on Hetzner server (46.224.206.37) is fully functional and serves as the reference implementation. All configuration files there should be used as the source of truth for creating templates.

### Certificate Management

Let's Encrypt has rate limits:

- 50 certificates per registered domain per week
- 5 duplicate certificates per week

For testing, use Let's Encrypt staging environment to avoid hitting limits. Production should use the regular Let's Encrypt API.

**Using Staging Environment** (for testing):

Set `use_staging: true` in your environment configuration:

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": true
  }
}
```

This automatically configures Caddy to use `https://acme-staging-v02.api.letsencrypt.org/directory`.

**Important Notes**:

- Staging certificates will show browser warnings (not trusted by browsers)
- Use staging only for testing the HTTPS flow, not for production
- Staging has much higher rate limits than production
- For production, omit `use_staging` or set it to `false` (default)

### Domain Requirements

Users must own and control the domains they configure. DNS records must point to the deployment server IP before certificate acquisition can succeed.

### Backward Compatibility

HTTPS support is entirely optional (omit the `https` configuration section). Existing deployments and configurations continue to work without modification. This is a pure addition, not a breaking change.

### Flexibility and Use Cases

The independent HTTPS control per service enables various deployment patterns:

**Security-Focused**: HTTPS only for Tracker API (protect API token), HTTP for public trackers
**Compliance**: HTTPS for all external services, HTTP for internal monitoring (Grafana on VPN)
**Migration**: Gradual rollout - enable HTTPS per service as domains become available

### Future Enhancements

After this task is complete, consider:

- Support for custom certificates (not just Let's Encrypt)
- Support for HTTP→HTTPS redirect configuration
- Support for additional Caddy features (rate limiting, caching, etc.)
- Integration with DNS providers for DNS-01 challenge (for wildcard certificates)

### Security Scan Results

The Caddy 2.10 image has 4 known vulnerabilities (3 HIGH, 1 CRITICAL) in Go dependencies, not in Caddy core. Alpine base image is clean. All vulnerabilities have fixed versions available upstream. This is acceptable for deployment with monitoring. See [security-scan.md](../research/caddy-tls-proxy-evaluation/security-scan.md) for full analysis.

### Design Decision: Service-Based TLS Configuration

**Chosen Architecture**: Service-based TLS configuration where each service has an optional `tls` field containing only the domain.

**Configuration Structure**:

1. **Common HTTPS Config** (top-level, shared configuration only):

   ```json
   {
     "https": {
       "admin_email": "admin@example.com",
       "use_staging": false // optional, defaults to false
     }
   }
   ```

2. **Service-Level TLS** (each service declares its own TLS configuration):

   ```json
   {
     "tracker": {
       "http_api": {
         "bind_address": "0.0.0.0:1212",
         "admin_token": "...",
         "tls": {
           "domain": "api.torrust-tracker.com"
         }
       },
       "http_trackers": [
         {
           "bind_address": "0.0.0.0:7070",
           "tls": {
             "domain": "http1.torrust-tracker.com"
           }
         }
       ]
     },
     "grafana": {
       "admin_user": "admin",
       "admin_password": "...",
       "tls": {
         "domain": "grafana.torrust-tracker.com"
       }
     }
   }
   ```

**Rationale for Service-Based Approach**:

**Alternative Considered** (centralized approach):

We considered a centralized approach where all TLS configuration would be in a single `https` section:

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "tracker_api": {
      "domain": "api.torrust-tracker.com",
      "port": 1212
    },
    "http_trackers": [
      {
        "name": "http1",
        "domain": "http1.torrust-tracker.com",
        "port": 7070
      }
    ],
    "grafana": {
      "domain": "grafana.torrust-tracker.com",
      "port": 3000
    }
  }
}
```

**Why We Rejected the Centralized Approach**:

1. **Service Cohesion**: In service-based architecture, all configuration for a service should be co-located. If the tracker API uses HTTPS, the domain and any proxy-related settings belong with the tracker API configuration, not scattered across multiple sections.

2. **Torrust Conventions**: The Torrust Tracker follows a pattern where the presence of a configuration section indicates the feature is enabled. For example, `tracker.http_api` presence means API is enabled. Similarly, `tls` section presence should mean TLS is enabled for that specific service.

3. **Behavioral Coupling**: When a service uses TLS behind Caddy, it may need additional configuration (e.g., trust proxy headers to get original client IP). Keeping TLS config with the service makes these relationships explicit.

4. **Port Duplication**: Centralized approach would duplicate port information - ports are already defined in `bind_address` for each service. Service-based approach uses existing `bind_address` fields, maintaining single source of truth.

5. **Conditional Dependencies**: Like Grafana requiring Prometheus, TLS on any service requires Caddy. The service-based approach makes it natural to check "does any service have a `tls` section?" rather than maintaining parallel configuration structures.

6. **Maintenance**: Adding a new HTTP-based service (e.g., a web UI) would require updates in two places with centralized approach. Service-based approach only requires adding the service with optional `tls` field.

**Implementation Consequences**:

- Each service DTO gets `tls: Option<TlsConfig>` field
- `HttpsConfig` at top level contains only shared configuration (`admin_email`, `use_staging`)
- Templates use conditional rendering: `{% if tracker.http_api.tls %}`
- Validation checks: if any service has `tls` → `https.admin_email` required
- Port extraction from existing `bind_address` fields using Tera `extract_port` filter
