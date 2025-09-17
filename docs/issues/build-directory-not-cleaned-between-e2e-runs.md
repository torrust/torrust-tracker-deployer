# Build Directory Not Cleaned Between E2E Test Runs

## Issue Summary

The E2E test suite does not properly clean the `build/` directory between test runs, causing stale cached template files to persist. This results in outdated configurations being used instead of the latest templates.

## Problem Description

When running E2E tests, the following sequence occurs:

1. **Template Rendering**: `RenderOpenTofuTemplatesStep` calls `TofuTemplateRenderer.render()`
2. **Directory Creation**: `create_build_directory()` uses `tokio::fs::create_dir_all()`
3. **File Copying**: Static templates are copied to `build/tofu/lxd/`
4. **Issue**: Existing files are NOT removed before copying new ones

## Evidence

- **Template Source**: `templates/tofu/lxd/main.tf` correctly uses `variable "instance_name"`
- **Build Output**: `build/tofu/lxd/main.tf` incorrectly shows `variable "container_name"`
- **Root Cause**: `tokio::fs::create_dir_all()` creates directories but does not clean existing content

## Impact

- Template changes are not reflected in E2E test runs
- Inconsistent behavior between fresh environments and cached environments
- Failed refactoring validation (instance name parameterization not working)
- Potential for stale configuration bugs in deployment workflows

## Technical Details

### Affected Components

- `src/tofu/template/renderer/mod.rs` - `TofuTemplateRenderer::create_build_directory()`
- `src/steps/rendering/opentofu_templates.rs` - `RenderOpenTofuTemplatesStep::execute()`
- E2E test workflow in `src/bin/e2e_tests.rs`

### Current Behavior

```rust
// In TofuTemplateRenderer::create_build_directory()
async fn create_build_directory(&self) -> Result<PathBuf, ProvisionTemplateError> {
    let build_tofu_dir = self.build_opentofu_directory();
    tokio::fs::create_dir_all(&build_tofu_dir)  // ❌ Does not clean existing content
        .await
        .map_err(|source| ProvisionTemplateError::DirectoryCreationFailed {
            directory: build_tofu_dir.display().to_string(),
            source,
        })?;
    Ok(build_tofu_dir)
}
```

### Expected Behavior

The build directory should be cleaned before template rendering to ensure fresh state:

```rust
async fn create_build_directory(&self) -> Result<PathBuf, ProvisionTemplateError> {
    let build_tofu_dir = self.build_opentofu_directory();

    // Clean existing content if directory exists
    if build_tofu_dir.exists() {
        tokio::fs::remove_dir_all(&build_tofu_dir).await?;
    }

    // Create fresh directory structure
    tokio::fs::create_dir_all(&build_tofu_dir).await?;
    Ok(build_tofu_dir)
}
```

## Proposed Solutions

### Option 1: Clean in Template Renderer (Recommended)

- Modify `create_build_directory()` to remove existing content
- Ensures fresh state for every template rendering operation
- Minimal code changes, focused responsibility

### Option 2: Clean in E2E Preflight Cleanup

- Add build directory cleanup to `preflight_cleanup::cleanup_lingering_resources()`
- Consistent with existing cleanup patterns
- Requires coordination between cleanup and rendering phases

### Option 3: Clean in RenderOpenTofuTemplatesStep

- Add cleanup logic directly in the rendering step
- More explicit control over cleanup timing
- Slightly more complex but very clear intent

## Testing Strategy

1. **Verify Issue**: Confirm stale files persist between E2E runs
2. **Implement Fix**: Apply chosen solution
3. **Validate Cleanup**: Ensure build directory is properly cleaned
4. **E2E Validation**: Run full E2E test suite to confirm template refresh
5. **Regression Testing**: Multiple consecutive E2E runs to verify consistency

## Related Issues

- Instance name parameterization refactor blocked by stale templates
- Potential for similar issues in Ansible template rendering
- Build directory management needs comprehensive review

## Priority

**High** - Blocks active refactoring work and could cause deployment inconsistencies.

## Labels

- `bug`
- `e2e-tests`
- `template-rendering`
- `build-system`
- `infrastructure`

---

**Created**: September 17, 2025  
**Status**: Open  
**Assignee**: TBD
