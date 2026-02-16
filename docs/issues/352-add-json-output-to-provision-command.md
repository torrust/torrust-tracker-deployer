# Add JSON Output to Provision Command

**Issue**: [#352](https://github.com/torrust/torrust-tracker-deployer/issues/352)
**Parent Epic**: [#348](https://github.com/torrust/torrust-tracker-deployer/issues/348) - Add JSON output format support
**Related**: [Roadmap Section 12.2](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-add-json-output-format-support), [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed

## Overview

Add machine-readable JSON output format (`--output-format json`) to the `provision` command. This enables automation workflows to programmatically extract the provisioned instance IP address and connection details without regex parsing of console output.

## Goals

- [ ] Implement JSON output format for provision command
- [ ] Preserve existing human-readable output as default
- [ ] Enable automation to extract instance IP reliably
- [ ] Follow the architecture pattern established in #349

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (`src/presentation/`)
**Module Path**: `src/presentation/views/commands/provision/`
**Pattern**: Strategy Pattern with TextView and JsonView (established in #349)

### Module Structure Requirements

- [ ] Follow view layer separation established in #349
- [ ] Use DTO pattern for data transfer (`ProvisionDetailsData`)
- [ ] Implement TextView and JsonView for format switching
- [ ] Wire output_format through ExecutionContext → Router → Controller

### Architectural Constraints

- [ ] No business logic in presentation layer (views only format existing data)
- [ ] No changes to application or domain layers
- [ ] Follow output handling conventions ([docs/contributing/output-handling.md](../contributing/output-handling.md))
- [ ] Use existing `OutputFormat` enum and `--output-format` flag from #349

### Anti-Patterns to Avoid

- ❌ Embedding output formatting logic in controller
- ❌ Mixing business logic with view formatting
- ❌ Changing the application command handler interface

## Specifications

### JSON Output Schema

#### Example 1: HTTPS Configuration (with custom domains)

```json
{
  "environment_name": "full-stack-lxd",
  "instance_name": "torrust-tracker-vm-full-stack-lxd",
  "instance_ip": "10.140.190.39",
  "ssh_username": "torrust",
  "ssh_port": 22,
  "ssh_private_key_path": "/home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa",
  "provider": "lxd",
  "provisioned_at": "2026-02-16T16:00:00Z",
  "domains": [
    "tracker1.example.com",
    "tracker2.example.com",
    "api.example.com",
    "grafana.example.com",
    "health.example.com"
  ]
}
```

#### Example 2: Non-HTTPS Configuration (no custom domains)

```json
{
  "environment_name": "simple-tracker",
  "instance_name": "torrust-tracker-vm-simple-tracker",
  "instance_ip": "10.140.190.40",
  "ssh_username": "torrust",
  "ssh_port": 22,
  "ssh_private_key_path": "/home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa",
  "provider": "lxd",
  "provisioned_at": "2026-02-16T16:00:00Z",
  "domains": []
}
```

> **Note on Schema Flexibility**: The JSON schema shown above is **not mandatory**. The actual JSON output should mirror the structure of the Rust `ProvisionDetailsData` DTO. If the natural Rust serialization (via `#[derive(Serialize)]`) produces a slightly different format that is easier to maintain or more idiomatic, **prefer the Rust-native structure**. The goal is simplicity and consistency with the codebase, not rigid adherence to a predefined schema. The examples above serve as a guide for the expected information, but field names and structure can be adjusted to match what `serde_json` naturally produces from the DTO.

### Field Descriptions

| Field                  | Type     | Description                                                       |
| ---------------------- | -------- | ----------------------------------------------------------------- |
| `environment_name`     | string   | Name of the environment                                           |
| `instance_name`        | string   | Full VM instance name                                             |
| `instance_ip`          | string   | IP address of the provisioned instance                            |
| `ssh_username`         | string   | SSH username for connecting to the instance                       |
| `ssh_port`             | number   | SSH port (typically 22)                                           |
| `ssh_private_key_path` | string   | Absolute path to SSH private key file for authentication          |
| `provider`             | string   | Infrastructure provider used (lxd or hetzner)                     |
| `provisioned_at`       | string   | ISO 8601 timestamp of provisioning completion                     |
| `domains`              | string[] | Custom domains configured (empty array for non-HTTPS deployments) |

### CLI Interface

```bash
# Human-readable output (default, unchanged)
torrust-tracker-deployer provision my-env

# JSON output (new)
torrust-tracker-deployer provision my-env --output-format json

# Short form
torrust-tracker-deployer provision my-env -o json
```

### Human-Readable Output (Reference - Must Not Change)

The default text output should remain unchanged.

**Test Command:**

```bash
# Using the AI training dataset environment config
torrust-tracker-deployer provision full-stack-lxd
```

**Actual Output (Captured 2026-02-16):**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: full-stack-lxd (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
⏳   ✓ Infrastructure provisioned (took 27.9s)
✅ Environment 'full-stack-lxd' provisioned successfully

Instance Connection Details:
  IP Address:        10.140.190.39
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa torrust@10.140.190.39 -p 22

⚠️  DNS Setup Required:
  Your configuration uses custom domains. Remember to update your DNS records
  to point your domains to the server IP: 10.140.190.39

  Configured domains:
    - tracker1.example.com
    - tracker2.example.com
    - api.example.com
    - grafana.example.com
    - health.example.com
```

**Critical Requirements:**

- ✅ Progress indicators must remain unchanged
- ✅ Connection details format must be preserved
- ✅ SSH command example must be included
- ✅ DNS warning must appear for HTTPS configurations
- ✅ All domain names must be listed when applicable

### Automation Use Cases

**Primary use case**: Extract instance IP address and SSH credentials for subsequent automation steps.

**Common automation workflows**:

1. **SSH Connection Automation**:
   - Extract IP, username, port, and private key path from JSON
   - Automated SSH connectivity testing
   - No manual key management required

2. **DNS Update Automation** (for HTTPS deployments):
   - Extract IP address and domains list from JSON
   - Automated DNS record updates (A records pointing to instance IP)
   - Multi-domain configuration support

3. **Complete Deployment Pipeline**:
   - Provision infrastructure → Get IP and credentials from JSON
   - Update DNS records with the IP (if domains present)
   - Wait for SSH to be available (using private key path)
   - Continue with configure/release/run commands

## Implementation Plan

### Phase 0: Understand Existing Architecture (Reference Implementation Available)

**Goal**: Study the create command's JSON output implementation and identify parallel structures in provision.

**Reference Implementation** - Study these files from issue #349:

- ✅ `src/presentation/views/commands/create/environment_details.rs` - DTO pattern
- ✅ `src/presentation/views/commands/create/text_view.rs` - TextView implementation
- ✅ `src/presentation/views/commands/create/json_view.rs` - JsonView implementation
- ✅ `src/presentation/views/commands/create/mod.rs` - Module exports
- ✅ `src/presentation/controllers/create/router.rs` - output_format extraction
- ✅ `src/presentation/controllers/create/subcommands/environment/handler.rs` - Format switching in `display_creation_results()`

**Existing Provision Structures:**

- ✅ `src/presentation/views/commands/provision/` exists with:
  - `connection_details.rs` - Existing view component (needs refactoring)
  - `dns_reminder.rs` - Existing view component
  - `mod.rs` - Module exports
- ✅ `src/presentation/controllers/provision/handler.rs` - Controller that needs output_format parameter

**Key Pattern from Create Command:**

```rust
// 1. Convert domain model to DTO
let details = EnvironmentDetailsData::from(environment);

// 2. Strategy Pattern for rendering
let output = match output_format {
    OutputFormat::Text => TextView::render(&details),
    OutputFormat::Json => JsonView::render(&details).map_err(|e| {
        Error::OutputFormatting { reason: format!("...: {e}") }
    })?,
};

// 3. Output the result
self.progress.result(&output)?;
```

**Action Items:**

- [ ] Review how create command implements the Strategy Pattern
- [ ] Identify where provision controller currently generates output
- [ ] Plan refactoring of existing `connection_details.rs` view into TextView
- [ ] Understand how `dns_reminder.rs` fits into the new structure

### Phase 1: Create Data Transfer Object

**Goal**: Define the data structure for provision results.

> **Important**: The DTO structure should prioritize simplicity and natural Rust serialization. The JSON schema in this spec is a guide, not a strict requirement. If adjusting field names, types, or structure simplifies the implementation or makes the JSON serialization more straightforward, **make those changes**. The DTO should be whatever structure makes sense for the Rust code, and the JSON will naturally follow via `#[derive(Serialize)]`.

- [ ] Create or verify `ProvisionDetailsData` struct in `src/presentation/views/commands/provision/provision_details.rs`
- [ ] Add fields: environment_name, instance_name, instance_ip, ssh_username, ssh_port, ssh_private_key_path, provider, provisioned_at, domains
- [ ] Add `#[derive(Debug, Clone, Serialize)]` for JSON support
- [ ] Implement `From<&Environment<Provisioned>>` conversion
- [ ] Extract timestamp from environment state
- [ ] Extract domains list from environment configuration (empty Vec for non-HTTPS configs)
- [ ] **Note**: Adjust field names/types if needed to match domain model naturally

**Files**:

- `src/presentation/views/commands/provision/provision_details.rs` (new or modify)

### Phase 2: Implement View Strategies

**Goal**: Create TextView and JsonView implementations.

- [ ] Create or verify `src/presentation/views/commands/provision/text_view.rs`
  - Implement `render(&ProvisionDetailsData) -> String` for text format
  - Match existing human-readable output exactly
- [ ] Create `src/presentation/views/commands/provision/json_view.rs`
  - Implement `render(&ProvisionDetailsData) -> Result<String, serde_json::Error>`
  - Pretty-print JSON output
- [ ] Update `src/presentation/views/commands/provision/mod.rs` to export both views

**Files**:

- `src/presentation/views/commands/provision/text_view.rs` (new or verify)
- `src/presentation/views/commands/provision/json_view.rs` (new)
- `src/presentation/views/commands/provision/mod.rs` (export views)

### Phase 3: Wire Format Switching in Controller

**Goal**: Add format switching logic to the provision command controller.

- [ ] Update `src/presentation/controllers/provision/router.rs`
  - Extract `output_format` from `context.output_format()`
  - Pass to controller's `execute()` method
- [ ] Update `src/presentation/controllers/provision/handler.rs`
  - Add `output_format: OutputFormat` parameter to `execute()` signature
  - Add `display_provision_results()` method with format switching:

    ```rust
    match output_format {
        OutputFormat::Text => TextView::render(&data),
        OutputFormat::Json => JsonView::render(&data).map_err(...)?,
    }
    ```

  - Call `self.progress.result(&output)?;` with formatted output

- [ ] Update `src/presentation/controllers/provision/errors.rs`
  - Add `OutputFormatting { reason: String }` error variant (if not already present from #349)

**Files**:

- `src/presentation/controllers/provision/router.rs`
- `src/presentation/controllers/provision/handler.rs`
- `src/presentation/controllers/provision/errors.rs`

### Phase 4: Update Tests

**Goal**: Ensure all tests pass with the new parameter.

- [ ] Update provision controller tests with `OutputFormat::Text` parameter
- [ ] Update any integration tests that call provision command
- [ ] Verify E2E tests still pass with default text output
- [ ] Add manual test cases (see Testing section below)

**Files**:

- `src/presentation/controllers/provision/tests/*.rs`
- Any E2E test files that use provision command

### Phase 5: Testing and Validation

**Goal**: Verify implementation with manual and automated tests.

- [ ] Test default text output unchanged
- [ ] Test JSON output with `--output-format json`
- [ ] Test JSON validation with `jq`
- [ ] Test IP extraction workflow
- [ ] Test with both LXD and Hetzner providers
- [ ] Run pre-commit checks

### Phase 6: Documentation

**Goal**: Document the JSON output feature for users.

- [ ] Update `docs/user-guide/commands/provision.md`
  - Add "Output Formats" section (similar to create command)
  - Document JSON schema with field descriptions
  - Add automation examples (IP extraction, CI/CD, Python)
  - Explain stdout/stderr separation
- [ ] Update `docs/user-guide/commands.md` (if not already updated from #349)
  - Ensure `--output-format` is in common options
- [ ] Update this specification with actual file paths found during implementation

**Files**:

- `docs/user-guide/commands/provision.md`
- `docs/user-guide/commands.md` (verify common options include --output-format)

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All 2230+ unit tests pass
- [ ] All doctests pass
- [ ] All linters pass (clippy, rustfmt, markdown, yaml, toml, cspell)
- [ ] No unused dependencies (cargo-machete)

**Architecture**:

- [ ] View layer properly separated (following MVC pattern from #349)
- [ ] Controller delegates output formatting to view
- [ ] No changes to application or domain layers
- [ ] Consistent with create command architecture

**Functionality**:

- [ ] With `--output-format json`, command outputs valid JSON to stdout
- [ ] JSON contains all specified fields with correct values
- [ ] JSON is parsable by `jq` and other standard tools
- [ ] Without flag (or with `--output-format text`), output is unchanged
- [ ] Instance IP address is correctly extracted
- [ ] Works with both LXD and Hetzner providers
- [ ] Progress logs go to stderr, JSON goes to stdout

**Documentation**:

- [ ] User guide updated with JSON output section
- [ ] JSON schema documented with field descriptions
- [ ] At least 3 automation examples provided (Shell, CI/CD, Python)
- [ ] IP extraction use case documented
- [ ] stdout/stderr separation explained

**User Experience**:

- [ ] Default behavior (no flag) identical to before
- [ ] JSON output is pretty-printed for readability
- [ ] Timestamps use ISO 8601 format
- [ ] Provider field matches actual provider used

## Testing

### Manual Test Cases

#### Test 1: Default Text Output Unchanged

```bash
torrust-tracker-deployer provision test-env
```

Expected: Human-readable text output with connection details (no JSON).

#### Test 2: JSON Output Format

```bash
torrust-tracker-deployer provision test-env --output-format json
```

Expected: Valid JSON with all fields present.

#### Test 3: JSON Validation with jq

```bash
torrust-tracker-deployer provision test-env -o json --log-output file-only | jq .
```

Expected: `jq` successfully parses and pretty-prints the JSON.

#### Test 4: Extract Instance IP

```bash
IP=$(torrust-tracker-deployer provision test-env -o json --log-output file-only | jq -r '.instance_ip')
echo "Provisioned IP: $IP"
```

Expected: IP address is extracted correctly.

#### Test 5: LXD Provider

```bash
# With LXD environment config
torrust-tracker-deployer provision lxd-test-env -o json
```

Expected: JSON includes `"provider": "lxd"`.

#### Test 6: Hetzner Provider

```bash
# With Hetzner environment config
torrust-tracker-deployer provision hetzner-test-env -o json
```

Expected: JSON includes `"provider": "hetzner"`.

#### Test 7: SSH Private Key Path Extraction

```bash
PRIVATE_KEY=$(torrust-tracker-deployer provision test-env -o json --log-output file-only | jq -r '.ssh_private_key_path')
echo "SSH Private Key: $PRIVATE_KEY"
# Verify the file exists
test -f "$PRIVATE_KEY" && echo "✓ Key file exists" || echo "✗ Key file not found"
```

Expected: Private key path is extracted correctly and file exists.

#### Test 8: Domains Field - HTTPS Configuration

```bash
# With HTTPS-enabled environment config (custom domains configured)
DOMAINS=$(torrust-tracker-deployer provision https-test-env -o json --log-output file-only | jq -r '.domains[]')
echo "Configured domains:"
echo "$DOMAINS"
```

Expected: JSON includes array of domain names (e.g., tracker.example.com, api.example.com).

#### Test 9: Domains Field - Non-HTTPS Configuration

```bash
# With non-HTTPS environment config (no custom domains)
DOMAINS=$(torrust-tracker-deployer provision http-test-env -o json --log-output file-only | jq -r '.domains')
echo "Domains: $DOMAINS"
```

Expected: JSON includes empty array `[]`.

#### Test 10: Output Channel Separation

```bash
torrust-tracker-deployer provision test-env -o json > output.json 2> logs.txt
```

Expected:

- `output.json` contains only JSON (no log messages)
- `logs.txt` contains progress logs (no JSON)

#### Test 11: Automation Workflow

```bash
#!/bin/bash
# Provision and immediately test SSH connectivity

ENV_NAME="automation-test"

# Provision instance
echo "Provisioning $ENV_NAME..."
JSON=$(torrust-tracker-deployer provision "$ENV_NAME" \
  --output-format json \
  --log-output file-only)

# Extract connection details
IP=$(echo "$JSON" | jq -r '.instance_ip')
USERNAME=$(echo "$JSON" | jq -r '.ssh_username')
PORT=$(echo "$JSON" | jq -r '.ssh_port')
PRIVATE_KEY=$(echo "$JSON" | jq -r '.ssh_private_key_path')

echo "Instance provisioned at $IP"

# Wait for SSH to be available
echo "Waiting for SSH..."
MAX_ATTEMPTS=60
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
  if ssh -i "$PRIVATE_KEY" -o ConnectTimeout=2 -o StrictHostKeyChecking=no \
         "$USERNAME@$IP" -p "$PORT" exit 2>/dev/null; then
    echo "✓ SSH connection successful"
    break
  fi
  ATTEMPT=$((ATTEMPT + 1))
  sleep 5
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
  echo "✗ Timeout waiting for SSH"
  exit 1
fi
```

Expected: Script successfully provisions and verifies SSH connectivity.

### Automation Examples

#### Shell Script - Complete Deployment

```bash
#!/bin/bash
set -e

ENV_NAME="production"

# 1. Provision infrastructure
echo "Step 1: Provisioning infrastructure..."
PROVISION_DATA=$(torrust-tracker-deployer provision "$ENV_NAME" \
  --output-format json \
  --log-output file-only)

IP=$(echo "$PROVISION_DATA" | jq -r '.instance_ip')
echo "✓ Instance provisioned: $IP"

# 2. Update DNS
echo "Step 2: Updating DNS..."
./scripts/update-dns.sh --domain tracker.example.com --ip "$IP"
echo "✓ DNS updated"

# 3. Wait for SSH
echo "Step 3: Waiting for SSH..."
PRIVATE_KEY=$(echo "$PROVISION_DATA" | jq -r '.ssh_private_key_path')
USERNAME=$(echo "$PROVISION_DATA" | jq -r '.ssh_username')
./scripts/wait-for-ssh.sh "$IP" "$USERNAME" 22 "$PRIVATE_KEY"
echo "✓ SSH ready"

# 4. Continue deployment
echo "Step 4: Configuring instance..."
torrust-tracker-deployer configure "$ENV_NAME"

echo "Step 5: Releasing application..."
torrust-tracker-deployer release "$ENV_NAME"

echo "Step 6: Starting services..."
torrust-tracker-deployer run "$ENV_NAME"

echo "✓ Deployment complete!"
```

#### CI/CD Pipeline (GitHub Actions)

```yaml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install deployer
        run: |
          cargo install --path .

      - name: Provision infrastructure
        id: provision
        run: |
          OUTPUT=$(torrust-tracker-deployer provision production \
            --env-file .github/production-config.json \
            --output-format json \
            --log-output file-only)

          echo "instance_ip=$(echo $OUTPUT | jq -r '.instance_ip')" >> $GITHUB_OUTPUT
          echo "instance_name=$(echo $OUTPUT | jq -r '.instance_name')" >> $GITHUB_OUTPUT

      - name: Configure DNS
        env:
          CLOUDFLARE_TOKEN: ${{ secrets.CLOUDFLARE_TOKEN }}
        run: |
          curl -X PUT "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
            -H "Authorization: Bearer $CLOUDFLARE_TOKEN" \
            -H "Content-Type: application/json" \
            --data '{"type":"A","name":"tracker.example.com","content":"${{ steps.provision.outputs.instance_ip }}"}'

      - name: Wait for SSH availability
        run: |
          IP="${{ steps.provision.outputs.instance_ip }}"
          # In CI/CD, SSH keys are typically configured separately
          # This example assumes SSH key is already loaded in the runner
          for i in {1..30}; do
            if ssh -o ConnectTimeout=2 -o StrictHostKeyChecking=no deployer@$IP exit 2>/dev/null; then
              echo "SSH ready"
              exit 0
            fi
            sleep 10
          done
          exit 1

      - name: Complete deployment
        run: |
          torrust-tracker-deployer configure production
          torrust-tracker-deployer release production
          torrust-tracker-deployer run production

      - name: Verify deployment
        run: |
          IP="${{ steps.provision.outputs.instance_ip }}"
          curl -f https://tracker.example.com/api/health || exit 1
```

#### Python Script - Multi-Region Deployment

```python
#!/usr/bin/env python3
"""
Provision multiple tracker instances across different regions.
"""
import json
import subprocess
import time
from typing import Dict, List

def provision_instance(env_name: str, region: str) -> Dict:
    """Provision instance and return connection details."""
    print(f"Provisioning {env_name} in {region}...")

    result = subprocess.run(
        [
            "torrust-tracker-deployer",
            "provision", env_name,
            "--output-format", "json",
            "--log-output", "file-only"
        ],
        capture_output=True,
        text=True,
        check=True
    )

    data = json.loads(result.stdout)
    print(f"✓ Provisioned {data['instance_name']} at {data['instance_ip']}")
    return data

def wait_for_ssh(ip: str, port: int, username: str, private_key_path: str, timeout: int = 300) -> bool:
    """Wait for SSH to become available."""
    start = time.time()
    while time.time() - start < timeout:
        result = subprocess.run(
            ["ssh", "-i", private_key_path, "-o", "ConnectTimeout=2",
             "-o", "StrictHostKeyChecking=no", f"{username}@{ip}", "-p", str(port), "exit"],
            capture_output=True
        )
        if result.returncode == 0:
            return True
        time.sleep(5)
    return False

def deploy_multi_region():
    """Deploy tracker instances across multiple regions."""
    regions = [
        {"env": "tracker-us-east", "region": "us-east"},
        {"env": "tracker-eu-west", "region": "eu-west"},
        {"env": "tracker-ap-south", "region": "ap-south"}
    ]

    instances = []

    # Provision all instances
    for config in regions:
        try:
            details = provision_instance(config["env"], config["region"])
            instances.append({
                "env": config["env"],
                "region": config["region"],
                "ip": details["instance_ip"],
                "name": details["instance_name"],
                "username": details["ssh_username"],
                "port": details["ssh_port"],
                "private_key": details["ssh_private_key_path"]
            })
        except subprocess.CalledProcessError as e:
            print(f"✗ Failed to provision {config['env']}: {e}")
            continue

    # Wait for all instances to be ready
    print("\nWaiting for SSH availability...")
    for instance in instances:
        print(f"  Checking {instance['name']}...")
        if wait_for_ssh(instance["ip"], instance["port"], instance["username"], instance["private_key"]):
            print(f"  ✓ {instance['name']} ready")
        else:
            print(f"  ✗ {instance['name']} timeout")

    # Save instance registry
    with open("instances.json", "w") as f:
        json.dump(instances, f, indent=2)

    print(f"\n✓ Deployed {len(instances)} instances")
    print("Instance details saved to instances.json")

if __name__ == "__main__":
    deploy_multi_region()
```

## Related Documentation

- [Epic #348 - Add JSON output format support](https://github.com/torrust/torrust-tracker-deployer/issues/348)
- [Issue #349 - Add JSON output to create command](https://github.com/torrust/torrust-tracker-deployer/issues/349) ✅ Completed (reference implementation)
- [Roadmap Section 12.2](../roadmap.md#12-add-json-output-format-support)
- [Output Handling Conventions](../contributing/output-handling.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [User Guide - Provision Command](../user-guide/commands/provision.md)

## Notes

### Foundation from Issue #349

The infrastructure for JSON output is already implemented:

- ✅ `OutputFormat` enum in `src/presentation/input/cli/output_format.rs`
- ✅ Global `--output-format` flag in `GlobalArgs`
- ✅ `ExecutionContext::output_format()` accessor
- ✅ Strategy Pattern established (TextView and JsonView)

This task only needs to apply the pattern to the provision command.

### Critical Field: Instance IP

The **instance_ip** field is the most important for automation. It must be:

- Correctly extracted from the provisioned environment state
- Available immediately after provisioning completes
- Usable for SSH connection attempts
- Valid for both LXD and Hetzner providers

### SSH Private Key Path

The **ssh_private_key_path** field is essential for SSH automation:

- Must be the absolute path to the private key file
- Required for automated SSH connections to the provisioned instance
- Should match the path specified in the environment configuration
- Enables fully automated SSH workflows without manual key management

### Domains Field

The **domains** array is conditionally populated:

- **HTTPS-enabled configs**: Contains all configured custom domains (tracker, API, Grafana, health check)
- **Non-HTTPS configs**: Empty array `[]`
- Used for DNS automation workflows
- Enables automated DNS record updates after provisioning
- Critical for multi-domain HTTPS deployments

### Provider-Specific Considerations

- **LXD**: IP is typically in private range (e.g., 192.168.x.x or 10.x.x.x)
- **Hetzner**: IP is public-facing (e.g., 116.203.x.x)
- Both should work the same way from the JSON output perspective
- SSH private key path is consistent across both providers

### Success Metrics

This implementation will be successful when:

- [ ] CI/CD pipelines can extract IP without regex
- [ ] SSH automation works without manual key management (using ssh_private_key_path)
- [ ] DNS automation can be triggered immediately after provisioning (using domains array)
- [ ] Multi-region deployments can be automated
- [ ] Pattern is established for remaining JSON output tasks (tasks 12.3, 12.4, 12.5)

## Reference Implementation (Issue #349)

The create command implementation serves as the reference pattern. Study these files before implementing:

> **Schema Flexibility Principle**: The JSON schemas in this specification are guides, not strict contracts. The actual JSON output should naturally follow from the Rust DTO structure with `#[derive(Serialize)]`. If the DTO needs different field names, types, or structure to better match the domain model or simplify serialization, **make those changes**. The implementation should prioritize clean Rust code over matching a predefined JSON schema. What matters is that the JSON contains all necessary information for automation - the exact structure can follow what's natural for the Rust implementation.

### DTO Pattern

**File**: `src/presentation/views/commands/create/environment_details.rs`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    pub data_dir: PathBuf,
    pub build_dir: PathBuf,
    pub created_at: DateTime<Utc>,
}

// Conversion from domain model to DTO (in presentation layer, not domain)
impl From<&Environment<Created>> for EnvironmentDetailsData {
    fn from(environment: &Environment<Created>) -> Self {
        Self {
            environment_name: environment.name().as_str().to_string(),
            instance_name: environment.instance_name().as_str().to_string(),
            data_dir: environment.data_dir().clone(),
            build_dir: environment.build_dir().clone(),
            created_at: environment.created_at(),
        }
    }
}
```

### TextView Pattern

**File**: `src/presentation/views/commands/create/text_view.rs`

```rust
pub struct TextView;

impl TextView {
    #[must_use]
    pub fn render(data: &EnvironmentDetailsData) -> String {
        let mut lines = Vec::new();

        lines.push("Environment Details:".to_string());
        lines.push(format!("1. Environment name: {}", data.environment_name));
        lines.push(format!("2. Instance name: {}", data.instance_name));
        lines.push(format!("3. Data directory: {}", data.data_dir.display()));
        lines.push(format!("4. Build directory: {}", data.build_dir.display()));

        lines.join("\n")
    }
}
```

### JsonView Pattern

**File**: `src/presentation/views/commands/create/json_view.rs`

```rust
pub struct JsonView;

impl JsonView {
    pub fn render(data: &EnvironmentDetailsData) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(data)
    }
}
```

### Controller Format Switching Pattern

**File**: `src/presentation/controllers/create/subcommands/environment/handler.rs`

```rust
fn display_creation_results(
    &mut self,
    environment: &Environment<Created>,
    output_format: OutputFormat,
) -> Result<(), CreateEnvironmentCommandError> {
    self.progress.complete(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ))?;

    self.progress.blank_line()?;

    // 1. Convert domain model to presentation DTO
    let details = EnvironmentDetailsData::from(environment);

    // 2. Strategy Pattern: Select view based on output format
    let output = match output_format {
        OutputFormat::Text => TextView::render(&details),
        OutputFormat::Json => JsonView::render(&details).map_err(|e| {
            CreateEnvironmentCommandError::OutputFormatting {
                reason: format!("Failed to serialize environment details as JSON: {e}"),
            }
        })?,
    };

    // 3. Output the rendered result
    self.progress.result(&output)?;

    Ok(())
}
```

### Router Pattern

**File**: `src/presentation/controllers/create/router.rs`

```rust
pub async fn route_command(
    action: CreateAction,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateCommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            let output_format = context.output_format();  // Extract from context
            context
                .container()
                .create_environment_controller()
                .execute(&env_file, working_dir, output_format)  // Pass to controller
                .await
                .map(|_| ())
                .map_err(CreateCommandError::Environment)
        }
        // ... other actions
    }
}
```

### Module Exports Pattern

**File**: `src/presentation/views/commands/create/mod.rs`

```rust
pub mod environment_details;
pub mod json_view;
pub mod text_view;

// Re-export main types for convenience
pub use environment_details::EnvironmentDetailsData;
pub use json_view::JsonView;
pub use text_view::TextView;
```

### Key Architectural Principles

1. **Separation of Concerns**:
   - DTO defines data structure (`provision_details.rs`)
   - Views handle formatting (`text_view.rs`, `json_view.rs`)
   - Controller orchestrates workflow (`handler.rs`)

2. **Strategy Pattern**:
   - Multiple rendering strategies for same data
   - Easy to add new formats (XML, YAML, etc.)
   - No modification to existing code (Open/Closed Principle)

3. **DDD Layering**:
   - Domain models stay clean (no presentation concerns)
   - DTOs and views in presentation layer
   - `From<&Environment<State>>` conversion in presentation layer

4. **Error Handling**:
   - JSON serialization errors mapped to domain-specific errors
   - Clear error messages for users

5. **Testing**:
   - Each view independently testable
   - Controller tests use `OutputFormat::Text` by default
   - Integration tests verify both formats work end-to-end

### For Provision Command Implementation

Apply the same pattern to provision command:

- **DTO**: `ProvisionDetailsData` with fields: environment_name, instance_name, instance_ip, ssh_username, ssh_port, ssh_private_key_path, provider, provisioned_at, domains
- **TextView**: Render existing human-readable format (reuse/refactor `connection_details.rs`)
- **JsonView**: Serialize DTO as pretty-printed JSON
- **Controller**: Add `display_provision_results()` method with format switching
- **Router**: Pass `output_format` from context to controller
- **Module**: Export all views and DTO
