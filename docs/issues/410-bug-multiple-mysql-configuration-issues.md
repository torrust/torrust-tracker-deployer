# Bug: Multiple MySQL Configuration Issues in Tracker Deployer

**Issue**: #410
**Parent Epic**: None
**Related**: #405 - Deploy Hetzner Demo Tracker and Document the Process,
[torrust/torrust-tracker#1606](https://github.com/torrust/torrust-tracker/issues/1606) — Document DSN password URL-encoding requirement for MySQL connection string

## Overview

Three related MySQL configuration bugs were discovered during the Hetzner demo
deployment (#405). All three stem from the same area of the codebase (MySQL setup in
the deployer) and are best fixed together.

**Bug 1 — MySQL DSN hardcoded in `tracker.toml` with no URL-encoding**: The tracker
configuration template builds the MySQL DSN by interpolating raw credential values
directly into the URL string. If the password contains URL-reserved characters the DSN
becomes malformed and the tracker fails to connect. Additionally, the DSN (including the
plaintext password) ends up in a mounted config file rather than an environment
variable, violating the project's secret-handling convention.

**Bug 2 — MySQL root password is not user-configurable**: The deployer silently derives
the MySQL root password as `{app_password}_root` (hardcoded in
`src/application/services/rendering/docker_compose.rs`). Users have no way to supply
their own root password via the environment configuration JSON. This also means the root
password is entirely predictable from the app password.

**Bug 3 — No validation that MySQL app username is not `"root"`**: The MySQL Docker
image reserves the `root` username for the built-in administrator account. If a user
supplies `"root"` as the app DB username in their environment JSON, Docker will refuse
to initialize the database (the `MYSQL_USER` variable cannot be set to `root`). The
domain type `MysqlConfig` validates for an empty username but not for this reserved
value.

## Goals

### Bug 1 — DSN in `tracker.toml`

- [x] Move the MySQL DSN out of `tracker.toml` and into an environment variable override,
      consistent with `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER` and
      `TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN`
- [x] Build the percent-encoded DSN in Rust and expose it in the `.env` file as
      `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH`
- [x] Pass the new env var into the tracker container via `docker-compose.yml.tera`
- [x] Remove the raw DSN line from `tracker.toml.tera` for the MySQL case
- [x] Remove the now-unused `MysqlTemplateConfig` from `TrackerContext`

### Bug 2 — Root password not configurable

- [x] Add an optional `root_password` field to the MySQL section of the environment
      configuration JSON schema
- [x] When a root password is provided by the user, use it; when omitted, generate a
      strong random password at environment creation time (application layer) rather
      than deriving it from the app password
- [x] Remove the `format!("{password}_root")` derivation from `create_mysql_contexts`

### Bug 3 — Reserved username not rejected

- [x] Add a `ReservedUsername` variant to `MysqlConfigError`
- [x] Reject `"root"` as the app DB username in `MysqlConfig::new()` with a clear,
      actionable error message

## Specifications

### Root Cause

Two distinct problems share the same fix:

**Problem 1 — URL encoding**: The MySQL DSN is a URL. Per RFC 3986, the user-info
component (`user:password@`) must percent-encode any character from the reserved set.
The current template interpolates raw values. Base64-generated secrets (common from
secret managers and AI agents) always contain `+` and `/`, which are reserved characters.

Common problematic characters:

| Character | URL encoding |
| --------- | ------------ |
| `@`       | `%40`        |
| `:`       | `%3A`        |
| `/`       | `%2F`        |
| `+`       | `%2B`        |
| `#`       | `%23`        |
| `?`       | `%3F`        |
| `%`       | `%25`        |

**Problem 2 — Secret in config file**: `tracker.toml` is written by the `deploy
tracker-config` step and mounted read-only into the container. Placing the raw DSN
(with password) there conflicts with the project convention of injecting secrets via
`.env`. The `.env` file already carries `MYSQL_PASSWORD` for the MySQL container — the
same secret should not also appear in a different form inside the tracker config file.

### Solution: env var override for `core.database.path`

The tracker supports runtime config overrides through env vars following the pattern
`TORRUST_TRACKER_CONFIG_OVERRIDE_<SECTION__KEY>`. This is already used for:

- `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`
- `TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN`

The fix adds `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` for the MySQL case.
The Rust rendering layer (not the Tera template) constructs the full percent-encoded DSN
and places it in the `.env` file. The tracker container receives it through
`docker-compose.yml` environment injection, overriding whatever `tracker.toml` says
(or does not say) about `core.database.path`.

### Affected Modules and Types

#### `Cargo.toml`

- Add `percent-encoding` crate dependency.

#### `src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`

- `TrackerServiceConfig`: add an optional field to carry the database path DSN
  (populated only for MySQL; `None` for SQLite).
- `EnvContext::new` (SQLite constructor): leave the new field as `None`.
- `EnvContext::new_with_mysql`: use `percent_encoding::utf8_percent_encode` with
  `NON_ALPHANUMERIC` to encode the username and password, construct the full DSN string,
  and store it in the new `TrackerServiceConfig` field.

#### `templates/docker-compose/.env.tera`

- Inside the `{%- if mysql %}` block, add a line that renders
  `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` from the new
  `tracker.database_path` (or equivalent) field.

#### `templates/docker-compose/docker-compose.yml.tera`

- In the tracker service `environment:` section, conditionally inject
  `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` from the `.env` file when
  MySQL is configured (use `{%- if database.mysql %}` similar to the existing
  `depends_on` block).

#### `templates/tracker/tracker.toml.tera`

- In the `{%- elif database_driver == "mysql" %}` block: remove the `path = ...` line.
  Replace with a comment explaining that the connection path is injected via the
  `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` environment variable.

#### `src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs`

- `MysqlTemplateConfig`: remove this struct entirely — none of its fields are used by
  the tracker template once the `path` line is gone.
- `TrackerContext::from_config`: remove the MySQL branch that builds
  `MysqlTemplateConfig`; the MySQL case reduces to setting `database_driver = "mysql"`
  (already handled by the shared driver field).
- Remove or update the existing unit test that asserts `mysql_password` on
  `MysqlTemplateConfig`.

### What Does Not Change — Bug 1

- `templates/docker-compose/.env.tera`: the `MYSQL_PASSWORD` line for the MySQL service
  is unchanged — it is used by the MySQL container, not the tracker.
- `TrackerContext` still carries `database_driver` and the SQLite config; only the
  MySQL-specific struct is removed.
- The `.env.tera` and `docker-compose.yml.tera` templates' overall structure are
  unchanged; only additive lines are inserted.

### Bug 2 — MySQL Root Password Not Configurable (auto-derived)

**Root cause**: In
`src/application/services/rendering/docker_compose.rs`, the `create_mysql_contexts`
function derives the root password unconditionally:

```rust
let root_password = format!("{password}_root");
```

This value is never read from the environment configuration JSON — the JSON schema has
no `root_password` field for MySQL. The deployer therefore always sets
`MYSQL_ROOT_PASSWORD` in `.env` to `{app_password}_root`, which:

- Is entirely predictable from the app password (security concern).
- Cannot be rotated independently of the app password.
- Offers no escape hatch for environments that require a specific root password.

**Fix**: Add an optional `root_password` to the MySQL section of the environment
configuration JSON. If the user provides it, use it. If they omit it, generate a
cryptographically random password at **environment creation time** (application layer)
instead of deriving it from the app password. Remove the `format!("{password}_root")`
derivation.

**Key design decision — generate at creation time, not render time**: The root password
is a domain invariant that must remain stable across multiple renders (e.g. re-deploying
without reprovisioning). Generating it at render time would produce a different
`MYSQL_ROOT_PASSWORD` on each render, breaking MySQL container restarts. Instead,
generation happens once in the application layer (`TryFrom<DatabaseSection>`) when the
environment is first created, and the value is persisted alongside the rest of the
environment config.

**Affected modules and types**:

- `Cargo.toml`: add `rand = "0.9"` dependency.
- `schemas/environment-config.json`: add optional `root_password` string to the MySQL
  database object.
- `src/shared/secrets/random.rs` (new file): `generate_random_password() -> Password`
  using `rand::rng()` (ThreadRng seeded from OsRng), guaranteeing one character from each
  class (lower, upper, digit, symbol), filled to 32 characters, then shuffled. Satisfies
  MySQL `validate_password MEDIUM` policy.
- `src/shared/secrets/mod.rs` and `src/shared/mod.rs`: re-export
  `generate_random_password`.
- `src/domain/tracker/config/core/database/mysql.rs` (`MysqlConfig`): `root_password`
  field is `Password` (non-optional) — the domain type always has a value. Constructor
  `new()` takes `root_password: Password`. Accessor `root_password() -> &Password` added.
  `MysqlConfigRaw` (the serde deserialization intermediate) keeps
  `root_password: Option<Password>` with `#[serde(default)]` for backward compatibility
  with persisted environments that pre-date this field; missing values are filled by
  calling `generate_random_password()` during deserialization.
- `src/application/command_handlers/create/config/tracker/tracker_core_section.rs`
  (`TryFrom<DatabaseSection>`): generation happens here — the optional user-supplied
  `root_password` is mapped to `Password` if present, or `generate_random_password()` is
  called if absent. This is the single point of generation for new environments.
- `src/application/services/rendering/docker_compose.rs` (`create_mysql_contexts`):
  `root_password` parameter is now `PlainPassword` (non-optional); call site passes
  `mysql_config.root_password().expose_secret().to_string()`. No generation logic
  remains here.

### Bug 3 — Reserved MySQL Username `"root"` Not Rejected

**Root cause**: The official MySQL Docker image initializes the database according to
several environment variables. The `MYSQL_USER` variable is used to create a regular
app user, but the MySQL image explicitly [rejects `root`](https://hub.docker.com/_/mysql)
for this variable because `root` is already created as the privileged superuser. If
`MYSQL_USER=root` is set, the container initialization will fail with an error like:

```text
[ERROR] [Entrypoint]: MYSQL_USER="root", MYSQL_USER and MYSQL_ROOT_USER cannot be the same.
```

The domain type `MysqlConfig::new()` in
`src/domain/tracker/config/core/database/mysql.rs` validates for an empty username but
does not check for the reserved value `"root"`:

```rust
if username.is_empty() {
    return Err(MysqlConfigError::EmptyUsername);
}
// ← missing: if username == "root" { return Err(ReservedUsername); }
```

The error is therefore deferred until Docker container startup, far from the source of
the misconfiguration.

**Fix**: Add `ReservedUsername` to `MysqlConfigError` and check `username == "root"` in
`MysqlConfig::new()`, before the `Ok(Self { ... })` return, with a clear actionable
error message.

**Affected modules and types**:

- `src/domain/tracker/config/core/database/mysql.rs`:
  - Add `ReservedUsername` variant to `MysqlConfigError` with a `help()` message
    explaining the constraint and directing the user to choose a different username (e.g.
    `"tracker_user"`).
  - In `MysqlConfig::new()`: add the reserved username check after the empty-username
    check.
  - Add a unit test `it_should_reject_root_as_username()` mirroring the existing
    `it_should_reject_empty_username_when_creating_mysql_config` test.

## Implementation Plan

Tasks are ordered from simplest to most complex.

### Phase 1: Reject reserved MySQL username (Bug 3)

- [x] In `MysqlConfigError` (`mysql.rs`): add `ReservedUsername` variant
- [x] Add `help()` arm for `ReservedUsername` with actionable fix instructions
- [x] In `MysqlConfig::new()`: add `if username == "root"` guard returning
      `Err(MysqlConfigError::ReservedUsername)`
- [x] Add unit test `it_should_reject_root_as_username`

### Phase 2: Make root password configurable (Bug 2)

- [x] `schemas/environment-config.json`: add optional `root_password` string to the
      MySQL database object
- [x] `MysqlConfig` (`mysql.rs`): `root_password` is `Password` (non-optional) in the
      domain — always has a value. `MysqlConfigRaw` uses `Option<Password>` for backward
      compat with persisted environments lacking the field.
- [x] `src/shared/secrets/random.rs` (new): `generate_random_password() -> Password`
      using mixed charset (lower + upper + digit + symbol), length 32, satisfies MySQL
      MEDIUM password policy
- [x] `TryFrom<DatabaseSection>` (`tracker_core_section.rs`): generates root password at
      environment creation time — not at render time — so it is stable across re-renders
- [x] `create_mysql_contexts` (`docker_compose.rs`): replaced `format!("{password}_root")`
      with `mysql_config.root_password().expose_secret().to_string()`; no generation
      logic remains here

### Phase 3: Move DSN to env var override and add URL-encoding (Bug 1)

- [x] Add `percent-encoding` to `Cargo.toml`
- [x] `TrackerServiceConfig` (`env/context.rs`): add optional database path field
- [x] `EnvContext::new_with_mysql`: percent-encode username and password with
      `utf8_percent_encode(..., USERINFO_ENCODE)` (custom AsciiSet preserving RFC 3986
      unreserved chars), build the full DSN string, store in the new field
- [x] `TrackerContext` (`tracker_config/context.rs`): remove `MysqlTemplateConfig` and
      the MySQL branch that builds it
- [x] `templates/docker-compose/.env.tera`: add
      `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` inside `{%- if mysql %}`,
      placed in the Tracker Service Configuration section
- [x] `templates/docker-compose/docker-compose.yml.tera`: inject the new env var into
      the tracker service `environment:` section, conditionally on `{%- if mysql %}`
- [x] `templates/tracker/tracker.toml.tera`: remove the MySQL `path =` line; add a
      comment explaining that the connection path is injected via the env var override

### Phase 4: Tests

- [x] `mysql.rs`: add `it_should_reject_root_as_username` unit test (Phase 1)
- [x] `env/context.rs`: add test that `new_with_mysql` produces a correctly
      percent-encoded DSN for a password containing special characters
- [x] `env/context.rs`: add test that `new` (SQLite) leaves the database path field as
      `None`
- [x] `tracker_config/context.rs`: remove or update the test that referenced
      `mysql_password` on `MysqlTemplateConfig`
- [x] Run `cargo test` to verify all tests pass (2314 passed)

### Phase 5: Linting and pre-commit

- [x] Run linters: `cargo run --bin linter all`
- [x] Run pre-commit: `./scripts/pre-commit.sh`

> **Status**: Phases 1–5 complete and committed.

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR to minimize
> back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria — Bug 3 (reserved username)**:

- [x] `MysqlConfig::new()` returns `Err(MysqlConfigError::ReservedUsername)` when
      username is `"root"`
- [x] `MysqlConfigError::ReservedUsername` has a `help()` message with an actionable fix
- [x] A unit test for the reserved username rejection exists and passes

**Task-Specific Criteria — Bug 2 (root password)**:

- [x] The environment configuration JSON schema accepts an optional `root_password` field
      in the MySQL database object
- [x] When `root_password` is provided in the env JSON it is used as `MYSQL_ROOT_PASSWORD`
      in the rendered `.env`
- [x] When `root_password` is omitted, a randomly generated password is used — it is
      **not** derived from the app password
- [x] `create_mysql_contexts` no longer contains `format!("{password}_root")`
- [x] The random password is generated once at environment creation time (not at render
      time), ensuring stability across multiple renders
- [x] The domain type `MysqlConfig.root_password` is always populated (`Password`,
      non-optional)

**Task-Specific Criteria — Bug 1 (DSN in tracker.toml)**:

- [x] The rendered `tracker.toml` for a MySQL deployment does **not** contain the
      database password
- [x] The rendered `.env` file contains
      `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` with a correctly
      percent-encoded DSN when MySQL is configured
- [x] The rendered `.env` file does **not** contain
      `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` when SQLite is configured
- [x] The rendered `docker-compose.yml` injects
      `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH` into the tracker service
      environment when MySQL is configured
- [x] A MySQL password containing URL-reserved characters (e.g. `@`, `+`, `/`) produces
      a valid, correctly encoded DSN in the `.env` file (verified with `tracker_p@ss!word#1`)
- [x] A MySQL password with only alphanumeric characters is rendered unchanged
- [x] `MysqlTemplateConfig` no longer exists in `tracker_config/context.rs`
- [x] `cargo machete` reports no unused dependencies

## Manual E2E Verification Test

This test verifies the fix end-to-end on a local LXD VM, using a MySQL password that
contains URL-reserved characters. It validates that the rendered templates contain the
expected values and that the tracker connects to MySQL successfully.

### Test Environment Configuration

Create the environment file `envs/mysql-special-chars-test.json`:

```json
{
  "environment": {
    "name": "mysql-special-chars-test",
    "instance_name": null
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-mysql-special-chars-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "mysql",
        "port": 3306,
        "database_name": "tracker",
        "username": "tracker_user",
        "password": "p@ss:w/ord+1"
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
      "bind_address": "127.0.0.1:1313"
    }
  }
}
```

> The password `p@ss:w/ord+1` contains `@` (`%40`), `:` (`%3A`), `/` (`%2F`),
> and `+` (`%2B`) — all URL-reserved characters that triggered the original bug.

### Step 1: Render and Inspect Artifacts (No Infrastructure Needed)

Before provisioning, verify the rendered templates have the correct values.

```bash
# Create the environment
cargo run -- create environment --env-file envs/mysql-special-chars-test.json

# Render artifacts to an output directory (use any placeholder IP)
cargo run -- render --env-name mysql-special-chars-test \
    --instance-ip 192.168.1.100 \
    --output-dir ./tmp/mysql-special-chars-test
```

**Verify `.env` contains the encoded DSN, NOT the raw password:**

```bash
grep "DATABASE__PATH" ./tmp/mysql-special-chars-test/.env
```

Expected — the DSN must use percent-encoded values:

```text
TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH='mysql://tracker_user:p%40ss%3Aw%2Ford%2B1@mysql:3306/tracker'
```

**Verify `tracker.toml` does NOT contain the password:**

```bash
grep -i "password\|p@ss\|p%40\|mysql://" ./tmp/mysql-special-chars-test/tracker/tracker.toml
```

Expected — no output (neither the raw password nor the DSN should appear in `tracker.toml`).

**Verify `docker-compose.yml` injects the new env var into the tracker service:**

```bash
grep "DATABASE__PATH" ./tmp/mysql-special-chars-test/docker-compose.yml
```

Expected:

```text
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH=${TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH}
```

**Verify `MYSQL_PASSWORD` in `.env` still holds the raw (unencoded) password** — it is used by the MySQL container directly, not in a URL:

```bash
grep "MYSQL_PASSWORD" ./tmp/mysql-special-chars-test/.env
```

Expected:

```text
MYSQL_PASSWORD='p@ss:w/ord+1'
```

### Step 2: Full Deployment on LXD

```bash
# Provision the VM
cargo run -- provision mysql-special-chars-test

# Configure the OS (Docker, firewall, storage directories)
cargo run -- configure mysql-special-chars-test

# Deploy compose files and config
cargo run -- release mysql-special-chars-test

# Start services
cargo run -- run mysql-special-chars-test
```

### Step 3: Get the Instance IP

```bash
export INSTANCE_IP=$(cat data/mysql-special-chars-test/environment.json \
    | jq -r '.Running.context.runtime_outputs.instance_ip')
echo "VM IP: $INSTANCE_IP"
```

### Step 4: Verify Containers Are Running and Healthy

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
    "docker ps --format 'table {{.Names}}\t{{.Status}}'"
```

Expected — both containers healthy:

```text
NAMES     STATUS
tracker   Up X seconds (healthy)
mysql     Up X seconds (healthy)
```

> If the tracker shows `(unhealthy)` or is restarting, the DSN was not encoded
> correctly and the bug is not fixed.

### Step 5: Verify Tracker Connects to MySQL

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
    "docker logs tracker 2>&1 | head -20"
```

Expected — no `Access denied` or `parse error` messages, tracker shows it started
and is listening.

### Step 6: Verify Tracker API is Reachable

```bash
curl -s http://$INSTANCE_IP:1212/api/v1/stats?token=MyAccessToken
```

Expected — JSON response with tracker statistics (not an error).

### Step 7: Cleanup

```bash
cargo run -- destroy mysql-special-chars-test
rm envs/mysql-special-chars-test.json
rm -rf ./tmp/mysql-special-chars-test
```

Also clean up the LXD profile if it was created:

```bash
lxc profile delete torrust-profile-mysql-special-chars-test
```

## Related Documentation

- [templates/tracker/tracker.toml.tera](../../templates/tracker/tracker.toml.tera) — affected template
- [templates/docker-compose/.env.tera](../../templates/docker-compose/.env.tera) — where the DSN override is added
- [templates/docker-compose/docker-compose.yml.tera](../../templates/docker-compose/docker-compose.yml.tera) — where the env var is injected
- [src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs](../../src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs) — main Rust change
- [src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs](../../src/infrastructure/templating/tracker/template/wrapper/tracker_config/context.rs) — cleanup
- [torrust/torrust-tracker#1606](https://github.com/torrust/torrust-tracker/issues/1606) — upstream issue documenting the DSN encoding requirement
- [docs/issues/405-deploy-hetzner-demo-tracker-and-document-process.md](405-deploy-hetzner-demo-tracker-and-document-process.md) — deployment issue where this bug was discovered
