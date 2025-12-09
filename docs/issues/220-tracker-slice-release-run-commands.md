# Tracker Slice - Release and Run Commands

**Issue**: #220
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)

## Overview

This task replaces the demo nginx service with the actual Torrust Tracker in the docker-compose stack. It implements the complete workflow for deploying a working BitTorrent tracker with SQLite database, including storage directory creation, database initialization, tracker configuration, and docker-compose integration.

## Goals

- [ ] Create storage directory structure for Tracker on provisioned VM
- [ ] Initialize SQLite database for Tracker
- [ ] Add `.env` template for docker-compose environment variables
- [ ] Add tracker configuration template (`tracker.toml`)
- [ ] Replace nginx with Torrust Tracker service in docker-compose
- [ ] Extend environment configuration to support tracker settings
- [ ] Deploy and verify tracker is running and accessible

## 🏗️ Architecture Requirements

**DDD Layers**: Infrastructure + Application
**Module Paths**:

- `src/infrastructure/templating/ansible/` - Ansible playbook and variable support (renamed from `external_tools`)
- `src/infrastructure/templating/docker_compose/` - Docker Compose template rendering (renamed from `external_tools`)
- `src/infrastructure/templating/tracker/` - New module for tracker-specific templates
- `src/domain/config/environment/` - Environment configuration schema

**Pattern**: Template System with Project Generator pattern

### Module Structure Requirements

- [ ] Follow template system architecture (see [docs/technical/template-system-architecture.md](../docs/technical/template-system-architecture.md))
- [ ] Use Project Generator pattern for tracker templates
- [ ] Register static templates explicitly in renderer
- [ ] Use `.tera` extension for dynamic templates

### Architectural Constraints

- [ ] Tracker templates isolated in separate module from docker-compose
- [ ] Each service (tracker, docker-compose) has its own renderer
- [ ] Environment configuration drives all template variable substitution
- [ ] Static Ansible playbooks reference centralized `variables.yml` (`templates/ansible/variables.yml.tera`)

### Anti-Patterns to Avoid

- ❌ Mixing tracker templates with docker-compose templates
- ❌ Hardcoding values that should be configurable
- ❌ Creating `.tera` templates for static files
- ❌ Forgetting to register static templates in renderer

## Implementation Strategy

The implementation follows an **incremental, testable approach** where each step can be manually verified before proceeding to the next. This ensures we always have a working deployment and can catch issues early.

### Key Principles

1. **Incremental Development**: Add one capability at a time, test it works
2. **Hardcoded First**: Start with fixed values in templates to validate pipeline
3. **Progressive Configuration**: Gradually expose variables to environment config
4. **Manual Verification**: Test each step manually before moving forward
5. **Service Isolation**: Treat tracker as independent service with own templates

## Specifications

### Storage Directory Structure

The Tracker requires the following directory structure on the provisioned VM:

```text
storage/
└── tracker/
    ├── etc/           # Configuration files (tracker.toml)
    ├── lib/           # Application data
    │   └── database/  # SQLite database files
    └── log/           # Log files
```

**Location on VM**: `/opt/torrust/storage/tracker/`

**Creation Method**: New Ansible playbook

### SQLite Database Initialization

**Database Name**: `tracker.db` (initially hardcoded, later configurable)
**Location**: `/opt/torrust/storage/tracker/lib/database/tracker.db`

**Initialization Command**:

```bash
touch ./storage/tracker/lib/database/tracker.db
echo ";" | sqlite3 ./storage/tracker/lib/database/tracker.db
```

