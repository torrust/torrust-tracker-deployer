# Draft Issue: Automated Verification of AI Training Dataset Freshness

**Status:** Draft  
**Priority:** Low  
**Category:** Developer Experience / Testing  
**Related:** Issue #339 - AI Training Resources

## Problem

The AI training dataset in `docs/ai-training/dataset/` requires manual regeneration whenever:

- Input examples in `dataset/environment-configs/` change
- Template files in `templates/` are modified
- Rendering code changes in a way that affects output

Currently, developers must manually run `scripts/generate-ai-training-outputs.sh` to regenerate the `rendered-templates/` directory.

**Risk:** The rendered outputs can become stale without anyone noticing, leading to:

- Outdated training data for AI models
- Inconsistencies between documentation and actual behavior
- Confusion for users examining the examples

## Current Workaround

Manual regeneration by running:

```bash
./scripts/generate-ai-training-outputs.sh
```

## Challenge

Templates contain dynamic timestamps that make direct directory comparison difficult:

```text
# Generated: 2026-02-13T10:02:22Z
```

Any comparison mechanism would need to:

- Strip/normalize timestamps before comparison, OR
- Use timestamp-agnostic comparison methods, OR
- Include deterministic timestamp generation for testing

## Potential Solutions

### Option 1: CI Workflow with Timestamp Normalization

Create a GitHub Actions workflow that:

1. Strips timestamp lines matching pattern `# Generated: YYYY-MM-DDTHH:MM:SSZ`
2. Compares normalized outputs with existing rendered templates
3. Fails if differences exist (beyond timestamps)

**Pros:** Catches stale outputs automatically  
**Cons:** Requires maintaining comparison logic, may have false positives

### Option 2: Metadata File with Input Hashes

Store a `.metadata.json` file alongside rendered outputs:

```json
{
  "generated_at": "2026-02-13T10:02:22Z",
  "input_hashes": {
    "environment_configs": "sha256:abc123...",
    "templates": "sha256:def456...",
    "rendering_code": "sha256:ghi789..."
  },
  "tool_version": "0.1.0"
}
```

CI checks if relevant paths changed since metadata was created.

**Pros:** Fast, deterministic, no false positives  
**Cons:** Requires metadata management, doesn't detect code logic changes

### Option 3: Deterministic Timestamp Flag

Add `--mock-timestamp` or `--fixed-timestamp` flag to render command:

```bash
torrust-tracker-deployer render ... --fixed-timestamp "2024-01-01T00:00:00Z"
```

Use this flag for test/CI renders to enable exact comparison.

**Pros:** Simple, enables deterministic testing  
**Cons:** CI outputs would have fake timestamps (but that's fine for testing)

### Option 4: Git-Based Staleness Check

Simple bash script that checks:

```bash
# Get last modification time of rendered outputs
rendered_mtime=$(find docs/ai-training/dataset/rendered-templates -type f -printf '%T@\n' | sort -n | tail -1)

# Compare with modification times of inputs and templates
inputs_mtime=$(find docs/ai-training/dataset/environment-configs templates src/infrastructure/templating -type f -printf '%T@\n' | sort -n | tail -1)

if (( inputs_mtime > rendered_mtime )); then
  echo "⚠️  Rendered outputs may be stale. Run: ./scripts/generate-ai-training-outputs.sh"
  exit 1
fi
```

**Pros:** Very simple, no changes to application code  
**Cons:** Doesn't catch all cases (e.g., code logic changes without file mtime change)

### Option 5: Pre-commit Hook (Auto-regeneration)

Add regeneration to pre-commit checks:

```bash
./scripts/generate-ai-training-outputs.sh
git add docs/ai-training/dataset/rendered-templates
```

**Pros:** Always up-to-date automatically  
**Cons:** Slow pre-commit (renders 15 examples), may be disruptive

## Recommendation

**Short-term (easiest):** Option 4 (Git-based staleness check)

- Minimal implementation effort
- Good enough for catching most cases
- Can be added to pre-commit or CI immediately

**Long-term (best):** Option 3 (Deterministic timestamp flag)

- Enables proper testing and comparison
- Clean separation of test vs production behavior
- More maintainable and robust

## Implementation Estimate

- **Option 4:** ~30 minutes (bash script + CI integration)
- **Option 3:** ~2-3 hours (add flag, update templates, tests)
- **Option 2:** ~4-5 hours (metadata generation, comparison logic)
- **Option 1:** ~5-6 hours (normalization logic, CI workflow, edge cases)

## Related Files

- Script: `scripts/generate-ai-training-outputs.sh`
- Inputs: `docs/ai-training/dataset/environment-configs/*.json`
- Outputs: `docs/ai-training/dataset/rendered-templates/*/`
- Templates: `templates/**/*.tera`
- Rendering code: `src/infrastructure/templating/**`

## Notes

- Not urgent since dataset is primarily for AI training, not runtime behavior
- Manual regeneration is acceptable for now (done periodically)
- Worth revisiting when/if we automate AI model training pipeline
