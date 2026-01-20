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
- [x] Implement security scanning for Caddy in CI/CD
- [x] Document HTTPS setup in user guide
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

- [x] Add `caddy:2.10` to security scan workflow matrix
- [x] Add SARIF upload step for Caddy scan results
- [x] Update `docs/security/docker/scans/README.md` with Caddy entry
- [x] Run security scan locally to verify configuration
- [x] Document vulnerability assessment (reference [docs/research/caddy-tls-proxy-evaluation/security-scan.md](../research/caddy-tls-proxy-evaluation/security-scan.md))

### Phase 5: Documentation (4-5 hours)

- [x] Create `docs/user-guide/services/https.md` with complete HTTPS setup guide:
  - [x] Overview of HTTPS architecture with Caddy
  - [x] Prerequisites (domain names, DNS configuration, firewall)
  - [x] **Global HTTPS configuration**: `admin_email` and `use_staging` options
  - [x] **Per-service TLS configuration**: `domain` and `use_tls_proxy` pattern
  - [x] Services supporting HTTPS (Tracker HTTP API, HTTP Trackers, Health Check API, Grafana)
  - [x] Let's Encrypt certificate process (automatic acquisition and renewal)
  - [x] **Let's Encrypt staging environment**: Document `use_staging: true` for testing (avoids rate limits)
  - [x] **Rate limits**: Document Let's Encrypt limits (50 certs/week, 5 duplicates/week)
  - [x] **Staging certificates warning**: Browser warnings expected (not trusted), only for testing
  - [x] Complete configuration example with all services HTTPS-enabled
  - [x] Verification commands for checking HTTPS functionality
  - [x] Troubleshooting section (DNS, firewall, certificates, Caddy logs)
  - [x] Architecture explanation (Caddy as TLS termination proxy)
- [x] Update `docs/user-guide/services/README.md` with HTTPS service entry
- [x] Update `docs/user-guide/README.md` with HTTPS reference in services section
- [x] Update `docs/user-guide/services/grafana.md` with TLS proxy fields documentation
- [x] Regenerate JSON schema (`schemas/environment-config.json`)

### Phase 6: E2E Testing (5-6 hours)

**Revised Strategy** (2026-01-20):

The original plan to test multiple HTTPS patterns is not feasible because the Torrust Tracker
has only one config option to enable the TLS proxy - we cannot have some HTTP trackers using
HTTPS while others use HTTP simultaneously. Instead, we'll take a simpler, more maintainable approach:

1. **Enable HTTPS for all HTTP trackers** in the E2E test configuration
2. **Use the `test` command** (smoke test) instead of manual validation
3. **Test non-HTTPS via UDP tracker** which never uses the Caddy proxy

This approach provides comprehensive HTTPS coverage while leveraging existing infrastructure.

Implementation Plan:

- **Step 1: Add smoke test execution to E2E workflow**
  - [ ] Add `run_smoke_tests()` method to `E2eTestRunner` in `src/testing/e2e/tasks/black_box/test_runner.rs`
    - [ ] Execute `cargo run --bin torrust-tracker-deployer -- test <env-name>` via `ProcessRunner`
    - [ ] The existing `test` command already supports HTTPS via `ServiceEndpoint::https()` with domain resolution
  - [ ] Call `test_runner.run_smoke_tests()` in `run_deployer_workflow()` after `run_services()`
  - [ ] Verify E2E tests pass on GitHub Actions (may require runner changes)
  - [ ] Commit and push to verify CI passes

- **Step 2: Enable HTTPS in E2E test configuration**
  - [ ] Modify `E2eConfigEnvironment::to_json_config()` in `src/testing/e2e/containers/tracker_ports.rs`:
    - [ ] Add `domain` and `use_tls_proxy: true` for each HTTP tracker
    - [ ] Add `domain` and `use_tls_proxy: true` for HTTP API
    - [ ] Add `domain` and `use_tls_proxy: true` for Grafana
    - [ ] Add `https` section with `admin_email` and `use_staging: true`
    - [ ] Use `.local` domains (e.g., `api.tracker.local`, `http1.tracker.local`)
  - [ ] Caddy's internal CA automatically handles `.local` domain certificates
  - [ ] Wait for Caddy certificate acquisition after `run_services()` (add brief delay or retry logic)

