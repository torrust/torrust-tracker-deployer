# Phase 1.1b - Manual E2E Testing for Backup Container

**Issue**: [#315 - Implement Backup Support](315-implement-backup-support.md)  
**Phase**: 1.1b (Manual Testing Checkpoint)  
**Status**: In Progress

## Objective

Manually test the backup container in a real deployment environment to verify:

1. Backup container builds and runs successfully
2. Backup files are created correctly for SQLite and MySQL databases
3. Config file backups work as expected
4. Other services (tracker, database, monitoring) remain healthy
5. Backup files contain valid data that can be restored

## Test Plan

### Test 1: SQLite Database Backup

**Environment**: `manual-sqlite-udp-only`

#### Steps

1. **Deploy the environment**:

   ```bash
   cargo run -- create environment --env-file envs/manual-sqlite-udp-only.json
   ```

2. **Build the backup container locally**:

   ```bash
   docker build -t torrust/backup:test docker/backup/
   ```

3. **Locate the generated docker-compose.yml**:

   ```bash
   # Should be at: build/manual-sqlite-udp-only/docker-compose/docker-compose.yml
   ls -la build/manual-sqlite-udp-only/docker-compose/
   ```

4. **Manually add backup service to docker-compose.yml**:

   Edit `build/manual-sqlite-udp-only/docker-compose/docker-compose.yml` and add:

   ```yaml
   services:
     # ... existing services ...

     backup:
       image: torrust/backup:test
       container_name: torrust-backup
       volumes:
         - ./backup/backup.conf:/etc/backup/backup.conf:ro
         - ./backup/backup-paths.txt:/etc/backup/backup-paths.txt:ro
         - ../tracker:/tracker:ro
         - backup-mysql:/backups/mysql
         - backup-sqlite:/backups/sqlite
         - backup-config:/backups/config
       networks:
         - torrust-network
       depends_on:
         - tracker
       restart: "no" # Manual backup, not automatic

   volumes:
     # ... existing volumes ...
     backup-mysql:
     backup-sqlite:
     backup-config:
   ```

5. **Create backup configuration file**:

   Create `build/manual-sqlite-udp-only/docker-compose/backup/backup.conf`:

   ```bash
   mkdir -p build/manual-sqlite-udp-only/docker-compose/backup
   cat > build/manual-sqlite-udp-only/docker-compose/backup/backup.conf << 'EOF'
   # SQLite Backup Configuration
   DB_TYPE=sqlite
   DB_PATH=/tracker/data/tracker.db
   BACKUP_RETENTION_DAYS=7
   BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
   EOF
   ```

6. **Create backup paths file**:

   Create `build/manual-sqlite-udp-only/docker-compose/backup/backup-paths.txt`:

   ```bash
   cat > build/manual-sqlite-udp-only/docker-compose/backup/backup-paths.txt << 'EOF'
   # Tracker configuration files
   /tracker/etc/tracker.toml
   EOF
   ```

7. **Start all services**:

   ```bash
   cd build/manual-sqlite-udp-only/docker-compose
   docker compose up -d
   ```

8. **Wait for tracker to initialize** (creates database):

   ```bash
   # Wait 10-15 seconds for tracker to start and create the database
   sleep 15
   docker compose logs tracker
   ```

9. **Run backup container**:

   ```bash
   docker compose up backup
   ```

10. **Verify backup files created**:

    ```bash
    # List backup volumes
    docker volume ls | grep backup

    # Inspect SQLite backup
    docker run --rm -v manual-sqlite-udp-only_backup-sqlite:/backups alpine ls -lah /backups

    # Inspect config backup
    docker run --rm -v manual-sqlite-udp-only_backup-config:/backups alpine ls -lah /backups
    ```

11. **Verify backup content**:

    ```bash
    # Copy SQLite backup to host for inspection
    docker run --rm -v manual-sqlite-udp-only_backup-sqlite:/backups -v $(pwd):/host alpine \
      cp /backups/$(ls /backups | grep sqlite_) /host/test-sqlite-backup.db.gz

    # Decompress and verify
    gunzip test-sqlite-backup.db.gz
    sqlite3 test-sqlite-backup.db ".tables"
    sqlite3 test-sqlite-backup.db "SELECT * FROM sqlite_master;"
    rm test-sqlite-backup.db

    # Verify config backup
    docker run --rm -v manual-sqlite-udp-only_backup-config:/backups -v $(pwd):/host alpine \
      cp /backups/$(ls /backups | grep config_) /host/test-config-backup.tar.gz

    tar -tzf test-config-backup.tar.gz
    rm test-config-backup.tar.gz
    ```

12. **Check other services are healthy**:

    ```bash
    docker compose ps
    docker compose logs tracker | tail -20
    curl http://localhost:1313/health  # Tracker health check
    ```

13. **Cleanup**:

    ```bash
    docker compose down -v
    cd ../../..
    cargo run -- destroy environment --name manual-sqlite-udp-only
    ```

#### Expected Results

- ✅ Backup container runs without errors
- ✅ SQLite database backup file created in `/backups/sqlite/sqlite_YYYYMMDD_HHMMSS.db.gz`
- ✅ Config backup file created in `/backups/config/config_YYYYMMDD_HHMMSS.tar.gz`
- ✅ SQLite backup contains valid database with correct schema
- ✅ Config backup contains `tracker.toml` at correct path
- ✅ Tracker and other services remain healthy
- ✅ Backup container exits with status 0

### Test 2: MySQL Database Backup

**Environment**: `manual-mysql-test`

#### Steps

1. **Deploy the environment**:

   ```bash
   cargo run -- create environment --env-file envs/manual-mysql-test.json
   ```

2. **Build the backup container** (if not already built):

   ```bash
   docker build -t torrust/backup:test docker/backup/
   ```

3. **Locate the generated docker-compose.yml**:

   ```bash
   ls -la build/manual-mysql-test/docker-compose/
   ```

4. **Manually add backup service to docker-compose.yml**:

   Edit `build/manual-mysql-test/docker-compose/docker-compose.yml` and add:

   ```yaml
   services:
     # ... existing services ...

     backup:
       image: torrust/backup:test
       container_name: torrust-backup
       volumes:
         - ./backup/backup.conf:/etc/backup/backup.conf:ro
         - ./backup/backup-paths.txt:/etc/backup/backup-paths.txt:ro
         - ../tracker:/tracker:ro
         - backup-mysql:/backups/mysql
         - backup-sqlite:/backups/sqlite
         - backup-config:/backups/config
       networks:
         - torrust-network
       depends_on:
         - mysql
         - tracker
       restart: "no"

   volumes:
     # ... existing volumes ...
     backup-mysql:
     backup-sqlite:
     backup-config:
   ```

5. **Create backup configuration file**:

   Create `build/manual-mysql-test/docker-compose/backup/backup.conf`:

   ```bash
   mkdir -p build/manual-mysql-test/docker-compose/backup
   cat > build/manual-mysql-test/docker-compose/backup/backup.conf << 'EOF'
   # MySQL Backup Configuration
   DB_TYPE=mysql
   DB_HOST=mysql
   DB_PORT=3306
   DB_USER=tracker_user
   DB_PASSWORD=tracker_password
   DB_NAME=tracker
   BACKUP_RETENTION_DAYS=7
   BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
   EOF
   ```

6. **Create backup paths file**:

   ```bash
   cat > build/manual-mysql-test/docker-compose/backup/backup-paths.txt << 'EOF'
   # Tracker configuration files
   /tracker/etc/tracker.toml
   EOF
   ```

7. **Start all services**:

   ```bash
   cd build/manual-mysql-test/docker-compose
   docker compose up -d
   ```

8. **Wait for MySQL and tracker to initialize**:

   ```bash
   sleep 20
   docker compose logs mysql | tail -20
   docker compose logs tracker | tail -20
   ```

9. **Run backup container**:

   ```bash
   docker compose up backup
   ```

10. **Verify backup files created**:

    ```bash
    # List backup volumes
    docker volume ls | grep backup

    # Inspect MySQL backup
    docker run --rm -v manual-mysql-test_backup-mysql:/backups alpine ls -lah /backups

    # Inspect config backup
    docker run --rm -v manual-mysql-test_backup-config:/backups alpine ls -lah /backups
    ```

11. **Verify backup content**:

    ```bash
    # Copy MySQL backup to host for inspection
    docker run --rm -v manual-mysql-test_backup-mysql:/backups -v $(pwd):/host alpine \
      cp /backups/$(ls /backups | grep mysql_) /host/test-mysql-backup.sql.gz

    # Decompress and verify SQL content
    gunzip test-mysql-backup.sql.gz
    head -50 test-mysql-backup.sql  # Should show MySQL dump header
    grep -i "CREATE TABLE" test-mysql-backup.sql
    rm test-mysql-backup.sql

    # Verify config backup
    docker run --rm -v manual-mysql-test_backup-config:/backups -v $(pwd):/host alpine \
      cp /backups/$(ls /backups | grep config_) /host/test-config-backup.tar.gz

    tar -tzf test-config-backup.tar.gz
    rm test-config-backup.tar.gz
    ```

12. **Check other services are healthy**:

    ```bash
    docker compose ps
    docker compose logs tracker | tail -20
    docker compose logs mysql | tail -20
    curl http://localhost:1313/health  # Tracker health check
    ```

13. **Test retention policy** (optional):

    ```bash
    # Run backup multiple times with short retention
    # Modify backup.conf: BACKUP_RETENTION_DAYS=0
    docker compose up backup
    sleep 2
    docker compose up backup

    # Verify old backups are cleaned up
    docker run --rm -v manual-mysql-test_backup-mysql:/backups alpine ls -lah /backups
    ```

14. **Cleanup**:

    ```bash
    docker compose down -v
    cd ../../..
    cargo run -- destroy environment --name manual-mysql-test
    ```

#### Expected Results

- ✅ Backup container runs without errors
- ✅ MySQL database backup file created in `/backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz`
- ✅ Config backup file created in `/backups/config/config_YYYYMMDD_HHMMSS.tar.gz`
- ✅ MySQL backup contains valid SQL dump with CREATE TABLE statements
- ✅ Config backup contains `tracker.toml` at correct path
- ✅ Tracker, MySQL, and other services remain healthy
- ✅ Backup container exits with status 0
- ✅ Retention policy works (old backups cleaned up)

## Success Criteria

Phase 1.1b is complete when:

1. ✅ SQLite backup test passes all checks
2. ✅ MySQL backup test passes all checks
3. ✅ Backup files can be restored successfully
4. ✅ No impact on running services
5. ✅ Documentation is updated with test results

## Next Phase

After successful completion of Phase 1.1b, proceed to:

- **Phase 1.2**: GitHub workflow for publishing to Docker Hub
- **Phase 1.2**: Security scanning setup

## Notes

- This is a **manual checkpoint** - automation comes in Phase 2
- Focus on verifying the backup container works correctly in real deployments
- Document any issues or improvements discovered during testing
- Backup container must be manually added to docker-compose.yml for now
- Later phases will automate the integration
