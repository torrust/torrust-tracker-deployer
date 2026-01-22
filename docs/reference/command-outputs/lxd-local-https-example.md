# Command Outputs: LXD Local Example (with TLS)

**Session Date**: 2026-01-22
**Provider**: LXD
**Domain**: `.local`
**Environment Name**: `lxd-local-https-example`
**TLS Proxy**: Enabled (Caddy)

## Environment Configuration

See: [environment-configs/lxd-local-https-example.json](./environment-configs/lxd-local-https-example.json)

## Key Differences from Non-TLS Version

This environment has `use_tls_proxy: true` enabled for all HTTP services:

- HTTP Tracker
- HTTP API
- Health Check API
- Grafana

The `show` command output includes:

- HTTPS URLs with domain names instead of IP-based HTTP URLs
- DNS setup instructions for `/etc/hosts`
- Note about internal ports not being directly accessible

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
cargo run -- create environment --env-file envs/lxd-local-https-example.json
```

**Output:**

```text
⏳ [1/3] Loading configuration...
⏳     → Loading configuration from 'envs/lxd-local-https-example.json'...
⏳   ✓ Configuration loaded: lxd-local-https-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Creating environment...
⏳     → Creating environment 'lxd-local-https-example'...
⏳     → Validating configuration and creating environment...
⏳   ✓ Environment created: lxd-local-https-example (took 1ms)
✅ Environment 'lxd-local-https-example' created successfully

Environment Details:
1. Environment name: lxd-local-https-example
2. Instance name: torrust-tracker-vm-lxd-local-https-example
3. Data directory: ./data/lxd-local-https-example
4. Build directory: ./build/lxd-local-https-example
```

---

## 2. Provision

```bash
cargo run -- provision lxd-local-https-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
⏳   ✓ Infrastructure provisioned (took 36.8s)
✅ Environment 'lxd-local-https-example' provisioned successfully

Instance Connection Details:
  IP Address:        10.140.190.254
  SSH Port:          22
  SSH Private Key:   /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa
  SSH Username:      torrust

Connect using:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa torrust@10.140.190.254 -p 22
```

---

## 3. Configure

```bash
cargo run -- configure lxd-local-https-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 43.4s)
✅ Environment 'lxd-local-https-example' configured successfully
```

---

## 4. Release

```bash
cargo run -- release lxd-local-https-example
```

**Output:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 15.9s)
✅ Release command completed successfully for 'lxd-local-https-example'
```

---

## 5. Run

```bash
cargo run -- run lxd-local-https-example
```

**Output:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 23.9s)
✅ Run command completed for 'lxd-local-https-example'
```

---

## 6. Test

```bash
cargo run -- test lxd-local-https-example
```

**Output:**

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Testing infrastructure...
⏳   ✓ Infrastructure tests passed (took 18ms)
✅ Infrastructure validation completed successfully for 'lxd-local-https-example'
```

---

## 7. Show

```bash
cargo run -- show lxd-local-https-example
```

**Output:**

```text
⏳ [1/3] Validating environment name...
⏳   ✓ Environment name validated: lxd-local-https-example (took 0ms)
⏳ [2/3] Loading environment...
⏳   ✓ Environment loaded: lxd-local-https-example (took 0ms)
⏳ [3/3] Displaying information...

Environment: lxd-local-https-example
State: Running
Provider: LXD
Created: 2026-01-22 15:37:51 UTC

Infrastructure:
  Instance IP: 10.140.190.254
  SSH Port: 22
  SSH User: torrust
  SSH Key: /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa

Connection:
  ssh -i /home/josecelano/Documents/git/committer/me/github/torrust/torrust-tracker-deployer-agent-01/fixtures/testing_rsa torrust@10.140.190.254

Tracker Services:
  UDP Trackers:
    - udp://udp.tracker.local:6969/announce
  HTTP Trackers (HTTPS via Caddy):
    - https://http.tracker.local/announce
  API Endpoint (HTTPS via Caddy):
    - https://api.tracker.local/api
  Health Check (HTTPS via Caddy):
    - https://health.tracker.local/health_check

Prometheus:
  Internal only (localhost:9090) - not exposed externally

Grafana (HTTPS via Caddy):
  https://grafana.tracker.local/

Note: HTTPS services require domain-based access. For local domains (*.local),
add the following to your /etc/hosts file:

  10.140.190.254   http.tracker.local api.tracker.local grafana.tracker.local health.tracker.local

Internal ports (7070, 1212, 3000, 1313) are not directly accessible when TLS is enabled.

Services are running. Use 'test' to verify health.
⏳   ✓ Information displayed (took 0ms)
```

## Key Observations

The `show` command for TLS-enabled environments includes:

1. **HTTPS URLs** - Uses `https://` protocol with domain names
2. **"(HTTPS via Caddy)"** labels - Indicates services are behind the TLS proxy
3. **DNS Setup Instructions** - Provides the `/etc/hosts` entry needed for local domains
4. **Port Access Note** - Warns that internal ports are not directly accessible

This additional output helps users understand:

- What DNS configuration is required
- How to access the services
- Why direct IP:port access won't work