- **Step 3: Verify HTTPS E2E tests pass**
  - [ ] Run E2E tests locally: `cargo run --bin e2e-deployment-workflow-tests`
  - [ ] Verify `test` command validates HTTPS endpoints correctly
  - [ ] Verify Caddy logs show successful certificate acquisition
  - [ ] Run all linters and pre-commit checks
  - [ ] Push to GitHub and verify CI passes

**Configuration Example** (E2E test config):

```json
{
  "tracker": {
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http1.tracker.local",
        "use_tls_proxy": true
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "domain": "api.tracker.local",
      "use_tls_proxy": true,
      "admin_token": "MyAccessToken"
    }
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "e2e-test-password",
    "domain": "grafana.tracker.local",
    "use_tls_proxy": true
  },
  "https": {
    "admin_email": "admin@tracker.local",
    "use_staging": true
  }
}
```

**Non-HTTPS coverage** (tested implicitly):

- UDP tracker - never uses Caddy proxy, validates non-TLS path
- Health Check API - can be tested independently without TLS

**Automated E2E Tests** (deferred - may not be needed):

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

| File               | Location on VM                             | Location in Container                |
| ------------------ | ------------------------------------------ | ------------------------------------ |
| docker-compose.yml | `/opt/torrust/docker-compose.yml`          | N/A                                  |
| .env               | `/opt/torrust/.env`                        | N/A                                  |
| Caddyfile          | `/opt/torrust/storage/caddy/etc/Caddyfile` | `/etc/caddy/Caddyfile` (bind mount)  |
| Tracker config     | `/opt/torrust/storage/tracker/etc/`        | `/etc/torrust/tracker/` (bind mount) |
| Caddy certificates | Docker volume `caddy_data`                 | `/data/`                             |

**App directory**: The application is deployed to `/opt/torrust/`, **NOT** `/home/torrust/app/`. This is the working directory for docker compose commands on the VM.

```bash
# Example: Check running containers on the VM
ssh -i fixtures/testing_rsa torrust@<VM_IP> "cd /opt/torrust && docker compose ps"

# Example: View docker-compose.yml on the VM
ssh -i fixtures/testing_rsa torrust@<VM_IP> "cat /opt/torrust/docker-compose.yml"
```

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

### Phase 7: CLI Command Compatibility with HTTPS (3-4 hours)

When HTTPS is enabled, the deployer commands must adapt their behavior to work with domain-based URLs instead of direct IP addresses, and handle internal ports that are no longer directly accessible.

#### 7.1: Update `test` command for HTTPS-enabled environments

**Current Problem**: The `test` command validates services by accessing them directly via IP and internal ports (e.g., `http://10.140.190.214:1212/api/health_check`). When TLS is enabled for a service:

1. The internal port (e.g., 1212) is not exposed externally - only Caddy ports (80, 443) are exposed
2. The service should be accessed via its HTTPS domain (e.g., `https://api.tracker.local`)

**Current Behavior** (fails when TLS enabled):

```text
$ cargo run -- test manual-https-test

⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: manual-https-test (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Testing infrastructure...
❌ Test command failed: Validation failed for environment 'manual-https-test': Remote action failed: Action 'running-services-validation' validation failed: Tracker API external health check failed: error sending request for url (http://10.140.190.214:1212/api/health_check). Check that tracker is running and firewall allows port 1212.
```

**Required Changes**:

- [ ] Detect if a service has TLS enabled from environment configuration
- [ ] For TLS-enabled services:
  - [ ] Use the configured domain with HTTPS protocol instead of IP with internal port
  - [ ] For local/test domains (e.g., `.local`), accept self-signed certificates from Caddy's local CA
  - [ ] Show clear message: "Testing via HTTPS endpoint: https://api.tracker.local"
