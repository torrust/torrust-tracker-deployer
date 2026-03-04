# Problems: create

Issues encountered while running the `create template` and `create environment` commands.

> This is a living document — problems are added as they occur.

<!--
Template for each problem:

## Problem: [Short description]

**Command**: `create template` | `create environment` | `validate`
**Severity**: [Blocker / Major / Minor]

### Symptom

What we observed (error messages, unexpected behavior).

### Root Cause

Why it happened.

### Resolution

How we fixed it.

### Prevention

How to avoid this in the future (if applicable).
-->

## Problem: Template generates `instance_name: null` with no explanation

**Command**: `create template`
**Severity**: Minor

### Symptom

Running `create template --provider hetzner` generates a config with `"instance_name": null` in the
`environment` section. It is unclear whether this field is optional or required, and what value the
deployer will use at runtime if it is left as `null`.

```json
{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME",
    "instance_name": null
  }
}
```

### Root Cause

`instance_name` is an optional field. When `null`, the deployer auto-generates the server name as
`torrust-tracker-vm-{env_name}` (for example, `torrust-tracker-vm-torrust-tracker-demo`). The
template does not document this, leaving users uncertain.

### Resolution

Leave the field as `null` to accept the auto-generated name, or provide a custom value following
instance naming rules (1–63 characters, ASCII letters/numbers/dashes, cannot start or end with a
dash).

For this deployment we left it as `null`, so the server will be named
`torrust-tracker-vm-torrust-tracker-demo`.

### Prevention

The template generator should add an inline comment or a companion `_comment` field explaining
the auto-generation behavior.

---

## Problem: Template defaults bind addresses to `0.0.0.0` (IPv4 only)

**Command**: `create template`
**Severity**: Major

### Symptom

The generated template binds all UDP and HTTP tracker sockets to `0.0.0.0`:

```json
{ "bind_address": "0.0.0.0:6969" }
```

This works for IPv4 clients but completely ignores IPv6 connections, which is unacceptable for a
public-facing tracker in 2026.

### Root Cause

The `create template` command uses `0.0.0.0` as the hardcoded default, which is the safe minimum
for local/LXD environments. No prompt or note about IPv6 is shown.

### Resolution

Change all public-facing bind addresses to `[::]` (the IPv6 any-address). On Linux, `[::]` also
accepts IPv4 connections via the IPv4-mapped address mechanism (`::ffff:x.x.x.x`), so only a
single socket per port is required:

```json
{ "bind_address": "[::]:6969", "domain": "udp1.torrust-tracker-demo.com" }
```

The `health_check_api` is internal (used only by Docker health probes) and remains on
`127.0.0.1:1313`.

### Prevention

The template generator should default to `[::]` for public-facing sockets, or at minimum include a
note explaining the dual-stack trade-off. A `validate` warning for `0.0.0.0` on Hetzner
deployments would also help.

---

## Problem: Template silently defaults to SQLite — no database choice presented

**Command**: `create template`
**Severity**: Major

### Symptom

The generated template uses SQLite without asking the user:

```json
{
  "database": {
    "driver": "sqlite3",
    "database_name": "tracker.db"
  }
}
```

For a production-facing demo tracker, SQLite has limitations (no concurrent writes, no easy
remote inspection). Users may not notice the default until they are deep into the deployment.

### Root Cause

The `create template` command uses SQLite as the default database because it requires no additional
configuration fields (no host, port, username, password). It is the simpler default for local
development. No prompt or warning is shown.

### Resolution

Manually change the database section to MySQL after generating the template:

```json
{
  "database": {
    "driver": "mysql",
    "host": "mysql",
    "port": 3306,
    "database_name": "torrust",
    "username": "root",
    "password": "<MYSQL_ROOT_PASSWORD>"
  }
}
```

Use a securely generated password (e.g., `openssl rand -base64 24`).

### Prevention

The `create template` command should either prompt the user for a database choice, or include a
prominent comment in the generated file noting that SQLite is the dev default and MySQL is
recommended for production.
