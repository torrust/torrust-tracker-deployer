# Implement Dynamic Image Detection for Vulnerability Scanning

**Issue**: #252
**Parent Epic**: #250 - Implement Automated Docker Image Vulnerability Scanning
**Related**:

- Epic specification: `docs/issues/250-epic-docker-image-vulnerability-scanning.md`
- Subissue 1: #251 - `docs/issues/251-implement-basic-trivy-scanning-workflow.md`
- Show command: #241 - `docs/issues/241-implement-environment-show-command.md`
- Docker Compose template: `templates/docker-compose/docker-compose.yml.tera`

## Overview

Enhance the Trivy scanning workflow to dynamically detect Docker images from environment configuration instead of using a hardcoded list. This eliminates manual maintenance and ensures the workflow automatically adapts when images change.

The solution leverages the `show` command (issue #241) to expose Docker image information stored in the environment data structure.

## Goals

- [ ] Convert hardcoded Docker Compose image references to Tera variables
- [ ] Store Docker image references in environment data structure
- [ ] Expose image information through `show` command
- [ ] Update Trivy workflow to dynamically detect images
- [ ] Eliminate manual image list maintenance

## 🏗️ Architecture Requirements

**DDD Layers**: Domain + Application + Infrastructure (CI/CD)

**Module Paths**:

- `src/domain/environment/mod.rs` - Add image information to domain model
- `src/infrastructure/external_tools/ansible/template/renderer/` - Update template variables
- `src/application/commands/show/` - Expose image information
- `templates/docker-compose/docker-compose.yml.tera` - Use variables for images
- `.github/workflows/docker-security-scan.yml` - Dynamic image detection

**Patterns**:

- Domain Layer: Value Objects for Docker image references
- Application Layer: Extend show command with image information
- Infrastructure Layer: Template rendering with constants
- CI/CD: Dynamic workflow based on environment output

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Domain model owns image configuration (immutable, validation)
- [ ] Application layer extracts and formats image data
- [ ] Infrastructure layer handles template rendering
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Image versions are **not** user-configurable (compatibility concerns)
- [ ] Images stored as constants in code (single source of truth)
- [ ] Template variables injected from constants (not from user config)
- [ ] Show command uses public API only (no template internals)
- [ ] Workflow uses CLI interface only (no direct file access)

### Anti-Patterns to Avoid

- ❌ Allowing users to override Docker image versions (compatibility risk)
- ❌ Duplicating image information across multiple files
- ❌ Hardcoding images in both template and workflow
- ❌ Workflow parsing template files directly (use show command)
- ❌ Exposing template implementation details through show command

## Specifications

### Docker Image References to Extract

From `templates/docker-compose/docker-compose.yml.tera`:

1. **Tracker**: `torrust/tracker:develop`
2. **MySQL**: `mysql:8.0`
3. **Grafana**: `grafana/grafana:11.4.0`
4. **Prometheus**: `prom/prometheus:v3.0.1`

### Domain Model Changes

#### New Value Objects

**Location**: `src/shared/docker_image.rs` (or `src/domain/docker_image.rs`)

```rust
use std::fmt;

/// Docker image reference with repository and tag
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DockerImage {
    repository: String,
    tag: String,
}

impl DockerImage {
    pub fn new(repository: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            tag: tag.into(),
        }
    }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the full image reference (e.g., "torrust/tracker:develop")
    pub fn full_reference(&self) -> String {
        format!("{}:{}", self.repository, self.tag)
    }
}

impl fmt::Display for DockerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.repository, self.tag)
    }
}

impl From<(&str, &str)> for DockerImage {
    fn from((repository, tag): (&str, &str)) -> Self {
        Self::new(repository, tag)
    }
}
```

#### Update Service Configurations

Each service configuration should own its Docker image information.

**Location**: `src/domain/tracker/config.rs`

```rust
use crate::shared::docker_image::DockerImage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    // ... existing fields ...
    pub docker_image: DockerImage,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            docker_image: DockerImage::new("torrust/tracker", "develop"),
        }
    }
}
```

**Location**: `src/domain/tracker/database/mod.rs`

```rust
use crate::shared::docker_image::DockerImage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    // ... existing fields ...
    pub docker_image: DockerImage,
}

impl DatabaseConfig {
    pub fn default_mysql() -> Self {
        Self {
            // ... existing defaults ...
            docker_image: DockerImage::new("mysql", "8.0"),
        }
    }
}
```

**Location**: `src/domain/prometheus/config.rs`

```rust
use crate::shared::docker_image::DockerImage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrometheusConfig {
    // ... existing fields ...
    pub docker_image: DockerImage,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            docker_image: DockerImage::new("prom/prometheus", "v3.0.1"),
        }
    }
}
```

**Location**: `src/domain/grafana/config.rs`

```rust
use crate::shared::docker_image::DockerImage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrafanaConfig {
    // ... existing fields ...
    pub docker_image: DockerImage,
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            docker_image: DockerImage::new("grafana/grafana", "11.4.0"),
        }
    }
}
```

### Template Changes

#### Update Docker Compose Template

**Location**: `templates/docker-compose/docker-compose.yml.tera`

The template context already provides service configurations through `DockerComposeContext`.
Update the template to use the new `docker_image` field from service configurations:

```yaml
services:
  tracker:
    image: {{ tracker.docker_image.repository }}:{{ tracker.docker_image.tag }}
    # ... rest of config ...

{% if database.driver == "mysql" %}
  mysql:
    image: {{ database.docker_image.repository }}:{{ database.docker_image.tag }}
    # ... rest of config ...
{% endif %}

{% if prometheus_config %}
  prometheus:
    image: {{ prometheus_config.docker_image.repository }}:{{ prometheus_config.docker_image.tag }}
    # ... rest of config ...
{% endif %}

{% if grafana_config %}
  grafana:
    image: {{ grafana_config.docker_image.repository }}:{{ grafana_config.docker_image.tag }}
    # ... rest of config ...
{% endif %}
```

#### Update Template Context

**Location**: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context/mod.rs`

The `DockerComposeContext` already has `prometheus_config` and `grafana_config` which will include the `docker_image` field.
For tracker and database, add dedicated image fields to the context:

```rust
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration (global - used by multiple services)
    pub database: DatabaseConfig,
    /// Tracker port configuration (global - used by multiple services)
    pub ports: TrackerPorts,
    /// Tracker configuration - includes docker_image field
    pub tracker: TrackerConfig,
    /// Prometheus configuration (optional) - includes docker_image field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus_config: Option<PrometheusConfig>,
    /// Grafana configuration (optional) - includes docker_image field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana_config: Option<GrafanaConfig>,
}
```

Update the builder in `context/builder.rs`:

```rust
impl DockerComposeContextBuilder {
    pub fn with_tracker(mut self, tracker: TrackerConfig) -> Self {
        self.tracker = Some(tracker);
        self
    }
}
```

````rust
impl ProjectGenerator for DockerComposeProjectGenerator {
    fn generate(&self, environment: &Environment) -> Result<(), ProjectGeneratorError> {
        // ... existing code ...

        let mut context = tera::Context::new();

        // Service configs already include docker_image field
        context.insert("tracker", &environment.tracker_config);
        context.insert("database", &environment.database_config);
        context.insert("prometheus_config", &environment.prometheus_config);
        context.insert("grafana_config", &environment.grafana_config);
### Show Command Enhancement

#### Add Image Information to Output

**Location**: `src/application/commands/show/formatter.rs`

Add section for Docker images by extracting from service configurations:

```rust
impl EnvironmentFormatter {
    fn format_docker_images(
        &self,
        tracker: &TrackerConfig,
        database: &DatabaseConfig,
        prometheus: Option<&PrometheusConfig>,
        grafana: Option<&GrafanaConfig>,
    ) -> String {
        let mut lines = vec![
            format!("Tracker: {}", tracker.docker_image.full_reference()),
            format!("Database: {}", database.docker_image.full_reference()),
        ];

        if let Some(prom) = prometheus {
            lines.push(format!("Prometheus: {}", prom.docker_image.full_reference()));
        }

        if let Some(graf) = grafana {
            lines.push(format!("Grafana: {}", graf.docker_image.full_reference()));
        }

        format!("Docker Images:\n  {}", lines.join("\n  "))
    }
}
````