- [ ] For non-TLS services:
  - [ ] Continue using direct IP and port access as before
- [ ] Update error messages to clarify the HTTPS testing behavior

**Expected Behavior After Fix**:

```text
Testing Tracker API via HTTPS: https://api.tracker.local/api/health_check ✅
Testing HTTP Tracker (non-TLS): http://10.140.190.214:7072/announce ✅
```

#### 7.2: Update `show` command for HTTPS-enabled environments ✅ COMPLETE

**Current Problem**: The `show` command displays service endpoints using only IP addresses and internal ports, which are misleading when HTTPS is enabled:

1. Displayed URLs may not work (internal ports not exposed)
2. Users don't know the correct HTTPS URLs to use
3. No indication that domain-based access is required

**Current Behavior** (shows incorrect URLs when TLS enabled):

```text
$ cargo run -- show manual-https-test

Environment: manual-https-test
State: Running
Provider: LXD
Created: 2026-01-14 11:08:00 UTC

Infrastructure:
  Instance IP: 10.140.190.214
  SSH Port: 22
  SSH User: torrust
  SSH Key: /home/.../fixtures/testing_rsa

Connection:
  ssh -i /home/.../fixtures/testing_rsa torrust@10.140.190.214

Tracker Services:
  UDP Trackers:
    - udp://10.140.190.214:6969/announce
  HTTP Trackers:
    - http://10.140.190.214:7070/announce  # ❌ Port not exposed (TLS enabled)
    - http://10.140.190.214:7071/announce  # ❌ Port not exposed (TLS enabled)
    - http://10.140.190.214:7072/announce  # ✅ Works (no TLS)
  API Endpoint:
    - http://10.140.190.214:1212/api       # ❌ Port not exposed (TLS enabled)
  Health Check:
    - http://10.140.190.214:1313/health_check

Prometheus:
  Internal only (localhost:9090) - not exposed externally

Grafana:
  http://10.140.190.214:3100/              # ❌ Port not exposed (TLS enabled)

Services are running. Use 'test' to verify health.
```

**Required Changes**:

- [x] Detect if a service has TLS enabled from environment configuration
- [x] For TLS-enabled services:
  - [x] Show HTTPS URL with configured domain: `https://api.tracker.local`
  - [ ] Show HTTP redirect URL: `http://api.tracker.local` (redirects to HTTPS) _(deferred - not essential)_
  - [x] Add note: "Direct IP access not available when TLS is enabled"
- [x] For non-TLS services:
  - [x] Show direct IP URL as before: `http://10.140.190.214:7072`
- [x] Add informational section explaining:
  - [x] "Services with TLS enabled must be accessed via their configured domain"
  - [x] "For local domains (\*.local), add entries to /etc/hosts pointing to the VM IP"
  - [x] "Internal ports are not directly accessible when TLS is enabled"

**Expected Output After Fix**:

```text
Environment: manual-https-test
State: Running
Provider: LXD
Created: 2026-01-14 11:08:00 UTC

Infrastructure:
  Instance IP: 10.140.190.214
  SSH Port: 22
  SSH User: torrust

Tracker Services:
  UDP Trackers:
    - udp://10.140.190.214:6969/announce
  HTTP Trackers (HTTPS via Caddy):
    - https://http1.tracker.local/announce
    - https://http2.tracker.local/announce
  HTTP Trackers (direct):
    - http://10.140.190.214:7072/announce
  API Endpoint (HTTPS via Caddy):
    - https://api.tracker.local/api

Grafana (HTTPS via Caddy):
  https://grafana.tracker.local/

Prometheus:
  Internal only (localhost:9090) - not exposed externally

Note: HTTPS services require domain-based access. For local domains (*.local),
add the following to your /etc/hosts file:

  10.140.190.214   api.tracker.local http1.tracker.local http2.tracker.local grafana.tracker.local

Internal ports (1212, 7070, 7071, 3000) are not directly accessible when TLS is enabled.
```

