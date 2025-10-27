# Creating Roadmap Issues

This guide explains how to create and document issues from the project roadmap (`docs/roadmap.md`). It covers the complete workflow from selecting a roadmap task to creating GitHub issues with proper documentation.

## 📋 Overview

The roadmap implementation process involves:

1. **Issue Documentation** - Create detailed specification in `docs/issues/`
2. **GitHub Issues** - Create epic and/or task issues with proper hierarchy
3. **Linking** - Connect issues bidirectionally and update roadmap
4. **File Management** - Follow naming conventions
5. **Quality Checks** - Run linters before committing

## 🎯 Issue Hierarchy

Our roadmap uses a three-level hierarchy:

```text
Roadmap (Issue #1)
└── Epic (e.g., Issue #2: "Scaffolding for main app")
    ├── Task (e.g., Issue #3: "Setup logging for production CLI")
    ├── Task (e.g., Issue #4: "Add configuration file support")
    └── Task (e.g., Issue #5: "Add help command")
```

### When to Create Each Type

- **Epic Issue**: For roadmap sections (e.g., "1. Add scaffolding for main app")

  - Represents a major feature area or capability
  - Contains multiple related tasks
  - Links directly to roadmap issue (#1)

- **Task Issue**: For individual roadmap items (e.g., "1.1 Setup logging")
  - Represents a single, implementable unit of work
  - Links to parent epic issue
  - Has detailed specification document

## 📝 Complete Workflow

### Step 1: Create Issue Specification Document

Create a detailed specification in `docs/issues/` with temporary name:

```bash
# Copy the template to create your specification document
cp docs/issues/SPECIFICATION-TEMPLATE.md docs/issues/setup-logging-for-production-cli.md

# Edit the document to fill in all sections
vim docs/issues/setup-logging-for-production-cli.md
```

#### Document Structure

Use the template at [`docs/issues/SPECIFICATION-TEMPLATE.md`](../issues/SPECIFICATION-TEMPLATE.md) as your starting point. The template includes:

- **Header**: Issue number, parent epic, and related links
- **Overview**: Clear description of what the task accomplishes
- **Goals**: High-level objectives with checkboxes
- **Specifications**: Detailed technical specifications with code examples
- **Implementation Plan**: Phased breakdown with specific, actionable subtasks
- **Acceptance Criteria**: Clear definition of what "done" means
- **Related Documentation**: Links to relevant docs, ADRs, and examples
- **Notes**: Additional context or considerations

#### Key Principles

- **Be Specific**: Include code examples, command-line arguments, file structures
- **Be Actionable**: Break work into small, trackable tasks with checkboxes
- **Provide Context**: Link to related documentation, ADRs, and examples
- **Estimate Time**: Help others understand scope
- **Define Success**: Clear acceptance criteria (see [Common Acceptance Criteria](#common-acceptance-criteria) below)
- **Specify Architecture**: Include DDD layer, module path, and architectural constraints (see template guidance below)

#### Architectural Requirements

When creating issues that involve code changes, always specify:

- **DDD Layer**: Which layer the change belongs to (Presentation, Application, Domain, Infrastructure)
- **Module Path**: Exact location in the codebase (`src/{layer}/{module}/`)
- **Architectural Constraints**: Dependencies, patterns, and anti-patterns to follow
- **Related Documentation**: Link to [docs/codebase-architecture.md](../docs/codebase-architecture.md) and other relevant architectural guidance

This ensures AI assistants and contributors understand not just **what** to implement, but **where** and **how** to implement it within the project's architectural patterns.

#### Common Acceptance Criteria

Every issue should include these standard acceptance criteria in addition to task-specific criteria:

**Quality Checks** (applies to every commit and PR):

```markdown
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
```

This verification ensures:

- ✅ No unused dependencies (`cargo machete`)
- ✅ All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
- ✅ All unit tests pass (`cargo test`)
- ✅ Documentation builds successfully (`cargo doc`)
- ✅ All E2E tests pass (config, provision, full suite)

**Why Include This?**

- **Reminder**: Makes quality requirements explicit and visible
- **AI Assistants**: Ensures automated tools run all checks before submitting work
- **Consistency**: Every contributor knows the baseline quality standards
- **Early Detection**: Catches issues before PR review

The pre-commit script is the single source of truth for all quality checks. By including it in acceptance criteria, we make it clear that passing these checks is **required**, not optional.

### Step 2: Create GitHub Epic Issue (if needed)

If this is the first task in a roadmap section, create an epic issue first:

1. **Go to GitHub Issues**: `https://github.com/torrust/torrust-tracker-deployer/issues`

2. **Click "New Issue"**

3. **Fill in Epic Details**: Use the template at [`docs/issues/EPIC-TEMPLATE.md`](../issues/EPIC-TEMPLATE.md) and fill in:

   - Title with the roadmap section name
   - Overview describing the epic's purpose
   - Roadmap reference with quote from the roadmap
   - Initial task list (will be updated as tasks are created)
   - Parent link to roadmap issue (#1)

4. **Add Labels**: `epic`, `roadmap`

5. **Set Parent**: In the right sidebar, under "Development", link to parent issue (#1)

6. **Create Issue** - Note the issue number (e.g., #2)

### Step 3: Create GitHub Task Issue

1. **Go to GitHub Issues**: `https://github.com/torrust/torrust-tracker-deployer/issues`

2. **Click "New Issue"**

3. **Fill in Task Details**: Use the template at [`docs/issues/GITHUB-ISSUE-TEMPLATE.md`](../issues/GITHUB-ISSUE-TEMPLATE.md) and fill in:

   - Title with the task name from roadmap
   - Overview with brief description
   - Specification link (will be updated after file rename)
   - Implementation plan with phased tasks
   - Acceptance criteria
   - Related links to parent epic and specification document

4. **Add Labels**: `task`, `roadmap`, relevant technical labels (`rust`, `cli`, etc.)

5. **Set Parent**: Link to parent epic issue

6. **Create Issue** - Note the issue number (e.g., #3)

### Step 4: Update Issue Specification Document

Update the specification document with the issue number:

```markdown
# [Task Title]

**Issue**: #3
**Parent Epic**: #2 - [Epic Name]
**Related**: [Links to related issues]

[Rest of document...]
```

### Step 5: Rename File with Issue Number

Follow the naming convention: `{issue-number}-{description}.md`

```bash
# Example: Issue #3
mv docs/issues/setup-logging-for-production-cli.md \
   docs/issues/3-setup-logging-for-production-cli.md
```

**Naming Rules**:

- Start with issue number
- Use lowercase
- Separate words with hyphens
- Keep descriptive but concise
- Match branch naming convention

### Step 6: Update GitHub Task Issue

Update the task issue with the correct file link:

```markdown
## Specification

See detailed specification: [docs/issues/3-setup-logging-for-production-cli.md](../docs/issues/3-setup-logging-for-production-cli.md)
```

### Step 7: Update GitHub Epic Issue

Update the epic issue task list with the new task:

```markdown
## Tasks

- [ ] #3 - Setup logging for production CLI
- [ ] #X - [Next task name]
- [ ] #X - [Another task name]
```

### Step 8: Update Roadmap

Update `docs/roadmap.md` to link the epic and task issues:

```markdown
## 1. Add scaffolding for main app (Epic: [#2](https://github.com/torrust/torrust-tracker-deployer/issues/2))

- 1.1 Setup logging (Task: [#3](https://github.com/torrust/torrust-tracker-deployer/issues/3))
- 1.2 Add configuration file support
- 1.3 Add help command
```

### Step 9: Pre-Commit Verification

Follow the [commit process](./commit-process.md) and run the pre-commit verification script:

```bash
./scripts/pre-commit.sh
```

**All checks must pass** before proceeding.

### Step 10: Commit Changes

Follow the [conventional commit format](./commit-process.md):

```bash
# Stage files
git add docs/roadmap.md docs/issues/

# Commit (note: we're on main, not on issue branch yet)
git commit -m "docs: add issue specification for roadmap task X.Y

- Create epic issue #X for roadmap section X
- Create task issue #Y for task X.Y
- Add detailed specification document
- Update roadmap with issue links"

# Push to remote
git push
```

## 🔍 Complete Example

Here's a complete example following the workflow:

### Scenario

Implementing roadmap task **1.1 Setup logging**

### Execution

```bash
# 1. Create initial specification document from template
cp docs/issues/SPECIFICATION-TEMPLATE.md docs/issues/setup-logging-for-production-cli.md
vim docs/issues/setup-logging-for-production-cli.md
# ... fill in all sections: overview, goals, specs, implementation plan ...

# 2. Create GitHub Epic Issue #2: "Scaffolding for main app"
# - Browser: Create issue, link to #1, add labels
# - Note issue number: #2

# 3. Create GitHub Task Issue #3: "Setup logging for production CLI"
# - Browser: Create issue, link to #2, add labels
# - Note issue number: #3

# 4. Update specification with issue numbers
vim docs/issues/setup-logging-for-production-cli.md
# Add: **Issue**: #3
# Add: **Parent Epic**: #2

# 5. Rename file with issue number
mv docs/issues/setup-logging-for-production-cli.md \
   docs/issues/3-setup-logging-for-production-cli.md

# 6. Update GitHub Task Issue #3
# - Browser: Update specification link to correct filename

# 7. Update GitHub Epic Issue #2
# - Browser: Add task to task list: "- [ ] #3 - Setup logging..."

# 8. Update roadmap
vim docs/roadmap.md
# Update section 1.1 with issue link

# 9. Run pre-commit checks (see docs/contributing/commit-process.md)
./scripts/pre-commit.sh

# 10. Commit and push
git add docs/roadmap.md docs/issues/
git commit -m "docs: add issue specification for roadmap task 1.1

- Create epic issue #2 for roadmap section 1
- Create task issue #3 for task 1.1
- Add detailed specification document
- Update roadmap with issue links"
git push
```

## 📋 Pre-Commit Checklist

Before committing, verify:

- [ ] Issue specification document is complete and detailed
- [ ] File is named with issue number: `{number}-{description}.md`
- [ ] Epic issue exists and links to roadmap (#1)
- [ ] Task issue exists and links to epic
- [ ] Issue specification references correct issue numbers
- [ ] GitHub task issue links to correct file path
- [ ] GitHub epic issue includes task in task list
- [ ] `docs/roadmap.md` updated with issue links
- [ ] All links are correct and work
- [ ] All linters pass (`./scripts/pre-commit.sh`)
- [ ] Commit message follows [conventional format](./commit-process.md)

## 🚨 Common Mistakes to Avoid

### ❌ Don't Create Implementation Branch Before Specification is Committed

```bash
# Wrong: Creating branch before specification is committed to main
git checkout -b 3-setup-logging-for-production-cli
# Problem: Specification should be on main first
```

✅ **Correct**: Commit specification to `main` first

**Exception**: If you plan to use GitHub Copilot agents, create and push the branch after committing the specification but before assigning Copilot to the issue

### ❌ Don't Forget to Rename File

```bash
# Wrong: File still has temporary name
docs/issues/setup-logging-for-production-cli.md
# Problem: Should have issue number prefix
```

✅ **Correct**: `docs/issues/3-setup-logging-for-production-cli.md`

### ❌ Don't Skip Bidirectional Linking

```bash
# Wrong: Only roadmap links to issue
# Missing: Issue linking back to roadmap
# Missing: Epic listing the task
```

✅ **Correct**: All issues linked in both directions

### ❌ Don't Commit Without Linting

```bash
# Wrong: git add && git commit without running checks
# Problem: May have markdown, yaml, or other linting issues
```

✅ **Correct**: Always run `./scripts/pre-commit.sh` first

## 🔗 Related Documentation

- [Branching Conventions](./branching.md) - Branch naming for implementation
- [Commit Process](./commit-process.md) - Conventional commit format
- [Project Roadmap](../roadmap.md) - The roadmap being implemented
- [Development Principles](../development-principles.md) - Quality standards

## 📞 Questions?

If you have questions about:

- **Issue structure**: Review existing issues in `docs/issues/`
- **GitHub linking**: Check how #2 and #3 are connected
- **Commit format**: See [commit-process.md](./commit-process.md)
- **Linting**: See [linting.md](./linting.md)

## 🚀 Next Steps

After completing this process, you have two options for implementation:

### Option 1: Manual Implementation (Default)

1. **Create Implementation Branch**: `git checkout -b {issue-number}-{description}`
2. **Start Implementation**: Follow the plan in the specification document
3. **Track Progress**: Check off tasks in the GitHub issue as you complete them
4. **Create PR**: When complete, create pull request for review

### Option 2: Using GitHub Copilot Agent (Optional)

If you want to use GitHub Copilot to assist with implementation:

1. **Create Implementation Branch First**:

   ```bash
   git checkout -b {issue-number}-{description}
   git push -u origin {issue-number}-{description}
   ```

   **Important**: The branch must exist on GitHub before assigning Copilot to the issue.

2. **Assign Copilot to the Issue**:

   - Go to the GitHub issue page
   - Use the GitHub Copilot interface to assign the agent
   - Copilot will use the existing branch to create a PR with proposed changes

3. **Review Copilot's Work**:

   - Review the generated PR carefully
   - Test the changes locally
   - Request modifications if needed
   - Approve and merge when satisfied

4. **Track Progress**: Check off completed tasks in the GitHub issue

**Note**: GitHub Copilot agents work best with well-defined specifications, which is why we create detailed issue documents first.

## 🧹 Cleaning Up Closed Issues

Over time, as issues are completed and closed on GitHub, the `docs/issues/` folder can accumulate documentation for closed issues. Periodically clean up these files to keep the repository focused on active work.

### When to Clean Up

- After completing a major milestone or epic
- During regular maintenance cycles
- When preparing releases
- Whenever the `docs/issues/` folder feels cluttered

### Cleanup Process

1. **List Current Issue Files**:

   ```bash
   ls docs/issues/
   ```

   Issue files follow the format: `{issue-number}-{description}.md`

2. **Check Issue Status on GitHub**:

   Use the GitHub CLI to check the status of each issue:

   ```bash
   # Check individual issue
   gh issue view 21 --json state --jq .state

   # Check multiple issues at once
   for issue in 21 22 23 24; do
     state=$(gh issue view $issue --json state --jq .state 2>/dev/null || echo "NOT_FOUND")
     echo "$issue:$state"
   done
   ```

   Issue states:

   - `OPEN` - Issue is still active (keep the file)
   - `CLOSED` - Issue has been completed (remove the file)
   - `NOT_FOUND` - Issue doesn't exist (remove the file)

3. **Remove Closed Issue Files**:

   ```bash
   # Remove specific closed issue files
   cd docs/issues/
   rm -f 21-fix-e2e-infrastructure-preservation.md \
         22-rename-app-commands-to-command-handlers.md \
         23-add-clap-subcommand-configuration.md \
         24-add-user-documentation.md
   ```

4. **Verify Remaining Files**:

   ```bash
   ls docs/issues/
   ```

   Only open issues and template files should remain.

5. **Commit the Changes**:

   ```bash
   # Stage the deletions
   git add docs/issues/

   # Commit with descriptive message
   git commit -m "chore: remove closed issue documentation files

   Removed X closed issue documentation files from docs/issues/:
   - #21: fix-e2e-infrastructure-preservation
   - #22: rename-app-commands-to-command-handlers
   - #23: add-clap-subcommand-configuration
   - #24: add-user-documentation

   All these issues have been closed on GitHub and no longer need
   local documentation files.

   Remaining open issues: #16, #17, #18, #19, #34"

   # Push to remote
   git push
   ```

### Important Notes

- **Keep Template Files**: Never delete `EPIC-TEMPLATE.md`, `GITHUB-ISSUE-TEMPLATE.md`, or `SPECIFICATION-TEMPLATE.md`
- **Verify Before Deleting**: Always double-check issue status before removing files
- **Document Removals**: Use descriptive commit messages listing which issues were removed
- **Team Communication**: Consider notifying the team before large cleanup operations
- **Git History**: Closed issue documentation remains available in git history if needed for future reference

---

By following this workflow, you ensure that roadmap tasks are properly documented, tracked, and implemented with high quality and consistency.
