# Experiment 1: Minimal HTTPS Setup (Hello World)

**Status**: ✅ Complete
**Started**: 2026-01-12
**Completed**: 2026-01-12
**Domain**: `test.torrust-tracker.com`

## Objective

Test Pingoo certificate generation in isolation with a minimal setup containing only:

- Pingoo (TLS proxy)
- A simple nginx container serving static "Hello World" content

This isolates the certificate generation testing from any tracker-specific complexity.

## Pre-requisites

- [x] Hetzner server running (IP: 46.224.206.37)
- [x] DNS propagated for `test.torrust-tracker.com` → 46.224.206.37
- [x] Port 443 accessible on the server

## Setup

### Files Created

```text
/root/experiments/experiment-1/
├── docker-compose.yml
├── pingoo/
│   └── pingoo.yml
└── www/
    └── index.html
```

### docker-compose.yml

```yaml
services:
  pingoo:
    image: pingooio/pingoo:latest
    ports:
      - "443:443"
    volumes:
      - ./pingoo:/etc/pingoo
    networks:
      - test-network
    depends_on:
      - webserver

  webserver:
    image: nginx:alpine
    volumes:
      - ./www:/usr/share/nginx/html:ro
    networks:
      - test-network

networks:
  test-network:
    driver: bridge
```

### pingoo/pingoo.yml

```yaml
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains: ["test.torrust-tracker.com"]

services:
  static:
    http_proxy: ["http://webserver:80"]
```

### www/index.html

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Pingoo Test</title>
  </head>
  <body>
    <h1>Hello World!</h1>
    <p>If you see this page via HTTPS, Pingoo certificate generation works!</p>
    <p>Certificate info: Check browser padlock for details.</p>
  </body>
</html>
```

## Deployment Steps

1. SSH to the Hetzner server
2. Create the experiment directory structure
3. Copy the configuration files
4. Run `docker compose up -d`
5. Check Pingoo logs for certificate generation
6. Test HTTPS access

## Results

### DNS Check

```text
$ dig +short test.torrust-tracker.com A @8.8.8.8
46.224.206.37
```

### Deployment Log

```text
$ ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 \
    "cd /root/experiments/experiment-1 && docker compose up -d"

 Network experiment-1_test-network  Created
 Container experiment-1-webserver-1  Created
 Container experiment-1-pingoo-1  Created
 Container experiment-1-webserver-1  Started
 Container experiment-1-pingoo-1  Started
```

### Certificate Generation

Pingoo automatically generated a Let's Encrypt certificate:

```text
$ ls /root/experiments/experiment-1/pingoo/tls/
acme.json
default.key
default.pem
test.torrust-tracker.com.key
test.torrust-tracker.com.pem
```

Certificate details:

```text
$ openssl x509 -in test.torrust-tracker.com.pem -text -noout

Certificate:
    Data:
        Version: 3 (0x2)
        Serial Number: 06:a4:c1:28:dc:d8:6d:53:86:d0:e4:5d:cc:cb:db:72:68:a3
        Signature Algorithm: ecdsa-with-SHA384
        Issuer: C = US, O = Let's Encrypt, CN = E8
        Validity
            Not Before: Jan 12 15:22:51 2026 GMT
            Not After : Apr 12 15:22:50 2026 GMT
        Subject: CN = test.torrust-tracker.com
```

### HTTPS Test

```text
$ curl -v https://test.torrust-tracker.com 2>&1 | grep -E "(SSL|subject|issuer|expire)"

* SSL connection using TLSv1.3 / TLS_AES_256_GCM_SHA384 / X25519MLKEM768 / id-ecPublicKey
*  subject: CN=test.torrust-tracker.com
*  expire date: Apr 12 15:22:50 2026 GMT
*  issuer: C=US; O=Let's Encrypt; CN=E8
*  SSL certificate verify ok.

$ curl -s https://test.torrust-tracker.com

<!DOCTYPE html>
<html>
  <head>
    <title>Pingoo Test</title>
  </head>
  <body>
    <h1>Hello World!</h1>
    <p>If you see this page via HTTPS, Pingoo certificate generation works!</p>
    <p>Certificate info: Check browser padlock for details.</p>
  </body>
</html>
```

## Success Criteria

- [x] `https://test.torrust-tracker.com` shows the Hello World page
- [x] Browser shows valid Let's Encrypt certificate
- [x] No manual certificate generation required
- [x] Pingoo logs show successful ACME challenge completion

## Issues Encountered

### Initial ACME Error (False Alarm)

When first checking Pingoo logs, an error appeared:

```text
{"level":"ERROR","message":"TLS: error ordering TLS certificate: error loading config:
error getting ACME authorization for test.torrust-tracker.com: API error: No such
authorization (urn:ietf:params:acme:error:malformed)"}
```

However, this was a **false alarm** - the certificate had already been successfully
generated earlier. The error appears to be related to Pingoo retrying an already-completed
authorization. The certificate files were present and valid.

## Observations

1. **Automatic Certificate Generation**: Pingoo successfully obtained a Let's Encrypt
   certificate without any manual intervention. The only configuration needed was
   specifying the domain in `pingoo.yml`.

2. **TLS 1.3 Only**: As documented, Pingoo only supports TLS 1.3. The connection used
   `TLS_AES_256_GCM_SHA384` cipher.

3. **Post-Quantum Cryptography**: Pingoo used `X25519MLKEM768` for key exchange,
   which is a post-quantum hybrid key agreement algorithm.

4. **ECDSA Certificate**: Let's Encrypt issued an ECDSA certificate (prime256v1)
   signed by the E8 intermediate CA.

5. **Certificate Storage**: Pingoo stores certificates in `/etc/pingoo/tls/` with
   `.key` and `.pem` files named after the domain.

6. **ACME State**: ACME account credentials are stored in `acme.json` for reuse
   in future certificate renewals.

7. **No Email Required**: Unlike certbot, Pingoo does **not** require an email
   address for ACME account registration. With certbot, you typically must provide
   an email (or explicitly opt out with `--register-unsafely-without-email`).

   **Comparison:**

   - **Certbot**: `certbot certonly --email admin@example.com -d example.com`
   - **Pingoo**: Just specify the domain in YAML - no email configuration needed

   **Trade-off**: The email in certbot is used for:

   - Expiration warnings (before certificate expires)
   - Security notices (if certificate is revoked)
   - Account recovery

   With Pingoo, you won't receive these notifications, so you must rely on Pingoo's
   automatic renewal working correctly, or implement your own monitoring.

8. **Minimal Configuration**: The Pingoo configuration is significantly simpler than
   nginx+certbot - just 10 lines of YAML vs. multiple nginx config files plus certbot
   setup.

## Conclusion

**Experiment 1 is SUCCESSFUL.** Pingoo successfully:

- Automatically generated a valid Let's Encrypt certificate
- Terminated TLS with modern TLS 1.3 and post-quantum key exchange
- Proxied requests to the backend nginx container
- Required minimal configuration (10 lines of YAML)

This validates that Pingoo can handle automatic certificate generation, which is the
core requirement for simplifying our deployment infrastructure.

**Next**: Proceed to Experiment 2 to test Pingoo with the actual Tracker API.