#### 7.3: Add TLS Support for Health Check API ✅ COMPLETE

**Current State**: The health check API (`health_check_api`) doesn't support TLS configuration like other HTTP services (HTTP trackers, Tracker API, Grafana).

**Problem**: Users may want to expose the health check API publicly with HTTPS for external monitoring systems, load balancers, or orchestration tools that need to verify service health.

**Solution**: Add an optional `tls` field to the health check API configuration, following the same service-based TLS pattern used by other services.

**Configuration Change**:

```json
{
  "tracker": {
    "health_check_api": {
      "bind_address": "0.0.0.0:1313",
      "tls": {
        "domain": "health.tracker.local"
      }
    }
  }
}
```

**Implementation Scope**:

- [x] Add `tls: Option<TlsConfig>` to health check API domain model
- [x] Add `tls: Option<TlsConfig>` to health check API DTOs
- [x] Update Caddyfile template to include health check when TLS is configured
- [x] Update show command to display HTTPS URL when health check has TLS
- [ ] Update test command to use HTTPS for health check when TLS is configured (deferred to 7.1)

> **Note**: JSON schema regeneration deferred to Phase 8.

#### 7.4: Handle Localhost-Bound Services in Show Command and Validation ✅ COMPLETE

**Current State**:

- Services can bind to localhost (`127.0.0.1` or `::1`)
- If TLS is configured for such a service, Caddy cannot reach the backend (Caddy runs in a separate container, localhost is not shared between containers)
- The show command incorrectly displays public IP URLs for localhost-bound services

**Problem Example**: Configuration has `"bind_address": "127.0.0.1:1313"` but show command displays `http://10.140.190.190:1313/health_check` which won't work because the service is only listening on localhost.

**Solution** (two parts):

##### Part A: Validation at Create Time

Fail environment creation if any service has BOTH:

- TLS configuration (`tls` section present)
- Localhost bind address (`127.0.0.1` or `::1`)

**Error message example**:

```text
Error: Invalid configuration for health_check_api

  The service binds to localhost (127.0.0.1:1313) but has TLS configured.
  Caddy cannot proxy to localhost-bound services (different container network).

  To fix, either:
  - Remove the 'tls' section to keep the service internal-only
  - Change bind_address to '0.0.0.0:1313' to expose the service through Caddy
```

**Implementation Notes**:

- Validation occurs in the domain layer when converting DTO to domain object (similar to the Grafana→Prometheus dependency validation)
- This is an internal rule per service, checked during DTO-to-domain conversion
- Services to validate: `health_check_api`, `http_api`, `http_trackers` (each individually)
- Grafana excluded: bind address is hardcoded (port 3000), not user-configurable
- Localhost detection: Check for `127.0.0.1` and `::1` (IPv6 localhost) only, not entire ranges

##### Part B: Show Command for Localhost Services (without TLS)

For services bound to localhost WITHOUT TLS, display:

```text
Health Check:
  Internal only (localhost:1313) - access via SSH tunnel
```

Instead of the incorrect:

```text
Health Check:
  - http://10.140.190.190:1313/health_check
```

**Implementation Notes**:

