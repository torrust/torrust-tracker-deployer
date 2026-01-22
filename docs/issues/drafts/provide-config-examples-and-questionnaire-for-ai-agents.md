# Provide Configuration Examples and Questionnaire for AI Agent Guidance

**Draft Issue** - To be created on GitHub after review

## Problem Statement

AI agents helping users create environment configurations face challenges:

1. **Many valid combinations** - Provider (LXD/Hetzner), database (SQLite/MySQL), trackers (UDP/HTTP/both), TLS, monitoring, etc.
2. **Complex dependencies** - HTTPS requires domains, MySQL requires credentials, Grafana requires Prometheus
3. **Validation rules** - Port conflicts, binding addresses, domain formats
4. **No structured guidance** - Agents must discover requirements through trial and error

Currently, agents have tools (`create template`, `validate`, JSON schema, documentation), but lack:

- A structured questionnaire to gather user requirements
- Example configurations mapping requirements to valid configs
- A dataset for training/RAG or few-shot prompting

## Proposed Solution

Create two resources to help AI agents:

### 1. Configuration Questionnaire Template

A structured decision tree that agents can follow to gather all required information:

```markdown
## Environment Configuration Questionnaire

### Basic Information

1. What name do you want for this environment?
   - Pattern: lowercase alphanumeric + hyphens, 3-50 chars

### Infrastructure Provider

2. Which provider do you want to use?
   - [ ] LXD (local development, testing)
   - [ ] Hetzner (cloud production)

   **If LXD:**
   - Use default LXD bridge network? (yes/no)

   **If Hetzner:**
   - What Hetzner API token?
   - What server location? (nbg1, fsn1, hel1, ash)
   - What server type? (cx22, cx32, cx42)

### Database Configuration

3. What database do you want to use?
   - [ ] SQLite (simpler, file-based, good for small deployments)
   - [ ] MySQL (better for high-load, production deployments)

   **If MySQL:**
   - Root password?
   - Tracker database user?
   - Tracker database password?

### Tracker Services

4. What tracker protocols do you want to enable?
   - [ ] UDP Tracker
   - [ ] HTTP Tracker
   - [ ] Both

   **For EACH tracker:**
   - Binding IP address? (default: 0.0.0.0)
   - Port number?
   - Do you want to use a custom domain?
     - If yes: What domain?
     - Do you want HTTPS? (HTTP trackers only)
       - If yes: What email for Let's Encrypt?

### APIs

5. Do you want to enable the HTTP API?
   - If yes: What port? Do you want a domain? HTTPS?

6. Do you want to expose the Health Check API?
   - If yes: What port? Do you want a domain? HTTPS?

### Monitoring (Optional)

7. Do you want to enable Prometheus metrics?
   - If yes: Do you want a domain? HTTPS?

8. Do you want to enable Grafana dashboards?
   - If yes: Admin password? Domain? HTTPS?

### SSH Access

9. What SSH private key file to use?
10. What SSH username? (default: root)
```

### 2. Example Configuration Dataset

A collection of real-world scenarios with:

- **User requirements** (natural language description)
- **Expected configuration** (valid JSON)
- **Key decisions explained** (why certain values were chosen)

#### Dataset Structure

```text
docs/ai-training/
├── README.md                          # Overview and usage instructions
├── questionnaire.md                   # Full questionnaire template
├── examples/
│   ├── 01-minimal-sqlite-lxd.md       # Simplest possible config
│   ├── 02-mysql-lxd-development.md    # Local dev with MySQL
│   ├── 03-production-hetzner-https.md # Full production setup
│   ├── 04-udp-only-tracker.md         # UDP-only, no HTTP
│   ├── 05-multi-tracker-setup.md      # Multiple UDP + HTTP trackers
│   ├── 06-monitoring-enabled.md       # With Prometheus + Grafana
│   └── ...
└── scenarios.json                     # Machine-readable dataset
```

#### Example Entry Format

````markdown
# Scenario: Minimal SQLite Development Environment

## User Requirements

"I want to quickly test the tracker locally. Just the basics -
SQLite database, one UDP tracker, one HTTP tracker. No monitoring,
no HTTPS, no custom domains. Using LXD."

## Key Decisions

- **Provider**: LXD (local testing)
- **Database**: SQLite (simplest, no credentials needed)
- **Trackers**: 1 UDP (port 6969), 1 HTTP (port 7070)
- **APIs**: HTTP API enabled for testing (port 1212)
- **Monitoring**: Disabled (not needed for basic testing)
- **TLS**: Disabled (no domains, local only)

## Expected Configuration

```json
{
  "name": "local-dev-test",
  "provider": {
    "type": "lxd"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "username": "root"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3"
      }
    },
    "udp_trackers": [
      {
        "enabled": true,
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "enabled": true,
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "enabled": true,
      "bind_address": "0.0.0.0:1212"
    },
    "health_check_api": {
      "enabled": false
    }
  }
}
```
````

