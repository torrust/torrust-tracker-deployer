# Implement Backup Support

**Issue**: #315
**Parent Epic**: #309 - Add backup support
**Depends On**: #310 - Research database backup strategies (completed)
**Related**: [Roadmap Tasks 7.2-7.5](../../roadmap.md#7-add-backup-support)

## Overview

Implement backup support for Torrust Tracker deployments based on the research
findings from Issue #310. This issue consolidates the original roadmap tasks
7.2-7.5 into a single incremental implementation.

The **maintenance-window hybrid approach** was selected during research as the
recommended solution. This approach uses a Docker container for backup logic
(portable, tested) with minimal host-level orchestration (crontab + script).

## Design Decisions

| Decision              | Choice                      | Rationale                                                                   |
| --------------------- | --------------------------- | --------------------------------------------------------------------------- |
| Container image       | Pre-built on Docker Hub     | Generic tool, stable. No need to rebuild at deploy time.                    |
| Existing environments | Not supported               | Deployer is deploy-only, not a manager. Must destroy and recreate.          |
| E2E testing           | Unit tests + follow-up      | Add unit tests now; backup verification is a follow-up issue.               |
| POC artifacts         | Split: docker/ + templates/ | Container build files to docker/, deployment config to templates/.          |
| Cron validation       | Validate format             | Validate 5-field cron format during config parsing.                         |
| Default template      | Include backup              | Template includes backup section by default (users can remove).             |
| Backup scope          | Complete backup             | No partial options - database + config, all or nothing.                     |
| Backup paths          | Static file                 | File locations are fixed in deployment structure, no dynamic config needed. |

## Goals

- [ ] Add backup service to Docker Compose template (conditional per database)
- [ ] Deploy backup container artifacts (Dockerfile, backup.sh) to VM
- [ ] Install crontab for scheduled maintenance-window backups
- [ ] Integrate backup configuration into environment creation schema
- [ ] Update documentation

## 🏗️ Architecture Requirements

**DDD Layers Involved**:

| Layer          | Module Path                                       | Responsibility                       |
| -------------- | ------------------------------------------------- | ------------------------------------ |
| Domain         | `src/domain/backup/`                              | `BackupConfig`, `CronSchedule` types |
| Application    | `src/application/command_handlers/create/config/` | `BackupSection` DTO, validation      |
| Application    | `src/application/command_handlers/release/steps/` | Backup deployment step               |
| Infrastructure | `src/infrastructure/templating/`                  | Template wrappers for rendering      |

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Domain types in `src/domain/backup/` (like `src/domain/https/`)
- [ ] DTOs in application layer with `to_domain()` conversion
- [ ] Respect dependency flow rules (dependencies flow toward domain)

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions with actionable help messages
- [ ] Domain types are always valid (validated on construction)
- [ ] Static templates registered in `ProjectGenerator`

### Anti-Patterns to Avoid

- ❌ Raw `String` types in domain layer (use validated value objects like `CronSchedule`)
- ❌ Domain layer depending on infrastructure
- ❌ Skipping validation in DTO → domain conversion
- ❌ Adding Ansible `when:` conditionals (use Rust step logic instead)

## Solution Architecture

```text
┌─────────────────────────────────────────────────────────────────────┐
│                           Host VM                                   │
├─────────────────────────────────────────────────────────────────────┤
│  Crontab (3:00 AM daily)                                            │
│       │                                                             │
│       ▼                                                             │
│  maintenance-backup.sh                                              │
│       │                                                             │
│       ├──► docker compose stop tracker                              │
│       │                                                             │
│       ├──► docker compose run --rm backup                           │
│       │         │                                                   │
│       │         ▼                                                   │
│       │    ┌────────────────────────────────────────┐               │
│       │    │ Backup Container                       │               │
│       │    │ - MySQL dump (if enabled)              │               │
│       │    │ - SQLite copy (if enabled)             │               │
│       │    │ - Config tar (if enabled)              │               │
│       │    │ - Compression (gzip)                   │               │
│       │    │ - Retention cleanup                    │               │
│       │    └────────────────────────────────────────┘               │
│       │                                                             │
│       └──► docker compose start tracker                             │
│                                                                     │
│  /opt/torrust/storage/backup/                                       │
│       ├── mysql/       (MySQL dumps)                                │
│       ├── sqlite/      (SQLite copies)                              │
│       └── config/      (Configuration archives)                     │
└─────────────────────────────────────────────────────────────────────┘
```

## POC Artifacts Reference

The research phase (Issue #310) produced production-ready artifacts:

| Artifact                  | POC Location                                                                               | Destination         | Status                    |
| ------------------------- | ------------------------------------------------------------------------------------------ | ------------------- | ------------------------- |
| `backup.sh`               | `docs/research/backup-strategies/solutions/maintenance-window/artifacts/backup-container/` | `docker/backup/`    | ✅ Tested (58 unit tests) |
| `Dockerfile`              | Same directory                                                                             | `docker/backup/`    | ✅ Tested                 |
| `maintenance-backup.sh`   | Same directory                                                                             | `templates/backup/` | ✅ Tested                 |
| `maintenance-backup.cron` | Same directory                                                                             | `templates/backup/` | ✅ Production-ready       |
| `backup-paths.txt`        | `.../backup-storage/etc/`                                                                  | `templates/backup/` | ✅ Static config          |

**Notes**:

- Container build files (`Dockerfile`, `backup.sh`) → `docker/backup/` for Docker Hub publishing
- Deployment config files → `templates/backup/` for VM deployment
- The original POC files remain in docs as reference

## Implementation Plan

This plan follows a **vertical slice approach** where each phase delivers a complete,
testable feature. After each step, perform manual E2E testing to verify the change
works correctly before proceeding.

> **Manual Testing Guide**: See [`docs/e2e-testing/manual/`](../e2e-testing/manual/README.md)
> for the complete guide on manual E2E testing procedures.

### Backup Configuration Reference

The backup configuration is added as a new section in the environment config file.
We need to test both MySQL and SQLite scenarios.

**Backup section (to be added to existing environment configs):**

```json
{
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

**Backup configuration fields:**

| Field            | Type    | Default       | Description                                        |
| ---------------- | ------- | ------------- | -------------------------------------------------- |
| `schedule`       | string  | `"0 3 * * *"` | Cron schedule for backups (default: 3:00 AM daily) |
| `retention_days` | integer | `7`           | Days to keep old backups before deletion           |

**Notes:**

- **Presence enables backup**: If `backup` section exists, backup is enabled. Omit entirely to disable.
- **Complete backup**: Backs up database (MySQL or SQLite based on driver) + all config files
- **All fields have defaults**: You can use `"backup": {}` to enable with all defaults

#### Sample: MySQL with Backup

Based on `envs/manual-test-mysql.json` with backup added:

```json
{
  "environment": {
    "name": "manual-test-mysql-backup"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/testing_rsa",
    "public_key_path": "/path/to/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-mysql-backup"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "mysql",
        "port": 3306,
        "database_name": "torrust_tracker",
        "username": "tracker_user",
        "password": "tracker_password"
      },
      "private": false
    },
    "udp_trackers": [{ "bind_address": "0.0.0.0:6969" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": { "bind_address": "127.0.0.1:1313" }
  },
  "mysql": { "root_password": "root_secret" },
  "prometheus": { "scrape_interval_in_secs": 15 },
  "grafana": { "admin_user": "admin", "admin_password": "admin" },
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

#### Sample: SQLite with Backup

Based on `envs/manual-test-sqlite-backup.json` with backup added:

```json
{
  "environment": {
    "name": "manual-test-sqlite-backup"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/testing_rsa",
    "public_key_path": "/path/to/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-sqlite-backup"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [{ "bind_address": "0.0.0.0:6969" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": { "bind_address": "127.0.0.1:1313" }
  },
  "prometheus": { "scrape_interval_in_secs": 15 },
  "grafana": { "admin_user": "admin", "admin_password": "admin" },
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

---

### Phase 1: Backup Container Image (Docker Hub) - Prerequisite

**Goal**: Publish pre-built backup container image to Docker Hub.

This is a prerequisite that can be done independently. The backup container is a
**generic tool** that receives configuration at runtime (environment variables,
mounted files).

#### Step 1.1: Create backup container directory

```text
docker/backup/
├── Dockerfile           # Backup container image
├── backup.sh            # Main backup script (58 unit tests)
└── README.md            # Container documentation
```

**Tasks**:

- [ ] Create `docker/backup/` directory
- [ ] Copy POC artifacts (`Dockerfile`, `backup.sh`) from research directory
- [ ] Add README.md documenting the container's purpose and usage
- [ ] Verify container builds and runs correctly locally

**Manual Testing**: Build and run container locally with test data.

#### Step 1.1b: Manual E2E Integration Test (Critical)

Before publishing to Docker Hub, validate the container works with the full
docker-compose stack. This mirrors the research POC approach:

**Procedure**:

1. Deploy a manual test environment (e.g., `manual-test-mysql`)
2. Build the backup container image locally
3. Manually add the backup service to the generated `docker-compose.yml`
4. Run `docker compose up` and verify:
   - Backup container starts and connects to MySQL (if applicable)
   - Backup files are created in `/opt/torrust/storage/backup/`
   - Container exits cleanly after backup completes
   - Other services (tracker, prometheus, grafana) remain healthy

**Why this matters**: Catches integration issues (network connectivity, volume
permissions, MySQL connection timing) before committing to Docker Hub publishing.

See [`docs/e2e-testing/manual/`](../e2e-testing/manual/README.md) for full
manual testing procedures.

#### Step 1.2: Create GitHub workflow for publishing

**Tasks**:

- [ ] Create `.github/workflows/backup-container.yaml`
- [ ] Publish to Docker Hub as `torrust/backup`
- [ ] Add Trivy security scanning
- [ ] Tag with version and `latest`

**Manual Testing**: Verify image is published and pullable from Docker Hub.

---

### Phase 2: Backup Service on First Run (Vertical Slice 1)

**Goal**: When a user creates an environment with backup enabled, the backup
container runs once when `docker compose up` starts the stack, creating an
initial backup.

**Value delivered**: User gets an automatic backup on first deployment.

#### Step 2.1: Add backup configuration to create command

Start from the top (command/config layer) and work down:

**Application Layer DTO** (`src/application/command_handlers/create/config/backup.rs`):

```rust
/// Backup configuration DTO (application layer)
#[derive(Debug, Clone, Deserialize)]
pub struct BackupSection {
    #[serde(default = "default_backup_schedule")]
    pub schedule: String,

    #[serde(default = "default_retention_days")]
    pub retention_days: NonZeroU32,
}
```

**Domain Layer** (`src/domain/backup/`):

```rust
/// Domain-level backup configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupConfig {
    schedule: CronSchedule,
    retention_days: NonZeroU32,
}

/// Validated cron schedule (5-field format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CronSchedule(String);
```

**Topology Domain** (`src/domain/topology/`):

The backup service must be registered in the topology module, which defines all
Docker Compose services, networks, and their relationships:

```rust
// src/domain/topology/service.rs - Add Backup variant
pub enum Service {
    Tracker,
    MySQL,
    Prometheus,
    Grafana,
    Caddy,
    Backup,  // NEW: Backup service
}

impl Service {
    pub fn name(&self) -> &'static str {
        match self {
            // ... existing ...
            Service::Backup => "backup",
        }
    }

    pub fn all() -> &'static [Service] {
        &[
            // ... existing ...
            Service::Backup,
        ]
    }
}
```

The `BackupConfig` must also implement topology traits for network derivation:

```rust
// src/domain/backup/config.rs

