# Docker Compose Topology Domain Model

## 📋 Overview

This refactoring plan addresses architectural issues in the docker-compose template rendering where infrastructure topology rules (networks, volumes, dependencies) are scattered between Rust code and Tera templates. The goal is to move all topology decisions to the domain layer, enforcing invariants and making the template a pure rendering layer.

**Target Files:**

- `templates/docker-compose/docker-compose.yml.tera`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/mod.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/builder.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/tracker.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/caddy.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/mysql.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/prometheus.rs`
- `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/grafana.rs`
- New: `src/domain/deployment/topology/` (new domain module)

**Scope:**

- Convert all named volumes to bind mounts (unified `./storage/` directory)
- Standardize bind mount representation in template context
- Create domain types for Docker Compose topology (networks, dependencies)
- Derive global network lists from service configurations (single source of truth)
- Move topology decision logic from template conditionals to domain layer
- Enforce invariants like "if a service uses a network, that network must be defined"

## 📊 Progress Tracking

**Total Active Proposals**: 6  
**Pre-Phase Tasks**: 1  
**Bugs to Fix**: 1  
**Total Postponed**: 0  
**Total Discarded**: 0  
**Completed**: 6  
**In Progress**: 0  
**Not Started**: 2

### Phase Summary

- **Bugs to Fix (Priority)**: ✅ 1/1 fixed (100%) - Fix before or during relevant phase
- **Pre-Phase - ADR Documentation**: ✅ 1/1 completed (100%)
- **Phase 0 - Bind Mount Standardization (Medium Impact, Low Effort)**: ✅ 2/2 completed (100%)
- **Phase 1 - Domain Network Types (High Impact, Medium Effort)**: ✅ 2/2 completed (100%)
- **Phase 2 - Topology Aggregate & Network Derivation (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)

### Master Task List

This table provides a single view of all work items for easy tracking:

| ID     | Phase     | Task                                                        | Status         | Depends On | Notes                                                                                     |
| ------ | --------- | ----------------------------------------------------------- | -------------- | ---------- | ----------------------------------------------------------------------------------------- |
| BUG-01 | Priority  | Remove invalid "Grafana without Prometheus" template branch | ✅ Completed   | -          | Dead code, see [BUG-01](#bug-01-template-handles-invalid-grafana-without-prometheus-case) |
| ADR-01 | Pre-Phase | Create ADR for Bind Mount Standardization                   | ✅ Completed   | -          | See [Pre-Phase Task](#pre-phase-task-create-adr-for-bind-mount-standardization)           |
| P0.1   | Phase 0   | Convert Named Volumes to Bind Mounts                        | ✅ Completed   | ADR-01     | See [Proposal 0.1](#proposal-01-convert-named-volumes-to-bind-mounts)                     |
| P0.2   | Phase 0   | Create BindMount Domain Type                                | ✅ Completed   | P0.1       | See [Proposal 0.2](#proposal-02-create-bindmount-domain-type)                             |
| P1.1   | Phase 1   | Create Network Domain Types                                 | ✅ Completed   | -          | See [Proposal 1.1](#proposal-11-create-network-domain-types)                              |
| P1.2   | Phase 1   | Migrate Service Configs to Use Network Enum                 | ✅ Completed   | P1.1       | See [Proposal 1.2](#proposal-12-migrate-service-configs-to-use-network-enum)              |
| P2.1   | Phase 2   | Create DockerComposeTopology Aggregate                      | ⏳ Not Started | P1.2       | See [Proposal 2.1](#proposal-21-create-dockercomposetopology-aggregate)                   |
| P2.2   | Phase 2   | Derive Required Networks in Context                         | ⏳ Not Started | P2.1       | See [Proposal 2.2](#proposal-22-derive-required-networks-in-context)                      |

**Status Legend**:

- ⏳ Not Started
- 🔄 In Progress
- ✅ Completed
- ⏸️ Blocked
- 🗑️ Discarded
- 📅 Postponed

### Completion History

Track completed items with dates and any relevant notes:

| Date       | ID     | Task                                                        | Commit/PR                                                               | Notes                                               |
| ---------- | ------ | ----------------------------------------------------------- | ----------------------------------------------------------------------- | --------------------------------------------------- |
| 2025-07-17 | ADR-01 | Create ADR for Bind Mount Standardization                   | [PR #289](https://github.com/torrust/torrust-tracker-deployer/pull/289) | Documented 9 reasons for bind mount standardization |
| 2025-07-17 | BUG-01 | Remove invalid "Grafana without Prometheus" template branch | [PR #291](https://github.com/torrust/torrust-tracker-deployer/pull/291) | Removed dead code from docker-compose template      |
| 2025-07-17 | P0.1   | Convert Named Volumes to Bind Mounts                        | [PR #293](https://github.com/torrust/torrust-tracker-deployer/pull/293) | Bind mounts with BindMount domain type              |
| 2025-07-17 | P0.2   | Create BindMount Domain Type                                | [PR #293](https://github.com/torrust/torrust-tracker-deployer/pull/293) | Storage playbooks with proper ownership             |
| 2025-07-17 | P1.1   | Create Network Domain Types                                 | [PR #295](https://github.com/torrust/torrust-tracker-deployer/pull/295) | Network enum in src/domain/topology/                |
| 2025-07-17 | P1.2   | Migrate Service Configs to Use Network Enum                 | [PR #295](https://github.com/torrust/torrust-tracker-deployer/pull/295) | All service configs use Vec<Network>                |

### Discarded Proposals

None at this time.

### Postponed Proposals

None at this time.

---

## 🚀 Implementation Strategy

This refactoring will be implemented as **5 separate PRs** to ensure safe, incremental progress with reviewable chunks.

### PR Plan

| PR       | Scope                          | Tasks      | Rationale                                                       |
| -------- | ------------------------------ | ---------- | --------------------------------------------------------------- |
| **PR 1** | ADR Documentation              | ADR-01     | Documentation first - establishes the "why" before code changes |
| **PR 2** | Bug Fix                        | BUG-01     | Small, independent fix - can be reviewed/merged quickly         |
| **PR 3** | Phase 0: Bind Mount Foundation | P0.1, P0.2 | Foundation work - other phases depend on this                   |
| **PR 4** | Phase 1: Network Domain Types  | P1.1, P1.2 | Domain types - clear boundary                                   |
| **PR 5** | Phase 2: Topology Aggregate    | P2.1, P2.2 | Final integration                                               |

### PR Dependencies

```text
PR 1 (ADR-01)
    │
    ▼
PR 3 (Phase 0) ◄─── PR 2 (BUG-01) can be merged independently
    │
    ▼
PR 4 (Phase 1)
    │
    ▼
PR 5 (Phase 2)
```

### Benefits of Multi-PR Approach

1. **Reviewable chunks**: Each PR is ~200-400 lines vs 1000+
2. **Early feedback**: Can catch issues before building on top
3. **Natural boundaries**: Each phase is self-contained
4. **Easy bisection**: If something breaks, easier to identify the cause
5. **Aligns with development principles**: "Safe, incremental refactoring"

### GitHub Issues

This refactoring is tracked by an **Epic issue** with child issues for each PR:

**Epic**: `[Epic] Docker Compose Topology Domain Model Refactoring`

| Issue   | Title                                                                 | PR   |
| ------- | --------------------------------------------------------------------- | ---- |
| Child 1 | `[ADR] Bind Mount Standardization for Docker Compose`                 | PR 1 |
| Child 2 | `[BUG] Remove invalid "Grafana without Prometheus" template branch`   | PR 2 |
| Child 3 | `[Refactor] Phase 0: Convert volumes to bind mounts with domain type` | PR 3 |
| Child 4 | `[Refactor] Phase 1: Create Network domain types`                     | PR 4 |
| Child 5 | `[Refactor] Phase 2: Create DockerComposeTopology aggregate`          | PR 5 |

---

## 🎯 Key Problems Identified

### 1. Mixed Volume Mount Types (Named vs Bind)

The template mixes two volume mount formats:

```yaml
# Bind mounts (host path → container path) - PREFERRED
- ./storage/tracker/lib:/var/lib/torrust/tracker:Z

