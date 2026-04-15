# AI Coding Agents and Secrets

## The Problem

Cloud-based AI coding agents (GitHub Copilot, Cursor, Windsurf, Codeium, etc.) process your code and context on remote servers. This means:

**Any secret the AI agent can see is transmitted to the AI provider's infrastructure.**

This includes:

- Configuration files containing API tokens (`envs/*.json`)
- Environment variables accessed via terminal commands
- File contents read as context for code assistance
- Command output displayed in terminals

## Why This Matters for Infrastructure Tools

Infrastructure tools like this deployer necessarily handle sensitive data:

- Cloud provider API tokens (Hetzner, AWS, etc.)
- Database credentials
- SSH private keys
- Admin authentication tokens

If you use an AI agent to help you configure or debug deployments, the agent needs access to this information to be useful.

## The Fundamental Trade-off

There is **no technical solution** that allows a cloud-based AI to:

1. Help you with secret-containing configurations
2. While keeping those secrets from the AI provider

These goals are mutually exclusive. You must choose one.

## Mitigation Options

### Option 1: Use Local AI Models (Most Secure)

Run AI models locally so secrets never leave your machine:

- **Ollama** - Easy to run local LLMs
- **llama.cpp** - Efficient local inference
- **LM Studio** - GUI for local models
- **Continue.dev** - IDE extension supporting local models

**Trade-off**: Local models are generally less capable than cloud models.

### Option 2: Vault-Based Secrets (Runtime Injection)

Store secrets in a dedicated secret management system and fetch them at runtime. The AI agent only sees placeholder references, never actual secrets.

#### How It Works

1. **Configuration files contain references**, not secrets:

   ```json
   {
     "provider": {
       "provider": "hetzner",
       "api_token": "vault:secret/hetzner#api_token"
     }
   }
   ```

2. **At runtime**, the application (or a wrapper script) resolves references:

   ```bash
   # Wrapper script fetches secrets before running deployer
   export HETZNER_TOKEN=$(vault kv get -field=api_token secret/hetzner)

   # Generate config with actual values
   envsubst < config-template.json > /tmp/config.json

   # Run deployer
   torrust-tracker-deployer create environment --env-file /tmp/config.json

   # Clean up
   rm /tmp/config.json
   ```

3. **AI agent sees only** the template with `vault:` references

#### Secret Management Tools

| Tool                      | Type              | Best For                      |
| ------------------------- | ----------------- | ----------------------------- |
| **HashiCorp Vault**       | Self-hosted/Cloud | Enterprise, multi-cloud       |
| **AWS Secrets Manager**   | Cloud             | AWS-native workloads          |
| **Azure Key Vault**       | Cloud             | Azure-native workloads        |
| **Google Secret Manager** | Cloud             | GCP-native workloads          |
| **1Password CLI**         | SaaS              | Teams already using 1Password |
| **Bitwarden CLI**         | Self-hosted/SaaS  | Open-source friendly          |
| **SOPS**                  | File encryption   | GitOps workflows              |
| **age/GPG**               | File encryption   | Simple encryption needs       |

#### Implementation Patterns

##### Pattern A: Environment Variable Injection

```bash
#!/bin/bash
# deploy.sh - Wrapper script that injects secrets

# Fetch secrets from vault
export HETZNER_API_TOKEN=$(vault kv get -field=api_token secret/deployer/hetzner)
export DB_PASSWORD=$(vault kv get -field=password secret/deployer/database)

# Create config from template (uses envsubst or similar)
cat > /tmp/deploy-config.json << EOF
{
  "provider": {
    "provider": "hetzner",
    "api_token": "${HETZNER_API_TOKEN}"
  },
  "tracker": {
    "database": {
      "password": "${DB_PASSWORD}"
    }
  }
}
EOF

# Run deployer
torrust-tracker-deployer create environment --env-file /tmp/deploy-config.json

# Secure cleanup
shred -u /tmp/deploy-config.json 2>/dev/null || rm -f /tmp/deploy-config.json
unset HETZNER_API_TOKEN DB_PASSWORD
```

##### Pattern B: 1Password CLI Example

```bash
#!/bin/bash
# Using 1Password CLI (op)

# Sign in (or use service account)
eval $(op signin)

# Inject secrets directly
op run --env-file=.env.1password -- torrust-tracker-deployer provision my-env
```

Where `.env.1password` contains:

```text
HETZNER_API_TOKEN=op://Vault/Hetzner/api-token
DB_PASSWORD=op://Vault/Database/password
```

##### Pattern C: SOPS Encrypted Files

```bash
# Decrypt config, use it, then clean up
sops -d envs/my-env.enc.json > /tmp/my-env.json
torrust-tracker-deployer create environment --env-file /tmp/my-env.json
shred -u /tmp/my-env.json
```

The encrypted file (`envs/my-env.enc.json`) can safely be in the workspace - AI only sees encrypted data.

##### Pattern D: Vault Agent Sidecar (Advanced)

For containerized deployments, use Vault Agent to inject secrets:

```yaml
# docker-compose with vault agent
services:
  vault-agent:
    image: hashicorp/vault
    command: agent -config=/etc/vault/agent.hcl
    volumes:
      - secrets:/secrets

  deployer:
    image: torrust/tracker-deployer
    volumes:
      - secrets:/secrets:ro
    depends_on:
      - vault-agent
```