impl PortDerivation for BackupConfig {
    /// Backup service exposes no ports
    fn derive_ports(&self) -> Vec<PortBinding> {
        vec![]
    }
}

impl NetworkDerivation for BackupConfig {
    /// Backup connects to Database network when MySQL is enabled
    /// (to access MySQL for database dumps)
    fn derive_networks(&self, enabled_services: &EnabledServices) -> Vec<Network> {
        if enabled_services.has(Service::MySQL) {
            vec![Network::Database]
        } else {
            vec![]  // SQLite: no network needed (file access via volume)
        }
    }
}

impl DependencyDerivation for BackupConfig {
    /// Backup depends on MySQL service being healthy when MySQL is enabled
    /// (must wait for MySQL to be ready before attempting database dump)
    fn derive_dependencies(&self, enabled_services: &EnabledServices) -> Vec<ServiceDependency> {
        if enabled_services.has(Service::MySQL) {
            vec![ServiceDependency {
                service: Service::MySQL,
                condition: DependencyCondition::ServiceHealthy,
            }]
        } else {
            vec![]  // SQLite: no external dependencies
        }
    }
}
```

**New types in `src/domain/topology/traits.rs`**:

```rust
/// Condition for service dependency (maps to docker-compose depends_on conditions)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyCondition {
    ServiceHealthy,    // condition: service_healthy
    ServiceStarted,    // condition: service_started
    ServiceCompletedSuccessfully,  // condition: service_completed_successfully
}