# Named volumes (volume name → container path) - PROBLEMATIC
- caddy_data:/data
```

Even within the same service (Caddy), both formats are used:

```yaml
volumes:
  - ./storage/caddy/etc/Caddyfile:/etc/caddy/Caddyfile:ro # Bind mount ✅
  - caddy_data:/data # Named volume ❌
  - caddy_config_vol:/config # Named volume ❌
```

**Problems with named volumes:**

- Data is hidden in `/var/lib/docker/volumes/` - not obvious to users
- Harder to back up - requires `docker volume` commands or finding the path
- Inconsistent with the rest of our storage strategy
- Named volumes must be declared in top-level `volumes:` section (extra complexity)

**Decision**: Use bind mounts exclusively. All persistent data goes to `./storage/{service}/` for:

- **Observability**: Users can see exactly where data is stored
- **Backup simplicity**: Just copy `./storage/` to back up everything
- **Consistency**: Same pattern for all services

### 2. Dual Sources of Truth for Networks

Service networks are computed dynamically in Rust:

```rust
// In TrackerServiceConfig::compute_networks()
fn compute_networks(has_prometheus: bool, has_mysql: bool, has_caddy: bool) -> Vec<String> {
    let mut networks = Vec::new();
    if has_prometheus { networks.push("metrics_network".to_string()); }
    if has_mysql { networks.push("database_network".to_string()); }
    if has_caddy { networks.push("proxy_network".to_string()); }
    networks
}
```

But the global network definitions are hardcoded in the template:

```yaml
networks:
{%- if mysql %}
  database_network:
    driver: bridge
{%- endif %}
```

**Problem**: These two sources can diverge. If a service adds a new network in Rust, the template must also be updated. Domain invariants are unenforceable.

### 3. Template Contains Domain Logic

The Tera template has conditionals that encode business rules:

```yaml
{%- if mysql or grafana or caddy %}
volumes:
{%- if mysql %}
  mysql_data:
    driver: local
{%- endif %}
```

**DDD Violation**: Templates should render, not decide. The domain should determine which resources are required based on service configurations.

### 4. Implicit Service Dependencies

Service dependency relationships are encoded directly in the template:

```yaml
mysql:
  depends_on:
    - tracker # Implicit: tracker depends on mysql for database
```

**Problem**: These relationships are business rules that should be in the domain, not scattered in templates.

---

## 🐛 Bugs Discovered During Analysis

During the review of domain rules encoded in the template, several bugs were discovered. These represent invalid configurations that the template handles but should never occur. **These bugs should be fixed as part of the refactoring, ideally before or during the relevant phase.**

### BUG-01: Template Handles Invalid "Grafana without Prometheus" Case

**Severity**: Medium (dead code / impossible state)  
**Location**: `templates/docker-compose/docker-compose.yml.tera` (Grafana depends_on section)  
**Related Rule**: DEP-04

**Problem**:

The template has an `{%- else %}` branch for when Grafana is enabled but Prometheus is NOT:

```yaml
grafana:
  depends_on:
{%- if prometheus %}
    prometheus:
      condition: service_healthy
{%- else %}
    - tracker  # <-- This branch handles an INVALID case
{%- endif %}
```

**Why it's a bug**:

- Grafana requires Prometheus as its data source - it has no purpose without it
- The environment creation should fail validation if `grafana.enabled = true` and `prometheus.enabled = false`
- This template branch can NEVER be reached in valid configurations
- The code is dead code that misleads readers into thinking this is a valid configuration

**Expected Behavior**:

1. Environment validation should reject `grafana.enabled && !prometheus.enabled`
2. Template should NOT have this `{%- else %}` branch at all
3. When Grafana is rendered, Prometheus is ALWAYS also enabled (invariant)

**Fix Required**:

1. Verify that environment validation already rejects this case (if not, add validation)
2. Remove the `{%- else %}` branch from the template
3. Simplify to always use `prometheus` dependency when Grafana is enabled

**Phase**: Should be fixed in Phase 1 (Domain Network Types) or earlier as a quick cleanup

---

### BUG-02: (Reserved for future bugs)

_Add additional bugs here as they are discovered during plan review._

---

## 🔍 Inconsistencies Discovered

This section documents inconsistencies found during plan review, both internal and with other repository documentation.

### Internal Inconsistencies Within This Document

**None found** ✅ - The document is internally consistent.

### Inconsistencies With Other Documentation

#### ISSUE-01: ADR grafana-integration-pattern.md Recommends Named Volumes

**Location**: [docs/decisions/grafana-integration-pattern.md](../../decisions/grafana-integration-pattern.md)

**Conflict**: The ADR explicitly states:

> "Grafana uses a named Docker volume, not a bind mount."
>
> **Rationale**:
>
> - Standard Grafana practice (official Grafana Docker documentation uses named volumes)
> - Named volumes are managed by Docker (automatic creation, cleanup)
> - Simpler for users (no host directory permissions issues)

But this refactoring plan proposes converting Grafana to use bind mounts.

**Resolution**: The ADR was written before the decision to standardize on bind mounts. When implementing Phase 0:

1. Create new ADR `bind-mount-standardization.md` (already planned as Pre-Phase Task)
2. This new ADR **supersedes** the volume-related decisions in `grafana-integration-pattern.md`
3. Update `grafana-integration-pattern.md` status to "Partially Superseded" with a reference to the new ADR
4. **Add to Phase 0 Implementation Checklist**: "Update grafana-integration-pattern.md ADR status"

---

#### ISSUE-02: Template Uses Named Volume + Bind Mount for Grafana (Mixed Pattern)

**Location**: `templates/docker-compose/docker-compose.yml.tera` (lines 147-149)

**Current State**:

```yaml
volumes:
  - grafana_data:/var/lib/grafana # Named volume (data)
  - ./storage/grafana/provisioning:/etc/grafana/provisioning:ro # Bind mount (config)
```

**Issue**: Grafana already uses BOTH patterns in the same service:

- Named volume for data persistence
- Bind mount for provisioning config

**Impact**: This inconsistency will be resolved by Phase 0, but it's worth noting that Grafana's bind mount for provisioning already follows our target pattern.

---

#### ISSUE-03: ADR grafana-integration-pattern.md Says "No Health Check" but Template Has One

**Location**: [docs/decisions/grafana-integration-pattern.md](../../decisions/grafana-integration-pattern.md) Section 5

**ADR States**:

> "Grafana service uses simple `depends_on` without health checks."

**Template Has**:

```yaml
grafana:
  healthcheck:
    test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/api/health"]
    interval: 10s
    ...