- Add `is_localhost_only: bool` field to `ServiceInfo` for relevant services (don't put message in URL field)
- Show "Internal only" message for localhost-bound services - never hide services from output
- Principle: Keep user informed about everything. If keeping a service internal was an error, the user catches it sooner rather than wondering why the service is missing from output.

**Implementation Scope**:

- [x] Add validation in domain layer to reject localhost + TLS combinations (during DTO-to-domain conversion)
- [x] Update show command to detect localhost-bound services
- [x] Add `is_localhost_only` field to `ServiceInfo` for health check, API, and HTTP trackers
- [x] Display "Internal only" message for internal-only services
- [x] Apply to: health check API, HTTP API, HTTP trackers (Grafana excluded - hardcoded port)

#### 7.5: Fix `on_reverse_proxy` Tracker Configuration Bug

**Problem**:

The Torrust Tracker has a configuration option `[core.net].on_reverse_proxy` that tells the tracker whether it's running behind a reverse proxy. When `true`, the tracker expects the `X-Forwarded-For` HTTP header to get the real client IP instead of the proxy's IP. This is critical for HTTP trackers to correctly identify peers.

Currently, in `templates/tracker/tracker.toml.tera`, this option is **hardcoded to `true`**:

```toml
[core.net]
on_reverse_proxy = true
```

This is wrong because:

1. When an HTTP tracker is exposed directly (no Caddy proxy), the tracker expects `X-Forwarded-For` headers that won't exist, causing incorrect peer identification
2. The current implementation assumes all HTTP trackers with TLS go through Caddy, but users might want to use the tracker's built-in TLS support without a proxy

**Tracker Configuration Limitation**:

The `on_reverse_proxy` option is **global** (in `[core.net]`), not per-tracker. This means:

- ALL HTTP trackers share the same setting
- You cannot have some trackers behind a proxy and others direct in the same deployment
- If ANY tracker uses a proxy, ALL trackers must be configured for proxy mode

This is a limitation in the Torrust Tracker itself (not the deployer). A proper fix would require the tracker to support per-tracker `on_reverse_proxy` settings.

**Upstream Issue**: [torrust/torrust-tracker#1640](https://github.com/torrust/torrust-tracker/issues/1640)

**How to Reproduce**:

1. Deploy the manual test environment with mixed TLS/non-TLS HTTP trackers:

   ```bash
   cargo run -- show manual-https-test
   ```

2. Verify the tracker config has `on_reverse_proxy = true` (set because trackers 7070, 7071 use TLS proxy):

   ```bash
   cat build/manual-https-test/tracker/tracker.toml | grep -A2 "core.net"
   # Output: [core.net]
   #         on_reverse_proxy = true
   ```

3. Make a direct HTTP announce request to the tracker on port 7072 (no proxy):

   ```bash
   curl -v "http://<VM_IP>:7072/announce?info_hash=%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00%00&peer_id=-TR3000-000000000000&port=6881&uploaded=0&downloaded=0&left=0&event=started"
   ```

4. Observe the failure response:

   ```text
   d14:failure reason208:Error resolving peer IP: missing or invalid the right most
   X-Forwarded-For IP (mandatory on reverse proxy tracker configuration)e
   ```

The tracker on port 7072 expects `X-Forwarded-For` header (due to global `on_reverse_proxy = true`) but doesn't receive it from direct requests, causing the announce to fail.

**Solution**:

Rename `tls` to a clearer structure with `domain` at the top level and `use_tls_proxy` as a separate boolean. The `tls` name was misleading because it doesn't map to the tracker's TLS config - the domain is only used for Caddy proxy configuration.

**Before** (current - using `tls` object):

```json
{
  "environment": {
    "name": "manual-https-test"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/fixtures/testing_rsa",
    "public_key_path": "/path/to/fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-manual-https-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "tls": {
          "domain": "http1.tracker.local"
        }
      },
      {
        "bind_address": "0.0.0.0:7071",
        "tls": {
          "domain": "http2.tracker.local"
        }
      },
      {
        "bind_address": "0.0.0.0:7072"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "tls": {
        "domain": "api.tracker.local"
      }
    },
    "health_check_api": {
      "bind_address": "0.0.0.0:1313",
      "tls": {
        "domain": "health.tracker.local"
      }
    }
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin-password",
    "tls": {
      "domain": "grafana.tracker.local"
    }
  },
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "https": {
    "admin_email": "admin@tracker.local",
    "use_staging": true
  }
}
```

**After** (proposed - using `domain` + `use_tls_proxy`):

```json
{
  "environment": {
    "name": "manual-https-test"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/fixtures/testing_rsa",
    "public_key_path": "/path/to/fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-manual-https-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http1.tracker.local",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7071",
        "domain": "http2.tracker.local",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7072"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "domain": "api.tracker.local",
      "use_tls_proxy": true
    },
    "health_check_api": {
      "bind_address": "0.0.0.0:1313",
      "domain": "health.tracker.local",
      "use_tls_proxy": true
    }
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin-password",
    "domain": "grafana.tracker.local",
    "use_tls_proxy": true
  },
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "https": {
    "admin_email": "admin@tracker.local",
    "use_staging": true
  }
}
```

**Configuration Semantics**:

| `domain` | `use_tls_proxy` | Meaning                                                   |
| -------- | --------------- | --------------------------------------------------------- |
| absent   | absent          | Direct HTTP, no proxy                                     |
| present  | absent          | HTTP with domain (for future use, e.g., DNS-based access) |
| present  | `true`          | HTTPS via Caddy proxy (TLS termination)                   |
| absent   | `true`          | **INVALID** - TLS proxy needs domain for virtual host     |

**Why `use_tls_proxy` (not `on_reverse_proxy`)?**:

The name `use_tls_proxy` accurately describes what our Caddy proxy does: **TLS termination**. This naming choice is intentional for future compatibility:

1. **Current state**: The tracker has a global `[core.net].on_reverse_proxy` option
2. **Future state**: The tracker may add per-tracker `on_reverse_proxy` support
3. **No conflict**: When that happens, we can expose both options without ambiguity:

```json
{
  "bind_address": "0.0.0.0:7071",
  "domain": "http2.tracker.local",
  "use_tls_proxy": true,
  "on_reverse_proxy": true
}
```

**Dependency Rule**: `use_tls_proxy: true` → tracker's `on_reverse_proxy` MUST be `true`. This is enforced automatically:

- When `use_tls_proxy: true`, the deployer sets the tracker's `[core.net].on_reverse_proxy = true`
- This is because Caddy sends `X-Forwarded-For` headers that the tracker must read

**Future Compatibility**: If the tracker adds per-tracker `on_reverse_proxy`:

- `use_tls_proxy` controls Caddy inclusion and implies `on_reverse_proxy: true`
- `on_reverse_proxy` could be explicitly set for edge cases (non-TLS reverse proxy)
- Validation: `use_tls_proxy: true` + `on_reverse_proxy: false` = **INVALID**

**Behavior**:

1. **Tracker config** (`[core.net].on_reverse_proxy`):
   - Set to `true` if ANY HTTP tracker has `use_tls_proxy: true`
   - Set to `false` otherwise
   - Note: This only affects HTTP trackers; other services ignore it

2. **Caddy config** (Caddyfile):
   - Include service in Caddy config only if `use_tls_proxy: true`
   - Requires `domain` to be present for the virtual host configuration

3. **Validation rules**:
   - `use_tls_proxy: true` requires `domain` to be present
   - Localhost bind addresses with `use_tls_proxy: true` should be rejected (proxy can't reach localhost)

**Known Limitation** (due to tracker's global setting):

If you have multiple HTTP trackers where some use `use_tls_proxy` and others don't, the ones without it will still receive the global `on_reverse_proxy = true` setting and may fail if they receive direct requests without `X-Forwarded-For` headers.

**Workaround**: Ensure all HTTP trackers in a deployment either ALL use the TLS proxy or NONE use it.

**Reference**: [Torrust Tracker Network Configuration](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/v2_0_0/network/struct.Network.html)

**Implementation Scope**:

The implementation is split into incremental steps, one service type at a time, to minimize risk and simplify review.

##### Step 7.5.1: HTTP Trackers

- [x] Add `domain: Option<String>` and `use_tls_proxy: Option<bool>` to `HttpTrackerSection` DTO
- [x] Update `HttpTrackerConfig` domain type to include `use_tls_proxy` and `domain`
- [x] Add validation: `use_tls_proxy: true` requires `domain` to be present
- [x] Add validation: `use_tls_proxy: true` with localhost bind address → reject
- [x] Update tracker config template (`templates/tracker/tracker.toml.tera`) to conditionally set `on_reverse_proxy` based on ANY HTTP tracker having `use_tls_proxy: true`
- [x] Update Caddy template (`templates/caddy/Caddyfile.tera`) to check `use_tls_proxy` for HTTP trackers
- [x] Update show command `ServiceInfo` for HTTP trackers
- [x] Update `envs/manual-https-test.json` for HTTP trackers only
- [x] Remove `TlsSection` from HTTP trackers (keep in other services temporarily)
- [x] Add unit tests for HTTP tracker validation
- [x] Run E2E tests to verify HTTP trackers work

##### Step 7.5.2: Tracker REST API

- [x] Add `domain: Option<String>` and `use_tls_proxy: Option<bool>` to `HttpApiSection` DTO
- [x] Update `HttpApiConfig` domain type
- [x] Add validation rules (same as HTTP trackers)
- [x] Update Caddy template for API
- [x] Update show command `ServiceInfo` for API
- [x] Update `envs/manual-https-test.json` for API
- [x] Remove `TlsSection` from API
- [x] Add unit tests for API validation
- [x] Run E2E tests

##### Step 7.5.3: Tracker Health Check API

- [x] Add `domain: Option<String>` and `use_tls_proxy: Option<bool>` to `HealthCheckApiSection` DTO
- [x] Update `HealthCheckApiConfig` domain type
- [x] Add validation rules
- [x] Update Caddy template for health check
- [x] Update show command `ServiceInfo` for health check
- [x] Update `envs/manual-https-test.json` for health check
- [x] Remove `TlsSection` from health check
- [x] Add unit tests
- [x] Run E2E tests

##### Step 7.5.4: Grafana

- [x] Add `domain: Option<String>` and `use_tls_proxy: Option<bool>` to `GrafanaSection` DTO
- [x] Update `GrafanaConfig` domain type
- [x] Add validation rules (note: Grafana has no configurable bind address, so localhost validation not needed)
- [x] Update Caddy template for Grafana
- [x] Update show command `ServiceInfo` for Grafana
- [x] Update `envs/manual-https-test.json` for Grafana
- [x] Remove `TlsSection` from Grafana
- [x] Add unit tests
- [x] Run E2E tests

##### Step 7.5.5: Cleanup and Final Verification

- [x] Remove `TlsSection` type completely (should be unused after all services migrated)
- [x] Remove `domain::tls` module completely (unused after migration)
- [x] Run full E2E test suite
- [x] Run all linters
- [x] Manual verification with `envs/manual-https-test.json`

### Phase 8: Schema Generation (30 minutes)

- [x] Regenerate JSON schema from Rust DTOs:

  ```bash
  cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
  ```

- [x] Verify schema includes HTTPS configuration section
- [x] Verify schema validation rules match Rust DTO constraints
- [x] Test schema with example HTTPS-enabled environment file
- [x] Commit updated schema file

### Phase 9: Create ADR (1 hour)

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

**CLI Command Compatibility**:

- [ ] `test` command works correctly with HTTPS-enabled services:
  - [ ] Uses HTTPS domain URLs for TLS-enabled services
  - [ ] Uses direct IP/port for non-TLS services
  - [ ] Accepts self-signed certificates for local domains (e.g., `*.local`)
  - [ ] Shows clear message indicating HTTPS test mode
- [ ] `show` command displays correct endpoints:
  - [ ] Shows HTTPS URLs with domains for TLS-enabled services
  - [ ] Shows direct IP/port for non-TLS services
  - [ ] Includes note about domain-based access requirement
  - [ ] Provides `/etc/hosts` configuration hint for local domains
  - [ ] Clarifies internal ports are not accessible when TLS is enabled

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