impl DependencyCondition {
    pub fn as_docker_compose_value(&self) -> &'static str {
        match self {
            Self::ServiceHealthy => "service_healthy",
            Self::ServiceStarted => "service_started",
            Self::ServiceCompletedSuccessfully => "service_completed_successfully",
        }
    }
}

/// A service dependency with its condition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDependency {
    pub service: Service,
    pub condition: DependencyCondition,
}

/// Trait for deriving service dependencies from configuration
pub trait DependencyDerivation {
    fn derive_dependencies(&self, enabled_services: &EnabledServices) -> Vec<ServiceDependency>;
}
```

**Tasks**:

- [ ] Create `src/domain/backup/` module with `BackupConfig`, `CronSchedule`, error types
- [ ] Register module in `src/domain/mod.rs`
- [ ] Add `Backup` variant to `Service` enum in `src/domain/topology/service.rs`
- [ ] Update `Service::name()` and `Service::all()` methods
- [ ] Update service count in topology tests (5 → 6 services)
- [ ] Implement `PortDerivation` for `BackupConfig` (no ports)
- [ ] Implement `NetworkDerivation` for `BackupConfig` (Database network if MySQL)
- [ ] Add `DependencyCondition`, `ServiceDependency` types to `src/domain/topology/`
- [ ] Add `DependencyDerivation` trait to `src/domain/topology/traits.rs`
- [ ] Implement `DependencyDerivation` for `BackupConfig` (depends on MySQL if enabled)
- [ ] Add `BackupSection` DTO to `src/application/command_handlers/create/config/`
- [ ] Add `backup: Option<BackupSection>` to root config DTO
- [ ] Implement `to_domain()` conversion with cron validation
- [ ] Update JSON schema (`schemas/environment-config.json`)
- [ ] Add unit tests for parsing, validation, network derivation, and dependency derivation

**Manual Testing**: Run `create environment` with backup config, verify:

- No parsing errors
- Network derivation: MySQL enabled → `database_network`, SQLite → no network
- Dependency derivation: MySQL enabled → `depends_on: mysql: service_healthy`, SQLite → no dependencies

#### Step 2.2: Add backup templates and docker-compose integration

Add templates that will be deployed to VM:

**Templates** (`templates/backup/`):

```text
templates/backup/
├── backup.conf.tera    # All config: settings, database, paths file reference
└── backup-paths.txt    # Static list of config files to backup
```

**Note**: For Phase 2, we need both files. The crontab files come in Phase 3.

#### Complete Template Additions

The following sections show the backup configuration templates and the Docker
Compose integration.

##### Backup Configuration Files

**File: `templates/backup/backup.conf.tera`** (all settings in one file):

```bash
# Backup service configuration
# Generated by Torrust Tracker Deployer