```

**Resolution**: The template was updated AFTER the ADR was written. The ADR is outdated but this is a positive improvement, not a problem. Consider updating the ADR to reflect current state as a separate cleanup task.

---

### Summary of Required Actions

| Issue    | Action                                                                       | Phase                  |
| -------- | ---------------------------------------------------------------------------- | ---------------------- |
| ISSUE-01 | Update `grafana-integration-pattern.md` ADR status to "Partially Superseded" | Phase 0                |
| ISSUE-03 | Update `grafana-integration-pattern.md` to reflect healthcheck addition      | N/A (separate cleanup) |

---

## 🧪 Domain Rules Analysis

This section documents all domain logic currently encoded in the template that must be moved to Rust and covered by unit tests.

### Network Rules

| Rule ID | Rule Description                                                                   | Current Location                                 | Test Name                                                                    |
| ------- | ---------------------------------------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------------------------------------- |
| NET-01  | If MySQL is enabled, `database_network` must be in required networks               | Template: `{%- if mysql %}`                      | `it_should_include_database_network_when_mysql_enabled`                      |
| NET-02  | If Prometheus is enabled, `metrics_network` must be in required networks           | Template: `{%- if prometheus %}`                 | `it_should_include_metrics_network_when_prometheus_enabled`                  |
| NET-03  | If Grafana is enabled, `visualization_network` must be in required networks        | Template: `{%- if grafana %}`                    | `it_should_include_visualization_network_when_grafana_enabled`               |
| NET-04  | If Caddy is enabled, `proxy_network` must be in required networks                  | Template: `{%- if caddy %}`                      | `it_should_include_proxy_network_when_caddy_enabled`                         |
| NET-05  | Tracker must be on `database_network` when MySQL is enabled                        | Rust: `TrackerServiceConfig::compute_networks()` | `it_should_connect_tracker_to_database_network_when_mysql_enabled`           |
| NET-06  | Tracker must be on `metrics_network` when Prometheus is enabled                    | Rust: `TrackerServiceConfig::compute_networks()` | `it_should_connect_tracker_to_metrics_network_when_prometheus_enabled`       |
| NET-07  | Tracker must be on `proxy_network` when Tracker needs TLS (see note below)         | Rust: `TrackerServiceConfig::compute_networks()` | `it_should_connect_tracker_to_proxy_network_when_tracker_needs_tls`          |
| NET-08  | Prometheus must be on `metrics_network` (always)                                   | Rust: `PrometheusServiceConfig::new()`           | `it_should_connect_prometheus_to_metrics_network`                            |
| NET-09  | Prometheus must be on `visualization_network` when Grafana is enabled              | Rust: `PrometheusServiceConfig::new()`           | `it_should_connect_prometheus_to_visualization_network_when_grafana_enabled` |
| NET-10  | Grafana must be on `visualization_network` (always)                                | Rust: `GrafanaServiceConfig::new()`              | `it_should_connect_grafana_to_visualization_network`                         |
| NET-11  | Grafana must be on `proxy_network` when Grafana has TLS enabled (see note below)   | Rust: `GrafanaServiceConfig::new()`              | `it_should_connect_grafana_to_proxy_network_when_grafana_has_tls`            |
| NET-12  | MySQL must be on `database_network` (always)                                       | Rust: `MysqlServiceConfig::new()`                | `it_should_connect_mysql_to_database_network`                                |
| NET-13  | Caddy must be on `proxy_network` (always)                                          | Rust: `CaddyServiceConfig::new()`                | `it_should_connect_caddy_to_proxy_network`                                   |
| NET-14  | **Invariant**: Every network used by any service must appear in required networks  | Template (scattered)                             | `it_should_include_all_service_networks_in_required_networks`                |
| NET-15  | **Invariant**: No network should appear in required networks if no service uses it | Template (scattered)                             | `it_should_not_include_unused_networks_in_required_networks`                 |

**NET-07 Detailed Rule (Tracker Proxy Network)**:

The Tracker needs `proxy_network` only when **at least one** of its HTTP endpoints uses TLS:

| Tracker Endpoint | TLS Config Field                 | Notes                              |
| ---------------- | -------------------------------- | ---------------------------------- |
| HTTP Trackers    | `http_trackers[].use_tls_proxy`  | Each HTTP tracker can opt into TLS |
| REST API         | `http_api.use_tls_proxy`         | Admin API endpoint                 |
| Healthcheck API  | `health_check_api.use_tls_proxy` | Internal health endpoint           |

Caddy being enabled for other services (e.g., Grafana TLS) does NOT require Tracker to join `proxy_network`.

**NET-11 Detailed Rule (Grafana Proxy Network)**:

Grafana needs `proxy_network` only when Grafana itself has TLS enabled (`grafana.use_tls_proxy = true`).

Caddy being enabled for other services (e.g., Tracker API TLS) does NOT require Grafana to join `proxy_network`. This allows configurations like:

- Tracker API with HTTPS + Grafana with HTTP (only Tracker on `proxy_network`)
- Both with HTTPS (both on `proxy_network`)
- Neither with HTTPS (Caddy not needed, no `proxy_network`)

### Service Dependency Rules

| Rule ID    | Rule Description                                                                          | Current Location                 | Test Name                                                             |
| ---------- | ----------------------------------------------------------------------------------------- | -------------------------------- | --------------------------------------------------------------------- |
| DEP-01     | Tracker depends on MySQL (with `service_healthy`) when MySQL is enabled                   | Template: `{%- if mysql %}`      | `it_should_make_tracker_depend_on_mysql_when_mysql_enabled`           |
| DEP-02     | Prometheus depends on Tracker (simple dependency)                                         | Template: hardcoded              | `it_should_make_prometheus_depend_on_tracker`                         |
| DEP-03     | Grafana depends on Prometheus (with `service_healthy`) when Prometheus is enabled         | Template: `{%- if prometheus %}` | `it_should_make_grafana_depend_on_prometheus_when_prometheus_enabled` |
| ~~DEP-04~~ | ⚠️ **BUG-01**: Invalid rule - Grafana without Prometheus should be rejected at validation | Template: `{%- else %}`          | **REMOVE** - this case should never occur                             |

### Port Exposure Rules

| Rule ID | Rule Description                                                                             | Current Location                                    | Test Name                                                |
| ------- | -------------------------------------------------------------------------------------------- | --------------------------------------------------- | -------------------------------------------------------- |
| PORT-01 | Tracker needs ports section if it has UDP ports OR HTTP ports without TLS OR API without TLS | Rust: `TrackerServiceConfig::new()`                 | `it_should_need_ports_section_when_has_udp_ports`        |
| PORT-02 | Tracker UDP ports are always exposed (UDP doesn't use TLS)                                   | Template: always renders                            | `it_should_always_expose_tracker_udp_ports`              |
| PORT-03 | Tracker HTTP ports WITHOUT TLS are exposed directly                                          | Template: iterates `http_tracker_ports_without_tls` | `it_should_expose_http_tracker_ports_without_tls`        |
| PORT-04 | Tracker HTTP ports WITH TLS are NOT exposed (Caddy handles them)                             | Rust: filters out TLS ports                         | `it_should_not_expose_http_tracker_ports_with_tls`       |
| PORT-05 | Tracker API port is exposed only when API has no TLS                                         | Template: `{%- if not tracker.http_api_has_tls %}`  | `it_should_expose_api_port_when_api_has_no_tls`          |
| PORT-06 | Tracker API port is NOT exposed when API has TLS (Caddy handles it)                          | Template: `{%- if not tracker.http_api_has_tls %}`  | `it_should_not_expose_api_port_when_api_has_tls`         |
| PORT-07 | Grafana port 3000 is exposed only when Grafana has no TLS                                    | Template: `{%- if not grafana.has_tls %}`           | `it_should_expose_grafana_port_when_grafana_has_no_tls`  |
| PORT-08 | Grafana port 3000 is NOT exposed when Grafana has TLS                                        | Template: `{%- if not grafana.has_tls %}`           | `it_should_not_expose_grafana_port_when_grafana_has_tls` |
| PORT-09 | Caddy always exposes ports 80, 443, 443/udp (fixed)                                          | Template: hardcoded                                 | `it_should_expose_caddy_standard_ports`                  |
| PORT-10 | Prometheus exposes port 9090 on localhost only (127.0.0.1)                                   | Template: hardcoded                                 | `it_should_expose_prometheus_on_localhost_only`          |
| PORT-11 | MySQL does NOT expose any ports (internal only)                                              | Template: no ports section                          | `it_should_not_expose_mysql_ports`                       |

### Volume Rules (Post-Refactor: Bind Mounts Only)

| Rule ID | Rule Description                                           | Current Location    | Test Name                                           |
| ------- | ---------------------------------------------------------- | ------------------- | --------------------------------------------------- |
| VOL-01  | Tracker has 3 bind mounts: lib, log, etc (with SELinux :Z) | Template: hardcoded | `it_should_have_tracker_bind_mounts_with_selinux`   |
| VOL-02  | Prometheus has 1 bind mount: etc (with SELinux :Z)         | Template: hardcoded | `it_should_have_prometheus_bind_mount_with_selinux` |
| VOL-03  | Caddy has 3 bind mounts: Caddyfile (ro), data, config      | Template: hardcoded | `it_should_have_caddy_bind_mounts`                  |
| VOL-04  | Grafana has 2 bind mounts: data, provisioning (ro)         | Template: hardcoded | `it_should_have_grafana_bind_mounts`                |
| VOL-05  | MySQL has 1 bind mount: data                               | Template: hardcoded | `it_should_have_mysql_bind_mount`                   |
| VOL-06  | Caddyfile mount must be read-only                          | Domain rule         | `it_should_make_caddyfile_mount_readonly`           |
| VOL-07  | Grafana provisioning mount must be read-only               | Domain rule         | `it_should_make_grafana_provisioning_readonly`      |

### Service Inclusion Rules

| Rule ID | Rule Description                                                 | Current Location                 | Test Name                                                |
| ------- | ---------------------------------------------------------------- | -------------------------------- | -------------------------------------------------------- |
| SVC-01  | Tracker service is always included                               | Template: no conditional         | `it_should_always_include_tracker_service`               |
| SVC-02  | Caddy is included when ≥1 HTTP service uses TLS (see note below) | Template: `{%- if caddy %}`      | `it_should_include_caddy_when_any_http_service_uses_tls` |
| SVC-03  | Prometheus service is included only when Prometheus is enabled   | Template: `{%- if prometheus %}` | `it_should_include_prometheus_service_when_enabled`      |
| SVC-04  | Grafana service is included only when Grafana is enabled         | Template: `{%- if grafana %}`    | `it_should_include_grafana_service_when_enabled`         |
| SVC-05  | MySQL service is included only when MySQL is enabled             | Template: `{%- if mysql %}`      | `it_should_include_mysql_service_when_enabled`           |

**SVC-02 Detailed Rule (Caddy TLS Termination)**:

Caddy is the TLS termination proxy. It is required when **at least one** of these HTTP services is configured to use HTTPS:

| HTTP Service     | TLS Config Field                 | Notes                              |
| ---------------- | -------------------------------- | ---------------------------------- |
| HTTP Trackers    | `http_trackers[].use_tls_proxy`  | Each HTTP tracker can opt into TLS |
| Tracker REST API | `http_api.use_tls_proxy`         | Admin API endpoint                 |
| Healthcheck API  | `health_check_api.use_tls_proxy` | Internal health endpoint           |
| Grafana Web UI   | `grafana.use_tls_proxy`          | Dashboard access                   |

**Services NOT proxied by Caddy**:

- **Prometheus** (port 9090): Only exposed on `127.0.0.1` (localhost) - not externally accessible
- **MySQL** (port 3306): Internal only, no external port exposure
- **UDP Trackers**: UDP protocol doesn't use TLS termination

The template's `{%- if caddy %}` is self-referential. The domain should derive Caddy requirement from:

```rust
fn needs_caddy(config: &DeploymentConfig) -> bool {
    config.http_trackers.iter().any(|t| t.use_tls_proxy)
        || config.http_api.use_tls_proxy
        || config.health_check_api.use_tls_proxy
        || config.grafana.map(|g| g.use_tls_proxy).unwrap_or(false)
}
```

### Aggregate Invariants (Cross-Cutting)

| Rule ID | Rule Description                                                                     | Test Name                                              |
| ------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------ |
| INV-01  | **Network Consistency**: Union of all service networks == required networks          | `it_should_derive_required_networks_from_all_services` |
| INV-02  | **No Orphan Networks**: Required networks contains no networks unused by any service | `it_should_not_have_orphan_networks`                   |
| INV-03  | **Dependency Validity**: All services in depends_on must be enabled                  | `it_should_only_depend_on_enabled_services`            |
| INV-04  | **No Circular Dependencies**: Service dependency graph is acyclic                    | `it_should_have_no_circular_dependencies`              |
| INV-05  | **Storage Isolation**: Each service's volumes use distinct host paths                | `it_should_have_isolated_storage_paths`                |

### Configuration Combination Tests

These tests verify correct behavior for common deployment scenarios:

| Scenario        | Services Enabled                               | Test Name                                        |
| --------------- | ---------------------------------------------- | ------------------------------------------------ |
| Minimal         | Tracker (SQLite)                               | `it_should_configure_minimal_deployment`         |
| With MySQL      | Tracker + MySQL                                | `it_should_configure_deployment_with_mysql`      |
| With Monitoring | Tracker + Prometheus + Grafana                 | `it_should_configure_deployment_with_monitoring` |
| Full HTTP       | Tracker + MySQL + Prometheus + Grafana         | `it_should_configure_full_http_deployment`       |
| Full HTTPS      | Tracker + MySQL + Prometheus + Grafana + Caddy | `it_should_configure_full_https_deployment`      |
| HTTPS Minimal   | Tracker + Caddy                                | `it_should_configure_https_minimal_deployment`   |

---

## 🚀 Refactoring Phases

> **Architectural Note**: This refactoring introduces domain types in
> `src/domain/deployment/topology/` following DDD principles. These types
> (`Network`, `Service`, `MountOption`, `BindMount`, `DockerComposeTopology`)
> represent business concepts and invariants. The existing service configs
> (`TrackerServiceConfig`, etc.) remain in the infrastructure layer
> (`src/infrastructure/templating/`) as they are concerned with template
> rendering—an infrastructure concern. The domain types are used by the
> infrastructure layer to ensure type safety and enforce invariants.

---

## Phase 0: Bind Mount Standardization (Foundation)

This phase converts all named volumes to bind mounts and standardizes how volume mounts are represented in the context. This simplifies the architecture by eliminating the need for top-level volume declarations.

### Pre-Phase Task: Create ADR for Bind Mount Standardization

**Status**: ⏳ Not Started  
**Priority**: P0 (before starting implementation)

Before implementing bind mount changes, document the decision in an ADR following [docs/decisions/README.md](../../decisions/README.md).

**ADR File**: `docs/decisions/bind-mount-standardization.md`

**Reasons to Document in ADR** (all must be included):

1. **Observability**
   - Users can see exactly where persistent data is stored
   - No need to search `/var/lib/docker/volumes/` for hidden data
   - File system tools (ls, du, find) work directly on data

2. **Backup Simplicity**
   - Single command backup: `cp -r ./storage/ backup/` or `rsync -av ./storage/ backup/`
   - No Docker-specific tooling required (no `docker volume` commands)
   - Standard backup tools and scripts work without modification
   - Incremental backups are straightforward

3. **Restore Simplicity**
   - Restore by copying files back to `./storage/`
   - No need to recreate Docker volumes before restore
   - Can restore to different machines without Docker volume migration

4. **Consistency**
   - Same pattern for all services (Tracker, Caddy, MySQL, Grafana, Prometheus)
   - Predictable directory structure: `./storage/{service}/{type}/`
   - Eliminates cognitive overhead of mixed volume types

5. **Portability**
   - Data directory can be moved between hosts by copying
   - No Docker volume export/import dance
   - Works with any container runtime that supports bind mounts

6. **Debugging & Troubleshooting**
   - Direct file inspection without entering containers
   - Easy to check file permissions, ownership, disk usage
   - Can modify config files directly for debugging
   - Log files accessible without `docker logs`

7. **Development Experience**
   - Easy to reset state by deleting directories
   - Can pre-populate data for testing scenarios
   - IDE file watchers can observe changes

8. **Deployment Architecture Simplification**
   - Eliminates top-level `volumes:` section in docker-compose.yml
   - No volume derivation logic needed (which volumes are required?)
   - Ansible only needs to create directories, not manage Docker volumes

9. **Security Visibility**
   - File permissions are visible and controllable
   - SELinux labels can be applied consistently (`:Z` suffix)
   - No hidden data in Docker-managed locations

**Alternatives to Document**:

1. **Named Volumes Only**: Rejected because data is hidden, backup requires Docker commands
2. **Mixed Approach**: Rejected because inconsistency creates confusion and maintenance burden
3. **Docker Volume Plugins**: Rejected as overkill for single-VM deployments

**Implementation Checklist**:

- [ ] Create ADR file `docs/decisions/bind-mount-standardization.md`
- [ ] Include all 9 reasons above in Context/Decision sections
- [ ] Document alternatives considered
- [ ] Add consequences (positive and negative)
- [ ] Reference this refactoring plan
- [ ] Add entry to ADR index in `docs/decisions/README.md`

---

### Proposal 0.1: Convert Named Volumes to Bind Mounts

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

Some services use named volumes which hide data in Docker's internal directories:

| Service | Current (Named Volume)          | Data Location                                    |
| ------- | ------------------------------- | ------------------------------------------------ |
| Caddy   | `caddy_data:/data`              | `/var/lib/docker/volumes/caddy_data/_data`       |
| Caddy   | `caddy_config_vol:/config`      | `/var/lib/docker/volumes/caddy_config_vol/_data` |
| Grafana | `grafana_data:/var/lib/grafana` | `/var/lib/docker/volumes/grafana_data/_data`     |
| MySQL   | `mysql_data:/var/lib/mysql`     | `/var/lib/docker/volumes/mysql_data/_data`       |

#### Proposed Solution

Convert all named volumes to bind mounts under `./storage/{service}/`:

| Service | New (Bind Mount)                          | Data Location            |
| ------- | ----------------------------------------- | ------------------------ |
| Caddy   | `./storage/caddy/data:/data`              | `./storage/caddy/data`   |
| Caddy   | `./storage/caddy/config:/config`          | `./storage/caddy/config` |
| Grafana | `./storage/grafana/data:/var/lib/grafana` | `./storage/grafana/data` |
| MySQL   | `./storage/mysql/data:/var/lib/mysql`     | `./storage/mysql/data`   |

Update template for Caddy:

```yaml
# Before
volumes:
  - ./storage/caddy/etc/Caddyfile:/etc/caddy/Caddyfile:ro
  - caddy_data:/data
  - caddy_config_vol:/config

