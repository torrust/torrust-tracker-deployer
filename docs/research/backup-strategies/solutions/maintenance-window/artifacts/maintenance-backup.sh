#!/bin/bash
# ============================================================================
# Maintenance Window Backup Script
# ============================================================================
# Host-level orchestration script that stops the tracker, runs a backup
# container once, and restarts the tracker.
#
# Usage:
#   ./maintenance-backup.sh [options]
#
# Options:
#   --dry-run     Show what would be done without executing
#   --force       Run backup even if services are down
#   --help        Show this help message
#
# Environment Variables:
#   COMPOSE_DIR        - Path to docker compose directory (default: /opt/tracker)
#   LOG_FILE           - Path to log file (default: /var/log/tracker-backup.log)
#   TRACKER_SERVICES   - Services to stop (default: "tracker")
#   BACKUP_SERVICE     - Backup service name (default: "backup")
#
# Exit Codes:
#   0 - Success
#   1 - Backup failed but tracker restarted
#   2 - Configuration error
#   3 - Services not running (skipped, unless --force)
# ============================================================================

set -euo pipefail

# =============================================================================
# Configuration (can be overridden via environment)
# =============================================================================

COMPOSE_DIR="${COMPOSE_DIR:-/opt/torrust}"
LOG_FILE="${LOG_FILE:-/opt/torrust/storage/backup/log/maintenance-backup.log}"
TRACKER_SERVICES="${TRACKER_SERVICES:-tracker}"
BACKUP_SERVICE="${BACKUP_SERVICE:-backup}"

# Runtime flags
DRY_RUN=false
FORCE=false

# =============================================================================
# Logging
# =============================================================================

log() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $1" | tee -a "$LOG_FILE"
}

log_error() {
    log "ERROR: $1"
}

log_warning() {
    log "WARNING: $1"
}

log_header() {
    log "=== $1 ==="
}

# =============================================================================
# Argument Parsing
# =============================================================================

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 2
                ;;
        esac
    done
}

show_help() {
    cat << 'EOF'
Maintenance Window Backup Script

Usage: maintenance-backup.sh [options]

Options:
  --dry-run     Show what would be done without executing
  --force       Run backup even if services are down
  --help        Show this help message

Environment:
  COMPOSE_DIR        Docker compose directory (default: /opt/tracker)
  LOG_FILE           Log file path (default: /var/log/tracker-backup.log)
  TRACKER_SERVICES   Services to stop (default: tracker)
  BACKUP_SERVICE     Backup service name (default: backup)
EOF
}

# =============================================================================
# Service Management
# =============================================================================

# Check if tracker services are running
# Returns: 0 if running, 1 if not
check_services_running() {
    cd "$COMPOSE_DIR"
    
    local running_count
    running_count=$(docker compose ps --status running --format json 2>/dev/null | \
        jq -r '.Name' 2>/dev/null | \
        grep -c "$TRACKER_SERVICES" || echo "0")
    
    [[ "$running_count" -gt 0 ]]
}

# Stop tracker services for maintenance
stop_tracker() {
    log "Stopping tracker services: $TRACKER_SERVICES"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log "[DRY-RUN] Would execute: docker compose stop $TRACKER_SERVICES"
        return 0
    fi
    
    cd "$COMPOSE_DIR"
    docker compose stop "$TRACKER_SERVICES"
}

# Start tracker services after maintenance
start_tracker() {
    log "Starting tracker services: $TRACKER_SERVICES"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log "[DRY-RUN] Would execute: docker compose start $TRACKER_SERVICES"
        return 0
    fi
    
    cd "$COMPOSE_DIR"
    docker compose start "$TRACKER_SERVICES"
}

# =============================================================================
# Backup Execution
# =============================================================================

# Run backup container once (single execution mode)
run_backup_container() {
    log "Running backup container (single execution)..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log "[DRY-RUN] Would execute: docker compose run --rm -e BACKUP_MODE=single $BACKUP_SERVICE"
        return 0
    fi
    
    cd "$COMPOSE_DIR"
    
    # Run backup container with single mode (no loop)
    # The container runs once and exits
    if docker compose run --rm \
        -e BACKUP_MODE=single \
        -e BACKUP_INTERVAL=0 \
        "$BACKUP_SERVICE"; then
        log "Backup container completed successfully"
        return 0
    else
        log_error "Backup container failed"
        return 1
    fi
}

# =============================================================================
# Main Execution
# =============================================================================

main() {
    parse_args "$@"
    
    log_header "Maintenance backup started"
    
    # Show configuration
    log "Configuration:"
    log "  Compose directory: $COMPOSE_DIR"
    log "  Tracker services:  $TRACKER_SERVICES"
    log "  Backup service:    $BACKUP_SERVICE"
    log "  Dry run:           $DRY_RUN"
    log "  Force:             $FORCE"
    
    # Check if services are running
    if ! check_services_running; then
        if [[ "$FORCE" == "true" ]]; then
            log_warning "Services not running, but --force specified. Proceeding..."
        else
            log_warning "Tracker services not running, skipping backup"
            log "Use --force to run backup anyway"
            exit 3
        fi
    fi
    
    # Track timing
    local start_time=$SECONDS
    local backup_result=0
    
    # Stop tracker
    stop_tracker
    
    # Run backup (capture result but don't exit on failure)
    if ! run_backup_container; then
        backup_result=1
    fi
    
    # Always restart tracker, even if backup failed
    start_tracker
    
    # Calculate duration
    local duration=$((SECONDS - start_time))
    
    if [[ "$backup_result" -eq 0 ]]; then
        log_header "Maintenance backup completed successfully in ${duration}s"
        exit 0
    else
        log_header "Maintenance backup completed with errors in ${duration}s"
        exit 1
    fi
}

# Run main only if script is executed (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