**Implementation**: Ansible playbook with conditional check (only create if doesn't exist)

### Docker Compose Environment File (`.env`)

**Template**: `templates/docker-compose/env.tera`
**Location on VM**: `/opt/torrust/docker-compose/.env`

**Initial Content** (tracker variables only):

```bash
# Tracker Configuration
TORRUST_TRACKER_CONFIG_TOML_PATH='/etc/torrust/tracker/tracker.toml'
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN={{ tracker_api_admin_token }}
```

**Notes**:

- `TORRUST_TRACKER_CONFIG_TOML_PATH` explicitly points to our deployed config file (prevents issues if default location changes)
- `tracker_api_admin_token` comes from environment configuration (user-provided during environment creation)
- Other services (Index, Grafana) omitted for this slice

### Tracker Configuration Template

**Template**: `templates/tracker/tracker.toml.tera`
**Location on VM**: `/opt/torrust/storage/tracker/etc/tracker.toml`

**Initial Content** (hardcoded, minimal configuration):

```toml
[metadata]
app = "torrust-tracker"
purpose = "configuration"
schema_version = "2.0.0"

[logging]
threshold = "info"

[core]
listed = false
private = false

[core.tracker_policy]
persistent_torrent_completed_stat = true

[core.announce_policy]
interval = 300
interval_min = 300

[core.net]
on_reverse_proxy = true

[core.database]
driver = "sqlite3"
path = "/var/lib/torrust/tracker/database/sqlite3.db"

[[udp_trackers]]
bind_address = "0.0.0.0:6868"

[[udp_trackers]]
bind_address = "0.0.0.0:6969"

[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:1212"
```

**Progressive Configuration Exposure**:

- **Step 1**: All values hardcoded (validates pipeline)
- **Step 2**: Expose tracker list (`udp_trackers`, `http_trackers`) and mode (`private` flag)
- **Future**: Expose database driver, intervals, logging level, etc.

### Docker Compose Service Definition

**Template**: `templates/docker-compose/docker-compose.yml` (existing file, replace content)

**Tracker Service Configuration**:

```yaml
services:
  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    tty: true
    restart: unless-stopped
    environment:
      - USER_ID=1000
      - TORRUST_TRACKER_CONFIG_TOML=${TORRUST_TRACKER_CONFIG_TOML}
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=sqlite3
      - TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN}
    networks:
      - backend_network
    ports:
      - 6868:6868/udp
      - 6969:6969/udp
      - 7070:7070
      - 1212:1212
    volumes:
      - ./storage/tracker/lib:/var/lib/torrust/tracker:Z
      - ./storage/tracker/log:/var/log/torrust/tracker:Z
      - ./storage/tracker/etc:/etc/torrust/tracker:Z
    logging:
      options:
        max-size: "10m"
        max-file: "10"

networks:
  backend_network: {}
```

**Reference**: [torrust-demo compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml)

### Environment Configuration Schema

**File**: `src/domain/config/environment/schema.rs`

**New Fields** (add to environment config):

```rust
/// Tracker deployment configuration
#[serde(skip_serializing_if = "Option::is_none")]
pub tracker: Option<TrackerConfig>,

// Separate struct for tracker configuration
// Structure mirrors the real tracker config but only includes user-configurable fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerConfig {
    /// Core tracker configuration
    pub core: TrackerCoreConfig,

    /// UDP tracker instances
    pub udp_trackers: Vec<UdpTrackerConfig>,

    /// HTTP tracker instances
    pub http_trackers: Vec<HttpTrackerConfig>,

    /// HTTP API configuration
    pub http_api: HttpApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerCoreConfig {
    /// Database file name (e.g., "tracker.db")
    pub database_name: String,

    /// Tracker mode: true for private tracker, false for public
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:7070")
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiConfig {
    /// Admin access token for HTTP API authentication
    pub admin_token: String,
}
```

**Example environment.json**:

```json
{
  "name": "my-tracker-env",
  "provider": "lxd",
  "tracker": {
    "core": {
      "database_name": "tracker.db",
      "private": false
    },
    "udp_trackers": [
      { "bind_address": "0.0.0.0:6868" },
      { "bind_address": "0.0.0.0:6969" }
    ],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "admin_token": "MyAccessToken"
    }
  }
}
```

## Implementation Plan

> **Implementation Order: Incremental + Testable**
>
> Each phase adds one capability and is manually testable before proceeding. This ensures we always have a working deployment and can catch issues early.

### Implementation Progress Tracking

Track completion status for each phase:

- [x] **Phase 0**: Rename Module for Clarity (30 mins) - ✅ Completed in commit 2d5625c
- [x] **Phase 1**: Create Storage Directories (30 mins) - ✅ Completed
- [x] **Phase 2**: Initialize SQLite Database (45 mins) - ✅ Completed
- [x] **Phase 3**: Add Docker Compose `.env` File (1 hour) - ✅ Completed
- [x] **Phase 4**: Add Tracker Configuration Template (1.5 hours) - ✅ Completed in commit 659e407
- [x] **Phase 5**: Replace Docker Compose Service (1 hour) - ✅ Completed in commit 59e3762
- [x] **Phase 6**: Add Environment Configuration Support (2 hours) - ✅ Completed in commit 52d7c2a
- [x] **Phase 7**: Configure Firewall for Tracker Ports (1 hour) - ✅ Completed (infrastructure: 6939553, wiring: TBD)
- [ ] **Phase 8**: Update E2E Tests for Tracker Validation (1.5 hours) - 🔨 In Progress

**Total Estimated Time**: ~10 hours

### Manual Testing Workflow

Each phase should be tested using the following end-to-end workflow. The specific verification steps for each phase are documented in the phase sections below.

#### Prerequisites

```bash
# Ensure you have a clean test environment
rm -rf build/test-env
rm -rf envs/test-env.json
```

#### Complete E2E Test Flow

**Recommended Workflow**: Use `create template` to generate environment configuration, then customize it with your values. This ensures proper structure and provides helpful placeholders.

```bash
# RECOMMENDED: Generate environment template first
cargo run -- create template --provider lxd > envs/test-env.json

# Edit the generated template and replace placeholders:
# - REPLACE_WITH_ENVIRONMENT_NAME → your environment name (e.g., "test-env")
# - REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH → path to SSH private key
# - REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH → path to SSH public key
# - REPLACE_WITH_LXD_PROFILE_NAME → LXD profile name (e.g., "test-profile")

# Alternative (manual creation - NOT recommended):
# You can create environment.json manually, but use the template as a reference
# to ensure correct structure. Example shown below for reference only.

cat > envs/test-env.json <<EOF
{
  "environment": {
    "name": "test-env",
    "instance_name": null
  },
  "ssh_credentials": {
    "private_key_path": "/absolute/path/to/fixtures/testing_rsa",
    "public_key_path": "/absolute/path/to/fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "test-profile"
  }
}
EOF

# 2. Create environment
cargo run -- create environment --env-file envs/test-env.json

# 3. Provision VM
cargo run -- provision test-env

# 4. Configure VM (creates storage, installs software)
cargo run -- configure test-env

# 5. Release (deploy configurations and docker-compose)
cargo run -- release test-env

# 6. Run services
cargo run -- run test-env

# 7. Get VM IP for SSH verification
VM_IP=$(cargo run -- show test-env | grep 'IP Address' | awk '{print $3}')
```

#### Common Verification Commands

```bash
# SSH into VM
ssh -i fixtures/testing_rsa ubuntu@$VM_IP

# Check docker services
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust/docker-compose && docker compose ps"

# View docker logs
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust/docker-compose && docker compose logs tracker"
```

#### Cleanup After Testing

```bash
# Destroy environment
cargo run -- destroy test-env

# Clean up files
rm -rf build/test-env
rm envs/test-env.json
```

### Phase 0: Rename Module for Clarity (30 mins)

**Goal**: Rename `src/infrastructure/external_tools/` to `src/infrastructure/templating/` to better reflect its purpose

**Rationale**: The current name "external_tools" is misleading because it mixes:

- **Deployment tools** (OpenTofu, Ansible): Third-party tools used during deployment
- **Runtime configurations** (Docker Compose, Tracker): Service configurations used at runtime

Everything in this module is actually about **template rendering logic**, so `templating` is more accurate and matches the pattern of other modules like `persistence` (which handles persistence logic, not just data).

**Tasks**:

- [ ] Rename directory: `src/infrastructure/external_tools/` → `src/infrastructure/templating/`
- [ ] Update all imports across the codebase
- [ ] Update documentation references (if any)
- [ ] Run pre-commit checks to verify nothing broke

**Verification**:

```bash
# After refactoring, verify no broken imports
cargo build
cargo test

# Verify all references updated (should return no results)
rg "external_tools" src/

# Run full pre-commit checks
./scripts/pre-commit.sh
```

### Phase 1: Create Storage Directories (30 mins)

**Goal**: Released VM has correct directory structure for tracker

**Architecture Note**: This step belongs in **ReleaseCommand**, not ConfigureCommand. The ConfigureCommand prepares the system (Docker, security updates, firewall), while ReleaseCommand deploys the application (creates app directories, deploys configs).

**Tasks**:

- [x] Create `templates/ansible/create-tracker-storage.yml` (static playbook)
- [x] Register playbook in `AnsibleProjectGenerator::copy_static_templates`
- [x] Create `CreateTrackerStorageStep` in `src/application/steps/application/create_tracker_storage.rs` following the pattern of `InstallDockerStep`
- [x] Add step invocation to `ReleaseCommandHandler::execute_release_workflow()` (before rendering templates)
- [x] Add `CreateTrackerStorage` to `ReleaseStep` enum and error handling
- [x] Run manual E2E test to verify directories are created

**Manual E2E Test Results** (✅ PASSED):

```bash
# Test executed: 2025-12-08 15:29 UTC
# Environment: test-phase1 (LXD VM)
# VM IP: 10.140.190.105

# Verified directory structure
$ ssh -i fixtures/testing_rsa torrust@$VM_IP "find /opt/torrust/storage/tracker -type d | sort"
/opt/torrust/storage/tracker
/opt/torrust/storage/tracker/etc
/opt/torrust/storage/tracker/lib
/opt/torrust/storage/tracker/lib/database
/opt/torrust/storage/tracker/log

# Verified ownership and permissions
$ ssh -i fixtures/testing_rsa torrust@$VM_IP "ls -ld /opt/torrust/storage/tracker/*"
drwxr-xr-x 2 torrust torrust 4096 Dec  8 15:29 /opt/torrust/storage/tracker/etc
drwxr-xr-x 3 torrust torrust 4096 Dec  8 15:29 /opt/torrust/storage/tracker/lib
drwxr-xr-x 2 torrust torrust 4096 Dec  8 15:29 /opt/torrust/storage/tracker/log

✅ All verification checks passed:
- Directory structure correct
- Ownership: torrust:torrust (ansible_user)
- Permissions: 0755 (drwxr-xr-x)
- Executed as part of ReleaseCommand workflow
- Idempotent operation
```

**Playbook Content**:

```yaml
---
- name: Create Tracker storage directories
  hosts: all
  become: true

  tasks:
    - name: Create Tracker directory structure
      ansible.builtin.file:
        path: "{{ item }}"
        state: directory
        mode: "0755"
        owner: "{{ ansible_user }}"
        group: "{{ ansible_user }}"
      loop:
        - /opt/torrust/storage/tracker/etc
        - /opt/torrust/storage/tracker/lib/database
        - /opt/torrust/storage/tracker/log
```

**Verification** (after running complete E2E workflow through step 4):

```bash
# Note: Use username "torrust" for all future tests (not "ubuntu")

# Verify directories exist on VM
ssh -i fixtures/testing_rsa torrust@$VM_IP "ls -la /opt/torrust/storage/tracker/"

# Expected: Three subdirectories (etc, lib, log) with correct permissions
# drwxr-xr-x 2 torrust torrust 4096 ... etc
# drwxr-xr-x 3 torrust torrust 4096 ... lib
# drwxr-xr-x 2 torrust torrust 4096 ... log
```

### Phase 2: Initialize SQLite Database (45 mins) ✅ COMPLETE

**Goal**: SQLite database file exists and is initialized

**Tasks**:

- [x] ~~Add database name variable to `templates/ansible/variables.yml.tera`~~ (Skipped - using hardcoded filename for now)
- [x] ~~Update `AnsibleVariablesRenderer` context to include database name~~ (Skipped - will be done in Phase 6)
- [x] Create `templates/ansible/init-tracker-database.yml` (static playbook)
- [x] Register playbook in `AnsibleProjectGenerator::copy_static_templates`
- [x] Create `InitTrackerDatabaseStep` in `src/application/steps/application/init_tracker_database.rs`
- [x] Add step invocation to `ReleaseCommandHandler::execute_release_workflow()` (after `CreateTrackerStorageStep`)
- [x] Add `InitTrackerDatabase` to `ReleaseStep` enum and error handling
- [x] Run manual E2E test to verify database file is created

**Manual E2E Test Results** (✅ PASSED):

```bash
# Test executed: 2025-12-08 15:47 UTC
# Environment: test-phase2 (LXD VM)
# VM IP: 10.140.190.228

# Verified database file exists
$ ssh -o StrictHostKeyChecking=no -i fixtures/testing_rsa torrust@$VM_IP "ls -la /opt/torrust/storage/tracker/lib/database/"
total 8
drwxr-xr-x 2 torrust torrust 4096 Dec  8 15:47 .
drwxr-xr-x 3 torrust torrust 4096 Dec  8 15:47 ..
-rw-r--r-- 1 torrust torrust    0 Dec  8 15:47 tracker.db

# Verified file attributes
$ ssh -o StrictHostKeyChecking=no -i fixtures/testing_rsa torrust@$VM_IP "stat /opt/torrust/storage/tracker/lib/database/tracker.db"
  File: /opt/torrust/storage/tracker/lib/database/tracker.db
  Size: 0               Blocks: 0          IO Block: 4096   regular empty file
Access: (0644/-rw-r--r--)  Uid: ( 1000/ torrust)   Gid: ( 1000/ torrust)

# Verified file type
$ ssh -o StrictHostKeyChecking=no -i fixtures/testing_rsa torrust@$VM_IP "file /opt/torrust/storage/tracker/lib/database/tracker.db"
/opt/torrust/storage/tracker/lib/database/tracker.db: empty

✅ All verification checks passed:
- Database file created: tracker.db
- Ownership: torrust:torrust (ansible_user)
- Permissions: 0644 (-rw-r--r--)
- File type: empty (expected for new SQLite database)
- Executed as part of ReleaseCommand workflow (after CreateTrackerStorage)
- Idempotent operation
```

**Implementation Notes**:

- Simplified implementation: hardcoded "tracker.db" filename instead of using variables
- Database initialization skipped for now (will add schema in future phases)
- Playbook uses `touch` with `state: touch` and `modification_time: preserve`
- Step placed in `application/` layer (application deployment, not system configuration)
- Integrated into ReleaseCommand workflow (not ConfigureCommand)

**Playbook Content** (`templates/ansible/init-tracker-database.yml`):

```yaml
---
# Initialize Torrust Tracker SQLite Database
- name: Initialize Tracker Database
  hosts: all
  become: true
  tasks:
    - name: Create empty SQLite database file
      ansible.builtin.file:
        path: /opt/torrust/storage/tracker/lib/database/tracker.db
        state: touch
        owner: "{{ ansible_user }}"
        group: "{{ ansible_user }}"
        mode: "0644"
        modification_time: preserve
        access_time: preserve

    - name: Verify database file exists
      ansible.builtin.stat:
        path: /opt/torrust/storage/tracker/lib/database/tracker.db
      register: db_file

    - name: Assert database file was created
      ansible.builtin.assert:
        that:
          - db_file.stat.exists
          - db_file.stat.isreg
          - db_file.stat.pw_name == ansible_user
        fail_msg: "Database file was not created properly"
        success_msg: "Database file created successfully"
```

### Phase 3: Add Docker Compose `.env` File (1 hour) ✅ COMPLETE

**Goal**: Docker compose has environment variables file

**Tasks**:

- [x] Rename template: `templates/docker-compose/env.tera` → `.env.tera` (File type requires `.env` extension)
- [x] Create wrapper types: `EnvContext` and `EnvTemplate` in `src/infrastructure/templating/docker_compose/template/wrappers/env/`
- [x] Create `EnvRenderer` in `src/infrastructure/templating/docker_compose/template/renderer/env.rs`
- [x] Refactor to Project Generator pattern: create `DockerComposeProjectGenerator` in `src/infrastructure/templating/docker_compose/template/renderer/project_generator.rs`
- [x] Update `RenderDockerComposeTemplatesStep` to use `DockerComposeProjectGenerator::render()`
- [x] Add `.env` format support to `src/domain/template/file.rs` (Format::Env, Extension::Env)
- [x] Update documentation: `docs/technical/template-system-architecture.md`
- [x] Run manual E2E test to verify `.env` file generation and deployment

**Manual E2E Test Results** (✅ PASSED):

```bash
# Test executed: 2025-12-08 16:35 UTC
# Environment: e2e-phase3-test (LXD VM)
# VM IP: 10.140.190.48

# Test workflow:
# 1. Generated environment template: cargo run -- create template --provider lxd > envs/e2e-phase3.json
# 2. Customized template with test values (name: e2e-phase3-test, profile: e2e-phase3-profile)
# 3. Created environment: cargo run -- create environment --env-file envs/e2e-phase3.json
# 4. Provisioned: cargo run -- provision e2e-phase3-test (27.4s)
# 5. Configured: cargo run -- configure e2e-phase3-test (101.1s)
# 6. Released: cargo run -- release e2e-phase3-test (deployment step)
# 7. Run: cargo run -- run e2e-phase3-test (8.0s)

# Verified .env file in build directory
$ cat build/e2e-phase3-test/docker-compose/.env
# Docker Compose Environment Variables
# This file contains environment variables used by docker-compose services

# Tracker Configuration
# Path to the tracker TOML configuration file inside the container
TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml

# Admin API token for tracker HTTP API access
# This overrides the admin token in the tracker configuration file
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=MyAccessToken

# Verified .env file deployed to VM
$ ssh -i fixtures/testing_rsa torrust@10.140.190.48 "cat /opt/torrust/.env"
# Docker Compose Environment Variables
# This file contains environment variables used by docker-compose services

# Tracker Configuration
# Path to the tracker TOML configuration file inside the container
TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml

# Admin API token for tracker HTTP API access
# This overrides the admin token in the tracker configuration file
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=MyAccessToken

# Verified file listing on VM
$ ssh -i fixtures/testing_rsa torrust@10.140.190.48 "ls -la /opt/torrust/"
total 20
drwxr-xr-x 3 root    root    4096 Dec  8 16:34 .
drwxr-xr-x 4 root    root    4096 Dec  8 16:33 ..
-rw-r--r-- 1 root    root     464 Dec  8 16:34 .env
-rw-r--r-- 1 root    root     685 Dec  8 16:33 docker-compose.yml
drwxr-xr-x 3 torrust torrust 4096 Dec  8 16:33 storage

✅ All verification checks passed:
- .env file generated in build directory: build/e2e-phase3-test/docker-compose/.env
- .env file deployed to VM: /opt/torrust/.env
- File contains hardcoded "MyAccessToken" as expected (Phase 6 will make this configurable)
- Permissions: 0644 (-rw-r--r--)
- Ownership: root:root (deployed via Ansible)
- File synchronization via deploy-compose-files.yml playbook working correctly
- Project Generator pattern properly orchestrating Wrapper → Renderer → Generator layers
```

**Architecture Implementation**:

Refactored to **Project Generator pattern** (three-layer architecture):

1. **Wrapper Layer**: Context + Template types

   - `EnvContext` - holds template variables (tracker_api_admin_token)
   - `EnvTemplate` - wraps context and rendered content

2. **Renderer Layer**: One renderer per template file

   - `EnvRenderer` - renders `.env.tera` → `.env` file

3. **Generator Layer**: Orchestrator for all renderers
   - `DockerComposeProjectGenerator` - manages all Docker Compose template generation
   - Calls `EnvRenderer` for dynamic templates
   - Copies static files (docker-compose.yml)

**Implementation Notes**:

- Template renamed: `env.tera` → `.env.tera` (File type needs proper extension for Format::Env)
- Hardcoded "MyAccessToken" in EnvContext (TODO comment: will be configurable in Phase 6)
- Removed old monolithic `DockerComposeTemplateRenderer` (~700 lines)
- New clean module structure (~30 lines in mod.rs, ~370 lines in project_generator.rs)
- Added comprehensive unit tests for all components
- All linters passing (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- All unit tests passing (1353 tests)

**Template Content** (`templates/docker-compose/.env.tera`):

```bash
# Docker Compose Environment Variables
# This file contains environment variables used by docker-compose services

# Tracker Configuration
# Path to the tracker TOML configuration file inside the container
TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml

# Admin API token for tracker HTTP API access
# This overrides the admin token in the tracker configuration file
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN={{ tracker_api_admin_token }}
```

**Deployment Flow**:

1. `RenderDockerComposeTemplatesStep` creates `EnvContext` with hardcoded "MyAccessToken"
2. Calls `DockerComposeProjectGenerator::render(&env_context)`
3. Generator calls `EnvRenderer::render()` to process `.env.tera`
4. Writes `.env` to `build/e2e-phase3-test/docker-compose/.env`
5. `DeployComposeFilesStep` synchronizes entire directory to VM via Ansible
6. Result: `/opt/torrust/.env` contains rendered environment variables

**Verification** (complete E2E workflow):

```bash
# Use template generation workflow (recommended):
cargo run -- create template --provider lxd > envs/test-env.json
# Customize the generated template with your values
# Then: cargo run -- create environment --env-file envs/test-env.json

# Verify .env file in build directory
cat build/test-env/docker-compose/.env

# Verify .env file deployed to VM
ssh -i fixtures/testing_rsa torrust@$VM_IP "cat /opt/torrust/.env"

# Expected content:
# TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml
# TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=MyAccessToken
```

### Phase 4: Add Tracker Configuration Template (1.5 hours)

**Goal**: Tracker has configuration file on VM

**Tasks**:

- [ ] Create `templates/tracker/` directory
- [ ] Create `templates/tracker/tracker.toml.tera` (hardcoded initially)
- [ ] Create `src/infrastructure/templating/tracker/` module
- [ ] Implement `TrackerProjectGenerator` following same pattern as Ansible/DockerCompose
- [ ] Create `RenderTrackerTemplatesStep` in `src/application/steps/rendering/` to render tracker templates to `build/<env>/tracker/`
- [ ] Create Ansible playbook `templates/ansible/deploy-tracker-config.yml` (static, no .tera extension) to copy config to VM
- [ ] Register playbook in `AnsibleProjectGenerator::copy_static_templates`
- [ ] Create `DeployTrackerConfigStep` in `src/application/steps/application/deploy_tracker_config.rs` following pattern of `DeployComposeFilesStep`
- [ ] Add both steps to `ReleaseCommandHandler::execute_release_workflow()` (render step first, then deploy step, both before `DeployComposeFilesStep`)

**Directory Structure**:

```text
templates/tracker/              # Source templates (embedded at compile time)
└── tracker.toml.tera

build/<env>/tracker/            # Rendered templates (generated at runtime)
└── tracker.toml

src/infrastructure/templating/tracker/
├── mod.rs
└── template/
    ├── mod.rs
    ├── error.rs
    └── renderer/
        ├── mod.rs
        └── project_generator.rs

src/application/steps/
├── rendering/
│   └── tracker_templates.rs    # RenderTrackerTemplatesStep
└── application/
    └── deploy_tracker_config.rs # DeployTrackerConfigStep
```

**Template Content** (`tracker.toml.tera` - hardcoded for now):

```toml
[metadata]
app = "torrust-tracker"
purpose = "configuration"
schema_version = "2.0.0"

[logging]
threshold = "info"

[core]
listed = false
private = false

[core.tracker_policy]
persistent_torrent_completed_stat = true

[core.announce_policy]
interval = 300
interval_min = 300

[core.net]
on_reverse_proxy = true

[core.database]
driver = "sqlite3"
path = "/var/lib/torrust/tracker/database/sqlite3.db"

[[udp_trackers]]
bind_address = "0.0.0.0:6868"

[[udp_trackers]]
bind_address = "0.0.0.0:6969"

[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:1212"
```

**ProjectGenerator Implementation** (follow same pattern as `AnsibleProjectGenerator`):

```rust
// src/infrastructure/templating/tracker/template/renderer/project_generator.rs
pub struct TrackerProjectGenerator {
    tera: Tera,
    output_dir: PathBuf,
}

impl TrackerProjectGenerator {
    pub fn new(template_dir: &Path, output_dir: &Path) -> Result<Self, TrackerTemplateError> {
        let mut tera = Tera::default();
        let pattern = template_dir.join("**/*.tera").to_string_lossy().to_string();
        tera.add_template_files(vec![(pattern.as_str(), None)])
            .map_err(TrackerTemplateError::from)?;

        Ok(Self {
            tera,
            output_dir: output_dir.to_path_buf(),
        })
    }

    pub fn generate_all_templates(&self, environment_config: &EnvironmentConfig) -> Result<(), TrackerTemplateError> {
        self.render_tracker_config(environment_config)?;
        Ok(())
    }

    fn render_tracker_config(&self, environment_config: &EnvironmentConfig) -> Result<(), TrackerTemplateError> {
        // Phase 4: Empty context (hardcoded values in template)
        // Phase 6: Extract tracker config and populate context with variables
        let context = Context::new();

        let content = self.tera.render("tracker.toml.tera", &context)?;
        let output_path = self.output_dir.join("tracker.toml");
        std::fs::write(output_path, content)?;
        Ok(())
    }
}
```

**Ansible Playbook** (`deploy-tracker-config.yml`):

```yaml
---
- name: Deploy Tracker configuration
  hosts: all
  become: true

  tasks:
    - name: Copy tracker.toml to VM
      ansible.builtin.copy:
        src: "{{ playbook_dir }}/../tracker/tracker.toml"
        dest: /opt/torrust/storage/tracker/etc/tracker.toml
        mode: "0644"
        owner: "{{ ansible_user }}"
        group: "{{ ansible_user }}"
```

**Verification** (after running complete E2E workflow through step 5):

```bash
# Verify tracker.toml in build directory
cat build/test-env/tracker/tracker.toml | head -20

# Verify tracker.toml deployed to VM
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/storage/tracker/etc/tracker.toml | head -20"

# Expected: First 20 lines of tracker configuration with correct metadata and settings
```

### Phase 5: Replace Docker Compose Service (1 hour)

**Goal**: Docker compose runs Torrust Tracker instead of nginx

**Tasks**:

- [ ] Update `templates/docker-compose/docker-compose.yml` with tracker service
- [ ] Remove demo-app (nginx) service definition
- [ ] Test docker-compose file syntax locally
- [ ] Run full workflow: configure → release → run

**Updated Docker Compose**:

```yaml
# Docker Compose configuration for Torrust Tracker deployment

services:
  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    tty: true
    restart: unless-stopped
    environment:
      - USER_ID=1000
      - TORRUST_TRACKER_CONFIG_TOML=${TORRUST_TRACKER_CONFIG_TOML}
      - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=sqlite3
      - TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN}
    networks:
      - backend_network
    ports:
      - 6868:6868/udp
      - 6969:6969/udp
      - 7070:7070
      - 1212:1212
    volumes:
      - ./storage/tracker/lib:/var/lib/torrust/tracker:Z
      - ./storage/tracker/log:/var/log/torrust/tracker:Z
      - ./storage/tracker/etc:/etc/torrust/tracker:Z
    logging:
      options:
        max-size: "10m"
        max-file: "10"

networks:
  backend_network: {}
```

**Verification** (after running complete E2E workflow through step 6):

```bash
# Verify tracker container is running
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cd /opt/torrust/docker-compose && docker compose ps"

# Expected: Container "tracker" with status "Up" and exposed ports (6868/udp, 6969/udp, 7070, 1212)

# Test tracker HTTP API responds
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "curl -s http://localhost:1212/api/v1/stats"

# Expected: JSON response with tracker statistics (e.g., {"torrents": 0, "seeders": 0, "leechers": 0, ...})

# Verify all expected ports are listening
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "docker compose ps | grep -E '6868|6969|7070|1212'"
```

### Phase 6: Add Environment Configuration Support (2 hours)

**Goal**: Users can configure tracker via environment.json

**Tasks**:

- [ ] Add `TrackerConfig` struct to `src/domain/config/environment/schema.rs`
- [ ] Update environment JSON schema validation
- [ ] Modify `TrackerProjectGenerator::render_tracker_config()` to accept environment config parameter and use Tera context
- [ ] Update `RenderTrackerTemplatesStep` to pass `environment.config()` to `TrackerProjectGenerator`
- [ ] Update `tracker.toml.tera` to use Tera variables for configurable sections (`tracker_core_private`, `udp_trackers`, `http_trackers`)
- [ ] Update `AnsibleVariablesRenderer` to extract tracker config from `environment.config().tracker` (used in Phases 1-2 Configure steps)
- [ ] Update `EnvFileRenderer` in `DockerComposeProjectGenerator` to extract `admin_token` from environment config
- [ ] Create example environment file with tracker configuration
- [ ] Update E2E tests to use tracker configuration

**Environment Config Schema**:

```rust
// src/domain/config/environment/schema.rs
// Structure mirrors the real tracker config but only includes user-configurable fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerConfig {
    pub core: TrackerCoreConfig,
    pub udp_trackers: Vec<UdpTrackerConfig>,
    pub http_trackers: Vec<HttpTrackerConfig>,
    pub http_api: HttpApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerCoreConfig {
    pub database_name: String,
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpTrackerConfig {
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpApiConfig {
    pub admin_token: String,
}

// Add to EnvironmentConfig
#[serde(skip_serializing_if = "Option::is_none")]
pub tracker: Option<TrackerConfig>,
```

**Updated Template** (`tracker.toml.tera`):

```toml
[core]
private = {{ tracker_core_private }}

{% for udp_tracker in udp_trackers %}
[[udp_trackers]]
bind_address = "{{ udp_tracker.bind_address }}"
{% endfor %}

{% for http_tracker in http_trackers %}
[[http_trackers]]
bind_address = "{{ http_tracker.bind_address }}"
{% endfor %}

# ... rest of config remains hardcoded for now ...
```

**Example Environment File**:

```json
{
  "name": "tracker-test",
  "provider": "lxd",
  "vm": {
    "instance_name": "tracker-test-vm",
    "ssh_username": "ubuntu",
    "ssh_key_path": "fixtures/testing_rsa"
  },
  "tracker": {
    "core": {
      "database_name": "tracker.db",
      "private": false
    },
    "udp_trackers": [
      { "bind_address": "0.0.0.0:6868" },
      { "bind_address": "0.0.0.0:6969" }
    ],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "admin_token": "MyAccessToken"
    }
  }
}
```

**Manual Test**:

```bash
# Create environment with custom tracker config
cat > envs/tracker-custom.json <<EOF
{
  "name": "tracker-custom",
  "provider": "lxd",
  "tracker": {
    "core": {
      "database_name": "my_tracker.db",
      "private": true
    },
    "udp_trackers": [
      { "bind_address": "0.0.0.0:6969" }
    ],
    "http_trackers": [],
    "http_api": {
      "admin_token": "CustomAdminToken123"
    }
  }
}
EOF

# Run full E2E workflow (steps 2-6 from Manual Testing Workflow section)
cargo run -- create tracker-custom --env-file envs/tracker-custom.json
cargo run -- provision tracker-custom
cargo run -- configure tracker-custom
cargo run -- release tracker-custom
cargo run -- run tracker-custom
```

**Verification** (after running complete E2E workflow with custom config):

```bash
# Get VM IP
VM_IP=$(cargo run -- show tracker-custom | grep 'IP Address' | awk '{print $3}')

# Verify custom database name was used
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "ls /opt/torrust/storage/tracker/lib/database/my_tracker.db"

# Verify private mode is enabled in config
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/storage/tracker/etc/tracker.toml | grep 'private = true'"

# Verify only UDP port 6969 is configured (not 6868)
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/storage/tracker/etc/tracker.toml | grep -A 1 'udp_trackers' | grep '6969'"

# Verify custom admin token is set in .env
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/docker-compose/.env | grep 'CustomAdminToken123'"

# Verify HTTP trackers array is empty (no HTTP tracker configured)
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "docker compose ps | grep -v '7070'"
```

### Phase 7: Configure Firewall for Tracker Ports (1 hour)

**Goal**: Open firewall ports for all configured tracker services (UDP trackers, HTTP trackers, HTTP API)

**Rationale**: The existing `configure-firewall.yml` playbook is critical for SSH access and should remain focused on that single responsibility. Adding tracker-specific firewall rules to a separate playbook provides:

- **Single Responsibility**: Each playbook does one thing well
- **Maintainability**: Tracker firewall changes don't affect critical SSH access rules
- **Extensibility**: Future services (nginx, grafana, etc.) can have their own firewall playbooks
- **Safety**: Critical SSH configuration remains isolated from service-specific changes

**Tasks**:

- [ ] Extract port numbers from environment config tracker configuration
- [ ] Add port extraction logic to `AnsibleVariablesRenderer` context (reads from `environment.config().tracker`)
- [ ] Create `templates/ansible/configure-tracker-firewall.yml.tera` (dynamic playbook using variables)
- [ ] Register playbook in `AnsibleProjectGenerator` as dynamic template (has `.tera` extension, rendered during template generation)
- [ ] Create `ConfigureTrackerFirewallStep` in `src/application/steps/system/configure_tracker_firewall.rs` following pattern of `ConfigureFirewallStep`
- [ ] Add step invocation to `ConfigureCommandHandler::execute_configuration_with_tracking()` (after `ConfigureFirewallStep`, conditional on `environment.config().tracker.is_some()`)

**Variable Addition** (`AnsibleVariablesRenderer`):

```rust
// Extract tracker ports from environment config
let mut tracker_udp_ports = Vec::new();
let mut tracker_http_ports = Vec::new();
let mut tracker_api_port = None;

if let Some(tracker_config) = &environment_config.tracker {
    // Extract UDP tracker ports
    for udp_tracker in &tracker_config.udp_trackers {
        if let Some(port) = extract_port(&udp_tracker.bind_address) {
            tracker_udp_ports.push(port);
        }
    }

    // Extract HTTP tracker ports
    for http_tracker in &tracker_config.http_trackers {
        if let Some(port) = extract_port(&http_tracker.bind_address) {
            tracker_http_ports.push(port);
        }
    }

    // Extract HTTP API port (default 1212 if not in bind_address)
    tracker_api_port = Some(1212); // Hardcoded for now, can be made configurable later
}

context.insert("tracker_udp_ports", &tracker_udp_ports);
context.insert("tracker_http_ports", &tracker_http_ports);
context.insert("tracker_api_port", &tracker_api_port);

// Helper function to extract port from bind_address (e.g., "0.0.0.0:6868" -> 6868)
fn extract_port(bind_address: &str) -> Option<u16> {
    bind_address.split(':').nth(1)?.parse().ok()
}
```

**Playbook Content** (`templates/ansible/configure-tracker-firewall.yml.tera`):

```yaml
---
# Configure Firewall for Tracker Services
# This playbook opens firewall ports for UDP trackers, HTTP trackers, and HTTP API.
# Must be run AFTER configure-firewall.yml (which sets up SSH access).
#
# Variables are loaded from variables.yml for centralized management.

- name: Configure firewall for Tracker services
  hosts: all
  become: true
  gather_facts: false
  vars_files:
    - variables.yml

  tasks:
    - name: Allow UDP tracker ports
      community.general.ufw:
        rule: allow
        port: "{{ item }}"
        proto: udp
        comment: "Torrust Tracker UDP"
      loop: { { tracker_udp_ports } }
      when: tracker_udp_ports | length > 0
      tags:
        - security
        - firewall
        - tracker

    - name: Allow HTTP tracker ports
      community.general.ufw:
        rule: allow
        port: "{{ item }}"
        proto: tcp
        comment: "Torrust Tracker HTTP"
      loop: { { tracker_http_ports } }
      when: tracker_http_ports | length > 0
      tags:
        - security
        - firewall
        - tracker

    - name: Allow Tracker HTTP API port
      community.general.ufw:
        rule: allow
        port: "{{ tracker_api_port }}"
        proto: tcp
        comment: "Torrust Tracker HTTP API"
      when: tracker_api_port is defined
      tags:
        - security
        - firewall
        - tracker
        - api

    - name: Reload UFW to apply changes
      community.general.ufw:
        state: reloaded
      tags:
        - security
        - firewall
        - reload
```

**ConfigureCommandHandler Update**:

```rust
// After configure-firewall.yml execution
ansible_runner.run_playbook("configure-firewall.yml")?;

// Configure tracker-specific firewall rules (if tracker is configured)
if environment_config.tracker.is_some() {
    ansible_runner.run_playbook("configure-tracker-firewall.yml")?;
}
```

**Verification** (after running complete E2E workflow through step 4):

```bash
# Verify UFW rules include tracker ports
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "sudo ufw status numbered"

# Expected output should include:
# [ X] 6868/udp         ALLOW IN    Anywhere    # Torrust Tracker UDP
# [ Y] 6969/udp         ALLOW IN    Anywhere    # Torrust Tracker UDP
# [ Z] 7070/tcp         ALLOW IN    Anywhere    # Torrust Tracker HTTP
# [AA] 1212/tcp         ALLOW IN    Anywhere    # Torrust Tracker HTTP API

# Test external connectivity to tracker ports from host machine
# (requires VM IP to be accessible from host)
nc -zv $VM_IP 6868  # Should fail (UDP doesn't respond to TCP connect)
nc -zvu $VM_IP 6868 # UDP connectivity test (may timeout but port should be open)
nc -zv $VM_IP 7070  # Should succeed after tracker is running
nc -zv $VM_IP 1212  # Should succeed after tracker is running

# Verify firewall reload happened without breaking SSH
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "echo 'SSH still works'"
```

**Manual E2E Test Results** (🔨 PARTIAL - Infrastructure tested, wiring pending):

````bash
# Test executed: 2025-12-09 08:10 UTC
# Test type: Full E2E test (e2e-tests-full)
# Environment: e2e-full (LXD VM)
# Status: ✅ PASSED (infrastructure components verified)

# Test workflow:
# 1. Preflight cleanup completed
# 2. Environment created from config (13.0s)
# 3. Infrastructure provisioned (28.1s)
# 4. Services configured (38.6s) - includes firewall configuration
# 5. Software released (7.5s)
# 6. Services started (10.0s)
# 7. Deployment validated (2.2s)
# 8. Infrastructure destroyed (2.8s)
# Total test duration: 102.2s

✅ All verification checks passed:
- Port extraction logic tested (10 unit tests passing)
- AnsibleVariablesContext accepts tracker configuration
- Variables template updated with tracker port variables
- Firewall playbook created and registered (13 playbooks total)
- ConfigureTrackerFirewallStep created and integrated
- ConfigureStep enum updated
- All 1390 tests passing

**Phase 7 Wiring Completed** (2025-12-09):

✅ Tracker configuration successfully wired through provision workflow:
- Updated `RenderAnsibleTemplatesStep` to accept and forward `TrackerConfig`
- Refactored `AnsibleTemplateService` to accept `UserInputs` instead of individual parameters
  - **Design improvement**: Pass cohesive `UserInputs` + `instance_ip` (runtime output)
  - Reduces parameter list from 4 to 2 parameters
  - Better separation of UserInputs (immutable) vs RuntimeOutputs (generated)
- Updated `ProvisionCommandHandler` to pass `UserInputs` from environment context
- Updated `RegisterCommandHandler` to use new signature

**Manual E2E Test Results** (✅ PASSED - 2025-12-09 08:52 UTC):

```bash
# Test environment: phase7-test (LXD VM)
# VM IP: 10.140.190.118

# Verified UFW firewall rules include all tracker ports:
$ ssh -i fixtures/testing_rsa torrust@10.140.190.118 "sudo ufw status numbered"

Status: active

     To                         Action      From
     --                         ------      ----
[ 1] 22/tcp                     ALLOW IN    Anywhere                   # SSH access (configured port 22)
[ 2] 6868/udp                   ALLOW IN    Anywhere                   # Torrust Tracker UDP
[ 3] 6969/udp                   ALLOW IN    Anywhere                   # Torrust Tracker UDP
[ 4] 7070/tcp                   ALLOW IN    Anywhere                   # Torrust Tracker HTTP
[ 5] 1212/tcp                   ALLOW IN    Anywhere                   # Torrust Tracker HTTP API
[ 6] 22/tcp (v6)                ALLOW IN    Anywhere (v6)              # SSH access (configured port 22)
[ 7] 6868/udp (v6)              ALLOW IN    Anywhere (v6)              # Torrust Tracker UDP
[ 8] 6969/udp (v6)              ALLOW IN    Anywhere (v6)              # Torrust Tracker UDP
[ 9] 7070/tcp (v6)              ALLOW IN    Anywhere (v6)              # Torrust Tracker HTTP
[10] 1212/tcp (v6)              ALLOW IN    Anywhere (v6)              # Torrust Tracker HTTP API

✅ All firewall rules verified:
- SSH port 22 configured (configure-firewall.yml)
- UDP tracker ports 6868, 6969 configured (configure-tracker-firewall.yml)
- HTTP tracker port 7070 configured (configure-tracker-firewall.yml)
- HTTP API port 1212 configured (configure-tracker-firewall.yml)
- All ports have correct "Torrust Tracker" comments
- IPv4 and IPv6 rules both present

# Verified variables.yml contains extracted tracker ports:
$ cat build/phase7-test/ansible/variables.yml | grep -A 5 "Tracker Firewall"

# Tracker Firewall Configuration
tracker_udp_ports:
  - 6868
  - 6969
tracker_http_ports:
  - 7070
tracker_api_port: 1212
````

**Test Results Summary**:

- ✅ Full E2E test passed (102.0s, all 1390 unit tests passing)
- ✅ Tracker ports correctly extracted from environment configuration
- ✅ Variables.yml populated with tracker firewall configuration
- ✅ UFW firewall rules applied for all tracker ports
- ✅ Port comments correctly identify "Torrust Tracker" services
- ✅ Both IPv4 and IPv6 rules configured
- ✅ All pre-commit checks passing

**Phase 7 Status**: ✅ **COMPLETE**

### Phase 8: Update E2E Tests for Tracker Validation (1.5 hours)

**Goal**: Replace demo nginx validation with real Torrust Tracker API health check validation using external-only validation strategy

**Context**: The current E2E tests (`src/bin/e2e_config_and_release_tests.rs`) validate that services are running by checking Docker Compose status and attempting an HTTP request to port 8080 (the old demo nginx service). Since we've replaced the demo app with the real Torrust Tracker, we need to update the validation to check the tracker's HTTP API health endpoint instead.

**Validation Philosophy**: External checks are a superset of internal checks. If external validation passes, it proves:

- Services are running inside the VM
- Firewall rules are configured correctly
- Services are accessible from outside the VM

This simplifies E2E tests and makes them easier to maintain. If external checks fail, debugging will reveal whether it's a service issue (check `docker compose ps` via SSH) or a firewall issue (service running but not accessible).

**Current Behavior** (Why tests don't fail):

- The `RunningServicesValidator::check_http_accessibility()` method attempts to `curl http://localhost:8080`
- This check fails (port 8080 is not open), but only logs a **warning** instead of failing the test
- The validation completes successfully despite the failed HTTP check
- This is by design for the demo slice - HTTP checks are optional/informational

**Tasks**:

- [x] Update `RunningServicesValidator` infrastructure (external validation via direct HTTP)

  - Changed from demo nginx port 8080 to tracker API port 1212
  - Uses tracker API health check endpoint: `http://<vm-ip>:1212/api/health_check`
  - Uses HTTP tracker health check endpoint: `http://<vm-ip>:7070/api/health_check`
  - Made tracker API check **required** (fails validation if check fails)
  - Made HTTP tracker check **optional** (logs warning if fails - may not have health endpoint)
  - Updated logging to reflect tracker validation (not "demo-app")
  - Added `reqwest` dependency for HTTP client

- [x] Refactor `execute()` method for better code quality

  - Extracted `validate_services_are_running()` private method (Docker Compose status check)
  - Extracted `check_service_health_status()` private method (health status check)
  - Extracted `validate_external_accessibility()` private method (external HTTP validation)
  - Extracted `check_tracker_api_external()` private method (tracker API health check)
  - Extracted `check_http_tracker_external()` private method (HTTP tracker health check)
  - Reduced `execute()` from ~90 lines to ~30 lines (orchestration only)

- [x] Update E2E test documentation comments

  - Removed references to "demo slice" and "temporary nginx service"
  - Updated comments in `src/testing/e2e/tasks/run_run_validation.rs` to reflect real tracker validation
  - Updated comments in `src/infrastructure/remote_actions/validators/running_services.rs`

- [ ] Update E2E tests to use external validation only
  - Remove internal SSH-based health checks from test code
  - Verify both tracker API (port 1212) and HTTP tracker (port 7070) are accessible externally
  - Include proper error messages for external validation failures

**Implementation Details**:

```rust
// External validation (direct HTTP from test runner)
impl RunningServicesValidator {
    async fn execute(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        // Step 1: Check Docker Compose services are running (via SSH)
        self.validate_services_are_running().await?;

        // Step 2: Check service health status (via SSH)
        self.check_service_health_status().await;

        // Step 3: Validate external accessibility (direct HTTP)
        self.validate_external_accessibility(server_ip).await?;

        Ok(())
    }

    /// Check tracker API accessibility from outside the VM
    async fn check_tracker_api_external(&self, server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        let url = format!("http://{}:1212/api/health_check", server_ip);
        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            return Err(ValidationError::TrackerApiUnhealthy);
        }

        Ok(())
    }

    /// Check HTTP tracker accessibility from outside the VM (optional check)
    async fn check_http_tracker_external(&self, server_ip: &IpAddr) {
        let url = format!("http://{}:7070/api/health_check", server_ip);
        if let Ok(response) = reqwest::get(&url).await {
            if response.status().is_success() {
                info!("HTTP Tracker health check passed");
            } else {
                warn!("HTTP Tracker returned non-success - may not have health endpoint");
            }
        }
    }
}
```

**Verification**:

```bash
# Run E2E tests to verify tracker external health checks
cargo run --bin e2e-config-and-release-tests

# Expected log output:
# - "Docker Compose services are running" (via SSH: docker compose ps)
# - "Tracker API is accessible from outside (external check passed)"
# - "HTTP Tracker is accessible from outside (external check passed)" (or warning if no endpoint)

# Validation should FAIL if:
# - Tracker services are not running (docker compose ps shows no running services)
# - External tracker API not accessible (port 1212 blocked or service not running)

# Validation should PASS when:
# - Services are running inside VM (docker compose ps shows "running")
# - Tracker API accessible externally (http://<vm-ip>:1212/api/health_check returns 200)
# - HTTP tracker accessible externally (http://<vm-ip>:7070/api/health_check returns 200)
```

**Manual Testing**:

```bash
# Create and deploy test environment
cargo run -- create template --provider lxd > envs/tracker-test.json
# Edit tracker-test.json with your values
cargo run -- create environment --env-file envs/tracker-test.json
cargo run -- provision tracker-test
cargo run -- configure tracker-test
cargo run -- release tracker-test
cargo run -- run tracker-test

# Get VM IP
VM_IP=$(cargo run -- show tracker-test | grep 'IP Address' | awk '{print $3}')

# Test: External validation (direct HTTP - verifies service AND firewall)
echo "=== External Validation (Direct HTTP) ==="
curl -sf http://$VM_IP:1212/api/health_check
# Expected: {"status":"ok"} or HTTP 200 (proves service is running AND firewall allows access)

curl -sf http://$VM_IP:7070/api/health_check
# Expected: {"status":"ok"} or HTTP 200 (proves service is running AND firewall allows access)

# If external validation fails, debug internally:
echo "=== Debug: Check if services are running ==="
ssh -i fixtures/testing_rsa torrust@$VM_IP "docker compose ps"
# Expected: Shows tracker services in "running" state

echo "=== Debug: Check internal connectivity ==="
ssh -i fixtures/testing_rsa torrust@$VM_IP "curl -sf http://localhost:1212/api/health_check"
# If this works but external fails, it's a firewall issue

# Run E2E tests to verify external validation
cargo run --bin e2e-config-and-release-tests
# Should complete successfully with external health check logs
```

**Why External-Only Validation?**

Previously implemented dual validation (internal via SSH + external direct HTTP), but simplified to external-only because:

1. **External is Superset**: External checks already validate service functionality
2. **Simpler E2E Tests**: Easier to maintain without redundant SSH-based checks
3. **Sufficient for Testing**: E2E tests only need to verify end-to-end accessibility
4. **Debugging Flexibility**: If external fails, can SSH in to check `docker compose ps` manually

**Phase 8 Status**: 🔨 **IN PROGRESS**

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] Storage directories created on VM (`/opt/torrust/storage/tracker/{etc,lib/database,log}`)
- [ ] SQLite database initialized and valid (`tracker.db`)
- [ ] Docker compose `.env` file deployed to VM
- [ ] Tracker configuration file (`tracker.toml`) deployed to VM
- [ ] Docker compose service definition updated with tracker service
- [ ] Firewall rules configured for all tracker ports (UDP trackers, HTTP trackers, HTTP API)
- [ ] Tracker container starts successfully via `docker compose up -d`
- [ ] Tracker HTTP API responds to health check (`curl http://localhost:1212/api/v1/stats`)
- [ ] Tracker ports accessible externally (firewall allows traffic)

**Configuration Requirements**:

- [ ] Environment config schema supports tracker configuration (mirrors real tracker config structure)
- [ ] Users can specify UDP tracker instances (array of `bind_address` strings)
- [ ] Users can specify HTTP tracker instances (array of `bind_address` strings)
- [ ] Users can toggle private/public mode (`core.private`)
- [ ] Database name is configurable (`core.database_name`)
- [ ] API admin token is configurable (`http_api.admin_token`)
- [ ] Template variables correctly substituted in `tracker.toml`

**Architecture Requirements**:

- [ ] Tracker templates isolated in `templates/tracker/` directory
- [ ] Tracker templating module follows Project Generator pattern (`src/infrastructure/templating/tracker/`)
- [ ] Static Ansible playbooks registered in renderer
- [ ] No mixing of tracker and docker-compose templates

**Testing Requirements**:

- [ ] Each phase manually testable with provided commands
- [ ] E2E tests updated to include tracker deployment
- [ ] Example environment file demonstrates tracker configuration

## Related Documentation

- [Template System Architecture](../technical/template-system-architecture.md)
- [Adding Ansible Playbooks](../contributing/templates.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Torrust Demo Reference](https://github.com/torrust/torrust-demo)
- [Issue #217 - Demo Slice](./217-demo-slice-release-run-commands.md)

## Notes

### Design Decisions

**Tracker as Separate Service**: Each service (tracker, index, grafana) will have its own template module and renderer. This provides:

- Clear separation of concerns
- Easier to enable/disable services in future
- Simpler to add new services incrementally

**Progressive Configuration Exposure**: We start with hardcoded values and gradually expose configuration options. This ensures:

- Each step is testable with working deployment
- Complexity added incrementally
- Pipeline validated early

**Static Playbooks with Centralized Variables**: Following Ansible best practices:

- Playbooks are static YAML (no `.tera` extension)
- Variables defined in `variables.yml.tera` (single source of truth)
- Reduces Rust boilerplate for each playbook

### Future Enhancements

After this slice is complete, future work can:

- Expose more tracker configuration options (database driver, intervals, logging)
- Add health checks and monitoring
- Support multiple tracker configurations (staging, production)
- Add tracker management commands (status, logs, restart)

### Implementation Time Estimate

- Phase 0: 30 mins (module rename refactoring)
- Phase 1: 30 mins (storage directories)
- Phase 2: 45 mins (database initialization)
- Phase 3: 1 hour (`.env` file)
- Phase 4: 1.5 hours (tracker configuration template)
- Phase 5: 1 hour (docker-compose service update)
- Phase 6: 2 hours (environment configuration)
- Phase 7: 1 hour (firewall configuration)
- Phase 8: 1.5 hours (E2E test validation update)

**Total**: ~10 hours

### Testing Strategy

Each phase includes:

1. **Build Verification**: Check files in `build/` directory
2. **Deployment Verification**: SSH into VM and verify files exist
3. **Runtime Verification**: Test services are running (docker compose, curl)

This three-level verification ensures issues are caught early.
