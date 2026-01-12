# Experiment 1: Minimal HTTPS Setup (Hello World)

**Status**: In Progress
**Started**: 2026-01-12
**Domain**: `test.torrust-tracker.com`

## Objective

Test Pingoo certificate generation in isolation with a minimal setup containing only:

- Pingoo (TLS proxy)
- A simple nginx container serving static "Hello World" content

This isolates the certificate generation testing from any tracker-specific complexity.

## Pre-requisites

- [x] Hetzner server running (IP: 46.224.206.37)
- [x] DNS propagated for `test.torrust-tracker.com` → 46.224.206.37
- [ ] Port 443 accessible on the server

## Setup

### Files Created

```text
experiments/pingoo-hello-world/
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
# TBD - waiting for DNS propagation
```

### Deployment Log

```text
# TBD
```

### Certificate Generation

```text
# TBD - will capture Pingoo logs showing ACME certificate request
```

### HTTPS Test

```text
# TBD - will test with curl and browser
```

## Success Criteria

- [ ] `https://test.torrust-tracker.com` shows the Hello World page
- [ ] Browser shows valid Let's Encrypt certificate
- [ ] No manual certificate generation required
- [ ] Pingoo logs show successful ACME challenge completion

## Issues Encountered

None yet.

## Observations

TBD after experiment completion.

## Conclusion

TBD after experiment completion.
