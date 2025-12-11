# Implementation Plans

This directory contains detailed implementation plans for complex changes that require multiple steps to complete.

## Purpose

When working on issues that involve:

- Significant architectural refactoring
- Multiple phases with dependencies
- Changes spanning many files across different layers
- Complex coordination between features

...we create detailed implementation plans here to:

- Track progress systematically
- Enable incremental commits with validation
- Document decision rationale for each step
- Provide clear recovery points if issues arise

## Difference from Other Documentation

- **`docs/roadmap/`**: High-level planned features and long-term vision
- **`docs/refactors/`**: Planned large-scale refactoring initiatives
- **`docs/implementation-plans/`**: Step-by-step execution plans for specific issues

## Structure

Each implementation plan document should include:

1. **Context**: Brief description of the issue and why the plan is needed
2. **Problem Analysis**: Architectural or technical issues being addressed
3. **Progress Tracking**: Checklist of all steps with completion status
4. **Phase Breakdown**: Logical grouping of related steps
5. **Detailed Steps**: For each step:
   - Clear commit message format
   - Specific actions to take
   - Files to create/modify/delete
   - Pre-commit protocol (tests + linters)
   - Time estimates

## Naming Convention

Files should be named: `issue-{number}-{short-description}.md`

Examples:

- `issue-220-test-command-architecture.md`
- `issue-315-database-migration-strategy.md`

## Workflow

1. Create the plan when issue complexity becomes apparent
2. Review and refine the plan before implementation
3. Follow the plan step-by-step with incremental commits
4. Update progress tracking as steps complete
5. Keep the plan updated if changes are needed during implementation
6. Archive completed plans in this directory for future reference
