# EPIC: Refactor Ansible Templates to Variables Pattern

**Issue**: [#19](https://github.com/torrust/torrust-tracker-deployer/issues/19)
**Type**: EPIC
**Parent Epic**: #16 - Finish ConfigureCommand - System Security Configuration
**Depends On**: #18 - Configure UFW Firewall
**Related**: [Parent Epic](./16-epic-finish-configure-command-system-security.md), [Template System Architecture](../technical/template-system-architecture.md)

## Overview

This epic tracks the refactoring of Ansible templates to use a centralized variables pattern similar to OpenTofu's `variables.tfvars.tera` approach. This consolidates multiple Tera templates into a single variables file, reducing complexity and establishing a consistent pattern for future Ansible template additions.

After implementing the security updates and firewall configuration, we now have 2 Tera templates (`inventory.yml.tera` and `configure-firewall.yml.tera`). This refactoring consolidates them into a single variables-based approach that will simplify future service additions.

## Sub-Tasks

This EPIC uses **vertical slice** approach - each sub-task is a complete, independently deployable increment:

- [ ] [#105](https://github.com/torrust/torrust-tracker-deployer/issues/105) - [Create Variables Template Infrastructure](./105-create-variables-template.md) - Complete implementation with unit tests and documentation
- [ ] [#106](https://github.com/torrust/torrust-tracker-deployer/issues/106) - [Convert Firewall to Variables Pattern](./106-convert-firewall-template-to-static.md) - Complete conversion including cleanup, documentation updates, and full validation

## Goals

- [ ] **Centralized Variables**: Create single `variables.yml.tera` for all Ansible variables
- [ ] **Template Consolidation**: Reduce template complexity through variables pattern
- [ ] **Consistent Pattern**: Match OpenTofu's elegant variables approach
- [ ] **Future-Proofing**: Establish pattern for easy addition of new services
- [ ] **Reduced Complexity**: Minimize Rust template handling boilerplate (~500 lines removed)
- [ ] **Maintain Functionality**: Ensure all existing functionality continues to work

## Specifications

### Current State Analysis

**Before Refactoring**:

- `templates/ansible/inventory.yml.tera` (variables: ansible_host, ansible_port, ansible_ssh_private_key_file)
- `templates/ansible/configure-firewall.yml.tera` (variables: ssh_port)
- Static templates: `install-docker.yml`, `install-docker-compose.yml`, `configure-security-updates.yml`, etc.

**After Refactoring**:

- `templates/ansible/inventory.yml.tera` (variables: ansible_host, ansible_port, ansible_ssh_private_key_file)
- `templates/ansible/variables.yml.tera` (all variables centralized)
- `templates/ansible/configure-firewall.yml` (static, uses vars_files)
- All other templates remain static

### New Centralized Variables Template

Create `templates/ansible/variables.yml.tera`:

### Refactored Firewall Template

Convert `templates/ansible/configure-firewall.yml.tera` → `templates/ansible/configure-firewall.yml`:

### Template Rendering Updates

Update Ansible template rendering logic to:

1. **Render variables.yml.tera** → `build/{env}/ansible/variables.yml`
2. **Copy static templates** as-is to `build/{env}/ansible/`
3. **Remove** rendering of `configure-firewall.yml.tera`

## 🔍 Architecture Context

### Template System Overview

The project uses a **two-phase template processing system** (see `docs/technical/template-system-architecture.md`):

1. **Phase 1 - Static File Copying**: Files without `.tera` extension are copied as-is (requires explicit registration)
2. **Phase 2 - Dynamic Rendering**: Files with `.tera` extension are processed for variable substitution (automatic)

### Current Two-Layer Architecture Problem

Each Tera template currently requires:

- **Wrapper Layer**: `*Template` + `*Context` classes with validation
- **Renderer Layer**: `*TemplateRenderer` with orchestration logic
- **Significant Boilerplate**: ~500+ lines per template

**Example**: `configure-firewall.yml.tera` requires:

- `FirewallPlaybookTemplate` + `FirewallPlaybookContext` (wrappers)
- `FirewallPlaybookTemplateRenderer` (renderer)
- Dedicated tests for each component

### Ansible Variable Loading Constraints

**CRITICAL**: Ansible inventory files **do NOT support `vars_files`** - that's only for playbooks.

**Solution**: Keep `inventory.yml.tera` as a Tera template for variable substitution. Only playbooks can use the centralized variables pattern.

**Updated Goal**: Reduce from 2 Tera templates → 2 Tera templates, but with simplified architecture:

- `inventory.yml.tera` - Remains Tera (inventory needs direct variable substitution)
- `variables.yml.tera` - New centralized variables file for playbooks
- `configure-firewall.yml.tera` → `configure-firewall.yml` - Convert to static (uses variables.yml)

## Implementation Approach

This epic uses **vertical slice methodology** - each sub-task is a complete, independently deployable increment that includes implementation, cleanup, documentation, and validation. This follows lean/agile principles rather than waterfall phases.

### Vertical Slice 1: Create Variables Template Infrastructure (Estimated: 2.5 days)

**Complete increment including:**

- Create `templates/ansible/variables.yml.tera` template file
- Implement `AnsibleVariablesContext` with validation
- Implement `AnsibleVariablesTemplate` wrapper
- Implement `VariablesTemplateRenderer` orchestrator
- Integrate into `AnsibleTemplateRenderer` workflow
- **Write unit tests for all components**
- **Add Rustdoc documentation**
- **Verify with linters and tests**

**Outcome**: System works with new variables infrastructure in place, fully tested and documented.

**See**: [Issue #105](./105-create-variables-template.md)

### Vertical Slice 2: Convert Firewall to Variables Pattern (Estimated: 4.5 days)

**Complete increment including:**

- Convert `configure-firewall.yml.tera` → `configure-firewall.yml` (static)
- Add `vars_files: [variables.yml]` to playbook
- Register as static template in `copy_static_templates()`
- Update `AnsibleClient` to accept extra arguments
- Update all call sites
- Remove firewall renderer from workflow
- **Delete old firewall renderer/wrapper code (~500 lines)**
- **Update template system architecture documentation**
- **Update contributing templates guide**
- **Update templates README**
- **Run full test suite (unit, config, linters)**
- **Verify build directory structure**
- **Document E2E test plan for human reviewer**

**Outcome**: System works with firewall using variables pattern, old code removed, documentation updated, all tests passing.

**See**: [Issue #106](./106-convert-firewall-template-to-static.md)

### Total Estimated Time

**7 days** (2.5 + 4.5)

### Why This Approach?

**Lean Principles**:

- Each task delivers complete, working functionality
- No separate "cleanup" or "testing" phases (waterfall anti-pattern)
- System is always in a working state after each task
- Can stop at any point with value delivered

**Benefits**:

- Faster feedback loops
- Reduced risk (smaller increments)
- Better quality (test/doc as you go, not later)
- More flexible (can reprioritize between tasks)

### Dependencies

- #106 requires #105 to be completed (variables template must exist)

## High-Level Architecture Changes

### Before Refactoring

```text
Current Architecture (2 Tera templates + per-template infrastructure):

templates/ansible/
  ├── inventory.yml.tera              (Tera: connection details)
  └── configure-firewall.yml.tera     (Tera: with ssh_port variable)

src/infrastructure/.../ansible/template/
  ├── wrappers/
  │   ├── inventory/
  │   │   ├── mod.rs                  (InventoryTemplate wrapper)
  │   │   └── context.rs              (InventoryContext)
  │   └── firewall_playbook/          (~150 lines)
  │       ├── mod.rs                  (FirewallPlaybookTemplate wrapper)
  │       └── context.rs              (FirewallPlaybookContext)
  └── renderer/
      ├── inventory.rs                (InventoryTemplateRenderer)
      └── firewall_playbook.rs        (~350 lines - dedicated renderer)
```

### After Refactoring

```text
New Architecture (2 Tera templates + centralized variables):

templates/ansible/
  ├── inventory.yml.tera              (Tera: connection details - UNCHANGED)
  ├── variables.yml.tera              (Tera: NEW - centralized system variables)
  └── configure-firewall.yml          (Static: uses vars_files)

src/infrastructure/.../ansible/template/
  ├── wrappers/
  │   ├── inventory/
  │   │   ├── mod.rs                  (InventoryTemplate wrapper - UNCHANGED)
  │   │   └── context.rs              (InventoryContext - UNCHANGED)
  │   └── variables/                  (NEW ~150 lines)
  │       ├── mod.rs                  (AnsibleVariablesTemplate wrapper)
  │       └── context.rs              (AnsibleVariablesContext)
  └── renderer/
      ├── inventory.rs                (InventoryTemplateRenderer - UNCHANGED)
      └── variables.rs                (NEW ~200 lines - VariablesTemplateRenderer)

[DELETED: ~500 lines]
  - wrappers/firewall_playbook/ directory
  - renderer/firewall_playbook.rs file
```

### Key Changes

- **Added**: `variables.yml.tera` template + supporting Rust infrastructure (~350 lines)
- **Converted**: `configure-firewall.yml.tera` → static `configure-firewall.yml` (no rendering needed)
- **Deleted**: Firewall playbook renderer and wrapper (~500 lines)
- **Net Result**: ~150 lines less code, simpler architecture

## Specifications Summary

For detailed specifications, see individual sub-task documentation files linked above.

### 1. Centralized Variables Template (`variables.yml.tera`)

- Contains all system configuration variables (ssh_port, future service ports)
- Follows OpenTofu's `variables.tfvars.tera` pattern
- Does NOT include connection details (those stay in inventory)
- **Details**: See [Issue #19.1](./19.1-create-variables-template.md)

### 2. Static Firewall Playbook (`configure-firewall.yml`)

- Converted from `.tera` template to static `.yml` file
- Uses `vars_files: [variables.yml]` to load centralized variables
- No dedicated Rust renderer needed
- **Details**: See [Issue #19.2](./19.2-convert-firewall-template-to-static.md)

### 3. Deleted Components

- `firewall_playbook.rs` renderer (~350 lines)
- `firewall_playbook/` wrapper directory (~150 lines)
- All related tests and imports
- **Details**: See [Issue #19.3](./19.3-clean-up-old-architecture.md)

### 4. Updated Documentation

- Template system architecture
- Contributing templates guide
- Templates README
- **Details**: See [Issue #19.4](./19.4-update-documentation.md)

### 5. Comprehensive Testing

- Unit tests for all new components
- Config tests for template generation
- Linters for code quality
- E2E test plan for human reviewer
- **Details**: See [Issue #19.5](./19.5-integration-testing-validation.md)

## Architecture Context

### Template System Overview

The project uses a **two-phase template processing system** (see `docs/technical/template-system-architecture.md`):

1. **Phase 1 - Static File Copying**: Files without `.tera` extension are copied as-is (requires explicit registration)
2. **Phase 2 - Dynamic Rendering**: Files with `.tera` extension are processed for variable substitution (automatic)

### Current Two-Layer Architecture Problem

Each Tera template currently requires:

- **Wrapper Layer**: `*Template` + `*Context` classes with validation
- **Renderer Layer**: `*TemplateRenderer` with orchestration logic
- **Significant Boilerplate**: ~500+ lines per template

**Example**: `configure-firewall.yml.tera` requires:

- `FirewallPlaybookTemplate` + `FirewallPlaybookContext` (wrappers)
- `FirewallPlaybookTemplateRenderer` (renderer)
- Dedicated tests for each component

### Ansible Variable Loading Constraints

**CRITICAL**: Ansible inventory files **do NOT support `vars_files`** - that's only for playbooks.

**Solution**: Keep `inventory.yml.tera` as a Tera template for variable substitution. Only playbooks can use the centralized variables pattern.

**Updated Goal**: Maintain 2 Tera templates with simplified architecture:

- `inventory.yml.tera` - Remains Tera (inventory needs direct variable substitution)
- `variables.yml.tera` - New centralized variables file for playbooks
- `configure-firewall.yml.tera` → `configure-firewall.yml` - Convert to static (uses variables.yml)

**Key Variables Structure**:

- `ssh_port` - System SSH port for firewall and service configuration
- Future service variables to be added as needed (mysql_port, tracker_port, etc.)

**Important Constraint**: Connection variables (`ansible_host`, `ansible_port`, `ansible_ssh_private_key_file`) remain in `inventory.yml.tera` because Ansible inventories don't support `vars_files`

## Detailed Implementation

All implementation details have been moved to the following sub-task documents:

### Task 1: Create Variables Template Infrastructure

See **[Issue #105 - Create Variables Template](./105-create-variables-template.md)** for complete implementation details including:

- Template file structure (`variables.yml.tera`)
- Context implementation (`AnsibleVariablesContext`)
- Wrapper implementation (`AnsibleVariablesTemplate`)
- Renderer implementation (`VariablesTemplateRenderer`)
- Module exports and integration
- Comprehensive test suite

### Task 2: Convert Firewall Template to Static (Complete Vertical Slice)

See **[Issue #106 - Convert Firewall Template to Static](./106-convert-firewall-template-to-static.md)** for complete implementation details including:

- Template conversion (remove `.tera`, add `vars_files`)
- Static file registration in `copy_static_templates()`
- `AnsibleClient` API enhancement (extra_args parameter)
- Call site updates across codebase
- Renderer workflow removal
- **Cleanup of old firewall renderer/wrapper code (~500 lines)**
- **Documentation updates (architecture, contributing guide, templates README)**
- **Full integration validation (unit tests, config tests, linters, E2E preparation)**

## Acceptance Criteria

Refer to individual sub-task acceptance criteria for detailed requirements:

- **Issue #105**: Variables template infrastructure created and tested
- **Issue #106**: Firewall playbook converted to static with complete cleanup, documentation updates, and validation

### High-Level Success Metrics

#### Architecture

- [ ] **Variables Template Created**: `templates/ansible/variables.yml.tera` with system configuration
- [ ] **Firewall Converted to Static**: `configure-firewall.yml` uses `vars_files: [variables.yml]`
- [ ] **Old Components Removed**: ~500 lines of firewall renderer/wrapper code deleted
- [ ] **Inventory Unchanged**: `inventory.yml.tera` remains (inventories don't support vars_files)

#### Functionality

- [ ] **All Tests Pass**: Unit, config, linters, and E2E tests validate the changes
- [ ] **Firewall Configures Correctly**: UFW rules apply using centralized variables
- [ ] **Pattern Established**: Clear documentation for adding future service variables

### Notes

For detailed implementation guidance, refer to the individual sub-task documents linked above. This epic tracks the overall architecture refactoring while sub-tasks contain the specific "baby steps" implementation plans.

**Important for AI Agents**: When implementing these changes, follow the execution order specified in the "Implementation Approach" section above. Each sub-task includes detailed phase-by-phase implementation guidance designed to keep tests green throughout the process.

## AI Agent Guidance

When implementing this EPIC, work through the sub-tasks in the specified order (#105 → #106). Each sub-task document contains:

- Detailed phase-by-phase implementation steps
- "Baby steps" approach to keep tests green
- Comprehensive validation checkpoints
- Anti-patterns to avoid

**Testing Split**:

- ✅ Agent can run: `cargo test`, `cargo run --bin e2e-config-tests`, `cargo run --bin linter all`
- ❌ Agent cannot run: `cargo run --bin e2e-tests-full` (requires LXD/VMs - human reviewer must execute)

**Note**: Task #106 is a complete vertical slice that includes implementation, cleanup of old code, documentation updates, and full validation all in one task.

## Detailed Acceptance Criteria

### Architecture

- [ ] **Variables Template Created**: `templates/ansible/variables.yml.tera` exists with system configuration variables
- [ ] **Firewall Template Converted**: `configure-firewall.yml` is static (no `.tera` extension) and uses `vars_files`
- [ ] **Inventory Template Unchanged**: `inventory.yml.tera` remains as Tera template (inventory limitation)
- [ ] **Old Components Removed**: `firewall_playbook` renderer and wrapper deleted (~500 lines)
- [ ] **AnsibleClient Enhanced**: Accepts optional extra arguments for passing variables file

### Functionality

- [ ] **Functionality Preserved**: All existing Ansible functionality continues to work
- [ ] **Variables Loaded**: Playbooks correctly load variables from `variables.yml` via `-e @variables.yml`
- [ ] **Firewall Configuration**: UFW firewall configures correctly using centralized variables
- [ ] **SSH Access Maintained**: SSH connectivity preserved throughout deployment
- [ ] **Pattern Established**: Clear pattern documented for adding future service variables

### Code Quality

- [ ] **Tests Pass**: All unit tests pass (`cargo test`)
- [ ] **Config Tests Pass**: E2E configuration tests pass (`cargo run --bin e2e-config-tests`)
- [ ] **Linters Pass**: All linters pass (`cargo run --bin linter all`)
- [ ] **No Compilation Errors**: Project compiles without warnings or errors
- [ ] **Test Coverage**: Comprehensive tests for `AnsibleVariablesContext`, `AnsibleVariablesTemplate`, and `VariablesTemplateRenderer`

### Documentation

- [ ] **Architecture Doc Updated**: `docs/technical/template-system-architecture.md` documents the variables pattern
- [ ] **Contributing Guide Updated**: `docs/contributing/templates.md` explains how to use centralized variables
- [ ] **Templates README Updated**: `templates/ansible/README.md` documents the variables pattern

## Notes

### **Timing Rationale**

This refactoring is perfectly timed because:

1. **Clear Justification**: We now have 2 Tera templates to consolidate
2. **Proven Pattern**: OpenTofu variables approach has been successful
3. **Future Value**: Establishes pattern before adding more services in roadmap 3.2+
4. **Natural Evolution**: Organic growth from 1→2 templates shows the pattern emerging

### E2E Validation (Human Reviewer)

- [ ] **Full E2E Tests Pass**: `cargo run --bin e2e-tests-full` completes successfully (local testing only)
- [ ] **Deployment Workflow Works**: Complete provision → configure → destroy cycle works
- [ ] **No Regressions**: All existing deployment features continue to function

## Related Documentation

- [Template System Architecture](../technical/template-system-architecture.md)
- [OpenTofu Variables Pattern](../../templates/tofu/lxd/variables.tfvars.tera)
- [Ansible Variables Documentation](https://docs.ansible.com/ansible/latest/playbook_guide/playbooks_variables.html)
- [Ansible vars_files Documentation](https://docs.ansible.com/ansible/latest/playbook_guide/playbooks_variables.html#defining-variables-in-files)
- [Parent Epic](./16-epic-finish-configure-command-system-security.md)

## Benefits

### **Architectural Consistency**

- Matches OpenTofu's successful `variables.tfvars.tera` pattern
- Consistent approach to variable management across infrastructure tools
- Single source of truth for environment-specific values

### **Reduced Complexity**

- Only 1 Tera template instead of multiple
- Less Rust boilerplate for template handling
- Simpler debugging and maintenance

### **Future-Proofing**

- Easy pattern for adding new services (just add variables, write static playbook)
- Scalable approach for the full roadmap implementation
- Clear separation of concerns (variables vs. logic)

### **Developer Experience**

- Easier to understand variable flow
- Centralized variable management
- Reduced cognitive overhead when adding new features

## Migration Strategy

### **Risk Mitigation**

- Incremental approach (one template at a time)
- Extensive testing at each step
- Preserve existing functionality throughout migration
- Clear rollback path if issues arise

### **Validation Strategy**

- Test each refactored template individually
- Run full E2E tests after each subtask
- Manual verification of critical functionality (SSH access, firewall rules)
- Compare before/after behavior for consistency

## 🤖 AI Agent Implementation Guidance

### Key Decisions Made

1. **Inventory Limitation**: Keep `inventory.yml.tera` as Tera template (Ansible inventories don't support `vars_files`)
2. **Variables Scope**: Only system configuration variables in `variables.yml` (connection details stay in inventory)
3. **AnsibleClient Generic**: Update signature to accept optional extra args for flexibility
4. **Clean Up Old Code**: Delete `firewall_playbook` renderer and wrapper as part of #106 (vertical slice)

### Implementation Order

Follow the subtasks in sequence:

1. **Task #105**: Create variables template infrastructure (new files, no breaking changes)
2. **Task #106**: Convert firewall template to static - complete vertical slice including:
   - Template conversion and API updates
   - Cleanup of old architecture
   - Documentation updates
   - Full integration validation

### Testing Strategy

**Agent Responsibilities**:

- Run unit tests: `cargo test`
- Run config tests: `cargo run --bin e2e-config-tests`
- Run linters: `cargo run --bin linter all`
- Fix any failures before creating PR

**Human Reviewer Responsibilities** (cannot be done by agent):

- Run full E2E tests: `cargo run --bin e2e-tests-full`
- Validate UFW firewall works in real VMs
- Verify no regressions in deployment workflow

### Common Pitfalls to Avoid

❌ **Don't**: Try to use `vars_files` in inventory.yml (not supported by Ansible)
❌ **Don't**: Forget to register `configure-firewall.yml` in `copy_static_templates`
❌ **Don't**: Leave old `firewall_playbook` renderer/wrapper code in place
❌ **Don't**: Hardcode `-e @variables.yml` in every AnsibleClient call (make it generic)

✅ **Do**: Keep inventory.yml.tera as Tera template
✅ **Do**: Update AnsibleClient signature to accept extra args
✅ **Do**: Delete old components after refactoring
✅ **Do**: Write comprehensive tests for new components

### Success Validation

Before submitting PR, verify:

- [ ] `cargo test` - All tests pass
- [ ] `cargo run --bin e2e-config-tests` - Config tests pass
- [ ] `cargo run --bin linter all` - All linters pass
- [ ] `cargo build` - No compilation errors
- [ ] No references to `firewall_playbook` in codebase (except this doc)
- [ ] `build/e2e-config/ansible/` contains `variables.yml` after config test run

## Notes

### **Timing Rationale**

This refactoring is perfectly timed because:

1. **Clear Justification**: We now have 2 Tera templates to consolidate
2. **Proven Pattern**: OpenTofu variables approach has been successful
3. **Future Value**: Establishes pattern before adding more services in roadmap 3.2+
4. **Natural Evolution**: Organic growth from 1→2 templates shows the pattern emerging

### **Implementation Order**

This should be the **final issue** in Epic 3.1 because:

- Security updates and firewall provide the business value
- Refactoring is architectural improvement/debt reduction
- Creates clean foundation for future Epic 3.2 work
- Demonstrates the pattern with real complexity

### **Success Metrics**

- Architecture: Simplified from per-template renderers to centralized variables ✅
- Functionality: All existing features continue to work ✅
- Maintainability: Easier to add future service variables ✅
- Consistency: Matches OpenTofu architectural pattern ✅
- Template Count: 2 Tera templates → 2 Tera templates (but simpler architecture)
