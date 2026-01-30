# Troubleshooting

Common issues encountered during the proof of concept and their solutions.

## SSH Connection Issues

### Host Key Changed Warning

**Symptom**:

```text
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
```

**Cause**: The LXD VM was recreated with the same IP but different host key.

**Solution**:

```bash
ssh-keygen -f ~/.ssh/known_hosts -R 10.140.190.35
```

## Docker Issues

### Container Not Starting

**Symptom**: Container exits immediately after starting.

**Debug**:

```bash
docker compose logs backup
docker compose exec backup sh  # If container is running
```

### Permission Denied

**Symptom**: Backup script can't write to `/backups/`.

**Solution**: Ensure the directory is mounted with correct permissions:

```yaml
volumes:
  - /opt/torrust/backups:/backups
```

And create the directory on the host:

```bash
sudo mkdir -p /opt/torrust/backups
sudo chown torrust:torrust /opt/torrust/backups
```

## MySQL Issues

### Connection Refused

**Symptom**: `mysqldump: Got error: 2003: Can't connect to MySQL server`

**Cause**: MySQL container not ready or wrong hostname.

**Solution**: Ensure `depends_on` with health check:

```yaml
backup:
  depends_on:
    mysql:
      condition: service_healthy
```

### Access Denied

**Symptom**: `Access denied for user 'tracker_user'@'%'`

**Cause**: Wrong credentials or user doesn't have required permissions.

**Debug**:

```bash
docker compose exec mysql mysql -u root -p -e "SHOW GRANTS FOR 'tracker_user'@'%';"
```

## Backup Issues

### Empty Backup File

**Symptom**: Backup file is 0 bytes.

**Debug**:

```bash
# Check if mysqldump runs correctly
docker compose exec backup mysqldump --single-transaction \
  -h mysql -u tracker_user -ptracker_password torrust_tracker | head -20
```

### Disk Space

**Symptom**: `No space left on device`

**Solution**:

```bash
# Check disk space
df -h

# Clean old backups
find /opt/torrust/backups -name "*.tar.gz" -mtime +7 -delete
```

## Environment Issues

### Environment Not Found

**Symptom**: `Environment 'manual-test-sidecar-backup' not found`

**Solution**: Ensure the environment was created:

```bash
ls -la data/manual-test-sidecar-backup/
cargo run -- create environment --env-file envs/manual-test-sidecar-backup.json
```

### VM Not Reachable

**Symptom**: SSH connection timeout.

**Debug**:

```bash
# Check LXD instance
lxc list | grep manual-test-sidecar-backup

# Check if instance is running
lxc info torrust-tracker-vm-manual-test-sidecar-backup
```