#### Example Output

```bash
$ torrust-tracker-deployer show my-environment

Environment: my-environment
State: Running
Provider: LXD

# ... existing output ...

Docker Images:
  Tracker: torrust/tracker:develop
  MySQL: mysql:8.0
  Grafana: grafana/grafana:11.4.0
  Prometheus: prom/prometheus:v3.0.1

# ... rest of output ...
```

### Workflow Update

#### Update Trivy Workflow to Use Show Command

**Location**: `.github/workflows/docker-security-scan.yml`

Add dynamic image detection:

```yaml
jobs:
  extract-images:
    name: Extract Docker Images from Environment
    runs-on: ubuntu-latest
    outputs:
      images: ${{ steps.extract.outputs.images }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Build deployer CLI
        run: cargo build --release

      - name: Create test environment
        run: |
          # Create minimal environment config for image extraction
          cat > /tmp/test-env.json <<EOF
          {
            "name": "ci-image-scan",
            "provider": {
              "type": "lxd",
              "ssh": {
                "username": "ubuntu",
                "private_key_path": "/tmp/key",
                "public_key_path": "/tmp/key.pub"
              }
            }
          }
          EOF

          # Create environment (doesn't provision, just stores config)
          ./target/release/torrust-tracker-deployer create --env-file /tmp/test-env.json

      - name: Extract Docker images
        id: extract
        run: |
          # Run show command and parse output for Docker images
          images=$(./target/release/torrust-tracker-deployer show ci-image-scan \
            | grep -A 5 "Docker Images:" \
            | grep -E "^\s+(Tracker|Database|Grafana|Prometheus):" \

  scan-extracted-images:
    name: Scan Extracted Docker Images
    needs: extract-images
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        image: ${{ fromJson(needs.extract-images.outputs.images) }}
    steps:
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ matrix.image }}
          format: "sarif"
          output: "trivy-results.sarif"
          severity: "HIGH,CRITICAL"
          exit-code: "1"

      - name: Upload Trivy results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: "trivy-results.sarif"
```

