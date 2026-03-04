# Run Command — Problems

## Problem: `run` fails — MySQL container is unhealthy (`MYSQL_USER="root"` rejected)

**Status**: Root cause identified (2026-03-04)
**Severity**: High — `run` is blocked

### Symptom

The `run` command fails at the "Start Docker Compose services" Ansible task:

```text
❌ Run command failed: Failed to start services in environment 'environment':
Ansible playbook 'run-compose-services' failed: Command
'ansible-playbook -v run-compose-services.yml' failed with exit code 2
```

The Docker Compose `up -d` output shows all networks and containers were created,
but `mysql` failed its health check and the dependent containers could not start:

```text
 Container mysql  Error
dependency failed to start: container mysql is unhealthy
```

### Root Cause

MySQL 8.4 refuses to start when `MYSQL_USER="root"` is set, because `root` is a
reserved/privileged MySQL user that cannot be created via the `MYSQL_USER`
environment variable. Its log shows this repeatedly:

```text
[ERROR] [Entrypoint]: MYSQL_USER="root", MYSQL_USER and MYSQL_PASSWORD are for
configuring a regular user and cannot be used for the root user
    Remove MYSQL_USER="root" and use one of the following to control the root
    user password:
    - MYSQL_ROOT_PASSWORD
    - MYSQL_ALLOW_EMPTY_PASSWORD
    - MYSQL_RANDOM_ROOT_PASSWORD
```

The generated `.env` file on the server contains:

```env
MYSQL_ROOT_PASSWORD='<configuredPassword>_root'
MYSQL_DATABASE='torrust'
MYSQL_USER='root'         ← INVALID: MySQL 8.4 rejects this
MYSQL_PASSWORD='<configuredPassword>'
```

### Why This Happened

The environment config (`envs/lxd-local-example.json`) was used as the basis for
the Hetzner deployment and had `"username": "root"` for the MySQL database
config.

The deployer's template rendering service
(`src/application/services/rendering/docker_compose.rs`) passes `username`
directly as `MYSQL_USER`. It is designed to work with a **non-root application
user** — the `root` MySQL user is managed separately via `MYSQL_ROOT_PASSWORD`,
which is auto-derived as `{configured_password}_root`.

Using `"username": "root"` in the environment config is an unsupported
configuration: it produces `MYSQL_USER=root` which MySQL 8.4 rejects.

### Fix Required

The environment config must use a **non-root MySQL username** (e.g. `torrust`).

**Required change in the env config** (`envs/lxd-local-example.json` →
`envs/hetzner-demo.json`):

```json
"database": {
  "driver": "mysql",
  "config": {
    "host": "mysql",
    "port": 3306,
    "database_name": "torrust",
    "username": "torrust",    ← was "root"
    "password": "..."
  }
}
```

After updating the env config:

1. Re-run `release` to regenerate the `.env` and `docker-compose.yml` with the
   correct username.
2. Re-run `run` to start the services.

Note: the environment state must first be reset from `RunFailed` to `Released`
before `release` can be retried. See
[observations.md](../../observations.md#potential-manual-recovery-via-state-snapshot-untested)
for the state recovery procedure.

### Related Deployer Improvement

The deployer should validate that `MYSQL_USER` is not `root` at environment
creation or template rendering time, and return a clear error instead of
silently generating an invalid configuration.
