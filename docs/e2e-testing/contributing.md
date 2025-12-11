# Contributing to E2E Tests

This guide explains how to extend and modify E2E tests when adding new features or making changes.

## 🏗️ Infrastructure Changes

For OpenTofu, LXD, or cloud-init modifications:

1. **Update infrastructure lifecycle tests** in `src/bin/e2e_infrastructure_lifecycle_tests.rs`
2. **Add validation methods** for new infrastructure components
3. **Test locally**: `cargo run --bin e2e-infrastructure-lifecycle-tests`
4. **Verify CI passes** on `.github/workflows/test-e2e-infrastructure.yml`

### Example: Adding New Cloud-init Validation

```rust
// In e2e_infrastructure_lifecycle_tests.rs

async fn validate_new_cloud_init_feature(
    ssh_client: &SshClient,
) -> Result<(), Box<dyn std::error::Error>> {
    // Add your validation logic
    let output = ssh_client.execute("check-new-feature")?;
    assert!(output.contains("expected-result"));
    Ok(())
}
```

## 🔧 Deployment Workflow Changes

For Ansible playbooks or software installation modifications:

1. **Update deployment workflow tests** in `src/bin/e2e_deployment_workflow_tests.rs`
2. **Add validation methods** for new software components
3. **Update Docker image** in `docker/provisioned-instance/` if needed
4. **Test locally**: `cargo run --bin e2e-deployment-workflow-tests`
5. **Verify CI passes** on `.github/workflows/test-e2e-deployment.yml`

### Example: Adding New Software Installation Test

```rust
// In e2e_deployment_workflow_tests.rs

async fn validate_new_software(
    ssh_client: &SshClient,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate software is installed
    let version_output = ssh_client.execute("new-software --version")?;
    assert!(version_output.contains("v1.2.3"));

    // Validate software is configured correctly
    let config_output = ssh_client.execute("cat /etc/new-software/config")?;
    assert!(config_output.contains("expected-config"));

    Ok(())
}
```

## 🔄 End-to-End Integration

For comprehensive changes affecting multiple components:

1. **Test with complete workflow suite**: `cargo run --bin e2e-complete-workflow-tests`
2. **Verify both infrastructure and deployment suites pass independently**
3. **Update documentation** to reflect changes
4. **Consider split approach**: Can the change be tested in isolated suites?

## 🎯 Test Design Principles

When adding or modifying E2E tests, follow these principles:

### Infrastructure Lifecycle Tests

- **Focus**: Infrastructure readiness and basic VM setup
- **Network Dependencies**: Minimize network-heavy operations inside VM
- **Validation**: Verify infrastructure state, not application behavior
- **Cleanup**: Always ensure proper resource cleanup

### Deployment Workflow Tests

- **Focus**: Software functionality and deployment workflow
- **Network Access**: Reliable network access via Docker containers
- **Validation**: Verify application installation, configuration, and operation
- **State**: Sequential commands build on previous state

### Complete Workflow Tests

- **Focus**: Comprehensive validation for development workflows
- **Environment**: Local only (not CI-compatible)
- **Use Cases**: Integration testing, debugging complex issues
- **Coverage**: Full end-to-end deployment pipeline

### Independence

- Each suite should be runnable independently
- No shared state between test suites
- Each test should clean up after itself
- Tests should not depend on specific execution order

## 📝 Documentation Updates

When adding new E2E tests or modifying existing ones:

1. **Update relevant documentation files**:

   - [test-suites.md](test-suites.md) - If adding new test suites or changing validation
   - [running-tests.md](running-tests.md) - If adding new prerequisites or commands
   - [troubleshooting.md](troubleshooting.md) - If introducing new common issues
   - [architecture.md](architecture.md) - If changing testing architecture
   - [README.md](README.md) - If changing quick start or overview

2. **Update cross-references** to related documentation

3. **Add examples** for new features or complex changes

## 🔗 Related Documentation

For general contribution guidelines:

- [Contributing Guide](../contributing/README.md) - General contribution guidelines
- [Testing Conventions](../contributing/testing/README.md) - Unit testing standards
- [Error Handling](../contributing/error-handling.md) - Error handling patterns
- [Logging Guide](../contributing/logging-guide.md) - Logging best practices

## ✅ Pre-Submission Checklist

Before submitting changes to E2E tests:

- [ ] All relevant test suites pass locally
- [ ] CI tests pass on GitHub Actions
- [ ] Documentation is updated
- [ ] Code follows project conventions
- [ ] Commit messages follow [conventional commits](../contributing/commit-process.md)
- [ ] Pre-commit checks pass (`./scripts/pre-commit.sh`)
