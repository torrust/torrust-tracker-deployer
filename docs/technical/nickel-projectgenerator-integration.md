# Integrating Nickel Templates into ProjectGenerator Architecture

## Overview

This document describes how to integrate the new Nickel-based template system into the existing ProjectGenerator infrastructure. The current system uses Tera templates; this integration will enable a gradual migration to Nickel while maintaining backward compatibility.

## Current Architecture

### Three-Layer Project Generator Pattern

```text
Domain (Config structs)
    ↓
Context Builder (PrometheusContext, TrackerContext, etc.)
    ↓
Renderer (PrometheusConfigRenderer, TrackerConfigRenderer, etc.)
    ↓
ProjectGenerator (orchestration)
    ↓
Output Files (prometheus.yml, tracker.toml, etc.)
```

### Current Tera-Based Flow

```rust
pub struct PrometheusProjectGenerator {
    build_dir: PathBuf,
    prometheus_renderer: PrometheusConfigRenderer,
}

impl PrometheusProjectGenerator {
    pub fn render(
        &self,
        prometheus_config: &PrometheusConfig,
        tracker_config: &TrackerConfig,
    ) -> Result<(), PrometheusProjectGeneratorError> {
        // 1. Create build directory
        let prometheus_build_dir = self.build_dir.join("prometheus");
        std::fs::create_dir_all(&prometheus_build_dir)?;

        // 2. Build context
        let context = Self::build_context(prometheus_config, tracker_config);

        // 3. Render using Tera
        self.prometheus_renderer.render(&context, &prometheus_build_dir)?;

        Ok(())
    }
}
```

## Integration Strategy

### Phase 1: Add Nickel Rendering Capability (Non-Breaking)

Create a new `NickelProjectGenerator` trait and implementations **alongside** existing Tera renderers.

#### New Module Structure

```text
src/infrastructure/templating/
├── nickel/                          # NEW: Nickel-based rendering
│   ├── mod.rs
│   ├── renderer.rs                  # Core Nickel execution
│   └── project_generators/          # Template-specific generators
│       ├── prometheus.rs
│       ├── tracker.rs
│       ├── docker_compose.rs
│       ├── ansible.rs
│       └── tofu.rs
├── prometheus/
│   └── template/renderer/
│       ├── project_generator.rs     # EXISTING: Tera-based
│       └── nickel.rs               # NEW: Nickel implementation
├── tracker/
│   └── ...
└── ...
```

#### New Traits

```rust
// src/infrastructure/templating/nickel/renderer.rs

use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for Nickel template rendering
pub struct NickelRenderConfig {
    /// Path to Nickel template (.ncl file)
    pub template_path: PathBuf,
    /// Output format (yaml, toml, hcl, env, json)
    pub output_format: TemplateFormat,
    /// Destination file path
    pub output_path: PathBuf,
    /// Optional: additional import paths for Nickel modules
    pub import_paths: Vec<PathBuf>,
}

/// Supported output formats
#[derive(Debug, Clone, Copy)]
pub enum TemplateFormat {
    Yaml,
    Toml,
    Hcl,
    Env,
    Json,
}

impl TemplateFormat {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Hcl => "hcl",
            Self::Env => "env",
            Self::Json => "json",
        }
    }
}

/// Errors that can occur during Nickel rendering
#[derive(thiserror::Error, Debug)]
pub enum NickelRenderError {
    #[error("Nickel CLI not found. Install with: cargo install nickel-lang-cli")]
    NickelNotInstalled,

    #[error("Nickel template evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Output directory creation failed: {0}")]
    DirectoryCreationFailed(#[from] std::io::Error),

    #[error("Script execution failed: {0}")]
    ScriptExecutionFailed(String),

    #[error("Invalid output format: {0}")]
    InvalidFormat(String),
}

/// Core Nickel template renderer
pub struct NickelRenderer {
    /// Path to provisioning directory containing templates
    provisioning_dir: PathBuf,
    /// Script directory for rendering helpers
    script_dir: PathBuf,
}

impl NickelRenderer {
    /// Creates a new NickelRenderer
    ///
    /// # Arguments
    /// * `provisioning_dir` - Root provisioning directory (e.g., "provisioning/")
    pub fn new<P: AsRef<Path>>(provisioning_dir: P) -> Self {
        let provisioning_dir = provisioning_dir.as_ref().to_path_buf();
        let script_dir = provisioning_dir.join("scripts");

        Self {
            provisioning_dir,
            script_dir,
        }
    }

    /// Renders a Nickel template to the specified format
    pub fn render(&self, config: NickelRenderConfig) -> Result<(), NickelRenderError> {
        // Verify output directory exists
        if let Some(parent) = config.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Choose appropriate rendering script
        let script_path = self.get_render_script(config.output_format);

        // Execute rendering script
        self.execute_render_script(&script_path, &config)?;

        Ok(())
    }

    /// Gets the appropriate rendering script for the output format
    fn get_render_script(&self, format: TemplateFormat) -> PathBuf {
        match format {
            TemplateFormat::Yaml => self.script_dir.join("nickel-render-yaml.sh"),
            TemplateFormat::Toml => self.script_dir.join("nickel-render-toml.sh"),
            TemplateFormat::Hcl => self.script_dir.join("nickel-render-hcl.sh"),
            TemplateFormat::Env => self.script_dir.join("nickel-render-env.sh"),
            TemplateFormat::Json => self.script_dir.join("nickel-render.sh"),
        }
    }

    /// Executes the rendering script
    fn execute_render_script(
        &self,
        script_path: &Path,
        config: &NickelRenderConfig,
    ) -> Result<(), NickelRenderError> {
        let output = Command::new("bash")
            .arg(script_path)
            .arg(&config.template_path)
            .arg(&config.output_path)
            .output()
            .map_err(|e| {
                NickelRenderError::ScriptExecutionFailed(format!(
                    "Failed to execute {}: {}",
                    script_path.display(),
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NickelRenderError::EvaluationFailed(stderr.to_string()));
        }

        Ok(())
    }
}

/// Trait for Nickel-based project generators
pub trait NickelProjectGenerator {
    /// Renders all templates for this component
    fn render_nickel(&self) -> Result<(), NickelRenderError>;
}
```