# After
volumes:
  - ./storage/caddy/etc/Caddyfile:/etc/caddy/Caddyfile:ro
  - ./storage/caddy/data:/data
  - ./storage/caddy/config:/config
```

Remove the top-level `volumes:` section entirely (no longer needed).

#### Rationale

- **Observability**: All data is visible in `./storage/`
- **Backup simplicity**: `cp -r ./storage/ backup/` backs up everything
- **Consistency**: Same pattern for all services
- **Simplicity**: No need for top-level `volumes:` section or derivation logic

#### Benefits

- ✅ All persistent data in one predictable location
- ✅ Easy backup strategy (just copy `./storage/`)
- ✅ No hidden Docker volume directories
- ✅ Eliminates need for Phase 2 (volume derivation) - major simplification!

#### Implementation Checklist

> **Note**: Before implementing Ansible changes, read [docs/contributing/templates/README.md](../../contributing/templates/README.md).
> Ansible playbooks are **static files** (not `.tera` templates). Values are injected via a single
> `templates/ansible/variables.yml.tera` file, which simplifies rendering and keeps playbooks reusable.

- [ ] Update Caddy service volumes in template (2 changes: `caddy_data` → bind mount, `caddy_config_vol` → bind mount; Caddyfile is already a bind mount)
- [ ] Update Grafana service volumes in template (1 change: `grafana_data` → bind mount)
- [ ] Update MySQL service volumes in template (1 change: `mysql_data` → bind mount)
- [ ] Remove top-level `volumes:` section from template
- [ ] Create new Ansible playbook `create-grafana-storage.yml` (static, owner `472:472`)
- [ ] Create new Ansible playbook `create-mysql-storage.yml` (static, owner `999:999`)
- [ ] Register new static playbooks in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs` (in `copy_static_templates` method)
- [ ] Update `templates/ansible/variables.yml.tera` to include any new variables needed by playbooks
- [ ] Update deployment step orchestration to call new playbooks
- [ ] Test that services start correctly with bind mounts
- [ ] Verify TLS certificates persist across container restarts (Caddy)
- [ ] Verify Grafana dashboards persist
- [ ] Verify MySQL data persists
- [ ] Run E2E tests
- [ ] Run linter and fix any issues

