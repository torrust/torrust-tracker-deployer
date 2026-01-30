# Container Backup Architectures

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Context

The Torrust Tracker Deployer uses Docker Compose to orchestrate services. This provides:

- Portability (easy to move entire stack to another VM)
- Independent service updates
- Isolation between services

When implementing backups, we need to decide **where** the backup logic lives within this architecture.

## Approaches

### 1. Host-Level Crontab (Current Torrust Live Demo)

```text
┌────────────────────────────────────────────────────┐
│                      HOST                          │
│  ┌─────────────────────────────────────────────┐   │
│  │              crontab + script               │   │
│  │  sqlite3 /path/to/db ".backup ..."          │   │
│  └─────────────────────────────────────────────┘   │
│                        │                           │
│                        ▼                           │
│  ┌──────────────────────────────────────────────┐  │
│  │           Docker Compose Stack               │  │
│  │  ┌────────────┐  ┌────────────┐              │  │
│  │  │  Tracker   │  │   MySQL    │              │  │
│  │  │  (SQLite)  │  │            │              │  │
│  │  └─────┬──────┘  └─────┬──────┘              │  │
│  │        │               │                     │  │
│  │        ▼               ▼                     │  │
│  │    [volume]        [volume]                  │  │
│  └──────────────────────────────────────────────┘  │
│        ▲                   ▲                       │
│        │                   │                       │
│        └───── accessed via bind mount ─────────────┘
└────────────────────────────────────────────────────┘
```

**How it works:**

- Crontab runs on the host
- Backup scripts access database files via bind mounts
- Tools (sqlite3, mysqldump) installed on host

**Pros:**

| Advantage          | Description                                |
| ------------------ | ------------------------------------------ |
| Simple             | No additional containers, familiar crontab |
| Direct access      | Can access all volumes from host           |
| Central management | One place for all backup schedules         |

**Cons:**

| Disadvantage       | Description                                       |
| ------------------ | ------------------------------------------------- |
| Host dependency    | Backups tied to host configuration                |
| Non-portable       | Must reconfigure crontab when moving stack        |
| Tool installation  | Must install sqlite3, mysqldump, etc. on host     |
| Not "in the stack" | Backup config is separate from docker-compose.yml |

---

### 2. Centralized Backup Service

```text
┌──────────────────────────────────────────────────────┐
│              Docker Compose Stack                    │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │            backup-service                      │  │
│  │  (cron + sqlite3 + mysqldump + pg_dump + ...)  │  │
│  └───────────┬──────────────────┬─────────────────┘  │
│              │                  │                    │
│              ▼                  ▼                    │
│  ┌────────────────┐  ┌────────────────┐              │
│  │    Tracker     │  │     MySQL      │              │
│  │   (SQLite)     │  │                │              │
│  └───────┬────────┘  └───────┬────────┘              │
│          │                   │                       │
│          ▼                   ▼                       │
│      [tracker-data]     [mysql-data]                 │
│          ▲                   ▲                       │
│          │                   │                       │
│          └─── mounted to backup-service ─────────────┘
│                                                      │
│      [backup-storage] ◄─── backup output             │
└──────────────────────────────────────────────────────┘
```

**How it works:**

- Dedicated container runs cron and backup scripts
- Container has access to all data volumes
- All backup tools installed in one image

**Pros:**

| Advantage       | Description                         |
| --------------- | ----------------------------------- |
| Portable        | Backups are part of the stack       |
| Self-contained  | Everything in docker-compose.yml    |
| Single schedule | One place to manage all backup jobs |

**Cons:**

| Disadvantage   | Description                                           |
| -------------- | ----------------------------------------------------- |
| Kitchen sink   | Image grows with every database type                  |
| Tight coupling | Must update backup container when adding new services |
| Monolithic     | All backup tools in one image                         |
| Volume sprawl  | Must mount many volumes to one container              |

**Example docker-compose.yml:**

```yaml
services:
  tracker:
    image: torrust/tracker
    volumes:
      - tracker-data:/var/lib/torrust

  mysql:
    image: mysql:8
    volumes:
      - mysql-data:/var/lib/mysql

  backup:
    image: custom/backup-service # Contains cron, sqlite3, mysqldump
    volumes:
      - tracker-data:/data/tracker:ro
      - mysql-data:/data/mysql:ro
      - backup-storage:/backups
    environment:
      - BACKUP_SCHEDULE=0 2 * * *
```

---

### 3. Sidecar Pattern (Per-Service Backup)

