---
name: verify-template-changes
description: Fast workflow for verifying Tera template modifications using the render command without provisioning infrastructure. Use when modifying templates in templates/, testing template rendering, iterating on template development, or running E2E template verification. Triggers on "verify template", "test template changes", "template iteration", "render template", or "template development".
metadata:
  author: torrust
  version: "1.0"
---

# Verify Template Changes

This skill provides a fast development workflow for contributors modifying Tera templates. Instead of the slow full deployment cycle, use `create → render` for instant feedback on template changes.

## Why Use This Workflow?

```text
Full Deployment: create → provision (~5 min) → configure (~3 min) → release (~2 min)
Template Verify: create → render (~2 sec) → inspect → modify → render (~2 sec)

Time saved: ~10 minutes per iteration
```

**Benefits:**

- No infrastructure provisioning (skips LXD VM creation, cloud API calls)
- No remote operations (skips SSH, Ansible execution)
- Instant feedback (only template rendering, ~1-2 seconds)
- Repeatable (use `--force` to regenerate instantly)

## Workflow: Quick Template Iteration

### Step 1: Create a Test Environment

```bash
torrust-tracker-deployer create environment -f envs/lxd-local-example.json
```

### Step 2: Render Artifacts

```bash
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./template-test
```

### Step 3: Inspect the Templates You Modified

```bash
# Check specific rendered artifacts
cat template-test/ansible/playbooks/your-playbook.yml
cat template-test/docker-compose/docker-compose.yml
cat template-test/tracker/tracker.toml

# Validate syntax
yamllint template-test/ansible/playbooks/your-playbook.yml
```

### Step 4: Modify and Re-render

```bash
# Edit templates in templates/ directory
vim templates/ansible/playbooks/your-playbook.yml.tera

# Re-render with --force (no need to delete output)
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./template-test \
  --force
```

### Step 5: Compare Before/After

```bash
# Save old render, re-render, and diff
cp -r template-test /tmp/old-render
# (make template changes)
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./template-test \
  --force

diff -r /tmp/old-render/ template-test/
```

## Workflow: E2E Template Verification

When you're confident in template changes, verify them with the full deployment:

```bash
# Step 1: Quick render to verify syntax
torrust-tracker-deployer render \
  --env-name lxd-local-example \
  --instance-ip 10.0.0.100 \
  --output-dir ./quick-check

# Step 2: Lint generated artifacts
yamllint quick-check/ansible/playbooks/your-new-playbook.yml

# Step 3: If templates look good, test with real deployment
torrust-tracker-deployer provision lxd-local-example
torrust-tracker-deployer configure lxd-local-example
# ... continue full workflow
```

## Tips for Contributors

1. **Use render for template iteration** — Much faster than full deployment workflow
2. **Test with real configs** — Use actual environment configs from `envs/` directory
3. **Verify all artifact types** — Check not just the template you modified but all dependent files
4. **Use force flag** — `--force` for quick iteration without manual deletion
5. **Compare before/after** — Keep old renders to diff against new changes
6. **Validate syntax** — Use linters (yamllint, taplo) on generated YAML/TOML files
7. **Test multiple configs** — Render with different configurations to catch edge cases

## Output Directory Structure

```text
<output-dir>/
├── opentofu/           # Infrastructure code
├── ansible/            # Configuration playbooks
│   ├── inventory.ini
│   ├── ansible.cfg
│   └── playbooks/
├── docker-compose/     # Service definitions
│   ├── docker-compose.yml
│   └── .env
├── tracker/            # Tracker configuration
│   └── tracker.toml
├── prometheus/         # Monitoring (if configured)
│   └── prometheus.yml
├── grafana/            # Dashboards (if configured)
│   └── provisioning/
├── caddy/              # Reverse proxy (if HTTPS)
│   └── Caddyfile
└── backup/             # Backup scripts (if enabled)
    └── backup.sh
```

## Related Documentation

- **Template System Architecture**: `docs/contributing/templates/template-system-architecture.md`
- **Working with Tera Templates**: see the `work-with-tera-templates` skill in `dev/infrastructure/`
- **Render Command User Guide**: `docs/user-guide/commands/render.md`

## See Also

- For **rendering artifacts for manual deployment or validation**: see the `render-tracker-artifacts` skill in `usage/operations/`