#### What the AI Agent Sees

With vault-based secrets, your configuration files look like:

```json
{
  "environment": {
    "name": "production"
  },
  "provider": {
    "provider": "hetzner",
    "api_token": "PLACEHOLDER_FROM_VAULT"
  }
}
```

Or with reference syntax:

```json
{
  "provider": {
    "api_token": "{{vault:secret/hetzner#token}}"
  }
}
```

The AI can help you with the configuration structure, validation, and logic - but never sees actual credentials.

#### Current Limitation

> ⚠️ **Note**: The Torrust Tracker Deployer does **not** currently have native vault integration. You must use wrapper scripts (as shown above) to inject secrets at runtime.
>
> A future enhancement could add native support for vault references in configuration files.

**Trade-off**: Requires additional tooling setup and wrapper scripts. More complex workflow, but secrets never touch files the AI can read.

### Option 3: Exclude Sensitive Directories

Prevent AI agents from reading sensitive files:

**For GitHub Copilot** - Create `.copilotignore`:

```text
envs/
data/
build/
*.json
```

**For Cursor** - Create `.cursorignore`:

```text
envs/
data/
build/
```

**Trade-off**: The AI cannot help you with configuration files.

### Option 4: Workspace Isolation

- Open sensitive directories in a non-AI-enabled editor
- Use AI assistance only for non-sensitive code
- Create separate workspaces for secrets and code

**Trade-off**: Inconvenient workflow, manual context switching.

### Option 5: Accept the Risk (Informed Decision)

This option acknowledges that cloud AI providers will process your secrets, but accepts this based on risk assessment and trust in the provider's security practices.

#### What AI Providers Typically Promise

Most enterprise AI coding tools offer:

- **Data encryption** in transit and at rest
- **No training on your code** (for business/enterprise tiers)
- **Data retention limits** (often deleted within 30 days or less)
- **Compliance certifications** (SOC 2, GDPR, ISO 27001)
- **Data Processing Agreements** (DPAs) for enterprise customers

#### Provider-Specific Policies (as of 2025)

| Provider                  | Training on Code | Retention | Enterprise DPA |
| ------------------------- | ---------------- | --------- | -------------- |
| GitHub Copilot Business   | No               | Transient | Yes            |
| GitHub Copilot Enterprise | No               | Transient | Yes            |
| Cursor Business           | No               | Limited   | Yes            |
| Anthropic Claude (API)    | No (by default)  | 30 days   | Yes            |

> ⚠️ **Always verify current policies** - These change frequently. Free tiers often have different (less protective) policies than paid tiers.

#### When This Option May Be Acceptable

1. **Development/testing credentials** - Non-production API tokens with limited permissions
2. **Isolated test environments** - Tokens that can only access sandboxed resources
3. **Short-lived credentials** - Tokens that expire quickly or are rotated frequently
4. **Low-impact secrets** - Credentials for non-critical, easily replaceable resources

#### When This Option Is NOT Acceptable

1. **Production infrastructure credentials** - Tokens with access to live systems
2. **Payment/financial systems** - Any credentials touching money
3. **Personal data access** - Credentials to systems with PII/PHI
4. **Compliance-regulated environments** - HIPAA, PCI-DSS, FedRAMP, etc.
5. **Root/admin credentials** - Highly privileged access tokens

#### Risk Mitigation Within This Option

Even when accepting the risk, reduce exposure:

1. **Use least-privilege tokens** - Create API tokens with minimal permissions
2. **Set token expiration** - Short-lived tokens limit exposure window
3. **Rotate frequently** - Change credentials regularly
4. **Monitor usage** - Watch for unexpected API calls in provider dashboards
5. **Use separate credentials** - Different tokens for AI-assisted vs production work
6. **Audit AI context** - Periodically review what files the AI can access

#### Questions to Ask Your AI Provider

Before accepting this risk, get answers to:

1. Where is my data processed? (Geography, data residency)
2. Is my code/data used to train models?
3. How long is data retained?
4. Who at the provider can access my data?
5. What happens in a data breach?
6. Can I get a Data Processing Agreement?
7. What compliance certifications do you have?

**Trade-off**: Your secrets are processed by a third party, but with understood and (hopefully) acceptable risk.

## Recommendations

| Scenario                | Recommended Options                                                     |
| ----------------------- | ----------------------------------------------------------------------- |
| **Production secrets**  | Option 1 (local models) or Option 2 (vault-based)                       |
| **CI/CD pipelines**     | Option 2 (vault-based) - integrate with your existing secret management |
| **Development/testing** | Option 5 may be acceptable with non-production credentials              |
| **Enterprise teams**    | Option 2 (vault) + Option 5 (with enterprise DPA)                       |
| **Quick experiments**   | Option 3 (exclude dirs) or Option 5 (accept risk for throwaway tokens)  |

## This Is Not Unique to This Project

This security consideration applies to **any** infrastructure tool used with AI assistance:

- Terraform configurations
- Ansible playbooks
- Kubernetes secrets
- CI/CD pipeline configurations
- Docker Compose files with credentials

The same precautions should be taken regardless of the tool.

## Related Documentation

- [ADR: Configuration Directories as Secrets](../decisions/configuration-directories-as-secrets.md)
- [Docker Image Security](./docker-image-security-scans.md)
- [Secret Handling in Code](../contributing/secret-handling.md)