### Phase 2: Create Template-Specific Implementations

#### Example: Prometheus

```rust
// src/infrastructure/templating/prometheus/template/renderer/nickel.rs

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::tracker::TrackerConfig;
use crate::infrastructure::templating::nickel::{
    NickelRenderer, NickelRenderConfig, NickelRenderError, TemplateFormat,
};
use std::path::{Path, PathBuf};

pub struct PrometheusNickelProjectGenerator {
    build_dir: PathBuf,
    nickel_renderer: NickelRenderer,
}

impl PrometheusNickelProjectGenerator {
    const PROMETHEUS_BUILD_PATH: &'static str = "prometheus";
    const PROMETHEUS_TEMPLATE_PATH: &'static str = "provisioning/templates/prometheus/config.ncl";

    pub fn new<P: AsRef<Path>>(
        build_dir: P,
        provisioning_dir: P,
    ) -> Self {
        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            nickel_renderer: NickelRenderer::new(provisioning_dir),
        }
    }

    /// Renders Prometheus configuration using Nickel template
    ///
    /// This renders:
    /// - provisioning/templates/prometheus/config.ncl → prometheus/prometheus.yml
    pub fn render(
        &self,
        _prometheus_config: &PrometheusConfig,
        _tracker_config: &TrackerConfig,
    ) -> Result<(), NickelRenderError> {
        let prometheus_build_dir = self.build_dir.join(Self::PROMETHEUS_BUILD_PATH);

        let config = NickelRenderConfig {
            template_path: PathBuf::from(Self::PROMETHEUS_TEMPLATE_PATH),
            output_format: TemplateFormat::Yaml,
            output_path: prometheus_build_dir.join("prometheus.yml"),
            import_paths: vec![],
        };

        self.nickel_renderer.render(config)?;

        Ok(())
    }
}
```

#### Example: Docker Compose (Multi-Format)

```rust
// src/infrastructure/templating/docker_compose/template/renderer/nickel.rs

pub struct DockerComposeNickelProjectGenerator {
    build_dir: PathBuf,
    nickel_renderer: NickelRenderer,
}

impl DockerComposeNickelProjectGenerator {
    const DOCKER_COMPOSE_BUILD_PATH: &'static str = "docker-compose";
    const COMPOSE_TEMPLATE: &'static str = "provisioning/templates/docker-compose/compose.ncl";
    const ENV_TEMPLATE: &'static str = "provisioning/templates/docker-compose/env.ncl";

    pub fn render(
        &self,
        docker_config: &DockerComposeConfig,
    ) -> Result<(), NickelRenderError> {
        let docker_build_dir = self.build_dir.join(Self::DOCKER_COMPOSE_BUILD_PATH);

        // Render docker-compose.yml
        self.nickel_renderer.render(NickelRenderConfig {
            template_path: PathBuf::from(Self::COMPOSE_TEMPLATE),
            output_format: TemplateFormat::Yaml,
            output_path: docker_build_dir.join("docker-compose.yml"),
            import_paths: vec![],
        })?;

        // Render .env file
        self.nickel_renderer.render(NickelRenderConfig {
            template_path: PathBuf::from(Self::ENV_TEMPLATE),
            output_format: TemplateFormat::Env,
            output_path: docker_build_dir.join(".env"),
            import_paths: vec![],
        })?;

        Ok(())
    }
}
```

