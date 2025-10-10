# Decision: Repository Rename from "Torrust Tracker Deploy" to "Torrust Tracker Deployer"

## Status

Accepted

## Date

2025-10-10

## Context

This project started as a proof of concept (PoC) repository named "torrust-tracker-deploy-rust-poc". It is the third in a series of experimental approaches to building deployment infrastructure for the Torrust Tracker:

1. **Bash/OpenTofu/cloud-init PoC** (`torrust-tracker-deploy-bash-poc`) - Focused on Infrastructure as Code
2. **Perl/Ansible PoC** (`torrust-tracker-deploy-perl-poc`) - Focused on declarative configuration management
3. **Rust/Testing/LXD PoC** (`torrust-tracker-deploy-rust-poc`) - Current repository, focused on type-safe deployment tooling

After evaluating all three approaches, this Rust-based solution has been chosen as the definitive approach for the Torrust Tracker deployer. While still under active development, the repository is now transitioning from "proof of concept mode" to standard open-source development flow, with the goal of becoming production-ready.

The current name "Torrust Tracker Deploy" is ambiguous and doesn't clearly convey that this is a tool (agent noun) rather than a process description. Additionally, the repository URL includes the "-rust-poc" suffix which no longer reflects the maturity and purpose of the project.

## Decision

Rename the repository and project from:

- **Old Name**: Torrust Tracker Deploy
- **Old Repository**: `torrust-tracker-deploy-rust-poc`
- **Old URL**: `https://github.com/torrust/torrust-tracker-deploy-rust-poc`

To:

- **New Name**: Torrust Tracker Deployer
- **New Repository**: `torrust-tracker-deployer`
- **New URL**: `https://github.com/torrust/torrust-tracker-deployer`

The "-er" suffix makes it clear this is a tool that performs deployment, following common naming conventions for agent nouns in software (e.g., compiler, debugger, analyzer, deployer).

## Consequences

### Positive

- **Clearer Identity**: The name clearly identifies this as a deployment tool, not a process or state
- **Professional Positioning**: Removes the "proof of concept" designation, signaling this is the chosen definitive approach
- **Consistent Naming**: Follows established conventions for software tools using agent nouns
- **Better Discoverability**: More intuitive name for users searching for deployment tools
- **Clean History**: Opportunity to establish clear separation from experimental PoC repositories

### Negative

- **Breaking Change**: All existing references to the old repository name and URL must be updated
- **GitHub Redirect**: While GitHub automatically redirects old URLs, documentation should be updated to avoid confusion
- **Documentation Update**: Requires comprehensive update of all internal and external documentation
- **Learning Curve**: Contributors familiar with the old name will need to adjust

### Migration Tasks

1. **GitHub Repository Rename**: Rename repository on GitHub (automatic redirect will be created)
2. **Update Documentation**: Update all references in docs, READMEs, comments, and configuration files
3. **Update Previous PoCs**: Add deprecation notices to bash-poc and perl-poc repositories pointing to the new name
4. **Update URLs**: Change all hardcoded URLs in documentation and code
5. **Update CI/CD**: Verify GitHub Actions and workflows continue to function correctly
6. **Update Package Names**: Update Rust package names in Cargo.toml if they reference the repository name

## Alternatives Considered

### 1. "Torrust Tracker Installer"

**Rejected** - The term "installer" implies the tool installs the tracker on the local machine where the installer runs. This is misleading because the deployer provisions and configures remote virtual machines via SSH, not the local system. "Installer" suggests a more traditional software installation pattern (like apt, yum, or setup.exe) rather than infrastructure provisioning and remote deployment.

### 2. Keep "Torrust Tracker Deploy"

**Rejected** - Without the "-er" suffix, the name reads as either a noun describing the deployment state/process or a verb command, creating ambiguity. It doesn't clearly convey that this is a tool. Examples: "git deploy", "npm deploy" are commands, not tool names.

### 3. "Torrust Tracker Deploy Tool"

**Rejected** - While technically accurate, adding "Tool" is redundant. Well-established deployment tools don't include "Tool" in their name (e.g., Ansible, Terraform, not "Ansible Tool"). The "-er" suffix alone sufficiently conveys the tool nature.

### 4. "Torrust Deployer"

**Rejected** - While shorter, this loses the specificity that this tool is for deploying the Torrust Tracker specifically. The full name "Torrust Tracker Deployer" makes the purpose immediately clear and maintains namespace clarity within the broader Torrust ecosystem.

### 5. Keep "-rust-poc" suffix

**Rejected** - The "PoC" designation undermines confidence in production use. The project has evolved beyond proof of concept status. The "rust" language indicator is unnecessary in the repository name as it's clearly visible in the repository metadata, and many mature projects don't include the implementation language in their name.

## Related Decisions

- This decision supersedes the implicit naming from the initial PoC phase
- Future tooling in the Torrust ecosystem should consider following similar naming patterns (agent nouns with clear scope)

## References

- Previous PoC repositories:
  - Bash PoC: https://github.com/torrust/torrust-tracker-deploy-bash-poc
  - Perl PoC: https://github.com/torrust/torrust-tracker-deploy-perl-poc
- Main Torrust Tracker: https://github.com/torrust/torrust-tracker
- GitHub repository renaming documentation: https://docs.github.com/en/repositories/creating-and-managing-repositories/renaming-a-repository