#### Unit Tests Required

None for this proposal (template-only change). E2E tests verify persistence.

#### Testing Strategy

- E2E test: Deploy with HTTPS, verify certificates are obtained
- E2E test: Restart containers, verify data persists
- Manual test: Check that `./storage/` contains expected directories

#### Permissions Considerations

Some containers run as non-root users and need correct directory ownership:

| Service    | Container User                | Host Ownership        | Current Playbook Status                                      |
| ---------- | ----------------------------- | --------------------- | ------------------------------------------------------------ |
| Tracker    | `1000:1000` (USER_ID env var) | `ansible_user`        | ✅ `create-tracker-storage.yml` handles this                 |
| Prometheus | `65534:65534` (nobody)        | `ansible_user`        | ✅ `create-prometheus-storage.yml` handles this              |
| Caddy      | `root`                        | `ansible_user`        | ✅ `deploy-caddy-config.yml` handles this                    |
| Grafana    | `472:472` (grafana)           | **Must be `472:472`** | ❌ **MISSING**: No playbook creates `./storage/grafana/data` |
| MySQL      | `999:999` (mysql)             | **Must be `999:999`** | ❌ **MISSING**: No playbook creates `./storage/mysql/data`   |

**Required Ansible Changes:**

1. **Create new playbook** `create-grafana-storage.yml`:

   ```yaml
   - name: Create Grafana data directory
     ansible.builtin.file:
       path: /opt/torrust/storage/grafana/data
       state: directory
       mode: "0755"
       owner: "472"
       group: "472"
     when: grafana_enabled | default(false)
   ```

2. **Create new playbook** `create-mysql-storage.yml`:

   ```yaml
   - name: Create MySQL data directory
     ansible.builtin.file:
       path: /opt/torrust/storage/mysql/data
       state: directory
       mode: "0755"
       owner: "999"
       group: "999"
     when: mysql_enabled | default(false)
   ```

3. **Update** `deploy-grafana-provisioning.yml` to NOT set `ansible_user` ownership on data directory (it currently only handles provisioning, not data).

**Note**: Currently MySQL and Grafana use named volumes (`mysql_data`, `grafana_data`), which Docker creates automatically with correct internal ownership. Converting to bind mounts requires explicit directory creation with correct ownership BEFORE the containers start.

---

### Proposal 0.2: Create BindMount Domain Type

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal 0.1

#### Problem

Volume mounts are hardcoded directly in the template. There's no Rust representation.

#### Proposed Solution

Create a simple domain type for bind mounts with a type-safe options enum:

```rust
// src/domain/deployment/topology/volume.rs

/// Mount options for Docker bind mounts
///
/// These options control access mode and SELinux labeling behavior.
/// In practice, we use either read-only OR SELinux relabeling, not both,
/// because read-only config files don't need relabeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MountOption {
    /// Read-only mount (`:ro`) - used for config files that shouldn't be modified
    ReadOnly,
    /// SELinux private relabeling (`:Z`) - used for writable data directories
    /// Required on SELinux-enabled systems (RHEL, Fedora, CentOS)
    SELinux,
}

impl MountOption {
    /// Returns the option string for docker-compose.yml
    pub fn as_str(&self) -> &'static str {
        match self {
            MountOption::ReadOnly => "ro",
            MountOption::SELinux => "Z",
        }
    }
}

impl std::fmt::Display for MountOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A bind mount specification for a Docker container
///
/// Maps a host path (relative to docker-compose.yml) to a container path.
/// All persistent data uses bind mounts for visibility and easy backup.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BindMount {
    /// Host path (relative to docker-compose.yml location)
    /// Example: "./storage/tracker/lib"
    pub host_path: String,
    /// Container path
    /// Example: "/var/lib/torrust/tracker"
    pub container_path: String,
    /// Mount option (read-only or SELinux relabeling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<MountOption>,
}

impl BindMount {
    /// Creates a new bind mount with no options
    pub fn new(host_path: impl Into<String>, container_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            container_path: container_path.into(),
            option: None,
        }
    }

    /// Creates a read-only bind mount (for config files)
    pub fn read_only(host_path: impl Into<String>, container_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            container_path: container_path.into(),
            option: Some(MountOption::ReadOnly),
        }
    }

    /// Creates a bind mount with SELinux relabeling (for data directories)
    pub fn with_selinux(host_path: impl Into<String>, container_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            container_path: container_path.into(),
            option: Some(MountOption::SELinux),
        }
    }

    /// Renders the mount specification for docker-compose.yml
    /// Example: "./storage/tracker/lib:/var/lib/torrust/tracker:Z"
    pub fn to_compose_string(&self) -> String {
        match &self.option {
            Some(opt) => format!("{}:{}:{}", self.host_path, self.container_path, opt),
            None => format!("{}:{}", self.host_path, self.container_path),
        }
    }
}
```

Add `volumes: Vec<BindMount>` to each service config and update template to iterate:

```yaml
    volumes:
{%- for vol in tracker.volumes %}
      - {{ vol.host_path }}:{{ vol.container_path }}{% if vol.option %}:{{ vol.option }}{% endif %}
{%- endfor %}
```

#### Benefits

- ✅ Volume definitions move from template to Rust (single source of truth)
- ✅ Type-safe volume specifications with enum for options
- ✅ Consistent pattern across all services
- ✅ Easy to add new volumes programmatically
- ✅ Compiler prevents invalid option strings (no typos like "rO" or "z")

#### Implementation Checklist

- [ ] Create `src/domain/deployment/topology/mod.rs` module
- [ ] Create `src/domain/deployment/topology/volume.rs` with `BindMount` type
- [ ] Add `volumes: Vec<BindMount>` to `TrackerServiceConfig`
- [ ] Add `volumes: Vec<BindMount>` to `CaddyServiceConfig`
- [ ] Add `volumes: Vec<BindMount>` to `MysqlServiceConfig`
- [ ] Add `volumes: Vec<BindMount>` to `PrometheusServiceConfig`
- [ ] Add `volumes: Vec<BindMount>` to `GrafanaServiceConfig`
- [ ] Update template to render volumes from service configs
- [ ] Add unit tests for `BindMount` serialization
- [ ] Verify rendered output matches current behavior
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Unit Tests Required

From the Domain Rules Analysis:

- `it_should_have_tracker_bind_mounts_with_selinux` (VOL-01)
- `it_should_have_prometheus_bind_mount_with_selinux` (VOL-02)
- `it_should_have_caddy_bind_mounts` (VOL-03)
- `it_should_have_grafana_bind_mounts` (VOL-04)
- `it_should_have_mysql_bind_mount` (VOL-05)
- `it_should_make_caddyfile_mount_readonly` (VOL-06)
- `it_should_make_grafana_provisioning_readonly` (VOL-07)
- `it_should_have_isolated_storage_paths` (INV-05)

Unit tests for `MountOption` enum:

- `it_should_return_ro_string_for_readonly_option`
- `it_should_return_z_string_for_selinux_option`
- `it_should_display_mount_option_as_string`
- `it_should_serialize_mount_option_to_lowercase`

Unit tests for `BindMount` type:

- `it_should_create_bind_mount_without_options`
- `it_should_create_readonly_bind_mount`
- `it_should_create_selinux_bind_mount`
- `it_should_render_compose_string_without_options`
- `it_should_render_compose_string_with_readonly`
- `it_should_render_compose_string_with_selinux`
- `it_should_serialize_bind_mount_for_template`

---

## Phase 1: Domain Network Types (Core Infrastructure)

This phase creates domain types for Docker networks and migrates service configs to use them.

### Proposal 1.1: Create Network Domain Types

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Depends On**: None (Proposal 1.1 can start in parallel with Phase 0, but Proposal 1.2 may need the bind mount concepts from Phase 0.2 for consistency)

#### Problem

Networks are represented as raw strings with no type safety:

```rust
networks.push("metrics_network".to_string());
```

Network names are duplicated between service configs and the template's global section.

#### Proposed Solution

Create a domain enum for known networks:

```rust
// src/domain/deployment/topology/network.rs

/// Docker Compose networks used for service isolation
///
/// Each network serves a specific security purpose:
/// - Database: Isolates database access to only the tracker
/// - Metrics: Allows Prometheus to scrape tracker metrics
/// - Visualization: Allows Grafana to query Prometheus
/// - Proxy: Allows Caddy to reverse proxy to backend services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Network {
    /// Network for database access (Tracker ↔ MySQL)
    Database,
    /// Network for metrics scraping (Tracker ↔ Prometheus)
    Metrics,
    /// Network for visualization queries (Prometheus ↔ Grafana)
    Visualization,
    /// Network for TLS proxy (Caddy ↔ backend services)
    Proxy,
}

impl Network {
    /// Returns the network name as used in docker-compose.yml
    pub fn name(&self) -> &'static str {
        match self {
            Network::Database => "database_network",
            Network::Metrics => "metrics_network",
            Network::Visualization => "visualization_network",
            Network::Proxy => "proxy_network",
        }
    }

    /// Returns the network driver (always "bridge" for now)
    pub fn driver(&self) -> &'static str {
        "bridge"
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
```

#### Benefits

- ✅ Type-safe network references (no typos)
- ✅ Single source of truth for network names
- ✅ Network metadata (driver) in domain, not template
- ✅ Can derive global network list from service configs

#### Implementation Checklist

- [ ] Create `src/domain/deployment/topology/network.rs`
- [ ] Add `Network` enum with all known networks
- [ ] Add serialization that produces network names
- [ ] Add `Display` impl for template rendering
- [ ] Add unit tests
- [ ] Verify all tests pass

#### Unit Tests Required

Network enum tests:

- `it_should_return_correct_network_name_for_database`
- `it_should_return_correct_network_name_for_metrics`
- `it_should_return_correct_network_name_for_visualization`
- `it_should_return_correct_network_name_for_proxy`
- `it_should_return_bridge_driver_for_all_networks`
- `it_should_serialize_network_to_name_string`
- `it_should_display_network_as_name`

---

### Proposal 1.2: Migrate Service Configs to Use Network Enum

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal 1.1

#### Problem

Service configs use `Vec<String>` for networks, computed via string concatenation.

#### Proposed Solution

Change service configs to use `Vec<Network>`:

```rust
use crate::domain::deployment::topology::Network;

pub struct TrackerServiceConfig {
    // ...
    pub networks: Vec<Network>,
}

impl TrackerServiceConfig {
    /// Computes networks for the Tracker service
    ///
    /// Note: `tracker_needs_tls` means at least one Tracker HTTP endpoint
    /// uses TLS (API, healthcheck, or any HTTP tracker). This is different
    /// from "Caddy is enabled" - Caddy could be enabled for Grafana only.
    fn compute_networks(has_prometheus: bool, has_mysql: bool, tracker_needs_tls: bool) -> Vec<Network> {
        let mut networks = Vec::new();
        if has_prometheus { networks.push(Network::Metrics); }
        if has_mysql { networks.push(Network::Database); }
        if tracker_needs_tls { networks.push(Network::Proxy); }
        networks
    }
}

/// Helper to determine if Tracker needs TLS termination
///
/// Returns true if any Tracker HTTP endpoint uses TLS proxy.
fn tracker_needs_tls(config: &TrackerConfig) -> bool {
    config.http_api.use_tls_proxy
        || config.health_check_api.map_or(false, |h| h.use_tls_proxy)
        || config.http_trackers.iter().any(|t| t.use_tls_proxy)
}
```