## Validation Command

```bash
cargo run -- validate --env-file envs/local-dev-test.json
```

### 3. Machine-Readable Dataset (Optional)

For training or RAG systems, provide a JSON/JSONL dataset:

```json
{
  "scenarios": [
    {
      "id": "minimal-sqlite-lxd",
      "description": "Minimal local development with SQLite",
      "requirements": {
        "provider": "lxd",
        "database": "sqlite",
        "udp_trackers": 1,
        "http_trackers": 1,
        "https": false,
        "monitoring": false
      },
      "config_file": "examples/01-minimal-sqlite-lxd.json"
    }
  ]
}
```

## Proposed Example Scenarios

| ID  | Scenario              | Provider | Database | UDP | HTTP | HTTPS | Monitoring |
| --- | --------------------- | -------- | -------- | --- | ---- | ----- | ---------- |
| 01  | Minimal development   | LXD      | SQLite   | 1   | 1    | No    | No         |
| 02  | MySQL development     | LXD      | MySQL    | 1   | 1    | No    | No         |
| 03  | Production HTTPS      | Hetzner  | MySQL    | 2   | 2    | Yes   | Yes        |
| 04  | UDP-only tracker      | LXD      | SQLite   | 3   | 0    | No    | No         |
| 05  | HTTP-only with HTTPS  | Hetzner  | SQLite   | 0   | 2    | Yes   | No         |
| 06  | Full monitoring stack | LXD      | MySQL    | 1   | 1    | No    | Yes        |
| 07  | Multi-domain setup    | Hetzner  | MySQL    | 1   | 3    | Yes   | Yes        |
| 08  | Private tracker       | LXD      | MySQL    | 1   | 1    | No    | No         |
| 09  | High-availability     | Hetzner  | MySQL    | 4   | 4    | Yes   | Yes        |
| 10  | Minimal Hetzner       | Hetzner  | SQLite   | 1   | 1    | No    | No         |

## Benefits

### For AI Agents

1. **Structured guidance** - Questionnaire ensures all required info is gathered
2. **Few-shot learning** - Examples provide patterns to follow
3. **Reduced hallucination** - Real validated configs as reference
4. **Faster iteration** - Less trial-and-error with validate command

### For Training/RAG

1. **Curated dataset** - High-quality examples with explanations
2. **Coverage** - Scenarios covering common and edge cases
3. **Machine-readable** - JSON format for easy processing
4. **Versioned** - Updates with new features

### For Human Users

1. **Documentation** - Examples serve as user documentation
2. **Templates** - Starting points for custom configurations
3. **Understanding** - See how decisions map to config values

## Implementation Plan

### Phase 1: Questionnaire Template (1 hour)

- [ ] Create `docs/ai-training/questionnaire.md` with full decision tree
- [ ] Include validation rules and constraints for each question
- [ ] Add conditional logic notes (if X then ask Y)

### Phase 2: Core Examples (2-3 hours)

- [ ] Create 5 core scenarios (minimal, MySQL, production, UDP-only, monitoring)
- [ ] Each with requirements, decisions, and validated JSON config
- [ ] Test each config with `validate` command

### Phase 3: Extended Examples (2 hours)

- [ ] Add 5 more edge case scenarios
- [ ] Cover multi-tracker, multi-domain, high-availability patterns
- [ ] Document common mistakes and how to fix them

### Phase 4: Machine-Readable Dataset (1 hour)

- [ ] Create `scenarios.json` with structured metadata
- [ ] Link to individual config files
- [ ] Add README with usage instructions for training/RAG

## Integration with Agent Skills

This dataset can be referenced from the `create-environment-config` skill proposed in issue #274:

```yaml
---
name: create-environment-config
description: |
  Generate valid environment configuration JSON files for the deployer.
  Uses the questionnaire template and example scenarios for guidance.
---

## Before generating a config

1. Read the questionnaire: `docs/ai-training/questionnaire.md`
2. Find a similar example: `docs/ai-training/examples/`
3. Use `cargo run -- create template` to generate base
4. Customize based on user requirements
5. Validate with `cargo run -- validate --env-file`
```

## Related Documentation

- [Environment Configuration Schema](../../schemas/README.md)
- [JSON Schema](../../schemas/environment-config.json)
- [Create Command Documentation](../../docs/user-guide/commands/create.md)
- [Validate Command](../../docs/console-commands.md)
- [Issue #274 - Agent Skills](./274-consider-using-agentskills-io.md)

## Open Questions

1. Should examples include the full JSON or reference separate files?
2. Should we include "anti-patterns" (common mistakes)?
3. Should we version the dataset with deployer releases?
4. Should we include natural language variations of the same requirement?
