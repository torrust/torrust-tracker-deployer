# Decision: Configuration and Data Directories as Secrets

## Status

Accepted

## Date

2026-01-10

## Context

The Torrust Tracker Deployer requires sensitive information to operate:

- **API tokens** (e.g., Hetzner Cloud API token) for provisioning infrastructure
- **Database credentials** (username, password) for tracker configuration
- **SSH private keys** for remote server access
- **Admin tokens** for tracker HTTP API authentication

Initially, there was consideration of loading secrets from environment variables using Figment's env provider. However, this would add complexity while not fundamentally improving security because:

1. Secrets passed at environment creation time are persisted in the `data/` directory as part of the environment state JSON
2. Generated configuration files in `build/` also contain these secrets
3. The user's configuration files in `envs/` already contain secrets in plain text

Other infrastructure tools like Prometheus, Ansible, and Terraform follow a similar pattern where configuration files are treated as sensitive and users are responsible for securing them through file system permissions.

## Decision

We will treat all configuration files and generated directories as secrets rather than implementing environment variable-based secret injection:

1. **`envs/` directory** - User configuration files containing API tokens, passwords, and credentials
2. **`data/` directory** - Persisted environment state including all secrets
3. **`build/` directory** - Generated configuration files that may contain secrets

The application will:

- Continue reading configuration only from JSON files (no env var secret injection)
- Document that these directories must be secured with appropriate file permissions
- Keep these directories gitignored by default

Users are responsible for:

- Setting restrictive permissions (`chmod 700` for directories, `chmod 600` for config files)
- Not committing these directories to version control
- Securing access to the host system where these files reside

## Consequences

**Positive:**

- Simple, predictable configuration loading (JSON files only)
- Consistent with how other infrastructure tools handle secrets
- No additional complexity in the configuration parsing layer
- Clear security model: "protect these directories"

**Negative:**

- Users cannot inject secrets at runtime via environment variables
- Secrets exist in plain text on disk (requires proper file permissions)
- No integration with secret management tools (Vault, AWS Secrets Manager, etc.)

**Future considerations:**

- A future enhancement could add optional secret backend integration
- Volume encryption at the OS level can provide additional protection
- Container orchestration tools can inject secrets as files

## Alternatives Considered

### Environment Variable Secret Injection

Use Figment's env provider to allow secrets to be passed via environment variables like `TORRUST_TD_PROVIDER__API_TOKEN`.

**Rejected because:**

- Adds complexity to configuration loading
- Doesn't solve the fundamental issue (secrets are still persisted in state)
- Environment variables can leak in process listings and logs

### Secret Backend Integration (Vault, etc.)

Integrate with HashiCorp Vault or similar secret management tools.

**Deferred because:**

- Significant implementation effort
- Not all users need enterprise secret management
- Can be added as an optional feature later

### Encrypted State Files

Encrypt the `data/` directory or individual state files.

**Deferred because:**

- Adds complexity for key management
- Users can use OS-level encryption (LUKS, etc.) if needed
- Can be added as an optional feature later

## Related Decisions

- [Secrecy Crate for Sensitive Data Handling](./secrecy-crate-for-sensitive-data.md) - How secrets are handled in memory (using `secrecy` crate)
- [Configuration DTO Layer Placement](./configuration-dto-layer-placement.md) - Where configuration DTOs live in the architecture

## Security Warning: AI Coding Agents

> ⚠️ **Important**: If you use cloud-based AI coding agents (GitHub Copilot, Cursor, Windsurf, etc.), be aware that **any secret the agent can see is transmitted to the AI provider's infrastructure**.

This includes:

- Configuration files in `envs/` containing API tokens
- Environment variables the agent can access via terminal commands
- Any file content the agent reads as context

**Mitigations:**

1. **Use local AI models** - The only truly secure option (Ollama, llama.cpp, local LLMs)
2. **Exclude sensitive directories** - Use `.copilotignore`, `.cursorignore`, or similar
3. **Separate workspaces** - Don't open directories containing secrets in AI-enabled editors
4. **Accept the risk** - If using enterprise AI with appropriate data agreements

**The fundamental issue**: If an AI agent can help you deploy infrastructure, it needs to see or execute commands involving your secrets. There is no technical solution that allows a cloud AI to assist with secret-containing configurations without the AI provider having potential access to those secrets.

This is not specific to this project - it applies to any infrastructure tool used with AI assistance.

📖 **For detailed mitigation strategies**, see [AI Coding Agents and Secrets](../security/ai-agents-and-secrets.md).

## References

- [Prometheus Security Model](https://prometheus.io/docs/operating/security/) - Similar approach of treating configuration as sensitive
- [Ansible Vault](https://docs.ansible.com/ansible/latest/user_guide/vault.html) - Alternative approach for encrypted secrets
- [12-Factor App Config](https://12factor.net/config) - Environment variable approach (partially relevant)
