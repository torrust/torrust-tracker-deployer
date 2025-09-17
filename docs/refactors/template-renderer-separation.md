# Template Renderer Simplification Refactoring Plan

## 📋 Overview

**Problem**: Both `AnsibleTemplateRenderer` and `TofuTemplateRenderer` have too many responsibilities and hardcoded template-specific logic. With more Tera templates, they would become unmaintainable.

**Philosophy**: Discover the design step-by-step through iterative refactoring, rather than over-engineering upfront abstractions.

## 🎯 Goals

1. **Extract collaborators** for template-specific logic (one per Tera template)
2. **Simplify renderers** by removing hardcoded template knowledge
3. **Learn from experience** before creating abstractions
4. **Validate incrementally** - each step should improve the design

## 🚀 Iterative Approach

### Phase 1: Ansible Template Renderer (`estimated: 3-4 hours`)

**Focus**: Extract collaborator for `inventory.yml.tera` template handling.

#### Step 1.1: Extract InventoryTemplateRenderer

- **File**: `src/ansible/inventory_template_renderer.rs`
- **Purpose**: Handle all `inventory.yml.tera` specific logic
- **Status**: ❌ Not Started

**What to extract from AnsibleTemplateRenderer:**

```rust
// From: AnsibleTemplateRenderer::render_inventory_template()
// To: InventoryTemplateRenderer

pub struct InventoryTemplateRenderer {
    template_manager: Arc<TemplateManager>,
}

impl InventoryTemplateRenderer {
    pub fn render(
        &self,
        context: &InventoryContext,
        output_dir: &Path
    ) -> Result<(), InventoryTemplateError> {
        // All the inventory.yml.tera specific logic
        // - Get template path
        // - Read template content
        // - Create File object
        // - Create InventoryTemplate
        // - Render to output file
    }
}
```

#### Step 1.2: Update AnsibleTemplateRenderer to use collaborator

- **File**: `src/ansible/template_renderer.rs`
- **Purpose**: Compose with InventoryTemplateRenderer instead of handling directly
- **Status**: ❌ Not Started

```rust
pub struct AnsibleTemplateRenderer {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    inventory_renderer: InventoryTemplateRenderer,  // NEW
}

impl AnsibleTemplateRenderer {
    pub async fn render(&self, inventory_context: &InventoryContext) -> Result<(), ConfigurationTemplateError> {
        let build_ansible_dir = self.create_build_directory().await?;

        // Use collaborator instead of hardcoded logic
        self.inventory_renderer.render(inventory_context, &build_ansible_dir)?;

        self.copy_static_templates(&build_ansible_dir).await?;

        Ok(())
    }
}
```

### Phase 2: OpenTofu Template Renderer (`estimated: 3-4 hours`)

**Focus**: Apply lessons learned to extract collaborator for `cloud-init.yml.tera`.

#### Step 2.1: Extract CloudInitTemplateRenderer

- **File**: `src/tofu/cloud_init_template_renderer.rs`
- **Purpose**: Handle all `cloud-init.yml.tera` specific logic
- **Status**: ❌ Not Started

```rust
pub struct CloudInitTemplateRenderer {
    template_manager: Arc<TemplateManager>,
}

impl CloudInitTemplateRenderer {
    pub fn render(
        &self,
        ssh_credentials: &SshCredentials,
        output_dir: &Path
    ) -> Result<(), CloudInitTemplateError> {
        // All the cloud-init.yml.tera specific logic
        // - Get template path
        // - Read template content
        // - Create CloudInitContext from SSH credentials
        // - Create CloudInitTemplate
        // - Render to output file
    }
}
```

#### Step 2.2: Update TofuTemplateRenderer to use collaborator

- **File**: `src/tofu/template_renderer.rs`
- **Purpose**: Compose with CloudInitTemplateRenderer instead of handling directly
- **Status**: ❌ Not Started

```rust
pub struct TofuTemplateRenderer {
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
    ssh_credentials: SshCredentials,
    cloud_init_renderer: CloudInitTemplateRenderer,  // NEW
}

impl TofuTemplateRenderer {
    pub async fn render(&self) -> Result<(), ProvisionTemplateError> {
        let build_tofu_dir = self.create_build_directory().await?;

        self.copy_templates(&["main.tf"], &build_tofu_dir).await?;

        // Use collaborator instead of hardcoded logic
        self.cloud_init_renderer.render(&self.ssh_credentials, &build_tofu_dir)?;

        Ok(())
    }
}
```

