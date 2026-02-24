# Docker Compose Template Renders Invalid Empty `networks:` Key for Tracker Service

**Issue**: #382
**Parent Epic**: None
**Related**: Discovered during manual E2E testing for #380

## Overview

The docker-compose.yml Tera template renders an invalid empty `networks:` key for
the tracker service when no optional services (Caddy, MySQL, Prometheus) are
enabled. This produces invalid YAML that Docker Compose rejects with:

```text
services.tracker.networks must be a list
```

The bug means the entire deployment workflow (`create → provision → configure →
release → run`) succeeds until the final `run` command, which fails when Docker
Compose attempts to validate the generated file. **There is no early validation
that catches this invalid template output.**

## Goals

- [x] Fix the template to conditionally render the `networks:` key only when networks exist
- [x] Ensure consistency across all service blocks in the template
- [x] Add a unit test that renders the template with a minimal config (no optional services) and validates the output
- [x] Validate rendered docker-compose.yml with `docker compose config --quiet` after template rendering to fail early

## Root Cause Analysis

### The Bug

In `templates/docker-compose/docker-compose.yml.tera`, lines 103-106, the tracker
service renders `networks:` unconditionally:

```tera
    networks:
{%- for network in tracker.networks %}
      - {{ network }}
{%- endfor %}
```

When `tracker.networks` is an empty list, this produces:

```yaml
networks:
ports:
```

Which is invalid YAML — Docker Compose expects `networks:` to be followed by a
list, not another key.

### Why `tracker.networks` Can Be Empty

The `TrackerConfig::derive_networks()` method in
`src/domain/tracker/config/mod.rs` (lines 560-585) conditionally adds networks:

```rust
impl NetworkDerivation for TrackerConfig {
    fn derive_networks(&self, enabled_services: &EnabledServices) -> Vec<Network> {
        let mut networks = Vec::new();

        // NET-01: Metrics network if Prometheus enabled
        if enabled_services.has(Service::Prometheus) {
            networks.push(Network::Metrics);
        }

        // NET-02: Database network if MySQL enabled
        if enabled_services.has(Service::MySQL) {
            networks.push(Network::Database);
        }

        // NET-03: Proxy network if Caddy enabled
        if enabled_services.has(Service::Caddy) {
            networks.push(Network::Proxy);
        }

        networks
    }
}
```

When the configuration uses:

- **SQLite** (no MySQL) → no Database network
- **No domain names** (no Caddy/TLS) → no Proxy network
- **No Prometheus** → no Metrics network

Result: `tracker.networks` is an empty `Vec`, and the template renders an invalid
`networks:` key with no items.

### Why Only the Tracker Is Affected

The tracker is the **only service that is always present AND can have zero
networks**:

| Service    | Always Present?   | Can Have Zero Networks?  | Affected?       |
| ---------- | ----------------- | ------------------------ | --------------- |
| Tracker    | Yes               | Yes                      | Fixed (guarded) |
| Caddy      | No (needs domain) | No (always has proxy)    | Fixed (guarded) |
| Prometheus | No (needs domain) | No (always has metrics)  | Fixed (guarded) |
| Grafana    | No (needs domain) | No (always has metrics)  | Fixed (guarded) |
| MySQL      | No (needs mysql)  | No (always has database) | Fixed (guarded) |
| Backup     | Yes               | Yes                      | No (guarded)    |

The **backup** service correctly handles this case with a guard (line 238):

```tera
{%- if backup.networks | length > 0 %}
    networks:
{%- for network in backup.networks %}
      - {{ network }}
{%- endfor %}
{%- endif %}
```

The tracker service block is missing this guard.

## Reproduction

### Environment Configuration

The following minimal configuration triggers the bug — no domains, SQLite
database, no Prometheus/Grafana:

```json
{
  "environment": {
    "name": "json-output-test"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/fixtures/testing_rsa",
    "public_key_path": "/path/to/fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-json-output-test"
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
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "0.0.0.0:1313"
    }
  }
}
```

### Rendered Invalid Output

The generated `build/json-output-test/docker-compose/docker-compose.yml` contains:

```yaml
  tracker:
    <<: *defaults
    image: torrust/tracker:develop
    container_name: tracker
    environment:
      - USER_ID=1000
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=${TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER}
      - TORRUST_TRACKER_CONFIG_TOML_PATH=${TORRUST_TRACKER_CONFIG_TOML_PATH}
      - TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN}
    networks:
    ports:
      # BitTorrent UDP announce
      - "6969:6969/udp"
```

Note line `networks:` followed directly by `ports:` — this is invalid YAML.

### Steps to Reproduce

