# Improvements & Recommendations from Hetzner Demo Tracker Deployment

All deployer improvements and recommendations identified during this deployment,
collected in one place. Each entry links to the full description in the relevant
document.

---

## `create` command

### I-01 — Document `instance_name: null` auto-generation in template

The `create template` output contains `"instance_name": null` with no explanation
of the auto-generated value (`torrust-tracker-vm-{env_name}`). The template should
include an inline comment or a `_comment` field describing this behavior.

Full description: [commands/create/problems.md — instance_name: null unexplained](commands/create/problems.md#problem-template-generates-instance_name-null-with-no-explanation)

---

### I-02 — Default bind addresses to `[::]` (dual-stack) for public trackers

The `create template` command defaults to `0.0.0.0` (IPv4 only). Public trackers
should bind to `[::]`, which accepts both IPv4 and IPv6 on Linux. The template
generator should either default to `[::]` or include a note about the trade-off.

Full description: [commands/create/problems.md — Template defaults to `0.0.0.0`](commands/create/problems.md#problem-template-defaults-bind-addresses-to-0000-ipv4-only)

---

### I-03 — Prompt for database choice or note SQLite dev default

The `create template` command silently selects SQLite without informing the user.
It should either prompt for a database choice interactively or include a comment
noting that SQLite is the development default and MySQL is recommended for
production.

Full description: [commands/create/problems.md — Template silently defaults to SQLite](commands/create/problems.md#problem-template-silently-defaults-to-sqlite--no-database-choice-presented)

---

## `provision` command

### I-04 — Distinguish SSH failure reason in the probe loop

The SSH probe logs a generic "still waiting" message for every failed attempt
regardless of whether the port is unreachable (TCP timeout) or sshd is up
but authentication is rejected. Logging a different message per failure type
would significantly reduce investigation time.

Full description: [commands/provision/improvements.md — Distinguish SSH failure reason](commands/provision/improvements.md#1-distinguish-ssh-failure-reason-in-the-probe-loop)

---

### I-05 — Classify `error_kind` more precisely for SSH auth failures

A `WaitSshConnectivity` failure is always recorded as `NetworkConnectivity` in
the environment JSON, even when the root cause is authentication rejection (not
a network problem). A more specific `SshAuthenticationFailed` variant would
direct investigation to the right layer immediately.

Full description: [commands/provision/improvements.md — Classify error_kind more precisely](commands/provision/improvements.md#2-classify-error_kind-more-precisely-for-auth-failures)

---

### I-06 — Include per-attempt failure details in the provision trace file

The trace file only records a final summary. A condensed per-phase breakdown
of the SSH probe (how many attempts timed out vs. were rejected by sshd) would
be immediately actionable for operators without requiring analysis of
`data/logs/log.txt`.

Full description: [commands/provision/improvements.md — Per-attempt details in trace file](commands/provision/improvements.md#3-include-per-attempt-failure-details-in-the-trace-file)

---

### I-07 — Make SSH connectivity timeout configurable

The probe budget is hardcoded at 60 × 2 s = 120 s. Hetzner servers with
cloud-init user provisioning require over 3 minutes. This should be
configurable per provider, per env config, and via a CLI flag, with a longer
default for Hetzner.

Full description: [commands/provision/improvements.md — Configurable SSH connectivity timeout](commands/provision/improvements.md#4-support-configurable-ssh-connectivity-timeout)

---

### I-08 — Detect passphrase-protected SSH keys early and warn

The deployer does not check whether the configured SSH private key has a
passphrase. When running inside Docker (no agent, no TTY), a passphrase-protected
key silently fails every attempt. This should be caught at `create environment`
or `validate` time, with a clear actionable warning.

Full description: [commands/provision/improvements.md — Detect passphrase-protected keys early](commands/provision/improvements.md#7-detect-passphrase-protected-ssh-keys-early-and-warn-the-user)

---

### I-09 — Add `wait-for-ssh` command or `provision --resume` flag

When `provision` fails at `WaitSshConnectivity`, the server itself is healthy
but the environment must be destroyed and recreated from scratch. A
`wait-for-ssh` command or `provision --resume` flag would allow retrying only
the SSH probe step against an already-created server, saving the full
`tofu apply` + cloud-init cycle.

Full description: [commands/provision/improvements.md — `wait-for-ssh` command](commands/provision/improvements.md#5-add-a-wait-for-ssh-command-or---resume-flag-on-provision)

---

### I-10 — Include IPv6 address in `provision` output

The `provision` JSON output only includes the IPv4 instance IP. Hetzner
assigns an IPv6 address and /64 network to every server, but these are only
visible in the raw Tofu state file. Exposing them in the output avoids
operators having to consult the state file during post-provision steps like
floating IP setup.

Full description: [commands/provision/improvements.md — Include IPv6 in provision output](commands/provision/improvements.md#6-include-ipv6-address-in-provision-command-output)

---

## `run` command

### I-11 — Add lightweight post-start health check to `run`

The `Running` state only indicates that `docker compose up -d` returned exit
code 0, not that services are healthy. A lightweight poll of `docker compose ps`
after startup (until no container is in `starting` or `restarting` state) would
catch fast-failing containers — such as the tracker URL-encoding crash in this
deployment — without duplicating the full `test` smoke-test logic.

Full description: [commands/run/improvements.md — `Running` state does not guarantee healthy services](commands/run/improvements.md#improvement-running-state-does-not-guarantee-services-are-healthy)

---

## Cross-cutting

### I-12 — Add floating IP support to environment config and DNS checks

The `test` command compares resolved DNS addresses against the bare instance IP.
When a floating IP is in use (the recommended production setup), every domain
produces a false-positive DNS warning. The environment config should allow
specifying a `floating_ip` so the deployer uses it as the expected DNS target
and can optionally auto-assign it during provisioning.

Full description: [commands/improvements.md — Deployer not aware of floating IPs](commands/improvements.md#improvement-deployer-is-not-aware-of-floating-ips)

---

## Post-provision / operational

### I-13 — Write netplan floating IP config with correct permissions from the start

The post-provision guide writes the netplan file with `sudo tee`, which creates
it world-readable. Netplan then logs a warning requiring `chmod 600`. Using
`sudo install -m 600 /dev/stdin /etc/netplan/60-floating-ip.yaml` instead
avoids the warning and the manual fix step.

Full description: [post-provision/dns-setup.md — Improvements](post-provision/dns-setup.md#improvements)

---

## Summary

| ID   | Area           | Description                                                  |
| ---- | -------------- | ------------------------------------------------------------ |
| I-01 | `create`       | Document `instance_name` auto-generation in template         |
| I-02 | `create`       | Default bind addresses to `[::]` for public trackers         |
| I-03 | `create`       | Prompt for database choice or note SQLite dev default        |
| I-04 | `provision`    | Distinguish SSH failure reason in probe loop                 |
| I-05 | `provision`    | Classify `error_kind` more precisely for SSH auth failures   |
| I-06 | `provision`    | Include per-attempt SSH failure details in trace file        |
| I-07 | `provision`    | Make SSH connectivity timeout configurable                   |
| I-08 | `provision`    | Detect passphrase-protected SSH keys early and warn          |
| I-09 | `provision`    | Add `wait-for-ssh` command or `provision --resume` flag      |
| I-10 | `provision`    | Include IPv6 address in provision output                     |
| I-11 | `run`          | Add lightweight post-start health check                      |
| I-12 | Cross-cutting  | Add floating IP support to env config and DNS checks         |
| I-13 | Post-provision | Write netplan config with correct permissions from the start |