### Phase 3: Design Reevaluation (`estimated: 1-2 hours`)

#### Step 3.1: Assess Current State

**Questions to answer:**

1. **Are the renderers now focused?** Do they only handle directory creation and static file copying?
2. **Are the collaborators cohesive?** Does each handle exactly one Tera template?
3. **Is there duplication?** Are there patterns emerging between the collaborators?
4. **Do we need more extraction?** Are there still types with too many responsibilities?

#### Step 3.2: Decide Next Steps

**Possible outcomes:**

✅ **Good enough**: If types are focused and maintainable, stop here  
🔄 **More extraction needed**: Extract more collaborators (e.g., for static file handling)  
🏗️ **Abstract patterns**: If 3+ similar collaborators exist, consider trait extraction

## 📊 Progress Tracking

### Phase 1: Ansible (0/2 completed)

- [ ] Extract InventoryTemplateRenderer collaborator
- [ ] Update AnsibleTemplateRenderer to use collaborator

### Phase 2: OpenTofu (0/2 completed)

- [ ] Extract CloudInitTemplateRenderer collaborator
- [ ] Update TofuTemplateRenderer to use collaborator

### Phase 3: Reevaluation (0/2 completed)

- [ ] Assess current design state
- [ ] Decide on next iteration (if needed)

## 🎯 Success Criteria

**After Phase 1:**

- `AnsibleTemplateRenderer` no longer contains `render_inventory_template()` method
- Inventory-specific logic is isolated in `InventoryTemplateRenderer`
- All existing tests pass

**After Phase 2:**

- `TofuTemplateRenderer` no longer contains `render_cloud_init_template()` method
- Cloud-init-specific logic is isolated in `CloudInitTemplateRenderer`
- All existing tests pass

**After Phase 3:**

- Clear decision on whether further refactoring is needed
- Documentation updated with lessons learned

## 🔍 What We'll Learn

### From Ansible Refactoring:

- How to cleanly separate template-specific logic
- What the collaborator interface should look like
- Error handling patterns
- Testing strategies

### From OpenTofu Refactoring:

- Whether the Ansible patterns work for different contexts
- If there are common patterns worth abstracting
- Edge cases we missed in the first iteration

### From Reevaluation:

- Whether this level of separation is sufficient
- If trait extraction would actually help
- What other responsibilities might need extraction

## 🚫 What We're NOT Doing

- ❌ **No traits initially** - Discover patterns first
- ❌ **No generic renderers** - Keep it concrete
- ❌ **No complex error hierarchies** - Simple, focused errors
- ❌ **No over-abstraction** - Extract only what we actually need

## 📝 Implementation Notes

### Error Handling Strategy

Keep errors simple and focused per collaborator:

```rust
// For InventoryTemplateRenderer
#[derive(Error, Debug)]
pub enum InventoryTemplateError {
    #[error("Failed to read inventory template: {0}")]
    TemplateReadError(#[from] std::io::Error),

    #[error("Failed to render inventory template: {0}")]
    RenderError(#[from] crate::template::TemplateEngineError),

    // etc.
}
```

### Constructor Changes

Update constructors to include collaborators:

```rust
impl AnsibleTemplateRenderer {
    pub fn new(build_dir: PathBuf, template_manager: Arc<TemplateManager>) -> Self {
        let inventory_renderer = InventoryTemplateRenderer::new(template_manager.clone());

        Self {
            build_dir,
            template_manager,
            inventory_renderer,
        }
    }
}
```

### Testing Strategy

Test collaborators in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_renderer_should_render_template() {
        let renderer = InventoryTemplateRenderer::new(mock_template_manager());
        let context = mock_inventory_context();
        let temp_dir = TempDir::new().unwrap();

        let result = renderer.render(&context, temp_dir.path());

        assert!(result.is_ok());
        assert!(temp_dir.path().join("inventory.yml").exists());
    }
}
```

---

**Next Steps**: Start with Phase 1.1 - Extract `InventoryTemplateRenderer` from `AnsibleTemplateRenderer`.