```text
┌──────────────────────────────────────────────────────┐
│              Docker Compose Stack                    │
│                                                      │
│  ┌─────────────────────────┐  ┌─────────────────────┐│
│  │ Tracker Service Group   │  │ MySQL Service Group ││
│  │                         │  │                     ││
│  │ ┌─────────┐ ┌─────────┐ │  │┌────────┐ ┌────────┐││
│  │ │ Tracker │ │ Backup  │ │  ││ MySQL  │ │ Backup │││
│  │ │         │ │ Sidecar │ │  ││        │ │Sidecar │││
│  │ │         │ │(sqlite3)│ │  ││        │ │(mysql- │││
│  │ │         │ │         │ │  ││        │ │ dump)  │││
│  │ └────┬────┘ └────┬────┘ │  │└───┬────┘ └───┬────┘││
│  │      │           │      │  │    │          │     ││
│  │      ▼           ▼      │  │    ▼          ▼     ││
│  │   [tracker-data]        │  │  [mysql-data]       ││
│  │      ▲                  │  │    ▲                ││
│  │      └── shared ────────┘  │    └── shared ──────┘│
│  └─────────────────────────┘  └─────────────────────┘│
│                                                      │
│      [backup-storage] ◄─── all sidecars write here   │
└──────────────────────────────────────────────────────┘
```

**How it works:**

- Each service has a companion "sidecar" container
- Sidecar only has tools for that specific database
- Shares data volume with the main service
- Each sidecar runs its own cron

**Pros:**

| Advantage              | Description                              |
| ---------------------- | ---------------------------------------- |
| Separation of concerns | Each sidecar is purpose-built            |
| Lightweight images     | Only needed tools per sidecar            |
| Independent scaling    | Can have different schedules per service |
| Modular                | Adding new service = adding new sidecar  |

**Cons:**

| Disadvantage       | Description                                     |
| ------------------ | ----------------------------------------------- |
| Container count    | More containers to manage                       |
| Distributed config | Backup schedules spread across sidecars         |
| Resource overhead  | Each sidecar has its own cron process           |
| Coordination       | Harder to orchestrate (e.g., pause all backups) |

**Example docker-compose.yml:**

```yaml
services:
  tracker:
    image: torrust/tracker
    volumes:
      - tracker-data:/var/lib/torrust

  tracker-backup:
    image: torrust/sqlite-backup # Minimal: alpine + sqlite3 + cron
    volumes:
      - tracker-data:/data:ro
      - backup-storage:/backups
    environment:
      - BACKUP_SCHEDULE=0 2 * * *
      - BACKUP_PREFIX=tracker

  mysql:
    image: mysql:8
    volumes:
      - mysql-data:/var/lib/mysql

  mysql-backup:
    image: torrust/mysql-backup # Minimal: alpine + mysql-client + cron
    volumes:
      - backup-storage:/backups
    environment:
      - MYSQL_HOST=mysql
      - BACKUP_SCHEDULE=0 3 * * *
      - BACKUP_PREFIX=mysql
```

---

### 4. Backup Orchestrator Pattern

```text
┌──────────────────────────────────────────────────────┐
│              Docker Compose Stack                    │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │         backup-orchestrator                    │  │
│  │  (cron + script that triggers backups)         │  │
│  │  - No database tools installed                 │  │
│  │  - Just coordinates timing                     │  │
│  └───────────┬──────────────────┬─────────────────┘  │
│              │ docker exec      │ docker exec        │
│              ▼                  ▼                    │
│  ┌────────────────┐  ┌────────────────┐              │
│  │    Tracker     │  │     MySQL      │              │
│  │   (SQLite)     │  │                │              │
│  │  + sqlite3 CLI │  │  + mysqldump   │              │
│  └───────┬────────┘  └───────┬────────┘              │
│          │                   │                       │
│          ▼                   ▼                       │
│      [tracker-data]     [mysql-data]                 │
│                                                      │
│      [backup-storage] ◄─── shared backup volume      │
└──────────────────────────────────────────────────────┘
```

**How it works:**

- Thin orchestrator container just manages schedules
- Uses `docker exec` to run backup commands inside service containers
- Each service image includes its backup tools

**Pros:**

| Advantage                       | Description                             |
| ------------------------------- | --------------------------------------- |
| Central scheduling              | One place for all backup timing         |
| No volume mounting              | Tools run inside service container      |
| Services own their backup logic | Each service knows how to backup itself |

**Cons:**

| Disadvantage            | Description                              |
| ----------------------- | ---------------------------------------- |
| Docker socket access    | Orchestrator needs access to Docker API  |
| Security concern        | Container with Docker access is powerful |
| Image bloat             | Service images must include backup tools |
| Requires multi-process? | Not really - exec runs separately        |

