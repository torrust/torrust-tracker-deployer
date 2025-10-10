# Repository Rename: "Torrust Tracker Deploy" → "Torrust Tracker Deployer"

## 📋 Overview

This refactoring implements the repository rename from "Torrust Tracker Deploy" to "Torrust Tracker Deployer" as documented in [ADR: Repository Rename to Deployer](../decisions/repository-rename-to-deployer.md).

**Target Components:**

- Repository name and URL references
- Project name in documentation
- Package names and descriptions
- Code comments and module documentation
- GitHub Actions badge URLs
- External references (issues, discussions, commit messages)

**Scope:**

- Update all `torrust-tracker-deploy-rust-poc` → `torrust-tracker-deployer` references
- Update all "Torrust Tracker Deploy" → "Torrust Tracker Deployer" references
- Update GitHub repository URLs
- Update deprecation notices for previous PoC repositories (bash-poc and perl-poc)

## 📊 Progress Tracking

**Total Active Proposals**: 5
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 5

### Phase Summary

- **Phase 0 - GitHub Repository Rename (High Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Documentation Updates (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 2 - Code References (High Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 3 - Build Configuration (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 4 - PoC Deprecation Notices (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)

## 🎯 Key Changes Required

### 1. Repository URL Changes

- Old: `https://github.com/torrust/torrust-tracker-deploy-rust-poc`
- New: `https://github.com/torrust/torrust-tracker-deployer`

### 2. Project Name Changes

- Old: "Torrust Tracker Deploy"
- New: "Torrust Tracker Deployer"

### 3. Repository Slug Changes

- Old: `torrust-tracker-deploy-rust-poc`
- New: `torrust-tracker-deployer`

## 🚀 Refactoring Phases

---

## Phase 0: GitHub Repository Rename (Highest Priority)

This must be done first on GitHub as it affects all other changes. GitHub automatically creates redirects from the old URL.

### Proposal #0: Rename GitHub Repository

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: N/A  
**Completed**: -  
**Commit**: -

#### Problem

The current repository name includes the "-rust-poc" suffix which no longer reflects the maturity and production-ready status of the project.

#### Proposed Solution

1. Navigate to repository Settings on GitHub
2. Rename repository from `torrust-tracker-deploy-rust-poc` to `torrust-tracker-deployer`
3. GitHub will automatically create redirects from old URLs
4. Update local repository remotes

```bash
# After GitHub rename, update local clone
git remote set-url origin git@github.com:torrust/torrust-tracker-deployer.git
```

#### Rationale

- Must be done first as it affects all URL references
- GitHub's automatic redirects provide safety net
- Simple operation with immediate effect

#### Benefits

- ✅ Professional repository name
- ✅ Automatic URL redirects from old name
- ✅ Clear production status
- ✅ Shorter, cleaner URL

#### Implementation Checklist

- [ ] Create backup of local repository
- [ ] Rename repository on GitHub (Settings → Repository name)
- [ ] Verify automatic redirect works (test old URL)
- [ ] Update local repository remote URL
- [ ] Verify git operations work with new URL
- [ ] Notify team members to update their local clones

---

## Phase 1: Documentation Updates

Update all documentation files to reference the new repository name and URL.

### Proposal #1: Update Documentation Files

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #0  
**Completed**: -  
**Commit**: -

#### Problem

Documentation contains numerous references to the old repository name and URL that need updating for consistency and accuracy.

#### Affected Files

**README.md:**

- Line 1: GitHub Actions badge URLs (6 badges)
- Line 3: Main heading "# Torrust Tracker Deploy"

**docs/decisions/README.md:**

- Line 3: Project description
- Line 9: ADR table entry (already updated)

**docs/contributing/ directory:**

- `README.md`: GitHub issues/discussions URLs (lines 51-52)
- `branching.md`: Project name (line 3)
- `commit-process.md`: Project name (line 3)
- `debugging.md`: Project name (line 3)
- `error-handling.md`: Project name (lines 3, 617)
- `known-issues.md`: Project name (line 3)
- `linting.md`: Project name (line 3)
- `logging-guide.md`: Project name (line 3)
- `module-organization.md`: Project name (lines 3, 464)
- `spelling.md`: Project name (line 3)
- `templates.md`: Project name (line 3)
- `testing.md`: Project name (line 3)

**docs/ directory:**

- `console-commands.md`: Title and references (lines 1, 4)
- `deployment-overview.md`: References (line 4)
- `development-principles.md`: Project name (lines 3, 119)
- `e2e-testing.md`: Project name (line 3)
- `user-guide/template-customization.md`: Project name (line 3)
- `vm-providers.md`: Repository URL (line 106)

**docs/research/ directory:**

- `ansible-testing-strategy.md`: Project name (line 5)
- `docker-vs-lxd-ansible-testing.md`: Project references (lines 338, 545)
- `UX/console-app-output-patterns.md`: References (line 323)
- `UX/console-stdout-stderr-handling.md`: Project name (line 3)
- `UX/ux-design-discussion.md`: References (line 8)

**docs/research/mvvm-pattern-analysis/ directory:**

- `README.md`: Title and references (lines 1, 9, 17)
- `application-mvvm-analysis.md`: Multiple references
- `mvvm-pattern-overview.md`: References (line 87)
- `sessions/application-analysis-session.md`: References (line 10)
- `sessions/mvvm-learning-session.md`: Multiple references
- `conversation-logs/*.md`: Multiple references throughout

**docs/features/ directory:**

- `environment-state-management/feature-description.md`: Project name (lines 8, 281)

**docs/decisions/ directory:**

- `docker-testing-evolution.md`: Project name (line 19)
- `test-context-vs-deployment-environment-naming.md`: Project name (line 13)

**docs/github-actions-issues/ directory:**

- `docker-apt-cache-issue.md`: File paths and commit URLs (lines 11, 33, 100)

**scripts/setup/README.md:**

- Line 3: Project name

**templates/ansible/README.md:**

- Line 3: Project name

#### Proposed Solution

Systematically update all documentation files using find-and-replace:

1. Replace `torrust-tracker-deploy-rust-poc` → `torrust-tracker-deployer`
2. Replace `Torrust Tracker Deploy` → `Torrust Tracker Deployer`
3. Verify context for each replacement (some may be in ADR context where old name should remain)

Special cases:

- ADR files referencing the old name should keep them in historical context
- GitHub Actions workflow file paths will be updated automatically by badge URLs

#### Rationale

- Documentation is the primary interface for users and contributors
- Consistency across all documentation prevents confusion
- SEO and discoverability benefits

#### Benefits

- ✅ Consistent project identity across all documentation
- ✅ Better SEO with unified name
- ✅ Professional appearance
- ✅ Clear production status

#### Implementation Checklist

- [ ] Update README.md (badges and heading)
- [ ] Update all docs/contributing/\*.md files
- [ ] Update all docs/\*.md files
- [ ] Update docs/research/ subdirectories
- [ ] Update docs/features/ files
- [ ] Update docs/decisions/ files (preserve historical context)
- [ ] Update docs/github-actions-issues/ files
- [ ] Update scripts/setup/README.md
- [ ] Update templates/ansible/README.md
- [ ] Search for any missed references
- [ ] Run markdown linter
- [ ] Verify all internal links still work

---

## Phase 2: Code References

Update code comments, module documentation, and user-facing strings.

### Proposal #2: Update Code References

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Depends On**: Proposal #0  
**Completed**: -  
**Commit**: -

#### Problem

Code contains references to the old project name in comments, module docs, and user-facing strings.

#### Affected Files

**src/lib.rs:**

- Line 1: Module documentation comment

**src/main.rs:**

- Line 1: Module documentation comment
- Line 6: User-facing string in println!

**src/bin/ directory:**

- `linter.rs`: Lines 1, 9 (module docs)
- `e2e_config_tests.rs`: Lines 1, 85 (module docs and CLI about text)
- `e2e_provision_tests.rs`: Lines 1, 70 (module docs and CLI about text)
- `e2e_tests_full.rs`: Lines 1, 81 (module docs and CLI about text)

**src/infrastructure/persistence/filesystem/file_lock.rs:**

- Line 607: Issue reporting URL

#### Proposed Solution

Update all code references systematically:

```rust
// OLD:
//! Torrust Tracker Deploy

// NEW:
//! Torrust Tracker Deployer
```

```rust
// OLD:
println!("🏗️  Torrust Tracker Deploy");

// NEW:
println!("🏗️  Torrust Tracker Deployer");
```

```rust
// OLD:
#[command(about = "E2E tests for Torrust Tracker Deploy")]

// NEW:
#[command(about = "E2E tests for Torrust Tracker Deployer")]
```

```rust
// OLD:
- Please report at: https://github.com/torrust/torrust-tracker-deploy-rust-poc/issues

// NEW:
- Please report at: https://github.com/torrust/torrust-tracker-deployer/issues
```

#### Rationale

- User-facing strings should reflect the new name
- Documentation comments should be accurate
- Issue URLs should point to correct repository

#### Benefits

- ✅ Consistent naming in CLI output
- ✅ Accurate code documentation
- ✅ Correct issue reporting URLs
- ✅ Professional user experience

#### Implementation Checklist

- [ ] Update src/lib.rs module doc
- [ ] Update src/main.rs module doc and println!
- [ ] Update all src/bin/\*.rs files
- [ ] Update file_lock.rs issue URL
- [ ] Run cargo clippy
- [ ] Run cargo fmt
- [ ] Run cargo test
- [ ] Test CLI output manually
- [ ] Generate and verify cargo doc

---

## Phase 3: Build Configuration

Update build configuration files with new package names and descriptions.

### Proposal #3: Update Build Configuration

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #0  
**Completed**: -  
**Commit**: -

#### Problem

Build configuration files contain package descriptions and metadata referencing the old name.

#### Affected Files

**Cargo.toml:**

- Line 11: Package description

**packages/linting/Cargo.toml:**

- Line 5: Package description

#### Proposed Solution

Update package descriptions to reflect new project name:

```toml
# OLD: Cargo.toml
description = "Torrust Tracker Deployment Infrastructure with Ansible and OpenTofu"

# NEW: Cargo.toml (option 1)
description = "Torrust Tracker Deployer - Deployment Infrastructure with Ansible and OpenTofu"

# NEW: Cargo.toml (option 2 - more concise)
description = "Deployment infrastructure for Torrust Tracker using Ansible and OpenTofu"
```

```toml
# OLD: packages/linting/Cargo.toml
description = "Linting utilities for the Torrust Tracker Deploy project"

# NEW: packages/linting/Cargo.toml
description = "Linting utilities for the Torrust Tracker Deployer project"
```

#### Rationale

- Package metadata should reflect current project name
- Crates.io description (if published) should be accurate
- Cargo search results should show correct name

#### Benefits

- ✅ Accurate package metadata
- ✅ Professional crates.io presence (if published)
- ✅ Correct cargo search results
- ✅ Consistent branding

#### Implementation Checklist

- [ ] Update main Cargo.toml description
- [ ] Update packages/linting/Cargo.toml description
- [ ] Run cargo check
- [ ] Run cargo build
- [ ] Verify cargo metadata output
- [ ] Consider implications if published to crates.io

---

## Phase 4: Previous PoC Deprecation Notices

Add deprecation notices to previous proof-of-concept repositories.

### Proposal #4: Update Previous PoC Repositories

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #0, Proposal #1  
**Completed**: -  
**Commit**: -

#### Problem

The previous PoC repositories (bash-poc and perl-poc) should clearly indicate they are historical experiments and point users to the production-ready Torrust Tracker Deployer.

#### Affected Repositories

1. `torrust-tracker-deploy-bash-poc`
2. `torrust-tracker-deploy-perl-poc`

#### Proposed Solution

Add prominent deprecation notices to the README files of both repositories:

**For torrust-tracker-deploy-bash-poc:**

```markdown
# Torrust Tracker Deploy - Bash PoC (Historical)

> ⚠️ **PROOF OF CONCEPT - HISTORICAL REFERENCE ONLY**
>
> This repository contains the first proof of concept for Torrust Tracker deployment infrastructure.
> It is **no longer actively developed** and exists only as a historical reference.
>
> **👉 For production use, see [Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer)**
>
> ---

## Purpose

This was the first proof of concept in a series exploring deployment automation for the Torrust Tracker:

1. **[Bash/OpenTofu/cloud-init PoC](https://github.com/torrust/torrust-tracker-deploy-bash-poc)** (this repository)

   - **Technologies**: Bash scripts, OpenTofu, cloud-init, Docker Compose
   - **Focus**: Infrastructure as Code with libvirt/KVM and cloud deployment
   - **Status**: ✅ Historical reference - Completed its research goals

2. **[Perl/Ansible PoC](https://github.com/torrust/torrust-tracker-deploy-perl-poc)**

   - **Technologies**: Perl, OpenTofu, Ansible, libvirt/KVM, cloud-init, Docker Compose
   - **Focus**: Declarative configuration management with mature automation tools
   - **Status**: ✅ Historical reference - Completed its research goals

3. **[Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer)** (production)
   - **Technologies**: Rust, OpenTofu, Ansible, LXD, cloud-init, Docker Compose
   - **Focus**: Type-safe, performance-oriented deployment tooling
   - **Status**: 🚀 Production-ready - Active development

## What We Learned

This proof of concept successfully validated:

- ✅ Infrastructure as Code approach with OpenTofu/Terraform
- ✅ cloud-init for VM initialization
- ✅ libvirt/KVM for local testing
- ✅ Docker Compose for application orchestration
- ✅ Bash scripting feasibility for deployment automation

These learnings informed the design of the production Torrust Tracker Deployer.

## Migration Path

If you're using this PoC, migrate to the production [Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer):

1. Review the [Torrust Tracker Deployer README](https://github.com/torrust/torrust-tracker-deployer#readme)
2. Check the [migration guide](https://github.com/torrust/torrust-tracker-deployer/docs/migration-from-pocs.md) (if available)
3. Open an [issue](https://github.com/torrust/torrust-tracker-deployer/issues) if you need assistance

---

[Rest of original README content...]
```

**For torrust-tracker-deploy-perl-poc:**

```markdown
# Torrust Tracker Deploy - Perl PoC (Historical)

> ⚠️ **PROOF OF CONCEPT - HISTORICAL REFERENCE ONLY**
>
> This repository contains the second proof of concept for Torrust Tracker deployment infrastructure.
> It is **no longer actively developed** and exists only as a historical reference.
>
> **👉 For production use, see [Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer)**
>
> ---

## Purpose

This was the second proof of concept in a series exploring deployment automation for the Torrust Tracker:

1. **[Bash/OpenTofu/cloud-init PoC](https://github.com/torrust/torrust-tracker-deploy-bash-poc)**

   - **Technologies**: Bash scripts, OpenTofu, cloud-init, Docker Compose
   - **Focus**: Infrastructure as Code with libvirt/KVM and cloud deployment
   - **Status**: ✅ Historical reference - Completed its research goals

2. **[Perl/Ansible PoC](https://github.com/torrust/torrust-tracker-deploy-perl-poc)** (this repository)

   - **Technologies**: Perl, OpenTofu, Ansible, libvirt/KVM, cloud-init, Docker Compose
   - **Focus**: Declarative configuration management with mature automation tools
   - **Status**: ✅ Historical reference - Completed its research goals

3. **[Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer)** (production)
   - **Technologies**: Rust, OpenTofu, Ansible, LXD, cloud-init, Docker Compose
   - **Focus**: Type-safe, performance-oriented deployment tooling
   - **Status**: 🚀 Production-ready - Active development

## What We Learned

This proof of concept successfully validated:

- ✅ Ansible for declarative configuration management
- ✅ Perl as a viable deployment scripting language
- ✅ Integration of OpenTofu with configuration management tools
- ✅ Structured approach to deployment orchestration
- ✅ Importance of type safety and testing infrastructure

These learnings directly influenced the production Torrust Tracker Deployer's architecture.

## Migration Path

If you're using this PoC, migrate to the production [Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer):

1. Review the [Torrust Tracker Deployer README](https://github.com/torrust/torrust-tracker-deployer#readme)
2. Check the [migration guide](https://github.com/torrust/torrust-tracker-deployer/docs/migration-from-pocs.md) (if available)
3. Open an [issue](https://github.com/torrust/torrust-tracker-deployer/issues) if you need assistance

---

[Rest of original README content...]
```

#### Rationale

- Clear deprecation prevents users from investing in obsolete code
- Historical context helps understand the evolution of the project
- Migration path provides actionable guidance
- Links maintain discoverability of the production tool

#### Benefits

- ✅ Clear communication about repository status
- ✅ Guides users to production-ready solution
- ✅ Preserves historical context
- ✅ Maintains project continuity

#### Implementation Checklist

- [ ] Clone torrust-tracker-deploy-bash-poc locally
- [ ] Add deprecation notice to bash-poc README
- [ ] Commit and push bash-poc changes
- [ ] Clone torrust-tracker-deploy-perl-poc locally
- [ ] Add deprecation notice to perl-poc README
- [ ] Commit and push perl-poc changes
- [ ] Verify deprecation notices display correctly on GitHub
- [ ] Optional: Pin issues on both repos pointing to Deployer
- [ ] Optional: Add repository topics/tags indicating historical status

#### Testing Strategy

- Verify deprecation notices are prominent on GitHub
- Test all links to new repository
- Verify migration guidance is clear and actionable
- Check that badges and formatting render correctly

---

## 📈 Timeline

- **Start Date**: 2025-10-10
- **Phase 0 Target**: Same day (GitHub rename)
- **Phase 1 Target**: Within 1 day (documentation updates)
- **Phase 2 Target**: Within 1 day (code references)
- **Phase 3 Target**: Within 1 day (build config)
- **Phase 4 Target**: Within 2 days (PoC deprecation notices)
- **Estimated Completion**: 2025-10-12

## 📝 Notes

### Important Considerations

1. **GitHub Redirects**: GitHub automatically creates redirects, but it's still important to update all references for clarity
2. **Git History**: Renaming doesn't affect git history; all commits and history remain intact
3. **Local Clones**: Team members need to update their remote URLs after GitHub rename
4. **CI/CD**: GitHub Actions should continue to work, but verify all workflows after rename
5. **External Links**: Search engines and external sites may take time to update cached links

### Verification Checklist

After completing all phases:

- [ ] All GitHub Actions workflows pass
- [ ] All documentation links work correctly
- [ ] CLI tools display new name
- [ ] No broken references in search (grep entire repo)
- [ ] Cargo commands work correctly
- [ ] Tests pass
- [ ] Linters pass
- [ ] Previous PoCs display deprecation notices

### Communication Plan

1. Announce rename in GitHub Discussions
2. Update any external documentation or websites
3. Notify contributors via appropriate channels
4. Update any external references (package indexes, etc.)

## 🔗 Related Documents

- [ADR: Repository Rename to Deployer](../decisions/repository-rename-to-deployer.md)
- [Contributing Guidelines](../contributing/README.md)
- [Documentation Organization](../documentation.md)
