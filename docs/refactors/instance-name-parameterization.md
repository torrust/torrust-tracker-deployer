# Instance Name Parameterization Refactor

## Overview

This refactor aims to eliminate the hardcoded "torrust-vm" instance name throughout the codebase and make it configurable from the top-level main function in `src/bin/e2e_tests.rs`.

## Motivation

Currently, the instance name "torrust-vm" is hardcoded in multiple places across the codebase, including:

- Infrastructure configuration files (OpenTofu templates)
- Rust code literals and variables
- Ansible configurations
- Potentially workflows and comments

This creates several issues:

- Difficulty running multiple instances simultaneously
- Hard to customize instance names for different environments
- Tight coupling between configuration and implementation

## Summary of "torrust-vm" Usage Analysis

Based on a comprehensive search through the codebase, **61 occurrences** of "torrust-vm" were found across 4 main categories:

### 1. Configuration Files (2 files, 7 occurrences)

- `templates/tofu/lxd/main.tf` - OpenTofu variable default value
- `templates/ansible/inventory.yml.tera` - Ansible inventory template (5 places: comments, host names, lxc commands)

### 2. Rust Code Literals/Variables (5 files, 9 occurrences)

- `src/e2e/tasks/cleanup_infrastructure.rs` - E2E cleanup tasks
- `src/template/file.rs` - Template file testing
- `src/command_wrappers/opentofu/json_parser.rs` - JSON parser unit tests (5 places)
- `src/ansible/template/renderer/inventory.rs` - Inventory renderer tests (2 places)
- `tests/template_integration.rs` - Integration test assertions

### 3. GitHub Workflows (2 files, 13 occurrences)

- `.github/workflows/test-e2e.yml` - E2E workflow (3 places)
- `.github/workflows/test-lxd-provision.yml` - LXD provision workflow (10 places)

### 4. Documentation & Comments (4 files, 30 occurrences)

- `README.md` - Usage examples (5 places)
- `docs/e2e-testing.md` - Testing documentation (3 places)
- `docs/tofu-lxd-configuration.md` - Configuration documentation (22 places)
- `docs/refactors/instance-name-parameterization.md` - This refactor document (3 places)

## Goals

1. **Centralize Configuration**: Define the instance name in one place (`src/bin/e2e_tests.rs`)
2. **Parameter Propagation**: Pass the instance name down through the call chain
3. **Template Parameterization**: Convert static templates to Tera templates where needed
4. **Maintain Separation**: Use OpenTofu variables file strategy to avoid over-templating main.tf

## Strategy

### Template Strategy

- Keep `templates/tofu/lxd/main.tf` as a static template
- Create a new `variables.tf.tera` template for OpenTofu variables
- Inject the instance name through the variables file rather than directly into main.tf

### Refactor Direction

Start from low-level infrastructure details and work up to higher abstractions:

1. OpenTofu variable configuration
2. Template rendering system
3. Command wrappers
4. Higher-level orchestration
5. Top-level main function
6. Workflows (excluding `.github/workflows/test-lxd-provision.yml` - independent test for GitHub runners contract)

## Success Criteria

- [ ] Instance name defined once in `src/bin/e2e_tests.rs`
- [ ] All hardcoded "torrust-vm" references eliminated
- [ ] OpenTofu uses variables file for parameterization
- [ ] All tests (unit, linting, e2e) pass at each step
- [ ] No regression in functionality
- [ ] Clean separation of concerns maintained

## Implementation Notes

- Use baby steps: maximum 2 files changed per step
- Run full validation (linters, unit tests, e2e tests) after each step
- Maintain backwards compatibility during transition
- Document each step thoroughly
- Ask for confirmation before implementing each step
- Update execution plan after completing small tests
