#!/bin/bash
# =============================================================================
# bboard-rw Deployment Script for UAB Cloud.rc
# =============================================================================
# Runs on the remote cloud instance.
#
# Modes:
#   replace         Stop legacy /var/fider stack and deploy bboard-rw on WEB_PORT (default 80)
#   replace-prebuilt Stop legacy /var/fider stack and deploy from preloaded image
#   up              Start or rebuild full bboard-rw stack (web + db)
#   update          Rebuild and restart only web service from synced source
#   update-prebuilt Restart only web service from preloaded Docker image
#   status          Show compose status
#   down            Stop bboard-rw stack
#
# Usage:
#   ./deploy.sh replace
#   ./deploy.sh replace-prebuilt
#   WEB_PORT=80 ./deploy.sh up
#   ./deploy.sh update
#   ./deploy.sh update-prebuilt
# =============================================================================

set -euo pipefail

DEPLOYMENT_DIR="/var/bboard-rw"
LEGACY_DIR="/var/fider"
MODE="${1:-replace}"

usage() {
    echo "Usage: $0 [replace|replace-prebuilt|up|update|update-prebuilt|status|down]"
}

echo "========================================================================"
echo "bboard-rw deployment - mode: $MODE"
echo "========================================================================"

if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker is not installed."
    exit 1
fi
if ! docker compose version >/dev/null 2>&1; then
    echo "ERROR: docker compose is not available."
    exit 1
fi

if [ ! -d "$DEPLOYMENT_DIR" ]; then
    echo "ERROR: deployment directory not found: $DEPLOYMENT_DIR"
    exit 1
fi

cd "$DEPLOYMENT_DIR"

if [ ! -f docker-compose.yml ]; then
    echo "ERROR: docker-compose.yml not found in $DEPLOYMENT_DIR"
    exit 1
fi

# Load deployment env if present.
if [ -f .env ]; then
    set -a
    # shellcheck source=/dev/null
    source .env
    set +a
else
    echo "WARNING: .env not found in $DEPLOYMENT_DIR; using compose defaults."
fi

# Use persistent DB storage path automatically when available.
# Use a dedicated subdirectory to avoid initdb failing on non-empty mount roots
# (for example, files like lost+found on ext filesystems).
if [ -z "${POSTGRES_DATA_DIR:-}" ] && [ -d /mnt/postgres-data ]; then
    export POSTGRES_DATA_DIR="/mnt/postgres-data/bboard-rw-db"
fi

# Ensure data directory exists before compose startup.
if [ -n "${POSTGRES_DATA_DIR:-}" ]; then
    mkdir -p "${POSTGRES_DATA_DIR}" 2>/dev/null || sudo mkdir -p "${POSTGRES_DATA_DIR}" 2>/dev/null || true
fi

# Default to host port 80 unless caller overrides WEB_PORT.
export WEB_PORT="${WEB_PORT:-80}"

# Prevent a common production misconfiguration copied from local dev.
if [ -n "${DATABASE_URL:-}" ] && [[ "${DATABASE_URL}" == *"@localhost"* ]]; then
    echo "WARNING: DATABASE_URL points to localhost; overriding to use compose db service."
    export DATABASE_URL="postgres://${POSTGRES_USER:-postgres}:${POSTGRES_PASSWORD:-postgres}@db/${POSTGRES_DB:-realworld}"
fi

echo ""
echo "[1/3] Validating compose configuration..."
docker compose config >/dev/null
echo "Compose configuration is valid."

echo ""
echo "[2/3] Deploying services..."
case "$MODE" in
    replace)
        if [ -f "$LEGACY_DIR/docker-compose.yml" ]; then
            echo "Stopping legacy stack in $LEGACY_DIR ..."
            (cd "$LEGACY_DIR" && docker compose down) || true
        else
            echo "No legacy compose file found in $LEGACY_DIR (nothing to stop)."
        fi
        docker compose up -d --build
        ;;
    replace-prebuilt)
        if [ -f "$LEGACY_DIR/docker-compose.yml" ]; then
            echo "Stopping legacy stack in $LEGACY_DIR ..."
            (cd "$LEGACY_DIR" && docker compose down) || true
        else
            echo "No legacy compose file found in $LEGACY_DIR (nothing to stop)."
        fi
        docker compose up -d --no-build
        ;;
    up)
        docker compose up -d --build
        ;;
    update)
        docker compose up -d --no-deps --force-recreate --build web
        ;;
    update-prebuilt)
        docker compose up -d --no-deps --force-recreate --no-build web
        ;;
    status)
        docker compose ps
        exit 0
        ;;
    down)
        docker compose down
        exit 0
        ;;
    *)
        echo "ERROR: unknown mode '$MODE'"
        usage
        exit 1
        ;;
esac

echo ""
echo "[3/3] Verifying deployment..."
sleep 5
echo ""
echo "Container Status:"
docker compose ps
echo ""
echo "Recent web logs:"
docker compose logs --tail=30 web || true

echo ""
echo "========================================================================"
echo "Deployment complete"
echo "========================================================================"
echo "Expected URL: http://<floating-ip>:${WEB_PORT}"
echo ""
echo "Useful commands:"
echo "  Status:  docker compose ps"
echo "  Logs:    docker compose logs -f web"
echo "  Stop:    ./scripts/deploy.sh down"
echo ""
