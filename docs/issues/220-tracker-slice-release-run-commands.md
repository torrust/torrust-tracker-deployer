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
- [ ] **Phase 2**: Initialize SQLite Database (45 mins)
- [ ] **Phase 3**: Add Docker Compose `.env` File (1 hour)
- [ ] **Phase 4**: Add Tracker Configuration Template (1.5 hours)
- [ ] **Phase 5**: Replace Docker Compose Service (1 hour)
- [ ] **Phase 6**: Add Environment Configuration Support (2 hours)
- [ ] **Phase 7**: Configure Firewall for Tracker Ports (1 hour)

**Total Estimated Time**: ~8.5 hours

### Manual Testing Workflow

Each phase should be tested using the following end-to-end workflow. The specific verification steps for each phase are documented in the phase sections below.

#### Prerequisites

```bash
# Ensure you have a clean test environment
rm -rf build/test-env
rm -rf envs/test-env.json
```

#### Complete E2E Test Flow

```bash
# 1. Create environment configuration file
cat > envs/test-env.json <<EOF
{
  "name": "test-env",
  "provider": "lxd",
  "vm": {
    "instance_name": "test-env-vm",
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
    "http_trackers": [
      { "bind_address": "0.0.0.0:7070" }
    ],
    "http_api": {
      "admin_token": "TestAdminToken123"
    }
  }
}
EOF

# 2. Create environment
cargo run -- create test-env --env-file envs/test-env.json

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

### Phase 3: Add Docker Compose `.env` File (1 hour)

**Goal**: Docker compose has environment variables file

**Tasks**:

- [ ] Create `templates/docker-compose/env.tera` with tracker variables
- [ ] Create `EnvFileRenderer` in `src/infrastructure/templating/docker_compose/template/renderer/`
- [ ] Add renderer to `DockerComposeProjectGenerator::generate_all_templates()`
- [ ] Note: `.env` file will be automatically deployed to VM by existing `deploy-compose-files.yml` playbook (synchronizes entire docker-compose directory)

**Template Content** (`env.tera`):

```bash
# Tracker Configuration
TORRUST_TRACKER_CONFIG_TOML_PATH='/etc/torrust/tracker/tracker.toml'
TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN={{ tracker_api_admin_token }}
```

**Renderer Implementation**:

```rust
// src/infrastructure/templating/docker_compose/template/renderer/env_file.rs
use tera::{Context, Tera};
use crate::infrastructure::templating::docker_compose::template::error::DockerComposeTemplateError;

pub struct EnvFileRenderer;

impl EnvFileRenderer {
    pub fn render(tera: &Tera, tracker_api_admin_token: &str) -> Result<String, DockerComposeTemplateError> {
        let mut context = Context::new();
        context.insert("tracker_api_admin_token", tracker_api_admin_token);
        tera.render("env.tera", &context)
            .map_err(DockerComposeTemplateError::from)
    }
}
```

**ProjectGenerator Update**:

```rust
// src/infrastructure/templating/docker_compose/template/renderer/mod.rs
pub fn generate_all_templates(&self, environment_config: &EnvironmentConfig) -> Result<(), DockerComposeTemplateError> {
    // ... existing code ...

    // Render .env file with tracker config from environment
    let tracker_api_admin_token = environment_config
        .tracker
        .as_ref()
        .map(|t| t.http_api.admin_token.as_str())
        .unwrap_or("MyAccessToken"); // Fallback for backward compatibility
    let env_content = EnvFileRenderer::render(&self.tera, tracker_api_admin_token)?;
    self.write_template(".env", &env_content)?;

    Ok(())
}
```

**Verification** (after running complete E2E workflow through step 5):

```bash
# Verify .env file in build directory
cat build/test-env/docker-compose/.env

# Verify .env file deployed to VM
ssh -i fixtures/testing_rsa ubuntu@$VM_IP "cat /opt/torrust/docker-compose/.env"

# Expected content:
# TORRUST_TRACKER_CONFIG_TOML_PATH=/etc/torrust/tracker/tracker.toml
# TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=TestAdminToken123
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

**Total**: ~8.5 hours

### Testing Strategy

Each phase includes:

1. **Build Verification**: Check files in `build/` directory
2. **Deployment Verification**: SSH into VM and verify files exist
3. **Runtime Verification**: Test services are running (docker compose, curl)

This three-level verification ensures issues are caught early.