### Phase 3: Update Main ProjectGenerator (Optional)

Optionally update the main `ProjectGenerator` to support both Tera and Nickel:

```rust
// src/infrastructure/templating/project_generator.rs

pub struct ProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,

    // Tera-based (existing)
    prometheus_generator: PrometheusProjectGenerator,

    // Nickel-based (new)
    prometheus_nickel_generator: Option<PrometheusNickelProjectGenerator>,
}

impl ProjectGenerator {
    pub fn new_with_nickel<P: AsRef<Path>>(
        build_dir: P,
        template_manager: Arc<TemplateManager>,
        provisioning_dir: P,
    ) -> Self {
        let build_dir = build_dir.as_ref().to_path_buf();

        Self {
            build_dir: build_dir.clone(),
            template_manager,
            prometheus_generator: PrometheusProjectGenerator::new(&build_dir, template_manager),
            prometheus_nickel_generator: Some(
                PrometheusNickelProjectGenerator::new(&build_dir, provisioning_dir)
            ),
        }
    }

    /// Renders using Nickel if available, falls back to Tera
    pub fn render(
        &self,
        prometheus_config: &PrometheusConfig,
        tracker_config: &TrackerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Try Nickel first
        if let Some(ref nickel_gen) = self.prometheus_nickel_generator {
            return nickel_gen.render(prometheus_config, tracker_config)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }

        // Fall back to Tera
        self.prometheus_generator.render(prometheus_config, tracker_config)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
```

## Implementation Checklist

### For Each Template Type (Prometheus, Tracker, Docker Compose, Ansible, OpenTofu)

- [ ] Create `src/infrastructure/templating/{template_type}/template/renderer/nickel.rs`
- [ ] Implement `{Type}NickelProjectGenerator` struct
- [ ] Implement `render()` method with correct format
- [ ] Map template paths to output format
- [ ] Test rendering produces valid output
- [ ] Document any special considerations

### Core Infrastructure

- [ ] Create `src/infrastructure/templating/nickel/mod.rs`
- [ ] Create `src/infrastructure/templating/nickel/renderer.rs`
- [ ] Implement `NickelRenderer` with script execution
- [ ] Implement error handling and logging
- [ ] Add tracing instrumentation
- [ ] Add tests for script execution

### Integration Testing

- [ ] Test Nickel rendering produces same output as Tera (where applicable)
- [ ] Test error handling when nickel CLI not installed
- [ ] Test all template formats (YAML, TOML, HCL, ENV, JSON)
- [ ] E2E test generated configs work in deployment
- [ ] Test both Bash and Nushell script variants

### Documentation

- [ ] Document NickelRenderer API
- [ ] Document error types and handling
- [ ] Add examples for each template type
- [ ] Update contributing guidelines
- [ ] Document script requirements (bash, yq, jq, etc.)

## Key Considerations

### 1. Script Availability

The Nickel rendering scripts must be in `provisioning/scripts/`:

- `nickel-render-yaml.sh`
- `nickel-render-toml.sh`
- `nickel-render-hcl.sh`
- `nickel-render-env.sh`
- `nickel-render.sh` (generic)

Check existence and provide helpful error messages if missing.

### 2. Template Path Resolution

Templates are in `provisioning/templates/` relative to project root:

```rust
let template_path = ProjectRoot::path()
    .join("provisioning/templates/prometheus/config.ncl");
```

Ensure paths work both in development and when crate is used as a dependency.

### 3. Error Handling Strategy

Nickel rendering errors should be clear:

```text
Error: Nickel template evaluation failed for prometheus/config.ncl:
  Reason: Failed to import ../values/config.ncl: File not found

  Fix: Ensure provisioning/values/config.ncl exists and is readable
```

### 4. Context vs Nickel Values

**Tera approach** (existing):

```rust
let context = PrometheusContext {
    scrape_interval: "30s".to_string(),
    api_token: config.http_api.admin_token,
    api_port: config.http_api.bind_address.port(),
};
renderer.render(&context)?;
```

**Nickel approach** (new):

