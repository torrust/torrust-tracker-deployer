# Add Devcontainer Configuration for GitHub Codespaces

**Issue**: #328
**Parent Epic**: N/A
**Related**: Docker image support for deployer

## Overview

Add a devcontainer configuration to enable running the Torrust Tracker Deployer directly in GitHub Codespaces. This would allow users to deploy the tracker without installing any dependencies locally (OpenTofu, Ansible, LXD, etc.) by using a pre-configured development environment in the cloud.

## Goals

- [ ] Create a devcontainer configuration that includes all deployer dependencies
- [ ] Enable GitHub Codespaces support with the deployer template
- [ ] Document the Codespaces deployment workflow
- [ ] Provide guidance on security considerations (GitHub secrets management)

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (Infrastructure/Tooling)
**Module Path**: `.devcontainer/` (repository root)
**Pattern**: Development environment configuration

### Module Structure Requirements

- [ ] Follow standard devcontainer structure (see [VS Code Devcontainers specification](https://containers.dev/))
- [ ] Integrate with existing Docker image (`docker/deployer/`)
- [ ] Maintain compatibility with local development setup

### Architectural Constraints

- [ ] Configuration must support all deployment workflows except local LXD provider
- [ ] Should reuse existing Docker image when possible
- [ ] Must document security implications of using Codespaces

### Anti-Patterns to Avoid

- ❌ Don't duplicate dependency installation logic from existing Docker image
- ❌ Don't expose secrets in devcontainer configuration files
- ❌ Don't create provider-specific configurations (keep it generic)

## Specifications

### Devcontainer Configuration Structure

Create `.devcontainer/devcontainer.json` with the following capabilities:

```json
{
  "name": "Torrust Tracker Deployer",
  "image": "torrust/tracker-deployer:latest",
  "features": {},
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "redhat.vscode.yaml",
        "GitHub.copilot"
      ],
      "settings": {
        "chat.useAgentSkills": true,
        "evenBetterToml.formatter.allowedBlankLines": 1,
        "json.schemas": [
          {
            "fileMatch": ["envs/*.json"],
            "url": "./schemas/environment-config.json"
          }
        ]
      }
    }
  },
  "postCreateCommand": "cargo build",
  "remoteUser": "vscode"
}
```

### Supported Workflows

**Supported**:

- ✅ Create, provision, configure, release, run (cloud providers)
- ✅ All CLI commands and E2E tests (without LXD provider)
- ✅ Linting and code quality checks
- ✅ Documentation editing and generation

**Not Supported**:

- ❌ Local LXD provider (requires nested virtualization)

### User Workflow

1. Navigate to repository on GitHub
2. Click "Code" → "Codespaces" → "Create codespace on main"
3. Wait for devcontainer to initialize (installs all dependencies)
4. Run deployer: `cargo run -- create environment --env-file envs/your-config.json`
5. (Optional) Use GitHub Copilot to assist with deployment

### Security Considerations

**Pros**:

- Easy setup - no local installation required
- Consistent environment across all users
- GitHub-managed infrastructure

**Cons**:

- Secrets must be stored as GitHub Codespaces secrets
- Credentials visible to GitHub infrastructure
- Recommendation: Change secrets after deployment completion

**Mitigation**:

- Document how to use GitHub Codespaces secrets (not environment variables in config files)
- Provide post-deployment secret rotation guide
- Warn users about security trade-offs in documentation

## Implementation Plan

### Phase 1: Devcontainer Configuration (2-3 hours)

- [ ] Create `.devcontainer/devcontainer.json` with deployer image reference
- [ ] Add VS Code extensions for Rust, TOML, YAML, and GitHub Copilot
- [ ] Configure VS Code settings:
  - [ ] Enable agent skills for GitHub Copilot
  - [ ] Configure TOML formatter settings
  - [ ] Add JSON schema validation for environment files (`envs/*.json`)
- [ ] Configure post-create command to build the project
- [ ] Test basic functionality (building and running commands)

### Phase 2: Documentation (1-2 hours)

- [ ] Create `docs/user-guide/codespaces-deployment.md` with:
  - [ ] Step-by-step Codespaces setup guide
  - [ ] How to manage secrets in Codespaces
  - [ ] Supported vs unsupported workflows (LXD limitation)
  - [ ] Security considerations and secret rotation guide
  - [ ] Troubleshooting common Codespaces issues
- [ ] Update `docs/user-guide/README.md` with link to Codespaces guide
- [ ] Add Codespaces badge to main `README.md`

### Phase 3: Testing and Validation (1-2 hours)

- [ ] Test creating a new Codespace from the repository
- [ ] Validate all dependencies are correctly installed
- [ ] Run E2E tests (excluding LXD-dependent tests)
- [ ] Test with different cloud providers (if available)
- [ ] Verify GitHub Copilot integration works correctly

### Phase 4: Optional Enhancements (Future)

- [ ] Add devcontainer features for additional tools (jq, yq, etc.)
- [ ] Create a Codespaces template repository
- [ ] Add automated secret rotation scripts
- [ ] Explore alternatives to storing secrets in GitHub

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `.devcontainer/devcontainer.json` exists and is valid
- [ ] Devcontainer successfully builds in GitHub Codespaces
- [ ] All non-LXD commands work in Codespaces environment
- [ ] Documentation clearly explains:
  - [ ] How to create and use a Codespace
  - [ ] Security considerations and secret management
  - [ ] Limitations (LXD provider not supported)
- [ ] VS Code extensions are automatically installed
- [ ] GitHub Copilot integration is enabled with agent skills
- [ ] JSON schema validation works for environment files in `envs/` directory
- [ ] E2E tests (excluding LXD) pass in Codespaces environment

## Related Documentation

- [VS Code Devcontainers Documentation](https://code.visualstudio.com/docs/devcontainers/containers)
- [Devcontainer Specification](https://containers.dev/)
- [GitHub Codespaces Documentation](https://docs.github.com/en/codespaces)
- [GitHub Codespaces Secrets](https://docs.github.com/en/codespaces/managing-your-codespaces/managing-secrets-for-your-codespaces)
- [Agent Skills Specification](https://agentskills.io/specification)
- Project: `docker/deployer/` - Existing Docker image
- Project: `docs/user-guide/` - User documentation directory

## Notes

### Virtualization Limitation

GitHub Codespaces runs in containers, not VMs, which means nested virtualization is not available. This prevents using the LXD provider for local deployments. Users must use cloud providers (AWS, Azure, GCP, etc.) when deploying from Codespaces.

### Docker Image Reuse

The devcontainer configuration should reference the existing deployer Docker image (`docker/deployer/`) to avoid duplicating dependency installation logic. If modifications are needed, they should be made to the base Docker image, not the devcontainer configuration.

### Copilot Integration

With GitHub Copilot enabled and `chat.useAgentSkills: true`, users can ask Copilot to help with deployment tasks. Copilot can read the `AGENTS.md` file and skills in `.github/skills/` to provide contextual assistance.

Example prompts:

- "Deploy the tracker to Hetzner using the environment configuration"
- "Create a new environment configuration for MySQL database"
- "Run the E2E tests"

### Alternative: Codespaces Template Repository

In the future, we could create a separate template repository specifically for Codespaces that:

- Pre-configures common deployment scenarios
- Includes sample environment configurations
- Has secrets placeholder guidance built-in
- Could be forked for one-time deployments

This would be a separate enhancement tracked in a future issue.
