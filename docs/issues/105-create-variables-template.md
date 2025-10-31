# Create Variables Template

**Issue**: [#105](https://github.com/torrust/torrust-tracker-deployer/issues/105)
**Parent Epic**: [#19](https://github.com/torrust/torrust-tracker-deployer/issues/19) - Refactor Ansible Templates to Variables Pattern
**Related**: [Parent Epic](./19-refactor-ansible-templates-variables-pattern.md), [Template System Architecture](../technical/template-system-architecture.md)

## Overview

Create the centralized `variables.yml.tera` template and supporting Rust infrastructure (context, wrapper, renderer) to consolidate Ansible playbook variables into a single file. This establishes the foundation for the variables pattern that will simplify future playbook additions.

## Goals

- [ ] **Variables Template**: Create `templates/ansible/variables.yml.tera` with system configuration variables
- [ ] **Context Layer**: Implement `AnsibleVariablesContext` with validation
- [ ] **Wrapper Layer**: Implement `AnsibleVariablesTemplate` for rendering
- [ ] **Renderer Layer**: Implement `VariablesTemplateRenderer` for orchestration
- [ ] **Integration**: Hook into `AnsibleTemplateRenderer::render()` workflow
- [ ] **Test Coverage**: Comprehensive unit tests for all components

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure
**Module Path**: `src/infrastructure/external_tools/ansible/template/`
**Pattern**: Template Wrapper + Context + Renderer (existing pattern)

### Module Structure Requirements

- [ ] Follow existing template architecture pattern (see `inventory/` and `firewall_playbook/` modules)
- [ ] Separate concerns: context validation, template rendering, file operations
- [ ] Use Arc<TemplateManager> for template access

### Architectural Constraints

- [ ] Context must validate SSH port range (reuse existing validation)
- [ ] Template wrapper uses domain TemplateEngine for rendering
- [ ] Renderer orchestrates context → wrapper → file I/O
- [ ] Error types use thiserror with proper context

### Anti-Patterns to Avoid

- ❌ Mixing variable validation with template rendering
- ❌ Direct file I/O in context or wrapper (use renderer)
- ❌ Hardcoding template paths (use TemplateManager)

## Specifications

### Variables Template File

**File**: `templates/ansible/variables.yml.tera`

```yaml
---
# Centralized Ansible Variables
# This file contains all dynamic variables used across Ansible playbooks.
# It follows the same pattern as OpenTofu's variables.tfvars.tera for consistency.
#
# NOTE: The inventory file (inventory.yml.tera) cannot use this file because
# Ansible inventories don't support vars_files. Only playbooks can use vars_files.

# System Configuration
ssh_port: { { ssh_port } }
# Future service variables can be added here:
# mysql_port: {{ mysql_port }}
# tracker_port: {{ tracker_port }}
# prometheus_port: {{ prometheus_port }}
# grafana_port: {{ grafana_port }}
```

**Why Not Include Connection Variables?**

- `ansible_host`, `ansible_port`, `ansible_ssh_private_key_file` belong in inventory
- Ansible inventories don't support `vars_files`
- These must stay in `inventory.yml.tera`

### Context Implementation

**File**: `src/infrastructure/external_tools/ansible/template/wrappers/variables/context.rs`

```rust
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur when creating an `AnsibleVariablesContext`
#[derive(Debug, Error)]
pub enum AnsibleVariablesContextError {
    /// Invalid SSH port
    #[error("Invalid SSH port: {0}")]
    InvalidSshPort(#[from] crate::infrastructure::external_tools::ansible::template::wrappers::inventory::context::AnsiblePortError),
}

/// Context for rendering the variables.yml.tera template
///
/// This context contains system configuration variables used across
/// Ansible playbooks (but NOT inventory connection variables).
#[derive(Serialize, Debug, Clone)]
pub struct AnsibleVariablesContext {
    /// SSH port to configure in firewall and other services
    ssh_port: u16,
}

impl AnsibleVariablesContext {
    /// Creates a new context with the specified SSH port
    pub fn new(ssh_port: u16) -> Result<Self, AnsibleVariablesContextError> {
        // Validate SSH port using existing validation
        crate::infrastructure::external_tools::ansible::template::wrappers::inventory::context::AnsiblePort::new(ssh_port)?;

        Ok(Self { ssh_port })
    }

    /// Get the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }
}
```

### Wrapper Implementation

**File**: `src/infrastructure/external_tools/ansible/template/wrappers/variables/mod.rs`

```rust
//! Wrapper for templates/ansible/variables.yml.tera

pub mod context;

use crate::domain::template::file::File;
use crate::domain::template::{write_file_with_dir_creation, FileOperationError, TemplateEngineError};
use std::path::Path;

pub use context::{AnsibleVariablesContext, AnsibleVariablesContextError};

/// Wrapper for the variables template
#[derive(Debug)]
pub struct AnsibleVariablesTemplate {
    content: String,
}

impl AnsibleVariablesTemplate {
    /// Creates a new template with variable substitution
    pub fn new(
        template_file: &File,
        context: AnsibleVariablesContext,
    ) -> Result<Self, TemplateEngineError> {
        let mut engine = crate::domain::template::TemplateEngine::new();
        let validated_content = engine.render(
            template_file.filename(),
            template_file.content(),
            &context,
        )?;

        Ok(Self { content: validated_content })
    }

    /// Render the template to a file
    pub fn render(&self, output_path: &Path) -> Result<(), FileOperationError> {
        write_file_with_dir_creation(output_path, &self.content)
    }
}
```

### Renderer Implementation

**File**: `src/infrastructure/external_tools/ansible/template/renderer/variables.rs`

```rust
//! Variables template renderer

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::domain::template::file::File;
use crate::domain::template::{FileOperationError, TemplateManager, TemplateManagerError};
use crate::infrastructure::external_tools::ansible::template::wrappers::variables::{
    AnsibleVariablesContext, AnsibleVariablesTemplate,
};

/// Errors for variables template rendering
#[derive(Error, Debug)]
pub enum VariablesTemplateError {
    #[error("Failed to get template path for '{file_name}': {source}")]
    TemplatePathFailed {
        file_name: String,
        #[source]
        source: TemplateManagerError,
    },

    #[error("Failed to read template file '{file_name}': {source}")]
    TemplateReadFailed {
        file_name: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create File object for '{file_name}': {source}")]
    FileCreationFailed {
        file_name: String,
        #[source]
        source: crate::domain::template::file::Error,
    },

    #[error("Failed to create variables template: {source}")]
    TemplateCreationFailed {
        #[source]
        source: crate::domain::template::TemplateEngineError,
    },

    #[error("Failed to render variables template: {source}")]
    TemplateRenderFailed {
        #[source]
        source: FileOperationError,
    },
}

/// Renders the variables.yml.tera template
pub struct VariablesTemplateRenderer {
    template_manager: Arc<TemplateManager>,
}

impl VariablesTemplateRenderer {
    const VARIABLES_TEMPLATE_FILE: &'static str = "variables.yml.tera";
    const VARIABLES_OUTPUT_FILE: &'static str = "variables.yml";

    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    pub fn render(
        &self,
        context: &AnsibleVariablesContext,
        output_dir: &Path,
    ) -> Result<(), VariablesTemplateError> {
        tracing::debug!("Rendering variables template with system configuration");

        // Get template path
        let template_path = self
            .template_manager
            .get_template_path(&Self::build_template_path())
            .map_err(|source| VariablesTemplateError::TemplatePathFailed {
                file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Read template content
        let template_content = std::fs::read_to_string(&template_path)
            .map_err(|source| VariablesTemplateError::TemplateReadFailed {
                file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Create File object
        let template_file = File::new(Self::VARIABLES_TEMPLATE_FILE, template_content)
            .map_err(|source| VariablesTemplateError::FileCreationFailed {
                file_name: Self::VARIABLES_TEMPLATE_FILE.to_string(),
                source,
            })?;

        // Create and render template
        let variables_template = AnsibleVariablesTemplate::new(&template_file, context.clone())
            .map_err(|source| VariablesTemplateError::TemplateCreationFailed { source })?;

        let output_path = output_dir.join(Self::VARIABLES_OUTPUT_FILE);
        variables_template
            .render(&output_path)
            .map_err(|source| VariablesTemplateError::TemplateRenderFailed { source })?;

        tracing::debug!(
            "Successfully rendered variables template to {}",
            output_path.display()
        );

        Ok(())
    }

    fn build_template_path() -> String {
        format!("ansible/{}", Self::VARIABLES_TEMPLATE_FILE)
    }
}
```

## Implementation Plan

This task establishes the foundation without breaking existing functionality. Each step keeps tests green.

### Phase 1: Create Template File (0.25 days)

**Safe Steps**:

1. Create `templates/ansible/variables.yml.tera` with SSH port variable
2. Run linters to validate YAML syntax: `cargo run --bin linter yaml`
3. Commit: "feat: [#19.1] add centralized variables template file"

**Green Check**: Linters pass, no code changes yet.

### Phase 2: Implement Context Layer (0.25 days)

**Safe Steps**:

1. Create directory: `src/infrastructure/external_tools/ansible/template/wrappers/variables/`
2. Create `context.rs` with `AnsibleVariablesContext` and validation
3. Create `mod.rs` in variables directory
4. Write unit tests for context creation and validation
5. Run tests: `cargo test`
6. Commit: "feat: [#19.1] add AnsibleVariablesContext with validation"

**Green Check**: All tests pass, context fully tested.

### Phase 3: Implement Wrapper Layer (0.25 days)

**Safe Steps**:

1. Update `wrappers/variables/mod.rs` with `AnsibleVariablesTemplate`
2. Write unit tests for template rendering
3. Run tests: `cargo test`
4. Commit: "feat: [#19.1] add AnsibleVariablesTemplate wrapper"

**Green Check**: All tests pass, wrapper fully tested.

### Phase 4: Implement Renderer Layer (0.25 days)

**Safe Steps**:

1. Create `src/infrastructure/external_tools/ansible/template/renderer/variables.rs`
2. Implement `VariablesTemplateRenderer`
3. Write unit tests for renderer
4. Run tests: `cargo test`
5. Commit: "feat: [#19.1] add VariablesTemplateRenderer"

**Green Check**: All tests pass, renderer fully tested in isolation.

### Phase 5: Update Module Exports (0.25 days)

**Safe Steps**:

1. Update `wrappers/mod.rs` to export `variables` module
2. Update `renderer/mod.rs` to export `variables` module
3. Build project: `cargo build`
4. Run tests: `cargo test`
5. Commit: "feat: [#19.1] export variables module from wrappers and renderer"

**Green Check**: Project compiles, all tests pass, new components accessible.

### Phase 6: Integrate into AnsibleTemplateRenderer (0.5 days)

**Safe Steps**:

1. Add `variables_renderer` field to `AnsibleTemplateRenderer` struct
2. Update constructor to initialize `variables_renderer`
3. Add `VariablesRenderingFailed` error variant to `ConfigurationTemplateError`
4. Add helper method `create_variables_context()` (private, doesn't break anything)
5. Build project: `cargo build`
6. Run tests: `cargo test`
7. Commit: "feat: [#19.1] add variables renderer to AnsibleTemplateRenderer"

**Green Check**: Builds successfully, existing tests still pass.

### Phase 7: Hook into Render Workflow (0.5 days)

**Safe Steps**:

1. Add variables rendering call to `AnsibleTemplateRenderer::render()` method
2. Place AFTER firewall rendering (existing functionality unaffected)
3. Run unit tests: `cargo test`
4. Run config tests: `cargo run --bin e2e-config-tests`
5. Commit: "feat: [#19.1] integrate variables rendering into main workflow"

**Green Check**: Config tests pass, `variables.yml` generated in build directory.

### Phase 8: Final Validation (0.25 days)

**Safe Steps**:

1. Run all linters: `cargo run --bin linter all`
2. Run unit tests: `cargo test`
3. Run config tests: `cargo run --bin e2e-config-tests`
4. Manually verify `build/e2e-config/ansible/variables.yml` exists with correct content
5. Check file permissions and format
6. Commit: "feat: [#19.1] finalize variables template implementation"

**Green Check**: All tests pass, linters happy, variables file generated correctly.

## Acceptance Criteria

### Quality Checks

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

### Template File

- [ ] `templates/ansible/variables.yml.tera` exists with SSH port variable
- [ ] Template has correct Tera syntax: `{{ ssh_port }}`
- [ ] Template includes comments explaining usage
- [ ] YAML linting passes

### Context Layer

- [ ] `AnsibleVariablesContext` created with SSH port validation
- [ ] Reuses existing `AnsiblePort` validation from inventory context
- [ ] Unit tests for valid and invalid port numbers
- [ ] Error type uses thiserror with proper context

### Wrapper Layer

- [ ] `AnsibleVariablesTemplate` renders template with context
- [ ] Uses domain `TemplateEngine` for rendering
- [ ] Unit tests for successful rendering
- [ ] Unit tests for rendering errors

### Renderer Layer

- [ ] `VariablesTemplateRenderer` orchestrates full rendering process
- [ ] Uses `TemplateManager` for template access
- [ ] Proper error handling with context preservation
- [ ] Unit tests for all error paths

### Integration

- [ ] Variables renderer integrated into `AnsibleTemplateRenderer`
- [ ] `variables.yml` generated in build directory during rendering
- [ ] Config tests pass: `cargo run --bin e2e-config-tests`
- [ ] Generated file has correct SSH port value

### Code Quality

- [ ] All unit tests pass: `cargo test`
- [ ] All linters pass: `cargo run --bin linter all`
- [ ] No compilation warnings
- [ ] Module exports updated correctly
- [ ] Error handling follows project conventions

### Documentation

- [ ] Rustdoc comments added for all public types and methods
- [ ] Template file includes descriptive header comments
- [ ] Code examples in documentation are valid
- [ ] Inline comments explain non-obvious logic
- [ ] Error types have clear descriptions

## Related Documentation

- [Parent Epic](./19-refactor-ansible-templates-variables-pattern.md)
- [Template System Architecture](../technical/template-system-architecture.md)
- [Contributing: Templates](../contributing/templates.md)
- [Error Handling Guide](../contributing/error-handling.md)

## Notes

### Design Decisions

1. **SSH Port Validation**: Reuse existing validation from inventory context to avoid duplication
2. **Context Scope**: Only system configuration variables (not connection details) - connection variables stay in inventory
3. **Integration Point**: Render after firewall playbook to avoid disrupting existing workflow
4. **Error Handling**: Use thiserror for structured errors with proper source chaining

### Why This Is Safe

- No existing code modified until Phase 6
- Each phase builds incrementally with test coverage
- Existing template rendering continues to work
- Variables template is additive (doesn't break anything)

### Future Extensibility

This foundation enables:

- Easy addition of new service variables (MySQL, Prometheus, etc.)
- Consistent pattern for all future playbooks
- Reduced Rust boilerplate per service
