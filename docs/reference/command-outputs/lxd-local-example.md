# Command Outputs: LXD Local Example (without TLS)

**Session Date**: 2026-01-22
**Provider**: LXD
**Domain**: `.local`
**Environment Name**: `lxd-local-example`
**TLS Proxy**: Disabled

## Environment Configuration

See: [environment-configs/lxd-local-example.json](./environment-configs/lxd-local-example.json)

## Command Sequence

The full deployment workflow consists of these commands in order:

1. `create environment` - Create the environment from configuration
2. `provision` - Provision the VM infrastructure
3. `configure` - Configure the provisioned instance
4. `release` - Deploy application files
5. `run` - Start the services
6. `test` - Verify deployment health
7. `show` - Display environment information

---

## 1. Create Environment

```bash
cargo run -- create environment --env-file envs/lxd-local-example.json
```

**Output:**

```text
⏳ [1/3] Loading configuration...
⏳     → Loading configuration from 'envs/lxd-local-example.json'...
⏳   ✓ Configuration loaded: lxd-local-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Creating environment...
⏳     → Creating environment 'lxd-local-example'...
⏳     → Validating configuration and creating environment...
⏳   ✓ Environment created: lxd-local-example (took 1ms)
✅ Environment 'lxd-local-example' created successfully

Environment Details:
1. Environment name: lxd-local-example
2. Instance name: torrust-tracker-vm-lxd-local-example
3. Data directory: ./data/lxd-local-example
4. Build directory: ./build/lxd-local-example
```

---

## 2. Provision

```bash
cargo run -- provision lxd-local-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
⏳   ✓ Infrastructure provisioned (took 44.0s)
✅ Environment 'lxd-local-example' provisioned successfully

Instance Connection Details:
  IP Address:        10.140.190.35
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa torrust@10.140.190.35 -p 22
```

---

## 3. Configure

```bash
cargo run -- configure lxd-local-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 42.5s)
✅ Environment 'lxd-local-example' configured successfully
```

---

## 4. Release

```bash
cargo run -- release lxd-local-example
```

**Output:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 13.4s)
✅ Release command completed successfully for 'lxd-local-example'
```

---

## 5. Run

```bash
cargo run -- run lxd-local-example
```

**Output:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 22.9s)
✅ Run command completed for 'lxd-local-example'
```

---

## 6. Test

```bash
cargo run -- test lxd-local-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Testing infrastructure...
⏳   ✓ Infrastructure tests passed (took 18ms)
✅ Infrastructure validation completed successfully for 'lxd-local-example'
```

---

## 7. Show

```bash
cargo run -- show lxd-local-example
```

**Output:**

```text
⏳ [1/3] Validating environment name...
⏳   ✓ Environment name validated: lxd-local-example (took 0ms)
⏳ [2/3] Loading environment...
⏳   ✓ Environment loaded: lxd-local-example (took 0ms)
⏳ [3/3] Displaying information...

Environment: lxd-local-example
State: Running
Provider: LXD
Created: 2026-01-22 14:04:28 UTC

Infrastructure:
  Instance IP: 10.140.190.35
  SSH Port: 22
  SSH User: torrust
  SSH Key: /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa

Connection:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa torrust@10.140.190.35

Tracker Services:
  UDP Trackers:
    - udp://udp.tracker.local:6969/announce
  HTTP Trackers (direct):
    - http://10.140.190.35:7070/announce
  API Endpoint:
    - http://10.140.190.35:1212/api
  Health Check:
    - http://10.140.190.35:1313/health_check

Prometheus:
  Internal only (localhost:9090) - not exposed externally

Grafana:
  http://10.140.190.35:3000/

Services are running. Use 'test' to verify health.
⏳   ✓ Information displayed (took 0ms)
```