## Implementation Plan

### Phase 1: Domain Model Changes (1.5 hours)

- [ ] Create `DockerImage` value object in `src/shared/docker_image.rs`
- [ ] Add `docker_image` field to `TrackerConfig` in `src/domain/tracker/config.rs`
- [ ] Add `docker_image` field to `DatabaseConfig` in `src/domain/tracker/database/mod.rs`
- [ ] Add `docker_image` field to `PrometheusConfig` in `src/domain/prometheus/config.rs`
- [ ] Add `docker_image` field to `GrafanaConfig` in `src/domain/grafana/config.rs`
- [ ] Implement `Default` trait with image constants for each service
- [ ] Add unit tests for `DockerImage` value object
- [ ] Update environment serialization tests

### Phase 2: Template Updates (1 hour)

- [ ] Update Docker Compose template to use service-specific image variables
- [ ] Verify template renderer already passes service configs to context
- [ ] Test template rendering produces correct output
- [ ] Verify existing E2E tests still pass
- [ ] Update template documentation

### Phase 3: Show Command Enhancement (1 hour)

- [ ] Extend show command formatter to extract and display Docker images from service configs
- [ ] Add unit tests for image formatting
- [ ] Test show command output includes images
- [ ] Update show command documentation
- [ ] Add E2E test for show command with images

### Phase 4: Workflow Update (1.5 hours)