# =============================================================================
# General Settings
# =============================================================================

BACKUP_MODE=single
BACKUP_RETENTION_DAYS={{ backup.retention_days }}

# Path to file containing list of files/directories to backup
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt

# =============================================================================
# Database Configuration
# =============================================================================

{%- if mysql %}
DB_TYPE=mysql
DB_HOST=mysql
DB_PORT=3306
DB_USER={{ mysql.user }}
DB_PASSWORD={{ mysql.password }}
DB_NAME={{ mysql.database }}
{%- else %}
DB_TYPE=sqlite
DB_PATH=/data/storage/tracker/lib/tracker.db
{%- endif %}
```

**File: `templates/backup/backup-paths.txt`** (static, files to backup):

```text
/data/storage/tracker/etc/tracker.toml
/data/storage/prometheus/etc/prometheus.yml
/data/storage/grafana/provisioning
/data/storage/caddy/etc/Caddyfile
```

##### Docker Compose Integration

**File: `templates/docker-compose/docker-compose.yml.tera`**

Add before the `networks:` section (after the mysql service):

```yaml
{%- if backup %}

  # Backup service for database and configuration backups
  # Runs once on docker compose up (single mode), exits when complete
  # Scheduled backups are triggered via crontab on the host
  backup:
    <<: *defaults
    image: torrust/backup:latest
    container_name: backup
    restart: "no"  # Override defaults - backup runs once and exits
{%- if backup.dependencies | length > 0 %}
    depends_on:
{%- for dep in backup.dependencies %}
      {{ dep.service }}:
        condition: {{ dep.condition }}
{%- endfor %}
{%- endif %}
    volumes:
      # Backup configuration (sourceable by backup.sh)
      - ./backup/backup.conf:/etc/backup/backup.conf:ro
      - ./backup/backup-paths.txt:/etc/backup/backup-paths.txt:ro
      # Mount storage read-only for config file backup
      - ./storage:/data/storage:ro
      # Mount backup output directory read-write
      - ./storage/backup:/backups
{%- if backup.networks | length > 0 %}
    networks:
{%- for network in backup.networks %}
      - {{ network }}
{%- endfor %}
{%- endif %}
{%- endif %}
```

**Note on domain-driven template rendering**: Both `backup.networks` and
`backup.dependencies` are computed by the Rust topology domain layer - NOT
hardcoded with `{%- if mysql %}` conditionals in templates. This keeps domain
logic out of the presentation layer (Tera templates).

| Property              | Source                                        | Domain Rule                                         |
| --------------------- | --------------------------------------------- | --------------------------------------------------- |
| `backup.networks`     | `NetworkDerivation::derive_networks()`        | Connects to database_network when MySQL is enabled  |
| `backup.dependencies` | `DependencyDerivation::derive_dependencies()` | Depends on mysql service_healthy when MySQL enabled |

> **Technical Debt Note**: The current template
> (`templates/docker-compose/docker-compose.yml.tera`) has hardcoded
> `{%- if mysql %}` conditionals for the tracker service's `depends_on`. This
> is existing technical debt. For the backup service, we should implement it
> correctly with a `DependencyDerivation` trait from the start, following the
> same pattern as `NetworkDerivation`.

**Note**: No environment variables needed in docker-compose - all configuration
is injected via mounted config files. The backup container reads:

- `/etc/backup/backup.conf` - all settings (mode, retention, database)
- `/etc/backup/backup-paths.txt` - files to include in backup (referenced from
  backup.conf)

This approach scales well for future database support (PostgreSQL, etc.) by
adding conditionals to `backup.conf.tera`.

> **Design Rationale**: This configuration pattern aligns with how mature backup
> tools like [restic](https://restic.readthedocs.io/) handle configuration:
>
> - **Environment variables** for sensitive/common settings
> - **Simple text files** for lists (paths, exclude patterns)
> - **Sourceable key=value files** (like our `backup.conf`)
>
> Restic does NOT use a single structured config file (TOML/YAML/JSON) - it uses
> environment variables + simple list files. Our approach mirrors this pattern,
> keeping configuration bash-friendly without requiring complex parsing.

---

**Tasks**:

- [ ] Create `templates/backup/` directory
- [ ] Create `backup.conf.tera` and `backup-paths.txt`
- [ ] Register templates in `ProjectGenerator` (dynamic + static)
- [ ] Add `BackupTemplateWrapper` for template rendering context
- [ ] Add backup service to `docker-compose.yml.tera`
- [ ] Update backup.sh in container to source config file

**Manual Testing**: Deploy environment, verify backup service runs and creates backup files.

#### Step 2.3: Add backup step to Release command

Wire backup into the release workflow:

**Tasks**:

- [ ] Create `release/steps/backup.rs` module
- [ ] Add backup template rendering step (skip if backup not enabled)
- [ ] Add Ansible playbook to deploy `backup-paths.txt` to VM
- [ ] Wire into release workflow

**Manual Testing**: Full deployment with backup enabled:

1. `create environment --env-file ...`
2. `provision`
3. `configure`
4. `release`
5. `run`
6. Verify: SSH to VM, check `/opt/torrust/storage/backup/` contains backup files

#### Step 2.4: Update create template command

**Tasks**:

- [ ] Update `create template` to include backup section in generated config
- [ ] Backup enabled by default (users can remove if not needed)

**Manual Testing**: Run `create template`, verify backup section in output.

---

### Phase 3: Scheduled Backups via Crontab (Vertical Slice 2)

**Goal**: Add crontab configuration so backups run automatically on schedule.

**Value delivered**: User gets scheduled automated backups (e.g., daily at 3 AM).

#### Step 3.1: Add crontab templates

Add host-level scheduling files:

```text
templates/backup/
├── backup-paths.txt              # (from Phase 2)
├── maintenance-backup.sh         # Host script: stop tracker → backup → start
└── maintenance-backup.cron.tera  # Crontab with configurable schedule
```

**Tasks**:

- [ ] Copy `maintenance-backup.sh` from POC to templates
- [ ] Create `maintenance-backup.cron.tera` with schedule variable
- [ ] Register templates in `ProjectGenerator`

**Manual Testing**: Verify templates render correctly with test schedule.

#### Step 3.2: Add crontab installation playbook

```yaml
# templates/ansible/install-backup-crontab.yml
---
- name: Install backup crontab
  hosts: target_hosts
  become: true
  vars_files:
    - variables.yml
  tasks:
    - name: Copy maintenance backup script
      copy:
        src: "{{ backup_source_dir }}/maintenance-backup.sh"
        dest: "/usr/local/bin/maintenance-backup.sh"
        mode: "0755"

    - name: Install backup crontab
      copy:
        src: "{{ backup_source_dir }}/maintenance-backup.cron"
        dest: /etc/cron.d/tracker-backup
        mode: "0644"