1. Create an environment config with no domains and SQLite (see config above)
2. Run the full workflow: `create environment`, `provision`, `configure`, `release`
3. Run `run` → fails with `services.tracker.networks must be a list`

The bug only manifests at the `run` step because the generated file is not
validated until Docker Compose processes it.

## Implementation Plan

### Phase 1: Fix the Template

- [x] Add a conditional guard around the tracker's `networks:` block in
      `templates/docker-compose/docker-compose.yml.tera`, matching the pattern
      already used by the backup service:

  ```tera
  {%- if tracker.networks | length > 0 %}
      networks:
  {%- for network in tracker.networks %}
        - {{ network }}
  {%- endfor %}
  {%- endif %}
  ```

- [x] Audit all other service blocks in the template for the same pattern.
      Guards added for all five service blocks: `tracker`, `caddy`, `prometheus`,
      `grafana`, and `mysql`.

### Phase 2: Add Test Coverage

- [x] Add a unit test that renders the docker-compose template with a minimal
      config (SQLite, no domains, no Prometheus) and verifies the output does not
      contain an empty `networks:` key
      (`it_should_not_render_empty_networks_key_for_tracker_when_no_optional_services_are_configured`)
- [x] Add a unit test that verifies the rendered output contains the correct
      `networks:` key when Prometheus is enabled
      (`it_should_render_networks_key_for_tracker_when_prometheus_is_enabled`)

### Phase 3: Validate Rendered docker-compose.yml After Rendering

Add a validation step that runs `docker compose config --quiet` on the rendered
`docker-compose.yml` immediately after template rendering. This catches invalid
output **before** deployment reaches the VM, providing a fast fail with a clear
error message.

**Evidence that this works:**

```console
$ cd build/json-output-test/docker-compose
$ docker compose config --quiet
services.tracker.networks must be a list
$ echo $?
15
```

`docker compose config --quiet` validates the file structure without starting any
services. It returns exit code 0 on success and a non-zero exit code (15) with a
descriptive error on failure.

**Implementation approach:**

- [x] After template rendering in the `configure` step (where docker-compose.yml
      is generated), run `docker compose config --quiet` on the output file
      (`validate_docker_compose_file()` in `local_validator.rs`, called from
      `DockerComposeProjectGenerator::render()`)
- [x] If validation fails, report a clear error to the user indicating the
      rendered template is invalid, including the docker-compose error output
      (`DockerComposeValidationFailed` error variant with `help()` message)
- [x] This validation should run locally against the `build/<env>/docker-compose/`
      directory before files are uploaded to the VM
- [x] Requires `docker` to be available on the machine running the deployer (it
      already is, since Docker is a project dependency)
- [x] Add a unit/integration test covering the validation step
      (4 tests in `src/infrastructure/templating/docker_compose/local_validator.rs`)

**Benefits:**

- Catches template bugs at `configure` time instead of `run` time
- Provides immediate, actionable feedback
- Acts as a safety net for future template changes
- No additional dependencies required

### Phase 4: Documentation

- [x] Update the template documentation comments if needed (no changes required;
      guards are self-documenting)

## Acceptance Criteria

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [x] Minimal config (SQLite, no domains, no Prometheus) produces valid docker-compose.yml
- [x] Template guards are consistent across all service blocks
- [x] Unit test covers the minimal-config rendering scenario
- [x] Rendered docker-compose.yml is validated with `docker compose config --quiet` after template rendering
- [x] Invalid templates produce a clear, actionable error at `configure` time (not `run` time)
- [x] The `run` command succeeds with the configuration that previously failed
  - Verified with `envs/minimal-fix-test.json` (SQLite, no domains, no Prometheus):
    `create → provision → configure → release → run → test` all passed ✅

## Related Documentation

- Template source: `templates/docker-compose/docker-compose.yml.tera`
- Network derivation: `src/domain/tracker/config/mod.rs` (lines 560-585)
- Network topology: `src/domain/topology/network.rs`
- Template context builder: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/builder.rs`
- Local validator: `src/infrastructure/templating/docker_compose/local_validator.rs`
- ADR: [Docker Compose Local Validation Placement](../decisions/docker-compose-local-validation-placement.md)

## Notes

- All service `networks:` blocks (`tracker`, `caddy`, `prometheus`, `grafana`,
  `mysql`) are now guarded with `{%- if <service>.networks | length > 0 %}`,
  matching the pre-existing `backup` guard. The template is now consistent.
- This bug affects any "minimal" deployment configuration without optional services.
  As the project adds more minimal deployment examples, this will become a more
  common failure mode.
- Phase 3 addresses the systemic issue: even after fixing this specific bug,
  future template changes could introduce similar problems. The
  `docker compose config --quiet` validation acts as a safety net that catches any
  structural issues in generated docker-compose files.
