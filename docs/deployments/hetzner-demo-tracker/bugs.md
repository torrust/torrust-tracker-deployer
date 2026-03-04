# Bugs Found During Hetzner Demo Tracker Deployment

All deployer bugs discovered during this deployment, collected in one place.
Each entry links to the full description in the relevant command document.

> **Legend**: 🔴 Open &nbsp;|&nbsp; 🟢 Fixed (in this branch, pending merge)

---

## `create` command

### B-01 — Template defaults bind addresses to `0.0.0.0` (IPv4 only)

🔴 Open &nbsp;|&nbsp; **Severity**: Major

The `create template` command hard-codes `0.0.0.0` as the default bind address for
all UDP and HTTP tracker sockets. IPv6 clients cannot connect. Public trackers
should bind to `[::]` by default.

Full description: [commands/create/problems.md — Template defaults bind addresses to `0.0.0.0`](commands/create/problems.md#problem-template-defaults-bind-addresses-to-0000-ipv4-only)

---

### B-02 — Template silently defaults to SQLite with no database choice

🔴 Open &nbsp;|&nbsp; **Severity**: Major

The `create template` command uses SQLite without prompting the user or noting that
MySQL is the recommended choice for production. Users may not notice until
deep into the deployment.

Full description: [commands/create/problems.md — Template silently defaults to SQLite](commands/create/problems.md#problem-template-silently-defaults-to-sqlite--no-database-choice-presented)

---

### B-03 — `instance_name: null` generated with no explanation

🔴 Open &nbsp;|&nbsp; **Severity**: Minor

The `create template` command outputs `"instance_name": null` with no
documentation of what the deployer will do with a null value (it auto-generates
`torrust-tracker-vm-{env_name}`). The template should explain this.

Full description: [commands/create/problems.md — Template generates `instance_name: null`](commands/create/problems.md#problem-template-generates-instance_name-null-with-no-explanation)

---

## `provision` command

### B-04 — SSH probe budget too short for Hetzner (120 s hardcoded)

🔴 Open &nbsp;|&nbsp; **Severity**: Major

The deployer polls SSH with a fixed budget of 60 × 2 s = 120 s.
On Hetzner `ccx23`, cloud-init user provisioning can take over 3 minutes,
causing `provision` to fail even when the server eventually comes up healthy.

Full description: [commands/provision/problems.md — SSH connectivity times out](commands/provision/problems.md#problem-provisioning-fails--ssh-connectivity-to-newly-created-server-times-out)

---

### B-05 — Passphrase-protected SSH keys fail silently inside Docker

🔴 Open &nbsp;|&nbsp; **Severity**: Major

When the deployer runs inside Docker (the standard production workflow), there is
no SSH agent. If the configured deployment key has a passphrase, every probe
attempt silently returns `Permission denied`. Neither `create environment` nor
`validate` warns about this. The user sees repeated SSH failures with no
diagnostic pointing at the passphrase as the cause.

Full description: [commands/provision/problems.md — SSH probe always fails — passphrase-protected key](commands/provision/problems.md#problem-ssh-probe-always-fails-from-docker-container--passphrase-protected-private-key)

---

### B-06 — UDP tracker domains missing from `provision` output

🔴 Open &nbsp;|&nbsp; **Severity**: Minor

The `domains` array in the `provision` JSON output only contains HTTP-based
domains. The two UDP tracker domains configured in the environment are silently
omitted.

Full description: [commands/provision/bugs.md — UDP tracker domains missing](commands/provision/bugs.md#bug-udp-tracker-domains-missing-from-provision-output)

---

## `release` command

### B-07 — `release` fails when `docker` is not in PATH (Docker-based usage)

🟢 Fixed (this branch) &nbsp;|&nbsp; **Severity**: High

The `release` command validates the rendered `docker-compose.yml` locally by
running `docker compose config`. When the deployer runs inside its own Docker
container (where `docker` is not installed), this validation fails with
`ENOENT`. The `release` command is completely broken for Docker-based usage.

**Fix**: the validator now skips with a warning when `docker` is not in PATH
instead of hard-failing.

Full description: [commands/release/bugs.md — `release` fails when docker not in PATH](commands/release/bugs.md#bug-release-fails-when-deployer-runs-inside-docker-docker-not-in-path)

---

## `run` command

### B-08 — MySQL `"root"` username not rejected at creation time

🔴 Open &nbsp;|&nbsp; **Severity**: High

The deployer accepts `"root"` as the MySQL application username. MySQL 8.4
explicitly rejects `MYSQL_USER=root`. The error only surfaces at `run` time
when MySQL fails to start, not at `create environment` when the bad value is first
accepted.

Full description: [commands/run/bugs.md — MySQL `"root"` not rejected at creation time](commands/run/bugs.md#bug-1-mysql-app-username-root-is-not-rejected-at-environment-creation-time)

---

### B-09 — MySQL root password silently diverges from operator-configured password

🔴 Open &nbsp;|&nbsp; **Severity**: Medium

The deployer auto-derives the MySQL root password by appending `"_root"` to the
configured application password (`format!("{password}_root")`). The effective
root password is never surfaced to the operator, making it impossible to connect
to MySQL as `root` using the value from the env config.

Full description: [commands/run/bugs.md — MySQL root password silently diverges](commands/run/bugs.md#bug-2-mysql-root-password-silently-diverges-from-the-configured-password)

---

### B-10 — MySQL password not URL-encoded in `tracker.toml` connection string

🔴 Open &nbsp;|&nbsp; **Severity**: High

The tracker `tracker.toml` template renders the MySQL password raw into the
database connection URL. If the password contains URL-special characters (e.g.
`/`, `@`, `#`), the URL is malformed and the tracker crashes at startup with a
misleading `InvalidPort` error. Worked around in this deployment by manually
URL-encoding the password.

Full description: [commands/run/bugs.md — MySQL password not URL-encoded](commands/run/bugs.md#bug-3-mysql-password-is-not-url-encoded-in-the-tracker-connection-string)

---

## `test` command

### B-11 — DNS check produces false positives when a floating IP is in use

🔴 Open &nbsp;|&nbsp; **Severity**: Minor

The `test` command resolves each domain and compares the result against the
**instance IP**. When a floating IP is assigned and DNS points to it instead,
every domain triggers a false-positive warning (`expected 46.225.234.201, got
116.202.176.169`). The deployer has no concept of floating IPs.

Full description: [commands/improvements.md — Deployer is not aware of floating IPs](commands/improvements.md#improvement-deployer-is-not-aware-of-floating-ips)

---

## Summary

| ID   | Command     | Description                                           | Severity | Status   |
| ---- | ----------- | ----------------------------------------------------- | -------- | -------- |
| B-01 | `create`    | Template binds to `0.0.0.0` (IPv4 only)               | Major    | 🔴 Open  |
| B-02 | `create`    | Template defaults to SQLite silently                  | Major    | 🔴 Open  |
| B-03 | `create`    | `instance_name: null` unexplained                     | Minor    | 🔴 Open  |
| B-04 | `provision` | SSH probe budget too short for Hetzner (120 s)        | Major    | 🔴 Open  |
| B-05 | `provision` | Passphrase-protected SSH keys fail silently in Docker | Major    | 🔴 Open  |
| B-06 | `provision` | UDP tracker domains missing from output               | Minor    | 🔴 Open  |
| B-07 | `release`   | Fails when `docker` not in PATH (Docker-based usage)  | High     | 🟢 Fixed |
| B-08 | `run`       | MySQL `"root"` username not rejected at creation time | High     | 🔴 Open  |
| B-09 | `run`       | MySQL root password silently diverges                 | Medium   | 🔴 Open  |
| B-10 | `run`       | MySQL password not URL-encoded in connection string   | High     | 🔴 Open  |
| B-11 | `test`      | DNS check false positives with floating IP            | Minor    | 🔴 Open  |