```

**Tasks**:

- [ ] Create `install-backup-crontab.yml` playbook
- [ ] Register playbook as static template

**Manual Testing**: Run playbook manually, verify files copied to correct locations.

#### Step 3.3: Wire crontab into Configure command

**Tasks**:

- [ ] Add crontab installation to configure workflow (when backup enabled)
- [ ] Ensure proper execution order (after Docker setup)

**Manual Testing**: Full deployment, verify:

1. `/etc/cron.d/tracker-backup` exists with correct schedule
2. `/usr/local/bin/maintenance-backup.sh` is executable
3. Wait for cron time or trigger manually, verify backup created

#### Step 3.4: Update docker-compose to use profiles

Now that crontab handles scheduling, backup container should only run on-demand:

**Tasks**:

- [ ] Add `profiles: [backup]` to backup service in docker-compose template
- [ ] Update `maintenance-backup.sh` to use `docker compose run --rm backup`

**Manual Testing**: Verify backup only runs via cron, not on `docker compose up`.

---

### Phase 4: Documentation and Final Testing

**Goal**: Complete documentation and comprehensive testing.

> **Testing Reference**: Follow the procedures in [`docs/e2e-testing/manual/`](../e2e-testing/manual/README.md)
> for comprehensive manual verification.

#### Step 4.1: Documentation

**Tasks**:

- [ ] Add backup section to user guide (`docs/user-guide/backup.md`)
- [ ] Update environment config documentation
- [ ] Add troubleshooting section for common backup issues
- [ ] Update `docs/console-commands.md`

#### Step 4.2: E2E Test Updates

**Tasks**:

- [ ] Add backup configuration to E2E test environment configs
- [ ] Verify E2E tests pass with backup enabled
- [ ] Document manual verification steps for backup testing

## Progress Tracking

### Phase 1: Backup Container Image (Prerequisite)

- [ ] Step 1.1: Create backup container directory (`docker/backup/`)
- [ ] Step 1.2: Create GitHub workflow for publishing

### Phase 2: Backup Service on First Run

- [ ] Step 2.1: Add backup configuration to create command
- [ ] Step 2.2: Add backup templates and docker-compose integration
- [ ] Step 2.3: Add backup step to Release command
- [ ] Step 2.4: Update create template command

### Phase 3: Scheduled Backups via Crontab

- [ ] Step 3.1: Add crontab templates
- [ ] Step 3.2: Add crontab installation playbook
- [ ] Step 3.3: Wire crontab into Configure command
- [ ] Step 3.4: Update docker-compose to use profiles

### Phase 4: Documentation and Final Testing

- [ ] Step 4.1: Documentation
- [ ] Step 4.2: E2E Test Updates

## Scope Boundaries

### In Scope

- ✅ MySQL database backups (mysqldump)
- ✅ SQLite database backups (file copy)
- ✅ Configuration file backups (tar archive)
- ✅ Scheduled backups via crontab
- ✅ Backup retention (configurable days)
- ✅ Compression (gzip)

### Out of Scope

- ❌ Restore functionality (future enhancement)
- ❌ Remote backup destinations (S3, etc.) - user mounts volumes
- ❌ Backup encryption (future enhancement)
- ❌ Backup verification/testing (future enhancement)
- ❌ Backup notifications/alerts (future enhancement)

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Users can enable backup in environment configuration
- [ ] Backup container is deployed with docker-compose stack
- [ ] Crontab runs daily backups at configured time
- [ ] MySQL and SQLite databases are backed up correctly
- [ ] Configuration files are archived
- [ ] Old backups are cleaned up per retention policy
- [ ] Documentation covers backup usage and configuration
- [ ] All E2E tests pass

## Technical Notes

### Lessons Learned from POC

Key insights from Issue #310 research:

1. **Path translation**: Host path `/opt/torrust/storage/...` vs container path
   `/data/storage/...` - templates must use correct perspective

2. **Template conditionals**: Database type affects:
   - Environment variables (MySQL credentials vs SQLite path)
   - Network configuration (MySQL needs database_network)
   - Dependencies (MySQL vs tracker healthcheck)

3. **Container exit behavior**: `BACKUP_MODE=single` means container exits
   after backup. Use `profiles: [backup]` to prevent auto-start.

4. **Cron pre-installed**: Ubuntu cloud images include cron, no installation
   needed.

5. **Log rotation**: Add logrotate config for `/var/log/tracker-backup.log`

### Related Files to Modify

| File                                                   | Change                           |
| ------------------------------------------------------ | -------------------------------- |
| `docker/backup/`                                       | New: container build directory   |
| `.github/workflows/backup-container.yaml`              | New: publish container to Hub    |
| `templates/backup/`                                    | New: deployment config templates |
| `templates/docker-compose/docker-compose.yml.tera`     | Add backup service               |
| `templates/docker-compose/.env.tera`                   | Add backup env vars              |
| `src/domain/backup/`                                   | New: domain types (like https/)  |
| `src/domain/mod.rs`                                    | Register backup module           |
| `src/application/command_handlers/create/config/`      | Add BackupSection DTO            |
| `schemas/environment-config.json`                      | Add backup schema                |
| `src/application/command_handlers/release/steps/`      | Add backup.rs                    |
| `src/application/command_handlers/release/workflow.rs` | Add backup step                  |
| `templates/ansible/`                                   | Add backup playbooks             |

## Related Documentation

- [Research Conclusions](../research/backup-strategies/conclusions.md)
- [Maintenance Window Solution](../research/backup-strategies/solutions/maintenance-window/)
- [Lessons Learned](../research/backup-strategies/solutions/maintenance-window/implementation-recommendations.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Template System Architecture](../contributing/templates/template-system-architecture.md)
- [Manual E2E Testing Guide](../e2e-testing/manual/README.md)
