# Provide Configuration Examples and Questionnaire for AI Agent Guidance

**Issue**: [#339](https://github.com/torrust/torrust-tracker-deployer/issues/339)
**Parent Epic**: N/A (standalone task)
**Roadmap**: Task 11.3 - [Improve AI Agent Experience](../roadmap.md#11-improve-ai-agent-experience)
**Related**: [Issue #274 - Agent Skills](274-consider-using-agentskills-io.md), [Environment Configuration Schema](../../schemas/README.md)

## Overview

Create structured resources to help AI agents guide users through environment configuration creation. This includes a decision-tree questionnaire template and a curated dataset of example configurations covering common deployment scenarios (minimal development, production with HTTPS, monitoring-enabled, etc.).

AI agents currently have tools (`create template`, `validate`, JSON schema, documentation) but lack structured guidance for gathering requirements and mapping them to valid configurations. This leads to trial-and-error interactions and potential configuration errors.

## Goals

- [x] Provide structured questionnaire to gather all configuration requirements systematically
- [x] Create curated example dataset mapping user requirements to validated JSON configs
- [x] Reduce AI agent hallucination and trial-and-error through real validated examples
- [x] Serve as user-facing documentation and configuration templates
- [x] Enable machine-readable format for potential training/RAG use cases

## 🏗️ Architecture Requirements

**DDD Layer**: Application + Documentation
**Module Path**: `src/application/command_handlers/create/config/` (schema) + `docs/ai-training/` (examples)
**Pattern**: Schema extension + documentation-driven configuration guidance

### Module Structure Requirements

- [x] Add optional `description` field to environment configuration schema
- [x] Update JSON schema at `schemas/environment-config.json`
- [x] Update Rust DTO at `src/application/command_handlers/create/config/environment_config.rs`
- [x] Examples validated with `cargo run -- validate --env-file`
- [x] All examples are JSON files (no separate markdown documentation)

### Architectural Constraints

- [x] `description` field must be optional (not required for existing configs)
- [x] Description should be free-text string, 2-3 sentences recommended
- [x] All example configs must be validated against the updated schema
- [x] Questionnaire must align with validation rules in `src/application/command_handlers/create/config/`
- [x] Examples must not include sensitive data (use fixture keys only)

### Anti-Patterns to Avoid

- ❌ Examples with invalid or outdated configurations
- ❌ Questionnaire questions that don't map to schema fields
- ❌ Missing validation constraints or dependency rules

## Specifications

### Problem Context

AI agents helping users create environment configurations face challenges:

1. **Many valid combinations** - Provider (LXD/Hetzner), database (SQLite/MySQL), trackers (UDP/HTTP/both), TLS, monitoring, etc.
2. **Complex dependencies** - HTTPS requires domains, MySQL requires credentials, Grafana requires Prometheus
3. **Validation rules** - Port conflicts, binding addresses, domain formats
4. **No structured guidance** - Agents must discover requirements through trial and error

Missing resources:

- Structured questionnaire to gather user requirements
- Example configurations mapping requirements to valid configs
- Dataset for training/RAG or few-shot prompting

### Solution Components

#### 1. Configuration Questionnaire Template (`docs/ai-training/questionnaire.md`)

A structured decision tree that agents can follow to gather all required information:

**Example Structure**:

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

### APIs

5. Do you want to enable the HTTP API?
   - If yes: What port? Do you want a domain?

6. Do you want to expose the Health Check API?
   - If yes: What port? Do you want a domain?

### TLS/HTTPS Configuration

7. Do any HTTP services (HTTP trackers, APIs) use domains?
   - If yes: Do you want to enable HTTPS?
     - If yes: What email for Let's Encrypt? (single email for all HTTPS certificates)

### Monitoring (Optional)

8. Do you want to enable Prometheus metrics?
   - If yes: Do you want a domain? HTTPS?

9. Do you want to enable Grafana dashboards?
   - If yes: Admin password? Domain? HTTPS?

### SSH Access

10. What SSH private key file to use?
11. What SSH username? (default: root)
```

### 2. Example Configuration Dataset

A collection of real-world scenario configurations as JSON files. Each configuration includes a `description` field that captures:

- **User requirements** (what the user wants to accomplish)
- **Key decisions** (why certain values were chosen)
- **Use case context** (when this configuration is appropriate)

#### Dataset Structure

```text
docs/ai-training/
├── README.md                          # Overview, usage instructions, scenarios table
├── questionnaire.md                   # Full questionnaire template
└── examples/
    ├── 01-minimal-lxd.json
    ├── 02-full-stack-lxd.json
    ├── 03-minimal-hetzner.json
    ├── 04-full-stack-hetzner.json
    ├── 05-mysql-development.json
    ├── 06-production-https.json
    ├── 07-udp-only-tracker.json
    ├── 08-http-only-with-https.json
    ├── 09-monitoring-stack.json
    ├── 10-multi-domain.json
    ├── 11-private-tracker.json
    ├── 12-high-availability.json
    ├── 13-backup-focused.json
    ├── 14-lightweight-production.json
    └── 15-sqlite-monitoring.json
```

#### Example Configuration Format

Each JSON file contains a complete, validated environment configuration with a `description` field:

```json
{
  "environment": {
    "name": "local-dev-test"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-local-dev-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "health_check_api": {
      "bind_address": "0.0.0.0:1313"
    }
  }
}
```

> **Note**: This is a minimal configuration for local development with LXD. It includes only the essential services without monitoring, HTTPS, or custom domains. SSH paths shown are relative for readability - use absolute paths when validating.

**Validation**:

```bash
cargo run -- validate --env-file docs/ai-training/dataset/environment-configs/01-minimal-lxd.json
```

**Automated Testing**:

To ensure all examples remain valid as the schema evolves, an integration test will validate all example files:

```rust
// tests/validate_examples.rs
// Iterates over docs/ai-training/dataset/environment-configs/*.json
// Runs validate command on each file
// Fails if any example is invalid
```

This prevents regressions and ensures examples always work with the current schema.

### Proposed Example Scenarios

The dataset will include these 15 scenarios covering common use cases:

| ID  | Scenario                | Provider | Database | UDP | HTTP | HTTPS         | Prometheus | Grafana | Backup |
| --- | ----------------------- | -------- | -------- | --- | ---- | ------------- | ---------- | ------- | ------ |
| 01  | Minimal development     | LXD      | SQLite   | 1   | 1    | No            | No         | No      | No     |
| 02  | Full-stack all features | LXD      | MySQL    | 2   | 2    | Yes (staging) | Yes        | Yes     | Yes    |
| 03  | Minimal Hetzner         | Hetzner  | SQLite   | 1   | 1    | No            | No         | No      | No     |
| 04  | Full-stack Hetzner      | Hetzner  | MySQL    | 2   | 2    | Yes (prod)    | Yes        | Yes     | Yes    |
| 05  | MySQL development       | LXD      | MySQL    | 1   | 1    | No            | No         | No      | No     |
| 06  | Production HTTPS        | LXD      | MySQL    | 2   | 2    | Yes (staging) | Yes        | Yes     | No     |
| 07  | UDP-only tracker        | LXD      | SQLite   | 3   | 0    | No            | No         | No      | No     |
| 08  | HTTP-only with HTTPS    | LXD      | SQLite   | 0   | 2    | Yes (staging) | No         | No      | No     |
| 09  | Full monitoring stack   | LXD      | MySQL    | 1   | 1    | No            | Yes        | Yes     | No     |
| 10  | Multi-domain setup      | LXD      | MySQL    | 1   | 3    | Yes (staging) | Yes        | Yes     | No     |
| 11  | Private tracker         | LXD      | MySQL    | 1   | 1    | No            | No         | No      | No     |
| 12  | High-availability       | LXD      | MySQL    | 4   | 4    | Yes (staging) | Yes        | Yes     | No     |
| 13  | Backup-focused          | LXD      | MySQL    | 1   | 1    | Yes (staging) | No         | No      | Yes    |
| 14  | Lightweight production  | LXD      | SQLite   | 1   | 1    | Yes (staging) | No         | No      | Yes    |
| 15  | SQLite with monitoring  | LXD      | SQLite   | 1   | 1    | No            | Yes        | Yes     | No     |

**Note**: Scenarios are organized to show local (LXD) and cloud (Hetzner) extremes first:

- Scenarios 01-02: LXD minimal and full-stack (for local E2E testing)
- Scenarios 03-04: Hetzner minimal and full-stack (for cloud deployment)
- Scenarios 05-15: LXD-based variations covering specific use cases

**HTTPS Certificate Configuration**:

- **LXD scenarios with HTTPS** (02, 06, 08, 10, 12, 13, 14): Use Let's Encrypt **staging** certificates (`use_staging: true`) for safe testing without hitting production rate limits
- **Hetzner scenario with HTTPS** (04): Uses Let's Encrypt **production** certificates for real deployments

**Key Scenarios**:

- **Scenario 02** (Full-stack LXD): Comprehensive setup with every feature enabled - ideal for E2E manual testing and regression testing
- **Scenario 04** (Full-stack Hetzner): Mirrors scenario 02 for production cloud deployment
- **Scenario 13** (Backup-focused): Demonstrates backup without monitoring overhead (MySQL + HTTPS + backup)
- **Scenario 14** (Lightweight production): Simplest production setup (SQLite + HTTPS + backup, no MySQL/monitoring complexity)
- **Scenario 15** (SQLite with monitoring): Learn monitoring stack with SQLite simplicity

#### Scenario 02 Example: Full-Stack Local Configuration

The following configuration represents the complete full-stack scenario with all features enabled, running on LXD for local testing:

```json
{
  "environment": {
    "name": "full-stack-test"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-full-stack-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "mysql",
        "port": 3306,
        "database_name": "tracker",
        "username": "tracker_user",
        "password": "secure_password"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      },
      {
        "bind_address": "0.0.0.0:6970"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "tracker1.example.com",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7071",
        "domain": "tracker2.example.com",
        "use_tls_proxy": true
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "domain": "api.example.com",
      "use_tls_proxy": true
    },
    "health_check_api": {
      "bind_address": "0.0.0.0:1313",
      "domain": "health.example.com",
      "use_tls_proxy": true
    }
  },
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": true
  },
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin-password",
    "domain": "grafana.example.com",
    "use_tls_proxy": true
  },
  "backup": {
    "schedule": "0 2 * * *",
    "retention_days": 7
  }
}
```

> **Note**: SSH paths shown here are relative for readability. When validating, the `validate` command requires absolute paths. For example, update `"private_key_path"` to the full path like `"/home/user/project/fixtures/testing_rsa"`.

**Key Features Demonstrated**:

- **Provider**: LXD local (ideal for E2E testing and development)
- **Database**: MySQL with full credentials (robust database setup)
- **Trackers**: 2 UDP trackers + 2 HTTP trackers with domains and TLS
- **APIs**: HTTP API and Health Check API, both with domains and TLS
- **HTTPS**: All HTTP services configured with TLS via Caddy reverse proxy using Let's Encrypt staging certificates (safe for testing)
- **Monitoring**: Prometheus (15s scrape interval) + Grafana with domain and TLS
- **Backup**: Daily backups at 2 AM with 7-day retention
- **Domains**: All HTTP services have custom domains for testing HTTPS flow locally

This configuration exercises every deployment feature and serves as a comprehensive integration test for the entire stack. It's designed for local E2E testing before deploying to production environments. The `use_staging: true` setting ensures safe testing without hitting Let's Encrypt production rate limits.

### Benefits and Value

**For AI Agents**:

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

### Phase 1: Add Description Field to Schema (1 hour) ✅

- [x] Add optional `description` field to `schemas/environment-config.json`
- [x] Update Rust DTO in `src/application/command_handlers/create/config/environment_config.rs`
- [x] Field type: `Option<String>`, free-text, no length constraints at schema level
- [x] Update validation tests to accept configs with description field
- [x] Run tests to ensure backward compatibility (existing configs without description still work)

### Phase 2: Questionnaire Template (1 hour) ✅

- [x] Create `docs/ai-training/questionnaire.md` with full decision tree
- [x] Include validation rules and constraints for each question
- [x] Add conditional logic notes (if X then ask Y)

### Phase 3: Core Example Configurations (2-3 hours) ✅

- [x] Create 6 core scenario JSON files with description field
- [x] Scenarios: 01-minimal LXD, 02-full-stack LXD (staging), 03-minimal Hetzner, 04-full-stack Hetzner (production), 05-MySQL development, 09-monitoring stack
- [x] Each description includes use case + key decisions (2-3 sentences)
- [x] Validate each config with `cargo run -- validate --env-file`
- [x] Use fixture keys only (no real credentials)

### Phase 4: Extended Example Configurations (2-3 hours) ✅

- [x] Add 9 more scenario JSON files covering specific use cases
- [x] Cover: 06-production HTTPS (staging), 07-UDP-only, 08-HTTP-only HTTPS (staging), 10-multi-domain (staging), 11-private tracker, 12-high-availability (staging), 13-backup-focused (staging), 14-lightweight production (staging), 15-sqlite-monitoring
- [x] Validate all configs
- [x] Document common mistakes in README

### Phase 5: Documentation and Index (1 hour) ✅

- [x] Create `docs/ai-training/README.md` with overview and scenarios table
- [x] Include usage instructions for AI agents and human users
- [x] Add table mapping scenario IDs to files (like "Proposed Example Scenarios" in spec)
- [x] Include guidance on when to use each scenario type

### Phase 6: Integration Test for Examples (30 minutes) ✅

- [x] Create integration test at `tests/validate_ai_training_examples.rs`
- [x] Test iterates over all JSON files in `docs/ai-training/dataset/environment-configs/`
- [x] For each example: run `validate` command and assert success
- [x] Added 4 comprehensive test functions validating all aspects
- [x] Test ensures examples remain valid as schema evolves
- [x] Run test as part of CI to catch regressions early
- [x] Added regex dependency for pattern matching

**Completed**: Integration test created with 4 test functions:

- `it_should_validate_all_ai_training_example_configurations()` - Validates all 15 examples
- `it_should_verify_expected_number_of_examples()` - Ensures exactly 15 files exist
- `it_should_verify_example_naming_convention()` - Checks NN-descriptive-name.json pattern
- `it_should_verify_all_examples_have_descriptions()` - Verifies environment.description field

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [x] All linters pass (markdown, cspell)
- [x] All example JSON configurations validated with `cargo run -- validate --env-file`
- [x] Integration test passes: `cargo test validate_ai_training_examples` (validates all examples automatically)

**Task-Specific Criteria**:

- [x] Optional `description` field added to schema and validated
- [x] Backward compatibility maintained (configs without description still work)
- [x] Questionnaire template created at `docs/ai-training/questionnaire.md`
- [x] All 15 example JSON configurations created with description field
- [x] Each example validated with `cargo run -- validate --env-file`
- [x] All descriptions are 2-3 sentences covering use case + key decisions
- [x] Example configurations use fixture keys only (e.g., `fixtures/testing_rsa`)
- [x] README created with scenarios table, usage instructions, and guidance
- [x] Directory structure matches specification (JSON files only, no markdown per scenario)
- [x] Full-stack scenarios (02 and 04) include all features: MySQL, Prometheus, Grafana, backup, domains
- [x] Scenarios 01-02 (LXD minimal and full-stack) and 03-04 (Hetzner minimal and full-stack) demonstrate the complete spectrum
- [x] All scenarios except 03-04 use LXD for local testing consistency
- [x] All LXD scenarios with HTTPS (02, 06, 08, 10, 12, 13, 14) use staging certificates (`use_staging: true`)
- [x] Hetzner production scenario (04) uses production certificates (no `use_staging` or `use_staging: false`)
- [x] Scenario 13 demonstrates backup without monitoring overhead
- [x] Scenario 14 demonstrates lightweight production (SQLite + HTTPS + backup)
- [x] Scenario 15 demonstrates monitoring stack with SQLite simplicity
- [x] Integration test `tests/validate_ai_training_examples.rs` created and passing
- [x] All examples validated automatically by integration test (prevents regressions)

## Related Documentation

- [Environment Configuration Schema](../../schemas/README.md)
- [JSON Schema](../../schemas/environment-config.json)
- [Create Command Documentation](../user-guide/commands/create.md)
- [Validate Command](../console-commands.md)
- [Issue #274 - Agent Skills](./274-consider-using-agentskills-io.md)
- [Configuration DTO Layer Placement ADR](../decisions/configuration-dto-layer-placement.md)
- [Environment Config README](../../src/application/command_handlers/create/config/README.md) - Authoritative constraints

## Notes

### Integration with Agent Skills

This dataset can be referenced from the `create-environment-config` skill proposed in [Issue #274](./274-consider-using-agentskills-io.md):

```yaml
---
name: create-environment-config
description: |
  Generate valid environment configuration JSON files for the deployer.
  Uses the questionnaire template and example scenarios for guidance.
---

## Before generating a config

1. Read the questionnaire: `docs/ai-training/questionnaire.md`
2. Find a similar example: `docs/ai-training/dataset/environment-configs/`
3. Use `cargo run -- create template` to generate base
4. Customize based on user requirements
5. Validate with `cargo run -- validate --env-file`
```

### Open Questions

1. Should we add a maximum length recommendation for the description field (e.g., 500 chars)?
2. Should we include "anti-patterns" (common mistakes) as separate example configs?
3. Should we version the dataset with deployer releases?
4. Should the description field support structured format (e.g., "Use case: ... | Key decisions: ...") or remain free-text?