- [ ] Add `extract-images` job to workflow
- [ ] Parse show command output for images
- [ ] Convert image list to JSON array
- [ ] Update `scan-extracted-images` job to use dynamic list
- [ ] Remove hardcoded image list from workflow
- [ ] Test workflow extracts correct images

### Phase 5: Testing and Documentation (1 hour)

- [ ] Test complete workflow end-to-end
- [ ] Verify workflow adapts when images change
- [ ] Update security documentation
- [ ] Add architecture decision record (ADR) if needed
- [ ] Update troubleshooting guide

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All unit tests pass
- [ ] All E2E tests pass
- [ ] Template rendering tests pass

**Functional Requirements**:

- [ ] Docker images stored in service configurations with constants
- [ ] Each service config (`TrackerConfig`, `DatabaseConfig`, `PrometheusConfig`, `GrafanaConfig`) has `docker_image` field
- [ ] Template uses service-specific image variables instead of hardcoded values
- [ ] Show command displays Docker image information from service configs
- [ ] Workflow dynamically extracts images from show command
- [ ] Changing an image constant in service config updates workflow automatically
- [ ] No manual image list maintenance required

**Domain Model**:

- [ ] `DockerImage` value object validates image references
- [ ] Images stored in each service configuration independently
- [ ] Default images set from constants in service `Default` implementations
- [ ] Images serialized/deserialized correctly as part of service configs

**Show Command**:

- [ ] Show command output includes "Docker Images:" section
- [ ] Images displayed with full reference (repo:tag)
- [ ] Output parseable by workflow script
- [ ] Works for all environment states

**Workflow**:

- [ ] `extract-images` job successfully creates environment
- [ ] Image extraction from show command works correctly
- [ ] JSON array conversion succeeds
- [ ] Workflow scans all extracted images
- [ ] No hardcoded image list remains

**Testing**:

- [ ] Unit tests for `DockerImage` value object
- [ ] Unit tests for show command image formatting
- [ ] E2E test for show command with images
- [ ] Manual test: change image constant and verify workflow updates

**Documentation**:

- [ ] Show command documentation updated
- [ ] Workflow documentation explains dynamic detection
- [ ] Architecture decision documented (if needed)
- [ ] Troubleshooting guide updated

## Related Documentation

- Show command specification: `docs/issues/241-implement-environment-show-command.md`
- Codebase architecture: `docs/codebase-architecture.md`
- DDD layer placement: `docs/contributing/ddd-layer-placement.md`
- Template system: `docs/contributing/templates/template-system-architecture.md`
- Docker Compose template: `templates/docker-compose/docker-compose.yml.tera`

## Notes

### Why Not Allow User Configuration?

Docker image versions are **not** user-configurable for several reasons:

1. **Compatibility**: Image versions must be tested together (tracker, database, monitoring)
2. **Support**: Troubleshooting requires known working combinations
3. **Security**: We control which versions are used and can enforce security updates

Users who need custom images can modify the code (it's open source) but this is intentionally not exposed as a configuration option.

### Alternative Approaches Considered

#### 1. Parse Template Files Directly

**Rejected**: Couples workflow to template implementation details. If we move or reorganize templates, workflow breaks.

#### 2. Separate Configuration File

**Rejected**: Creates duplication. Images would be defined in both template and config file. Single source of truth is better.

#### 3. Workflow Script Extracts from Template

**Rejected**: Requires workflow to understand Tera syntax. Using public CLI interface (show command) is cleaner.

### Dependency on Issue #241

This task depends on issue #241 (implement show command) being completed first. The show command provides the public interface for exposing environment information.

**If #241 is not ready**:

- Implement minimal show command just for images
- Focus on domain model and template changes
- Defer workflow update to later

### Future Enhancements

1. **Image Version Validation**: Add checks for known vulnerable versions
2. **Image Pinning**: Use SHA256 digests instead of tags for reproducibility
3. **Custom Image Registries**: Support private registries for enterprise deployments
4. **Image Build Information**: Include build date, source commit in environment data