Create a wrapper type for template serialization:

```rust
/// Wrapper for serializing Network to its name string
#[derive(Serialize)]
#[serde(transparent)]
pub struct NetworkRef(String);

impl From<Network> for NetworkRef {
    fn from(net: Network) -> Self {
        Self(net.name().to_string())
    }
}
```

Update template to use network name:

```yaml
    networks:
{%- for network in tracker.networks %}
      - {{ network }}
{%- endfor %}
```

#### Benefits

- ✅ Type-safe network assignments
- ✅ Compiler catches invalid network references
- ✅ Ready for global network derivation in Phase 2

#### Implementation Checklist

- [ ] Create `NetworkRef` wrapper for serialization (or custom Serialize impl)
- [ ] Update `TrackerServiceConfig` to use `Vec<Network>`
- [ ] Update `CaddyServiceConfig` to use `Vec<Network>`
- [ ] Update `MysqlServiceConfig` to use `Vec<Network>`
- [ ] Update `PrometheusServiceConfig` to use `Vec<Network>`
- [ ] Update `GrafanaServiceConfig` to use `Vec<Network>`
- [ ] Update template to use new network format
- [ ] Verify rendered output matches current behavior
- [ ] Verify all tests pass

#### Unit Tests Required

From the Domain Rules Analysis (Network Rules):

**Tracker network rules:**

- `it_should_connect_tracker_to_database_network_when_mysql_enabled` (NET-05)
- `it_should_connect_tracker_to_metrics_network_when_prometheus_enabled` (NET-06)
- `it_should_connect_tracker_to_proxy_network_when_tracker_needs_tls` (NET-07)
- `it_should_not_connect_tracker_to_database_network_when_mysql_disabled`
- `it_should_not_connect_tracker_to_metrics_network_when_prometheus_disabled`
- `it_should_not_connect_tracker_to_proxy_network_when_tracker_has_no_tls`

**Prometheus network rules:**

- `it_should_connect_prometheus_to_metrics_network` (NET-08)
- `it_should_connect_prometheus_to_visualization_network_when_grafana_enabled` (NET-09)
- `it_should_not_connect_prometheus_to_visualization_network_when_grafana_disabled`

**Grafana network rules:**

- `it_should_connect_grafana_to_visualization_network` (NET-10)
- `it_should_connect_grafana_to_proxy_network_when_grafana_has_tls` (NET-11)
- `it_should_not_connect_grafana_to_proxy_network_when_grafana_has_no_tls`
- `it_should_not_connect_grafana_to_proxy_network_when_caddy_disabled`

**MySQL network rules:**

- `it_should_connect_mysql_to_database_network` (NET-12)

**Caddy network rules:**

- `it_should_connect_caddy_to_proxy_network` (NET-13)

---

## Phase 2: Topology Aggregate & Network Derivation

This phase completes the domain model by deriving the global networks section from service configurations.

### Proposal 2.1: Create DockerComposeTopology Aggregate

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposals 0.2, 1.2

#### Problem

Networks are still derived separately from services. There's no aggregate that enforces:

- Every network used by a service is defined in the global section
- Service dependencies are consistent

The template still has conditional logic:

```yaml
networks:
{%- if mysql %}
  database_network:
    driver: bridge
{%- endif %}
```

#### Proposed Solution

Create a topology aggregate that collects all networks from services:

```rust
// src/domain/deployment/topology/mod.rs

use std::collections::HashSet;

/// Services in the Docker Compose deployment
///
/// This enum provides type-safe service identification, preventing typos
/// and enabling exhaustive matching in domain logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Service {
    Tracker,
    MySQL,
    Prometheus,
    Grafana,
    Caddy,
}

impl Service {
    /// Returns the service name as used in docker-compose.yml
    pub fn name(&self) -> &'static str {
        match self {
            Service::Tracker => "tracker",
            Service::MySQL => "mysql",
            Service::Prometheus => "prometheus",
            Service::Grafana => "grafana",
            Service::Caddy => "caddy",
        }
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Docker Compose deployment topology
///
/// This aggregate ensures all invariants are maintained:
/// - Networks used by services are derived and always defined
/// - Service dependencies are explicitly modeled
pub struct DockerComposeTopology {
    /// All services in the deployment
    services: Vec<ServiceTopology>,
}

/// Topology information for a single service
pub struct ServiceTopology {
    pub service: Service,  // Type-safe, not `name: String`
    pub networks: Vec<Network>,
    pub volumes: Vec<BindMount>,
}

impl DockerComposeTopology {
    /// Returns all networks required by enabled services
    ///
    /// This is the single source of truth - the template's `networks:` section
    /// should iterate over this, not use conditionals.
    pub fn required_networks(&self) -> Vec<Network> {
        let unique: HashSet<Network> = self.services.iter()
            .flat_map(|s| s.networks.iter().copied())
            .collect();

        // Return in deterministic order for template stability
        let mut networks: Vec<Network> = unique.into_iter().collect();
        networks.sort_by_key(|n| n.name());
        networks
    }
}
```

#### Benefits

- ✅ Domain aggregate with enforced invariants
- ✅ Single source of truth for all topology
- ✅ Template is pure rendering with no decisions
- ✅ Testable domain logic
- ✅ Type-safe service identification (no typos like `"trakcer"`) <!-- cspell:disable-line -->

#### Unit Tests Required

From the Domain Rules Analysis (Aggregate Invariants):

- `it_should_derive_required_networks_from_all_services` (INV-01)
- `it_should_not_have_orphan_networks` (INV-02)
- `it_should_return_networks_in_deterministic_order`

Service enum tests:

- `it_should_return_correct_service_name_for_tracker`
- `it_should_return_correct_service_name_for_mysql`
- `it_should_return_correct_service_name_for_prometheus`
- `it_should_return_correct_service_name_for_grafana`
- `it_should_return_correct_service_name_for_caddy`
- `it_should_display_service_as_name`

---

### Proposal 2.2: Derive Required Networks in Context

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal 2.1

#### Problem

The template's `networks:` section is still conditional based on which services are enabled.

#### Proposed Solution

Add `required_networks` to context and derive from service configs:

> **Note**: The `DockerComposeTopology` aggregate from Proposal 2.1 enforces that
> all networks used by services are registered in the global section. This proposal
> implements the derivation logic that uses the topology to build the context.

```rust
pub struct DockerComposeContext {
    // ... existing service fields ...

    /// All networks required by enabled services (derived)
    ///
    /// This list is computed from the networks used by all services.
    /// The template should iterate over this for the global `networks:` section.
    pub required_networks: Vec<NetworkDefinition>,
}

/// A network definition for the global `networks:` section
#[derive(Serialize)]
pub struct NetworkDefinition {
    pub name: String,
    pub driver: String,
}

impl From<Network> for NetworkDefinition {
    fn from(net: Network) -> Self {
        Self {
            name: net.name().to_string(),
            driver: net.driver().to_string(),
        }
    }
}
```

Derive in the builder:

```rust
impl DockerComposeContextBuilder {
    pub fn build(self) -> DockerComposeContext {
        // ... build services ...

        // Derive required networks from all service network assignments
        let required_networks = self.derive_required_networks();

        DockerComposeContext {
            // ... services ...
            required_networks,
        }
    }

    fn derive_required_networks(&self) -> Vec<NetworkDefinition> {
        let mut networks: HashSet<Network> = HashSet::new();

        // Collect from tracker (always present)
        networks.extend(self.tracker.networks.iter().copied());

        // Collect from optional services
        if let Some(ref prometheus) = self.prometheus {
            networks.extend(prometheus.networks.iter().copied());
        }
        if let Some(ref grafana) = self.grafana {
            networks.extend(grafana.networks.iter().copied());
        }
        if let Some(ref caddy) = self.caddy {
            networks.extend(caddy.networks.iter().copied());
        }
        if let Some(ref mysql) = self.mysql {
            networks.extend(mysql.networks.iter().copied());
        }

        // Sort for deterministic output
        let mut result: Vec<NetworkDefinition> = networks
            .into_iter()
            .map(NetworkDefinition::from)
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
}
```