```rust
// No explicit context needed - Nickel imports from:
// - provisioning/values/config.ncl (user configuration)
// - provisioning/schemas/*.ncl (type contracts)
// - provisioning/validators/*.ncl (constraint validators)
// Just call the template directly
renderer.render(template_path)?;
```

The configuration comes from Nickel's import system, not passed as Rust struct.

### 5. Gradual Migration Path

**Option A: Parallel Systems** (recommended)

- Both Tera and Nickel implementations coexist
- Each template type migrates independently
- No breaking changes to existing code
- Transition over multiple releases

#### Option B: Progressive Replacement

- Start with simple templates (Prometheus)
- Graduate to complex ones (Tracker)
- Remove Tera when all templates migrated

#### Option C: Feature Flag

- Add `nickel-templates` Cargo feature
- Enable/disable per use case
- Production uses Tera, new deployments use Nickel

## Example: Complete Prometheus Migration

### Step 1: Create Nickel Renderer Module

```rust
// src/infrastructure/templating/nickel/mod.rs
pub mod renderer;
pub use renderer::{NickelRenderer, NickelRenderConfig, NickelRenderError, TemplateFormat};
```

### Step 2: Implement Prometheus

```rust
// src/infrastructure/templating/prometheus/template/renderer/nickel.rs
pub struct PrometheusNickelProjectGenerator { ... }
impl PrometheusNickelProjectGenerator {
    pub fn render(...) -> Result<(), NickelRenderError> { ... }
}
```

### Step 3: Add to Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_render_prometheus_config_to_yaml() {
        let renderer = PrometheusNickelProjectGenerator::new(
            "build/",
            "provisioning/",
        );

        let result = renderer.render(&prometheus_config, &tracker_config);

        assert!(result.is_ok());
        assert!(Path::new("build/prometheus/prometheus.yml").exists());
    }
}
```

### Step 4: Update ProjectGenerator

```rust
// src/infrastructure/templating/project_generator.rs
pub fn new_with_nickel(...) -> Self {
    Self {
        prometheus_nickel_generator: Some(PrometheusNickelProjectGenerator::new(...)),
        ...
    }
}
```

## Alternative: Simpler Wrapper

If full integration is complex, implement a simple wrapper:

```rust
// src/infrastructure/templating/nickel/simple_renderer.rs

pub fn render_nickel_template(
    template_path: &Path,
    output_format: &str,
    output_path: &Path,
) -> Result<(), NickelRenderError> {
    let script = match output_format {
        "yaml" => "nickel-render-yaml.sh",
        "toml" => "nickel-render-toml.sh",
        "hcl" => "nickel-render-hcl.sh",
        "env" => "nickel-render-env.sh",
        _ => return Err(NickelRenderError::InvalidFormat(output_format.to_string())),
    };

    let output = Command::new("bash")
        .arg(script)
        .arg(template_path)
        .arg(output_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NickelRenderError::EvaluationFailed(stderr.to_string()));
    }

    Ok(())
}
```

Then use it in ProjectGenerator:

```rust
pub fn render_prometheus_nickel(&self) -> Result<(), NickelRenderError> {
    render_nickel_template(
        &PathBuf::from("provisioning/templates/prometheus/config.ncl"),
        "yaml",
        &self.build_dir.join("prometheus/prometheus.yml"),
    )
}
```

## Testing Strategy

### Unit Tests

Test script execution and error handling:

```rust
#[test]
fn it_should_execute_nickel_render_script() {
    // Mock or use actual scripts
    // Verify output file created
}

#[test]
fn it_should_handle_missing_script() {
    // Verify helpful error message
}
```

### Integration Tests

Test actual Nickel rendering:

```rust
#[test]
fn it_should_render_prometheus_identically_to_tera() {
    // Render with Tera
    // Render with Nickel
    // Compare outputs
}
```

### E2E Tests

Test generated configs work in deployment:

```rust
#[tokio::test]
async fn it_should_deploy_with_nickel_generated_config() {
    // Generate config with Nickel
    // Deploy to test environment
    // Verify services run correctly
}
```

## Summary

This integration approach:

- ✅ **Non-breaking**: Nickel templates work alongside Tera
- ✅ **Gradual**: Migrate one template type at a time
- ✅ **Simple**: Leverages existing shell scripts
- ✅ **Testable**: Clear error handling and validation
- ✅ **Maintainable**: Keeps Rust code minimal
- ✅ **Flexible**: Easy to switch between or run both

The Nickel system is already complete with 9 templates and rendering scripts. This integration simply connects it to the existing ProjectGenerator infrastructure.
