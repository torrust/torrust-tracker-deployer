# Decision: Bind Mount Standardization for Docker Compose

## Status

Accepted

## Date

2026-01-24

## Context

The Docker Compose template currently uses a mix of named volumes and bind mounts for persistent data:

```yaml
# Bind mounts (host path → container path) - CURRENT PATTERN
- ./storage/tracker/lib:/var/lib/torrust/tracker:Z

# Named volumes (volume name → container path) - PROBLEMATIC
- caddy_data:/data
- grafana_data:/var/lib/grafana
- mysql_data:/var/lib/mysql
```

This inconsistency creates several problems:

### 1. Observability

- Named volumes hide data in `/var/lib/docker/volumes/` - not obvious to users
- Users cannot easily see where persistent data is stored
- File system tools (ls, du, find) don't work directly on named volume data

### 2. Backup Complexity

- Named volumes require `docker volume` commands or finding the internal path
- No single command can back up all data
- Docker-specific tooling is required
- Standard backup scripts don't work without modification

### 3. Restore Complexity

- Restoring named volumes requires Docker volume recreation
- Cannot simply copy files to restore data
- Migration between hosts requires Docker volume export/import

### 4. Inconsistency

- Some services use bind mounts, others use named volumes
- Different patterns for different services create cognitive overhead
- No predictable directory structure

### 5. Portability Limitations

- Named volumes cannot be moved between hosts by copying files
- Docker volume export/import dance is required
- Tied to Docker's internal storage format

### 6. Debugging & Troubleshooting Difficulties

- Cannot directly inspect files without entering containers
- Checking file permissions, ownership, disk usage is difficult
- Cannot modify config files directly for debugging
- Log files not accessible without `docker logs`

### 7. Development Experience

- Cannot easily reset state by deleting directories
- Cannot pre-populate data for testing scenarios
- IDE file watchers cannot observe changes in named volumes

### 8. Deployment Architecture Complexity

- Named volumes require a top-level `volumes:` section in docker-compose.yml
- Must derive which volumes are required based on enabled services
- Ansible must manage both directories and Docker volumes

### 9. Security Visibility

- File permissions are hidden inside Docker volume directories
- SELinux labels cannot be applied consistently
- Data locations are not transparent to users

## Decision

Standardize on **bind mounts exclusively** for all persistent data in Docker Compose deployments.

All persistent data will be stored under `./storage/{service}/`:

| Service    | Bind Mount                                 | Data Location              |
| ---------- | ------------------------------------------ | -------------------------- |
| Tracker    | `./storage/tracker/lib:/var/lib/torrust`   | `./storage/tracker/lib`    |
| Tracker    | `./storage/tracker/log:/var/log/torrust`   | `./storage/tracker/log`    |
| Tracker    | `./storage/tracker/etc:/etc/torrust`       | `./storage/tracker/etc`    |
| Caddy      | `./storage/caddy/data:/data`               | `./storage/caddy/data`     |
| Caddy      | `./storage/caddy/config:/config`           | `./storage/caddy/config`   |
| Caddy      | `./storage/caddy/etc/Caddyfile:/etc/...`   | `./storage/caddy/etc`      |
| Grafana    | `./storage/grafana/data:/var/lib/grafana`  | `./storage/grafana/data`   |
| Prometheus | `./storage/prometheus/etc:/etc/prometheus` | `./storage/prometheus/etc` |
| MySQL      | `./storage/mysql/data:/var/lib/mysql`      | `./storage/mysql/data`     |

Mount options:

- `:ro` - Read-only for config files that shouldn't be modified
- `:Z` - SELinux private relabeling for writable data directories

## Consequences

### Positive

- **Simplified backup**: Single command `cp -r ./storage/ backup/` backs up everything
- **Easy restore**: Copy files back to `./storage/` to restore
- **Full observability**: All persistent data is visible at predictable paths
- **Consistent pattern**: Same approach for all services
- **Portable**: Data directory can be moved between hosts by copying
- **Easy debugging**: Direct file inspection without entering containers
- **Better development experience**: Reset state by deleting directories
- **Simpler deployment**: No top-level `volumes:` section needed in docker-compose.yml
- **Security visibility**: File permissions are visible and controllable

### Negative

- **Explicit directory creation**: Directories must be created with correct permissions before container start
- **Permission management**: Must ensure correct ownership for non-root containers (Grafana: 472:472, MySQL: 999:999)
- **SELinux handling**: Must apply `:Z` suffix for writable directories on SELinux systems
- **Additional Ansible playbooks**: Need playbooks to create directories with correct ownership

### Risks

- **Breaking change**: Existing deployments using named volumes will need migration
- **Permission errors**: Incorrect directory ownership will prevent containers from starting

### Mitigation

- Create new Ansible playbooks for Grafana and MySQL directory creation with correct ownership
- Document migration path for existing deployments
- E2E tests will verify correct permission handling

## Alternatives Considered

### 1. Named Volumes Only

**Rejected** because:

- Data is hidden in `/var/lib/docker/volumes/`
- Backup requires Docker-specific commands
- Inconsistent with our observability principles
- Users cannot easily access or inspect persistent data

### 2. Mixed Approach (Current State)

**Rejected** because:

- Inconsistency creates confusion and maintenance burden
- Different services have different storage patterns
- No single backup strategy works for all services
- Cognitive overhead for developers and operators

### 3. Docker Volume Plugins

**Rejected** because:

- Overkill for single-VM deployments
- Adds complexity and external dependencies
- Our deployment model is single-VM, not distributed
- Standard bind mounts meet all our requirements

## Related Decisions

- [Grafana Integration Pattern](./grafana-integration-pattern.md) - This ADR supersedes the volume recommendations in that decision
- [Configuration Directories as Secrets](./configuration-directories-as-secrets.md) - Related security considerations for data directories

## References

- [Docker Compose bind mounts documentation](https://docs.docker.com/compose/compose-file/07-volumes/)
- [SELinux and Docker](https://docs.docker.com/storage/bind-mounts/#configure-the-selinux-label)
- [Refactoring Plan: Docker Compose Topology Domain Model](../refactors/plans/docker-compose-topology-domain-model.md)