Update template:

```yaml
{%- if required_networks | length > 0 %}
networks:
{%- for net in required_networks %}
  {{ net.name }}:
    driver: {{ net.driver }}
{%- endfor %}
{%- endif %}
```

#### Benefits

- ✅ Networks derived from service configurations
- ✅ Impossible to use an undefined network
- ✅ Template has no conditional logic for networks
- ✅ **Invariant enforced**: "if a service uses a network, it must be defined"

#### Implementation Checklist

- [ ] Add `NetworkDefinition` type
- [ ] Add `required_networks: Vec<NetworkDefinition>` to `DockerComposeContext`
- [ ] Implement `derive_required_networks()` in builder
- [ ] Update template to iterate over `required_networks`
- [ ] Remove all `{%- if mysql %}` style conditionals from networks section
- [ ] Verify rendered output matches current behavior
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Unit Tests Required

From the Domain Rules Analysis (Network Rules - Global):

- `it_should_include_database_network_when_mysql_enabled` (NET-01)
- `it_should_include_metrics_network_when_prometheus_enabled` (NET-02)
- `it_should_include_visualization_network_when_grafana_enabled` (NET-03)
- `it_should_include_proxy_network_when_caddy_enabled` (NET-04)
- `it_should_include_all_service_networks_in_required_networks` (NET-14)
- `it_should_not_include_unused_networks_in_required_networks` (NET-15)
- `it_should_not_include_database_network_when_mysql_disabled`
- `it_should_not_include_metrics_network_when_prometheus_disabled`
- `it_should_not_include_visualization_network_when_grafana_disabled`
- `it_should_not_include_proxy_network_when_caddy_disabled`

From the Domain Rules Analysis (Configuration Combinations):

- `it_should_configure_minimal_deployment` (no networks needed)
- `it_should_configure_deployment_with_mysql` (database_network only)
- `it_should_configure_deployment_with_monitoring` (metrics + visualization)
- `it_should_configure_full_http_deployment` (database + metrics + visualization)
- `it_should_configure_full_https_deployment` (all four networks)
- `it_should_configure_https_minimal_deployment` (proxy_network only)

---

## 📈 Timeline

- **Start Date**: TBD
- **Estimated Duration**: 1.5-2 weeks
  - Phase 0: 2-3 days
  - Phase 1: 2-3 days
  - Phase 2: 2-3 days

## 🧪 Additional Test Coverage (Future Phases)

The following domain rules are identified but will be addressed in future refactoring phases:

### Service Dependencies (Future Phase)

Currently, service dependencies are hardcoded in the template. A future phase could:

- Add a `ServiceDependency` domain type
- Derive dependencies from service relationships
- Test all DEP-\* rules:
  - `it_should_make_tracker_depend_on_mysql_when_mysql_enabled` (DEP-01)
  - `it_should_make_prometheus_depend_on_tracker` (DEP-02)
  - `it_should_make_grafana_depend_on_prometheus_when_prometheus_enabled` (DEP-03)
  - ~~DEP-04~~: Removed - this was BUG-01 (Grafana without Prometheus is invalid)
  - `it_should_only_depend_on_enabled_services` (INV-03)
  - `it_should_have_no_circular_dependencies` (INV-04)

### Port Exposure (Future Phase)

Port exposure logic is partially in Rust, partially in template. A future phase could:

- Add a `PortMapping` domain type
- Derive port exposure from service configuration
- Test all PORT-\* rules:
  - `it_should_need_ports_section_when_has_udp_ports` (PORT-01)
  - `it_should_always_expose_tracker_udp_ports` (PORT-02)
  - `it_should_expose_http_tracker_ports_without_tls` (PORT-03)
  - `it_should_not_expose_http_tracker_ports_with_tls` (PORT-04)
  - `it_should_expose_api_port_when_api_has_no_tls` (PORT-05)
  - `it_should_not_expose_api_port_when_api_has_tls` (PORT-06)
  - `it_should_expose_grafana_port_when_grafana_has_no_tls` (PORT-07)
  - `it_should_not_expose_grafana_port_when_grafana_has_tls` (PORT-08)
  - `it_should_expose_caddy_standard_ports` (PORT-09)
  - `it_should_expose_prometheus_on_localhost_only` (PORT-10)
  - `it_should_not_expose_mysql_ports` (PORT-11)

### Service Inclusion (Future Phase)

Service inclusion is currently determined by optional fields in context. Tests:

- `it_should_always_include_tracker_service` (SVC-01)
- `it_should_include_caddy_service_when_enabled` (SVC-02)
- `it_should_include_prometheus_service_when_enabled` (SVC-03)
- `it_should_include_grafana_service_when_enabled` (SVC-04)
- `it_should_include_mysql_service_when_enabled` (SVC-05)

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals reviewed for technical feasibility
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Aligns with [DDD Layer Placement](../../contributing/ddd-layer-placement.md)
- [ ] Implementation plan is clear and actionable

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing (unit + E2E)
- [ ] All linters passing
- [ ] Template output unchanged (behavioral equivalence)
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../../development-principles.md)
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md)
- [Contributing Guidelines](../../contributing/README.md)
- [ADR: Environment Variable Injection in Docker Compose](../../decisions/environment-variable-injection-in-docker-compose.md)

## 💡 Notes

### Migration Strategy

Each phase is designed to be independently mergeable:

- Phase 0 converts to bind mounts and adds volume types (behavioral change: data location moves)
- Phase 1 adds network types but doesn't change rendered output
- Phase 2 derives networks but doesn't change rendered output

**Important**: Phase 0 is a behavioral change for existing deployments. Users with existing named volumes will need to migrate their data. Consider documenting a migration guide.

### Why This Order?

1. **Volumes first** (Phase 0): Simplifies architecture by eliminating named volumes entirely
2. **Networks second** (Phase 1): Adds type safety without changing output
3. **Derivation last** (Phase 2): Completes the single-source-of-truth goal

### Why Bind Mounts Only?

We chose to use bind mounts exclusively because:

- **Observability**: All data is visible in `./storage/` - aligns with project principle "If it happens, we can see it"
- **User Friendliness**: Users can easily find and back up their data
- **Simplicity**: No need for top-level `volumes:` section or derivation logic
- **Consistency**: Same pattern for all services

Named volumes were originally used because some tutorials default to them, but they provide no benefit for our Linux server deployments.

### Alternative Considered: Keep Named Volumes

We considered keeping the Bind/Named distinction and deriving the `volumes:` section, but rejected it because:

- Adds complexity without benefit for our use case
- Named volumes hide data in Docker internals
- Users can't easily back up data
- Inconsistent with our "observability" principle

---

## 📖 Service Quick Reference

This appendix provides a quick lookup of which rules apply to each service. See the Domain Rules Analysis section for full rule descriptions.

| Service    | Networks       | Dependencies | Ports      | Volumes | Inclusion |
| ---------- | -------------- | ------------ | ---------- | ------- | --------- |
| Tracker    | NET-05, 06, 07 | DEP-01       | PORT-01–06 | VOL-01  | SVC-01    |
| MySQL      | NET-01, 12     | -            | PORT-11    | VOL-05  | SVC-05    |
| Prometheus | NET-02, 08, 09 | DEP-02       | PORT-10    | VOL-02  | SVC-03    |
| Grafana    | NET-03, 10, 11 | DEP-03       | PORT-07–08 | VOL-04  | SVC-04    |
| Caddy      | NET-04, 13     | -            | PORT-09    | VOL-03  | SVC-02    |

**Cross-cutting invariants** (apply to all services): NET-14, NET-15, INV-01–05

**Read-only mount rules**: VOL-06 (Caddy), VOL-07 (Grafana)

---

**Created**: January 22, 2026  
**Last Updated**: January 23, 2026  
**Status**: 📋 Planning