---

### 5. External Backup Tool (Restic, Borg, etc.)

```text
┌──────────────────────────────────────────────────────┐
│              Docker Compose Stack                    │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │           restic-backup                        │  │
│  │  (restic + cron + database pre-backup hooks)   │  │
│  └───────────┬──────────────────┬─────────────────┘  │
│              │                  │                    │
│              ▼                  ▼                    │
│  ┌────────────────┐  ┌────────────────┐              │
│  │    Tracker     │  │     MySQL      │              │
│  └───────┬────────┘  └───────┬────────┘              │
│          │                   │                       │
│          ▼                   ▼                       │
│      [tracker-data]     [mysql-data]                 │
│          ▲                   ▲                       │
│          └─── accessed by restic ────────────────────┘
│                                                      │
│      → Remote storage (S3, B2, SFTP, etc.)           │
└──────────────────────────────────────────────────────┘
```

**How it works:**

- Use established backup tool (Restic, Borg, Duplicati)
- Container runs the backup tool + database-specific pre-hooks
- Supports remote storage, encryption, deduplication

**Pros:**

| Advantage      | Description                               |
| -------------- | ----------------------------------------- |
| Battle-tested  | Mature backup tools with many features    |
| Remote storage | Built-in support for S3, B2, SFTP         |
| Deduplication  | Efficient storage for incremental backups |
| Encryption     | Built-in encryption at rest               |

**Cons:**

| Disadvantage     | Description                               |
| ---------------- | ----------------------------------------- |
| Learning curve   | New tool to learn and configure           |
| Pre-hooks needed | Still need database-specific dump scripts |
| Overkill?        | May be more than needed for simple setups |

---

## Comparison Matrix

| Aspect          | Host Crontab   | Centralized | Sidecar          | Orchestrator     | External Tool |
| --------------- | -------------- | ----------- | ---------------- | ---------------- | ------------- |
| Portability     | ❌ Poor        | ✅ Good     | ✅ Good          | ⚠️ Medium        | ✅ Good       |
| Simplicity      | ✅ Simple      | ⚠️ Medium   | ⚠️ Medium        | ⚠️ Medium        | ❌ Complex    |
| Container count | ➖ None        | +1          | +N (per service) | +1               | +1            |
| Image size      | N/A            | Large       | Small            | Small            | Medium        |
| Security        | ⚠️ Host access | ✅ Good     | ✅ Good          | ❌ Docker socket | ✅ Good       |
| Flexibility     | ⚠️ Limited     | ⚠️ Limited  | ✅ High          | ⚠️ Medium        | ✅ High       |
| Tool coupling   | Host tools     | All-in-one  | Per-service      | In-service       | Pre-hooks     |

## Key Questions to Answer

1. **How portable does the solution need to be?**
   - If backups should "just work" when moving the stack → avoid host crontab

2. **How many database types will we support?**
   - Just SQLite → any approach works
   - SQLite + MySQL + PostgreSQL → sidecar or external tool scales better

3. **Do we want backups in docker-compose.yml?**
   - Yes → centralized, sidecar, or orchestrator
   - No → host crontab or external system

4. **Is Docker socket access acceptable?**
   - If no → avoid orchestrator pattern

5. **Do we need remote/cloud backup storage?**
   - If yes → external tool (Restic, Borg) is attractive

## Recommendation (Preliminary)

For the Torrust Tracker Deployer, the **sidecar pattern** appears most aligned with the project's goals:

- **Modular**: Each database type has its own lightweight backup image
- **Portable**: Everything is in docker-compose.yml
- **Scalable**: Adding PostgreSQL = adding a PostgreSQL backup sidecar
- **No Docker socket**: More secure
- **Single-process containers**: Each sidecar runs one thing (cron → backup script)

However, this is preliminary. We should evaluate:

- Existing sidecar backup images (e.g., `prodrigestivill/postgres-backup-local`)
- Whether the added container count is acceptable
- User preference for centralized vs distributed backup config

## Open Questions

- [ ] Should backup schedules be configurable per-environment?
- [ ] Where should backups be stored? (local volume, remote, both)
- [ ] Should we support backup rotation/retention policies?
- [ ] How do we handle backup verification in containers?
- [ ] What about backup monitoring/alerting?

## References

- [Docker Compose Sidecar Pattern](https://docs.docker.com/compose/how-tos/multiple-compose-files/)
- [Restic Backup Tool](https://restic.net/)
- [Borg Backup](https://www.borgbackup.org/)
- [postgres-backup-local (example sidecar)](https://github.com/prodrigestivill/docker-postgres-backup-local)
