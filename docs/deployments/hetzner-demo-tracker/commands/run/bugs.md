# Run Command — Bugs

## Bug 1: MySQL app username `"root"` is not rejected at environment creation time

**Status**: Open — no fix applied yet
**Severity**: High — silently generates an invalid configuration that prevents MySQL from starting

### Symptom

The `run` command fails because the MySQL container is unhealthy. MySQL 8.4
refuses to start with `MYSQL_USER="root"` and prints the following error
repeatedly in its logs:

```text
[ERROR] [Entrypoint]: MYSQL_USER="root", MYSQL_USER and MYSQL_PASSWORD are for
configuring a regular user and cannot be used for the root user.
    Remove MYSQL_USER="root" and use one of the following to control the root
    user password:
    - MYSQL_ROOT_PASSWORD
    - MYSQL_ALLOW_EMPTY_PASSWORD
    - MYSQL_RANDOM_ROOT_PASSWORD
```

Because MySQL never reaches a healthy state, the dependent containers (tracker,
caddy, prometheus, grafana) all fail to start.

### Root Cause

The deployer accepts any string as `username` in the MySQL database block of the
environment JSON config. When `username` is `"root"`, the template renderer
places it into `MYSQL_USER` in the generated `.env` file:

```env
MYSQL_USER='root'   ← MySQL 8.4 rejects this
```

MySQL's `MYSQL_USER` environment variable is intended solely for creating a new
**non-root application user** on first boot. The root user is managed exclusively
via `MYSQL_ROOT_PASSWORD`. Passing `MYSQL_USER=root` is explicitly rejected since
MySQL 8.4.

The deployer does not validate this constraint. It silently renders an unusable
configuration, which only fails later at `run` time (when MySQL actually starts),
not at creation or release time.

### Affected Code

- `src/application/services/rendering/docker_compose.rs` —
  `create_mysql_contexts()` passes `username` directly to `MYSQL_USER` without
  checking for the reserved value `"root"`.
- `templates/docker-compose/.env.tera` — renders `MYSQL_USER='{{ mysql.user }}'`
  verbatim.

### Proposed Fix

The deployer should reject `"root"` as a MySQL application username at
**environment creation time**, returning a clear user-facing error such as:

```text
❌ Invalid MySQL configuration: username "root" is reserved.
   Use a non-root application username (e.g. "torrust").
   The MySQL root user is managed automatically via MYSQL_ROOT_PASSWORD.
```

The validation should live in the domain layer, close to the environment config
parsing step, so that it fails early and never reaches the rendering stage.

---

## Bug 2: MySQL root password silently diverges from the configured password

**Status**: Open — no fix applied yet
**Severity**: Medium — the root password is unguessable from the env config, but
the effective root password is inconsistent with what the operator provided

### Symptom

The MySQL root user password set inside the container is **not** the password
specified in the environment JSON config. If any tool or script tries to connect
to MySQL as `root` using the configured password it will be denied.

With `password = "secret"` in the env config, the `.env` file on the server
contains:

```env
MYSQL_ROOT_PASSWORD='secret_root'  ← not "secret"
MYSQL_PASSWORD='secret'
```

The auto-derived root password (`secret_root`) is never surfaced to the operator.

### Root Cause

The feature was never implemented. In
`src/application/services/rendering/docker_compose.rs`, `create_mysql_contexts()`
contains a placeholder that derives the root password by appending `"_root"` to
the configured password, with a comment acknowledging the gap:

```rust
// For MySQL, generate a secure root password (in production, this should be managed securely)
let root_password = format!("{password}_root");
```

The comment explicitly says this should be managed securely in production, but
no proper implementation was ever added. The `"_root"` suffix is a stub, not a
deliberate design decision. As a result the root password silently diverges from
the configured password and the gap is invisible to the operator.

### Affected Code

- `src/application/services/rendering/docker_compose.rs` —
  the `format!("{password}_root")` derivation inside `create_mysql_contexts()`.

### Proposed Fix

The root password should be either:

1. **Explicitly configurable** — expose a separate `root_password` field in the
   environment JSON config so the operator controls it directly; or
2. **Randomly generated at environment creation time** and stored in the
   environment state so it can be retrieved if needed.

Option 2 is preferred because it eliminates any predictable relationship between
the app password and the root password.

A migration path for existing deployments should be documented and the old
`format!("{password}_root")` derivation removed.
