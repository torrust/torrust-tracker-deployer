# Decision: Per-Service TLS Configuration with domain + use_tls_proxy

## Status

Accepted

## Date

2026-01-20

## Context

When implementing HTTPS support for the deployer, we needed to decide how to configure TLS for each service. The deployer manages multiple HTTP services:

- Tracker REST API (HTTP API)
- One or more HTTP Trackers
- Grafana monitoring UI
- Health Check API

Key design considerations:

1. **Flexibility**: Some services might need HTTPS while others use HTTP
2. **Domain routing**: Caddy routes by domain name (subdomain-based routing)
3. **TLS proxy opt-in**: Services should explicitly opt into TLS proxying
4. **Configuration clarity**: Users should clearly see what each setting controls
5. **Validation**: Invalid combinations should be caught at configuration time

### Original Design (tls section)

The initial design used a nested `tls` section within each service:

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin",
    "tls": {
      "domain": "grafana.example.com"
    }
  }
}
```

This implied: if `tls` section exists, use HTTPS.

### Problem with Original Design

1. **Implicit activation**: Presence of section implies activation (no explicit flag)
2. **Domain without TLS**: What if user wants domain for other purposes but not TLS?
3. **TLS without domain**: Error case is less obvious
4. **Nested complexity**: Extra level of nesting for just one field

## Decision

We use **flat configuration with two explicit fields** at the service level:

- `domain` (string, optional) - Domain name for the service
- `use_tls_proxy` (boolean, optional) - Whether to route through Caddy for HTTPS

### Implementation

Each service configuration includes these optional fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrafanaSection {
    pub admin_user: String,
    pub admin_password: PlainPassword,

    /// Domain name for external HTTPS access (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Whether to use TLS proxy via Caddy (default: false)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tls_proxy: Option<bool>,
}
```

### Configuration Matrix

| domain | use_tls_proxy | Result                                         |
| ------ | ------------- | ---------------------------------------------- |
| None   | None/false    | HTTP access via IP:port                        |
| Some   | false         | Domain configured but not proxied (future use) |
| Some   | true          | HTTPS via Caddy with automatic certificates    |
| None   | true          | **Validation Error** - TLS requires domain     |

### Example Configurations

**HTTPS-enabled Grafana**:

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin",
    "domain": "grafana.example.com",
    "use_tls_proxy": true
  }
}
```

**HTTP-only Grafana (no TLS)**:

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin"
  }
}
```

**Domain without TLS (future-proofing)**:

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin",
    "domain": "grafana.example.com"
  }
}
```

### Global HTTPS Section

A global `https` section provides configuration shared across all TLS-enabled services:

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": false
  }
}
```

This section is **required** when any service has `use_tls_proxy: true`.

### Validation Rules

1. If `use_tls_proxy: true` is set, `domain` is required for that service
2. If any service has `use_tls_proxy: true`, global `https.admin_email` is required
3. If `https.admin_email` is provided but no service uses TLS proxy, validation fails

### Naming Choice: `use_tls_proxy` vs `use_tls`

We deliberately chose the name `use_tls_proxy` instead of `use_tls` or `tls_enabled` to avoid confusion with the tracker's **native TLS support**.

The Torrust Tracker supports TLS termination directly without a proxy through its [`TslConfig`](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/struct.TslConfig.html) configuration:

```rust
pub struct TslConfig {
    pub ssl_cert_path: Utf8PathBuf,
    pub ssl_key_path: Utf8PathBuf,
}
```

This native TLS configuration is available for [`HttpTracker`](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/type.HttpTracker.html) via the `tsl_config` field.

**Why this matters for the deployer**:

- **Future feature**: The deployer may eventually support enabling native TLS on the tracker (without a reverse proxy)
- **Naming conflict**: Using `use_tls` could create ambiguity between proxy-based and native TLS
- **Clear semantics**: `use_tls_proxy` explicitly indicates "route traffic through Caddy for TLS termination"

By using `use_tls_proxy`, we reserve the namespace for potential future fields like:

- `use_native_tls` - Enable TLS directly on the tracker
- `tls_cert_path` / `tls_key_path` - Certificate paths for native TLS

This naming also aligns with the related decision about [uniform HTTP tracker TLS requirement](./uniform-http-tracker-tls-requirement.md), where all HTTP trackers must share the same TLS proxy setting due to the tracker's global `on_reverse_proxy` configuration.

## Consequences

### Positive

- **Explicit activation**: `use_tls_proxy: true` clearly states intent
- **Flat structure**: No nested `tls` section, easier to read and edit
- **Future flexibility**: `domain` can be used for other purposes without TLS
- **Clear validation**: Invalid combinations produce specific error messages
- **Symmetry**: Same pattern used for all services (HTTP API, HTTP trackers, Grafana, Health Check)

### Negative

- **Two fields instead of one**: More verbose when enabling TLS
- **Refactoring cost**: Required changing existing design mid-implementation

### Neutral

- **Boolean defaults**: `use_tls_proxy` defaults to `false` (HTTP)

## Alternatives Considered

### 1. Nested tls Section (Original Design)

```json
{
  "grafana": {
    "tls": {
      "domain": "grafana.example.com"
    }
  }
}
```

**Rejected because**:

- Implicit activation (section presence = enabled)
- Extra nesting for single field
- Domain and TLS tightly coupled

### 2. Single Boolean with Auto-Domain

```json
{
  "grafana": {
    "use_https": true
  }
}
```

**Rejected because**:

- Domain still needed for Caddy routing
- Would require deriving domain from environment name (too magical)

### 3. Domain-Only (Implicit TLS)

```json
{
  "grafana": {
    "domain": "grafana.example.com"
  }
}
```

**Rejected because**:

- Domain might be wanted without TLS in future use cases
- No explicit opt-in for TLS proxy

## Related Decisions

- [caddy-for-tls-termination.md](./caddy-for-tls-termination.md) - Why Caddy was chosen
- [uniform-http-tracker-tls-requirement.md](./uniform-http-tracker-tls-requirement.md) - Why all HTTP trackers must use same TLS setting

## References

- [Issue #272 - Add HTTPS Support with Caddy](https://github.com/torrust/torrust-tracker-deployer/issues/272)
- [Commit: Replace tls with domain+use_tls_proxy](https://github.com/torrust/torrust-tracker-deployer/pull/273) (refactoring commits)
- [Tracker HttpTracker Configuration](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/type.HttpTracker.html) - Native HTTP tracker configuration with optional TLS
- [Tracker TslConfig Documentation](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/struct.TslConfig.html) - Native TLS configuration for direct TLS termination
