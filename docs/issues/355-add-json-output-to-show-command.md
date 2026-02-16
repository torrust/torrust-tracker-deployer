# Add JSON Output to Show Command

**Issue**: [#355](https://github.com/torrust/torrust-tracker-deployer/issues/355)
**Parent Epic**: [#348](https://github.com/torrust/torrust-tracker-deployer/issues/348) - Add JSON output format support
**Related**: [Roadmap Section 12.3](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support), [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed, [Issue #352 - Add JSON output to provision command](https://github.com/torrust/torrust-tracker-deployer/issues/352) ✅ Completed

**Implementation Status**: ⏳ **NOT STARTED**

## Overview

Add machine-readable JSON output format (`--output-format json`) to the `show` command. This enables automation workflows and AI agents to programmatically extract environment state, instance IP address, and service URLs without parsing human-readable text.

## Goals

- [ ] Implement JSON output format for show command
- [ ] Preserve existing human-readable output as default
- [ ] Enable automation to extract instance IP and comprehensive environment state
- [ ] Follow the architecture pattern established in #349 and #352

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (`src/presentation/`)
**Module Path**: `src/presentation/views/commands/show/`
**Pattern**: Strategy Pattern with TextView and JsonView (established in #349 and #352)

### Current Module Structure

```rust
src/presentation/views/commands/show/
├── views/
│   ├── mod.rs            (TextView - human-readable output)
│   ├── basic.rs          (Basic environment info section)
│   ├── infrastructure.rs (Infrastructure details section)
│   ├── tracker_services.rs (Tracker service endpoints)
│   ├── prometheus.rs     (Prometheus service info)
│   ├── grafana.rs        (Grafana service info)
│   ├── https_hint.rs     (HTTPS configuration hints)
│   └── next_step.rs      (State-aware guidance)
└── mod.rs
```

### Required Module Structure

Following the refactor from PR #354 (Separate View Data from Views), the show command already has a `views/` subdirectory. The structure is ready for adding a parallel `JsonView`:

```rust
src/presentation/views/commands/show/
├── views/
│   ├── mod.rs            (Re-exports TextView and JsonView)
│   ├── text_view.rs      (Renamed from mod.rs - TextView implementation)
│   ├── json_view.rs      (NEW - JsonView implementation)
│   ├── basic.rs          (Helper for TextView)
│   ├── infrastructure.rs (Helper for TextView)
│   ├── tracker_services.rs (Helper for TextView)
│   ├── prometheus.rs     (Helper for TextView)
│   ├── grafana.rs        (Helper for TextView)
│   ├── https_hint.rs     (Helper for TextView)
│   └── next_step.rs      (Helper for TextView)
└── mod.rs                (Re-exports views module)
```

**Key Changes Required:**

1. Rename `views/mod.rs` to `views/text_view.rs` (TextView stays the same, just file organization)
2. Create new `views/json_view.rs` with `JsonView` struct
3. Update `views/mod.rs` to re-export both `TextView` and `JsonView`
4. Wire output_format through ExecutionContext → Router → Controller

**Note**: The show command does not have or need separate view_data DTOs. The `EnvironmentInfo` DTO from the application layer (`src/application/command_handlers/show/info/`) already serves this purpose. Both TextView and JsonView consume the same `EnvironmentInfo` DTO.

### Architectural Constraints

- [ ] No business logic in presentation layer (views only format existing data)
- [ ] No changes to application or domain layers
- [ ] Follow output handling conventions ([docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] Use existing `OutputFormat` enum and `--output-format` flag from #349

### Anti-Patterns to Avoid

- ❌ Embedding output formatting logic in controller
- ❌ Mixing business logic with view formatting
- ❌ Changing the application command handler interface
- ❌ Creating redundant view_data DTOs (use existing `EnvironmentInfo`)

## Specifications

### JSON Output Schema

The JSON output should follow the structure of the `EnvironmentInfo` DTO from the application layer. The schema varies by environment state:

#### Example 1: Created State (Minimal Info)

```json
{
  "name": "my-env",
  "state": "Created",
  "provider": "lxd",
  "created_at": "2026-02-16T10:00:00Z",
  "state_name": "created"
}
```

#### Example 2: Provisioned State (With Infrastructure)

```json
{
  "name": "my-env",
  "state": "Provisioned",
  "provider": "lxd",
  "created_at": "2026-02-16T10:00:00Z",
  "state_name": "provisioned",
  "infrastructure": {
    "instance_ip": "10.140.190.39",
    "ssh_port": 22,
    "ssh_user": "torrust",
    "ssh_key_path": "/home/user/.ssh/testing_rsa"
  }
}
```

#### Example 3: Running State (Full Stack)

```json
{
  "name": "full-stack-lxd",
  "state": "Running",
  "provider": "lxd",
  "created_at": "2026-02-16T10:00:00Z",
  "state_name": "running",
  "infrastructure": {
    "instance_ip": "10.140.190.39",
    "ssh_port": 22,
    "ssh_user": "torrust",
    "ssh_key_path": "/home/user/.ssh/testing_rsa"
  },
  "services": {
    "udp_trackers": ["udp://10.140.190.39:6969"],
    "http_trackers": ["http://10.140.190.39:7070/announce"],
    "https_http_trackers": ["https://tracker1.example.com/announce"],
    "api_url": "http://10.140.190.39:1212/api/v1",
    "api_https_url": "https://api.example.com/api/v1",
    "health_check_url": "http://10.140.190.39:1212/api/health_check",
    "health_check_https_url": "https://health.example.com/api/health_check",
    "localhost_services": {
      "udp_trackers": [],
      "http_trackers": [],
      "api_url": null,
      "health_check_url": null
    },
    "tls_domains": [
      {
        "domain": "tracker1.example.com",
        "service_type": "HTTP Tracker"
      },
      {
        "domain": "api.example.com",
        "service_type": "API"
      }
    ]
  },
  "prometheus": {
    "url": "http://10.140.190.39:9090"
  },
  "grafana": {
    "url": "http://10.140.190.39:3000",
    "https_url": "https://grafana.example.com",
    "default_username": "admin",
    "default_password": "admin"
  }
}
```

> **Note on Schema Flexibility**: The JSON schema shown above is **not mandatory**. The actual JSON output should mirror the structure of the Rust `EnvironmentInfo` DTO (and its nested types: `InfrastructureInfo`, `ServiceInfo`, `PrometheusInfo`, `GrafanaInfo`). If the natural Rust serialization (via `#[derive(Serialize)]`) produces a slightly different format that is easier to maintain or more idiomatic, **prefer the Rust-native structure**. The goal is simplicity and consistency with the codebase, not rigid adherence to a predefined schema. The examples above serve as a guide for the expected information, but field names and structure can be adjusted to match what `serde_json` naturally produces from the DTOs.

### Field Descriptions

#### Base Fields (Always Present)

| Field        | Type   | Description                                                 |
| ------------ | ------ | ----------------------------------------------------------- |
| `name`       | string | Environment name                                            |
| `state`      | string | Human-readable state (e.g., "Created", "Provisioned")       |
| `provider`   | string | Infrastructure provider (lxd, hetzner)                      |
| `created_at` | string | ISO 8601 timestamp of environment creation                  |
| `state_name` | string | Internal state name (created, provisioned, configured, etc) |

#### Infrastructure Fields (Available After Provisioning)

| Field          | Type   | Description                                 |
| -------------- | ------ | ------------------------------------------- |
| `instance_ip`  | string | IP address of the provisioned instance      |
| `ssh_port`     | number | SSH port (typically 22)                     |
| `ssh_user`     | string | SSH username for connecting to the instance |
| `ssh_key_path` | string | Absolute path to SSH private key file       |

#### Service Fields (Available After Release/Run)

Nested under `services` object:

| Field                    | Type     | Description                                 |
| ------------------------ | -------- | ------------------------------------------- |
| `udp_trackers`           | string[] | UDP tracker announce URLs                   |
| `http_trackers`          | string[] | HTTP tracker announce URLs (direct IP)      |
| `https_http_trackers`    | string[] | HTTPS tracker announce URLs (via Caddy TLS) |
| `api_url`                | string   | Direct API endpoint URL (HTTP)              |
| `api_https_url`          | string   | HTTPS API endpoint URL (via Caddy TLS)      |
| `health_check_url`       | string   | Direct health check URL (HTTP)              |
| `health_check_https_url` | string   | HTTPS health check URL (via Caddy TLS)      |
| `localhost_services`     | object   | Services bound to localhost (internal only) |
| `tls_domains`            | array    | Configured TLS domains with service types   |

#### Prometheus Fields (Available After Release/Run)

Nested under `prometheus` object:

| Field | Type   | Description              |
| ----- | ------ | ------------------------ |
| `url` | string | Prometheus dashboard URL |

#### Grafana Fields (Available After Release/Run)

Nested under `grafana` object:

| Field              | Type   | Description                                 |
| ------------------ | ------ | ------------------------------------------- |
| `url`              | string | Direct Grafana dashboard URL (HTTP)         |
| `https_url`        | string | HTTPS Grafana dashboard URL (via Caddy TLS) |
| `default_username` | string | Default Grafana username (admin)            |
| `default_password` | string | Default Grafana password (admin)            |

### CLI Interface

```bash
# Human-readable output (default, unchanged)
torrust-tracker-deployer show my-env

# JSON output (new)
torrust-tracker-deployer show my-env --output-format json

# Short form
torrust-tracker-deployer show my-env -o json
```

### Human-Readable Output (Reference - Must Not Change)

The default text output should remain unchanged. The show command produces different output based on environment state:

#### Created State Output

```text
Environment: my-env
State:       Created
Provider:    LXD
Created:     2026-02-16 10:00:00 UTC

Next Step:
Run 'provision' to create infrastructure.
```

#### Provisioned State Output

```text
Environment: my-env
State:       Provisioned
Provider:    LXD
Created:     2026-02-16 10:00:00 UTC

Infrastructure:
  Instance IP:       10.140.190.39
  SSH Port:          22
  SSH Username:      torrust
  SSH Private Key:   /home/user/.ssh/testing_rsa

Connect using:
  ssh -i /home/user/.ssh/testing_rsa torrust@10.140.190.39

Next Step:
Run 'configure' to set up the system.
```

#### Running State Output (Full Stack with HTTPS)

```text
Environment: full-stack-lxd
State:       Running
Provider:    LXD
Created:     2026-02-16 10:00:00 UTC

Infrastructure:
  Instance IP:       10.140.190.39
  SSH Port:          22
  SSH Username:      torrust
  SSH Private Key:   /home/user/.ssh/testing_rsa

Connect using:
  ssh -i /home/user/.ssh/testing_rsa torrust@10.140.190.39

Tracker Services:

  UDP Trackers:
    • udp://10.140.190.39:6969

  HTTP Trackers (Direct):
    • http://10.140.190.39:7070/announce

  HTTP Trackers (HTTPS):
    • https://tracker1.example.com/announce

  API (Direct):
    • http://10.140.190.39:1212/api/v1

  API (HTTPS):
    • https://api.example.com/api/v1

  Health Check (Direct):
    • http://10.140.190.39:1212/api/health_check

  Health Check (HTTPS):
    • https://health.example.com/api/health_check

Prometheus:
  • http://10.140.190.39:9090

Grafana:
  • Dashboard: http://10.140.190.39:3000
  • Dashboard (HTTPS): https://grafana.example.com
  • Default Login: admin / admin

HTTPS Services:
  Your tracker is configured to use HTTPS with custom domains.
  To access HTTPS services locally, add these entries to /etc/hosts:

    10.140.190.39 tracker1.example.com
    10.140.190.39 api.example.com
    10.140.190.39 health.example.com
    10.140.190.39 grafana.example.com

  You can then access:
    • Tracker: https://tracker1.example.com/announce
    • API: https://api.example.com/api/v1
    • Grafana: https://grafana.example.com

Next Step:
Your deployment is complete and running.
Monitor: http://10.140.190.39:9090 (Prometheus) and http://10.140.190.39:3000 (Grafana)
```

**Critical Requirements:**

- ✅ All sections must remain unchanged (format, content, structure)
- ✅ State-aware output (fields shown based on environment state)
- ✅ HTTPS hints only shown when custom domains are configured
- ✅ Next step guidance must be state-aware

### Automation Use Cases

**Primary use cases**: Extract environment state, instance IP, and service URLs for automation and monitoring.

**Common automation workflows**:

1. **State Verification**:
   - Check if environment is in expected state
   - Validate deployment progression
   - Trigger next deployment step based on state

2. **IP and Credentials Extraction**:
   - Extract IP, SSH username, port, and private key path from JSON
   - Use in subsequent automation steps (SSH access, DNS updates, etc.)
   - No manual parsing of text output required

3. **Service Discovery**:
   - Extract all service URLs (UDP trackers, HTTP trackers, API, health check)
   - Automated smoke testing of endpoints
   - Service registration in monitoring systems

4. **HTTPS Domain Management**:
   - Extract list of configured TLS domains
   - Automated DNS verification
   - Certificate monitoring setup

5. **Monitoring Integration**:
   - Extract Prometheus and Grafana URLs
   - Automated dashboard provisioning
   - Alert configuration

6. **AI Agent Workflows**:
   - Parse structured data without regex
   - Extract specific fields by path (e.g., `.infrastructure.instance_ip`)
   - Reduce hallucination risk with clear, typed data

## Implementation Approach

### Step 1: Add Serde Derives to DTOs

The `EnvironmentInfo` and related DTOs in `src/application/command_handlers/show/info/` need `#[derive(Serialize)]`:

```rust
// src/application/command_handlers/show/info/mod.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]  // Add Serialize
pub struct EnvironmentInfo {
    // ... existing fields
}

#[derive(Debug, Clone, Serialize)]  // Add Serialize
pub struct InfrastructureInfo {
    // ... existing fields
}
```

Also add to nested DTOs:

- `ServiceInfo` (in `info/tracker.rs`)
- `PrometheusInfo` (in `info/prometheus.rs`)
- `GrafanaInfo` (in `info/grafana.rs`)
- `TlsDomainInfo` (in `info/tracker.rs`)
- `LocalhostServiceInfo` (in `info/tracker.rs`)

### Step 2: Reorganize Views Module

```rust
// src/presentation/views/commands/show/views/mod.rs
mod text_view;  // Renamed from previous mod.rs content
mod json_view;  // NEW

pub use text_view::TextView;
pub use json_view::JsonView;

// Helper modules for TextView (keep existing)
mod basic;
mod grafana;
mod https_hint;
mod infrastructure;
mod next_step;
mod prometheus;
mod tracker_services;
```

### Step 3: Create JsonView

```rust
// src/presentation/views/commands/show/views/json_view.rs
use crate::application::command_handlers::show::info::EnvironmentInfo;
use serde_json;

pub struct JsonView;

impl JsonView {
    #[must_use]
    pub fn render(info: &EnvironmentInfo) -> String {
        serde_json::to_string_pretty(info)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
    }
}
```

### Step 4: Update Controller

Wire the view selection in `src/presentation/controllers/show/handler.rs`:

```rust
use crate::presentation::views::commands::show::{TextView, JsonView};
use crate::presentation::views::OutputFormat;

// In the handler function:
let output = match ctx.output_format() {
    OutputFormat::Text => TextView::render(&info),
    OutputFormat::Json => JsonView::render(&info),
};
```

## Testing Strategy

### Unit Tests

Add unit tests in `src/presentation/views/commands/show/views/json_view.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn it_should_render_created_state_as_json() {
        // Test minimal state
    }

    #[test]
    fn it_should_render_provisioned_state_with_infrastructure() {
        // Test with infrastructure info
    }

    #[test]
    fn it_should_render_running_state_with_all_services() {
        // Test full stack
    }

    #[test]
    fn it_should_handle_serialization_error() {
        // Test error handling
    }
}
```

### Manual Testing

Test with real environments in different states:

```bash
# Created state
torrust-tracker-deployer show my-env --output-format json | jq

# Provisioned state
torrust-tracker-deployer show my-env --output-format json | jq '.infrastructure.instance_ip'

# Running state
torrust-tracker-deployer show full-stack-lxd --output-format json | jq '.services.udp_trackers'
```

## Success Criteria

- [ ] JSON output correctly serializes `EnvironmentInfo` and nested DTOs
- [ ] JSON output varies correctly by state (created → provisioned → running)
- [ ] Text output remains unchanged (existing tests pass)
- [ ] Unit tests cover all state variations
- [ ] Manual testing validates real-world usage
- [ ] All linters pass
- [ ] User documentation updated

## Related Documentation

- [EPIC #348 - Add JSON output format support](../issues/348-epic-add-json-output-format-support.md)
- [Issue #349 - Add JSON output to create command](../issues/349-add-json-output-to-create-command.md) ✅ Completed
- [Issue #352 - Add JSON output to provision command](../issues/352-add-json-output-to-provision-command.md) ✅ Completed
- [Roadmap Section 12](../roadmap.md#12-add-json-output-format-support)
- [Output Handling Guide](../contributing/output-handling.md)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)

## Notes

### Why Show Command JSON is Valuable

The `show` command is a **read-only query** that consolidates all environment information in one place:

1. **Single Source of Truth**: One command provides state, infrastructure, and all service URLs
2. **AI Agent Friendly**: Structured data eliminates regex parsing and reduces hallucination
3. **Automation Hub**: Enables "query then act" patterns (check state → extract IP → take action)
4. **Debugging Aid**: Quick JSON dump for troubleshooting and logging

### Comparison with Other Commands

- **create**: Returns paths and references (where to find more info)
- **provision**: Returns critical connection details (IP, SSH credentials)
- **show**: Returns comprehensive current state (everything at once)

The `show` command with JSON output effectively becomes an API endpoint for querying deployment state.

---

**Created**: 2026-02-16
**Status**: ⏳ Not Started
