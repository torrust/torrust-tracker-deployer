# Add JSON Output to Run Command

**Issue**: [#357](https://github.com/torrust/torrust-tracker-deployer/issues/357)
**Parent Epic**: [#348](https://github.com/torrust/torrust-tracker-deployer/issues/348) - Add JSON output format support
**Related**: [Roadmap Section 12.4](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support), [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed, [Issue #352 - Add JSON output to provision command](https://github.com/torrust/torrust-tracker-deployer/issues/352) ✅ Completed, [Issue #355 - Add JSON output to show command](https://github.com/torrust/torrust-tracker-deployer/issues/355) ✅ Completed

**Implementation Status**: ⏳ **NOT STARTED**

## Overview

Add machine-readable JSON output format (`--output-format json`) to the `run` command. This enables automation workflows and AI agents to programmatically extract service URLs and verify which services are running without parsing human-readable text.

## Goals

- [ ] Implement JSON output format for run command
- [ ] Preserve existing human-readable output as default
- [ ] Enable automation to extract service URLs and verify service availability
- [ ] Follow the architecture pattern established in #349, #352, and #355

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (`src/presentation/`)
**Module Path**: `src/presentation/views/commands/run/`
**Pattern**: Strategy Pattern with TextView and JsonView (established in previous implementations)

### Current Module Structure

The run command currently uses shared service URL views and does not have dedicated view components:

```rust
src/presentation/
├── controllers/run/
│   ├── handler.rs        (RunCommandController - uses shared views)
│   └── mod.rs
└── views/commands/
    └── shared/
        └── service_urls/
            ├── compact.rs (CompactServiceUrlsView)
            └── dns_hint.rs (DnsHintView)
```

### Required Module Structure

Create a new views module for the run command following the pattern from create, provision, and show commands:

```rust
src/presentation/views/commands/run/
├── mod.rs                (Re-exports TextView and JsonView)
├── text_view.rs          (NEW - TextView for human-readable output)
└── json_view.rs          (NEW - JsonView for machine-readable output)
```

**Key Changes Required:**

1. Create `src/presentation/views/commands/run/` directory
2. Create `text_view.rs` with `TextView` (uses existing `CompactServiceUrlsView` and `DnsHintView`)
3. Create `json_view.rs` with `JsonView` (serializes service information)
4. Update `RunCommandController` to use view selection based on `output_format`
5. Wire output_format through ExecutionContext → Router → Controller

**Note**: The run command should use the same `ServiceInfo` and `GrafanaInfo` DTOs from the application layer (`src/application/command_handlers/show/info/`) that are already used by the show command. These DTOs are already serializable (`#[derive(Serialize)]`).

### Architectural Constraints

- [ ] No business logic in presentation layer (views only format existing data)
- [ ] No changes to application or domain layers
- [ ] Follow output handling conventions ([docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] Use existing `OutputFormat` enum and `--output-format` flag from #349
- [ ] Reuse existing `ServiceInfo` and `GrafanaInfo` DTOs (no new DTOs needed)

### Anti-Patterns to Avoid

- ❌ Embedding output formatting logic in controller
- ❌ Mixing business logic with view formatting
- ❌ Changing the application command handler interface
- ❌ Creating redundant DTOs (reuse existing `ServiceInfo` and `GrafanaInfo`)

## Specifications

### JSON Output Schema

The JSON output should serialize the service information after services have been started. The schema uses the existing `ServiceInfo` and `GrafanaInfo` DTOs:

#### Example 1: Basic Configuration (HTTP-only, No Grafana)

```json
{
  "environment_name": "my-env",
  "state": "Running",
  "services": {
    "udp_trackers": ["udp://10.140.190.39:6969/announce"],
    "https_http_trackers": [],
    "direct_http_trackers": ["http://10.140.190.39:7070/announce"],
    "localhost_http_trackers": [],
    "api_endpoint": "http://10.140.190.39:1212/api",
    "api_uses_https": false,
    "api_is_localhost_only": false,
    "health_check_url": "http://10.140.190.39:1313/health_check",
    "health_check_uses_https": false,
    "health_check_is_localhost_only": false,
    "tls_domains": []
  },
  "grafana": null
}
```

#### Example 2: Full Stack with HTTPS

```json
{
  "environment_name": "full-stack-https",
  "state": "Running",
  "services": {
    "udp_trackers": ["udp://tracker.example.com:6969/announce"],
    "https_http_trackers": ["https://http.tracker.example.com/announce"],
    "direct_http_trackers": ["http://10.140.190.39:7070/announce"],
    "localhost_http_trackers": [],
    "api_endpoint": "https://api.tracker.example.com/api",
    "api_uses_https": true,
    "api_is_localhost_only": false,
    "health_check_url": "https://health.tracker.example.com/health_check",
    "health_check_uses_https": true,
    "health_check_is_localhost_only": false,
    "tls_domains": [
      {
        "domain": "http.tracker.example.com",
        "internal_port": 7070
      },
      {
        "domain": "api.tracker.example.com",
        "internal_port": 1212
      },
      {
        "domain": "grafana.tracker.example.com",
        "internal_port": 3000
      },
      {
        "domain": "health.tracker.example.com",
        "internal_port": 1313
      }
    ]
  },
  "grafana": {
    "url": "https://grafana.tracker.example.com/",
    "uses_https": true
  }
}
```

### Field Descriptions

#### Top-Level Fields

| Field              | Type           | Description                                                                  |
| ------------------ | -------------- | ---------------------------------------------------------------------------- |
| `environment_name` | string         | Name of the environment                                                      |
| `state`            | string         | Current environment state (always "Running" for run command)                 |
| `services`         | object         | Tracker service information (from `ServiceInfo` DTO)                         |
| `grafana`          | object \| null | Grafana service information (from `GrafanaInfo` DTO), null if not configured |

#### Service Fields (From `ServiceInfo` DTO)

All fields under the `services` object come from the existing `ServiceInfo` DTO:

| Field                            | Type     | Description                                        |
| -------------------------------- | -------- | -------------------------------------------------- |
| `udp_trackers`                   | string[] | UDP tracker announce URLs                          |
| `https_http_trackers`            | string[] | HTTPS HTTP tracker announce URLs (via Caddy TLS)   |
| `direct_http_trackers`           | string[] | Direct HTTP tracker announce URLs (IP-based)       |
| `localhost_http_trackers`        | object[] | Localhost-only HTTP trackers (SSH tunnel required) |
| `api_endpoint`                   | string   | API endpoint URL                                   |
| `api_uses_https`                 | boolean  | Whether API is accessed via HTTPS through Caddy    |
| `api_is_localhost_only`          | boolean  | Whether API is bound to localhost                  |
| `health_check_url`               | string   | Health check endpoint URL                          |
| `health_check_uses_https`        | boolean  | Whether health check uses HTTPS through Caddy      |
| `health_check_is_localhost_only` | boolean  | Whether health check is bound to localhost         |
| `tls_domains`                    | object[] | TLS domain configurations for HTTPS services       |

#### Grafana Fields (From `GrafanaInfo` DTO)

If Grafana is configured, the `grafana` object contains:

| Field        | Type    | Description                           |
| ------------ | ------- | ------------------------------------- |
| `url`        | string  | Grafana dashboard URL                 |
| `uses_https` | boolean | Whether Grafana is accessed via HTTPS |

### Human-Readable Output (Reference - Must Not Change)

The existing human-readable output must remain unchanged. This is the **actual current output** captured from a live deployment to ensure we can verify nothing breaks when adding JSON support.

#### Basic HTTP-only Output (Actual)

```text
✅ Run command completed for 'test-run-output-basic'

Services are now accessible:
  Tracker (UDP):  udp://udp.tracker.local:6969/announce
  Tracker (HTTP): http://10.140.190.133:7070/announce
  API:            http://10.140.190.133:1212/api
  Health Check:   http://10.140.190.133:1313/health_check
  Grafana:        http://10.140.190.133:3000/

Tip: Run 'torrust-tracker-deployer show test-run-output-basic' for full details
```

#### Full Stack with HTTPS Output (Actual)

```text
✅ Run command completed for 'test-run-output-https'

Services are now accessible:
  Tracker (UDP):  udp://udp.tracker.local:6969/announce
  Tracker (HTTP): https://http.tracker.local/announce
  API:            https://api.tracker.local/api
  Health Check:   https://health.tracker.local/health_check
  Grafana:        https://grafana.tracker.local/


Note: HTTPS services require DNS configuration. See 'show' command for details.

Tip: Run 'torrust-tracker-deployer show test-run-output-https' for full details
```

**Critical Requirements:**

- ✅ All sections must remain unchanged (format, content, structure)
- ✅ Only publicly accessible services are shown (localhost-only services excluded)
- ✅ UDP trackers use domain names when configured (not IP addresses)
- ✅ HTTPS hint is a simple note referencing the show command
- ✅ Tip about the show command must be present
- ✅ Success message format: `✅ Run command completed for '<env-name>'`

## Use Cases

### 1. Extract Service URLs for Testing

Automation scripts can parse the JSON output to extract service URLs and verify they are accessible:

```bash
#!/bin/bash
ENV_NAME="my-env"

# Start services and get JSON output
OUTPUT=$(torrust-tracker-deployer run "$ENV_NAME" --output-format json)

# Extract and test API endpoint
API_URL=$(echo "$OUTPUT" | jq -r '.services.api_endpoint')
if [ "$API_URL" != "null" ]; then
    echo "Testing API at: $API_URL"
    curl -s "$API_URL/stats"
fi

# Extract and test health check
HEALTH_URL=$(echo "$OUTPUT" | jq -r '.services.health_check_url')
if [ "$HEALTH_URL" != "null" ]; then
    echo "Testing health check at: $HEALTH_URL"
    curl -s "$HEALTH_URL"
fi
```

### 2. Verify Service Configuration

AI agents can verify that services are configured as expected:

```bash
#!/bin/bash
# Verify HTTPS is enabled for API
USES_HTTPS=$(torrust-tracker-deployer run my-env -o json | \
    jq -r '.services.api_uses_https')

if [ "$USES_HTTPS" = "true" ]; then
    echo "✓ API is properly configured with HTTPS"
else
    echo "⚠ API is not using HTTPS"
fi
```

### 3. Integration with CI/CD Pipelines

CI/CD pipelines can use JSON output to verify deployment success:

```bash
#!/bin/bash
# Start services and verify all expected services are running
OUTPUT=$(torrust-tracker-deployer run production-env -o json)

# Check if UDP tracker is available
UDP_COUNT=$(echo "$OUTPUT" | jq '.services.udp_trackers | length')
if [ "$UDP_COUNT" -gt 0 ]; then
    echo "✓ UDP tracker is running"
else
    echo "✗ UDP tracker not found"
    exit 1
fi

# Check if API is available
API_URL=$(echo "$OUTPUT" | jq -r '.services.api_endpoint')
if [ "$API_URL" != "null" ]; then
    echo "✓ API is available at: $API_URL"
else
    echo "✗ API not found"
    exit 1
fi
```

### 4. DNS Configuration Automation

Automation can extract TLS domain information to configure DNS:

```bash
#!/bin/bash
# Extract domains and instance IP for DNS configuration
OUTPUT=$(torrust-tracker-deployer run my-env -o json)

INSTANCE_IP=$(torrust-tracker-deployer show my-env -o json | \
    jq -r '.infrastructure.instance_ip')

DOMAINS=$(echo "$OUTPUT" | jq -r '.services.tls_domains[].domain')

if [ -n "$DOMAINS" ]; then
    echo "Configure these DNS A records pointing to $INSTANCE_IP:"
    echo "$DOMAINS" | while read -r domain; do
        echo "  $domain -> $INSTANCE_IP"
    done
fi
```

## Implementation Strategy

### Phase 1: Create Views Module Structure

1. Create `src/presentation/views/commands/run/` directory
2. Create `mod.rs` with re-exports
3. Create `text_view.rs` with `TextView` implementation
4. Create `json_view.rs` with `JsonView` implementation

### Phase 2: Implement TextView

Refactor existing output logic from `RunCommandController` into `TextView`:

- Use `CompactServiceUrlsView` for service URLs
- Use `DnsHintView` for DNS configuration hints
- Include tip about using the show command

### Phase 3: Implement JsonView

Create JSON output:

- Create a simple DTO for the JSON structure (environment name, state, services, grafana)
- Serialize `ServiceInfo` and `GrafanaInfo` (already have `#[derive(Serialize)]`)
- Use `serde_json::to_string_pretty()` for JSON output

### Phase 4: Wire View Selection in Controller

Update `RunCommandController` to:

- Accept `output_format: OutputFormat` parameter
- Match on `output_format` to call appropriate view
- Pass environment name, services, and grafana info to views

### Phase 5: Update Router

Update `src/presentation/dispatch/router.rs`:

- Pass `context.output_format()` to `RunCommandController`

### Phase 6: Testing

- [ ] Unit tests for TextView (verify existing output format)
- [ ] Unit tests for JsonView (verify JSON structure)
- [ ] Integration tests with real environments
- [ ] Verify all linters pass

## Testing Strategy

### Unit Tests

**TextView Tests** (`src/presentation/views/commands/run/text_view.rs`):

```rust
#[test]
fn it_should_render_basic_http_only_output() {
    // Test that TextView produces the expected human-readable format
}

#[test]
fn it_should_render_https_output_with_dns_hint() {
    // Test that TextView includes DNS hints when TLS domains are configured
}

#[test]
fn it_should_include_grafana_when_configured() {
    // Test that Grafana URL is shown when configured
}
```

**JsonView Tests** (`src/presentation/views/commands/run/json_view.rs`):

```rust
#[test]
fn it_should_render_basic_json_output() {
    // Test JSON structure for basic HTTP-only configuration
}

#[test]
fn it_should_render_full_stack_json_output() {
    // Test JSON structure for full stack with HTTPS
}

#[test]
fn it_should_produce_valid_json() {
    // Verify output can be parsed with serde_json::from_str
}

#[test]
fn it_should_handle_null_grafana() {
    // Test that grafana field is null when not configured
}
```

### Manual Testing

Test with real environments at different states:

```bash
# Test basic HTTP-only environment
cargo run -- run basic-env --output-format json | jq .

# Test full stack with HTTPS
cargo run -- run https-env --output-format json | jq .

# Verify human-readable output is unchanged
cargo run -- run basic-env --output-format text

# Test with environment that has no Grafana
cargo run -- run no-monitoring-env --output-format json | jq '.grafana'
```

## Documentation Requirements

### User Documentation

Update [`docs/user-guide/commands/run.md`](../user-guide/commands/run.md):

1. Add `--output-format` option to command syntax
2. Add JSON output format section with examples
3. Add automation use cases (similar to show command docs)
4. Include examples of extracting service URLs with `jq`

### Developer Documentation

- Ensure architecture follows pattern from #349, #352, and #355
- Document view selection logic in controller
- Reference shared `ServiceInfo` and `GrafanaInfo` DTOs

## Success Criteria

- [ ] JSON output format implemented for run command
- [ ] Human-readable text output unchanged (backward compatible)
- [ ] JSON structure follows existing DTO schemas
- [ ] All service URLs are included in JSON output
- [ ] TLS domain information is included when configured
- [ ] Grafana information is included when configured
- [ ] Unit tests pass for both TextView and JsonView
- [ ] Manual testing verifies correct output for different configurations
- [ ] All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- [ ] User documentation updated with JSON examples
- [ ] Architecture follows Strategy Pattern from previous implementations

## References

- [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed
- [Issue #352 - Add JSON output to provision command](https://github.com/torrust/torrust-tracker-deployer/issues/352) ✅ Completed
- [Issue #355 - Add JSON output to show command](https://github.com/torrust/torrust-tracker-deployer/issues/355) ✅ Completed
- [EPIC #348 - Add JSON output format support](https://github.com/torrust/torrust-tracker-deployer/issues/348)
- [Roadmap Section 12](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support)
- [Output Handling Guide](../contributing/output-handling.md)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
