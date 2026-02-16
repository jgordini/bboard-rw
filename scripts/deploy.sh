#!/bin/bash
# =============================================================================
# bboard-rw Deployment Script for UAB Cloud.rc
# =============================================================================
# Runs on the remote cloud instance.
#
# Modes:
#   replace         Stop legacy /var/fider stack and deploy bboard-rw
#   replace-prebuilt Stop legacy /var/fider stack and deploy from preloaded image
#   up              Start or rebuild full bboard-rw stack
#   update          Rebuild and restart only web service from synced source
#   update-prebuilt Restart only web service from preloaded Docker image
#   tls-init        Start/rebuild caddy TLS proxy when TLS is enabled
#   renew-certs     No-op helper; Caddy renews certificates automatically
#   status          Show compose status
#   down            Stop bboard-rw stack
#
# Usage:
#   ./deploy.sh replace
#   ./deploy.sh replace-prebuilt
#   ENABLE_LETSENCRYPT=true DOMAIN=uabspark.com ./deploy.sh up
#   ./deploy.sh update
#   ./deploy.sh update-prebuilt
#   ./deploy.sh tls-init
#   ./deploy.sh renew-certs
# =============================================================================

set -euo pipefail

DEPLOYMENT_DIR="${DEPLOYMENT_DIR:-/var/bboard-rw}"
LEGACY_DIR="${LEGACY_DIR:-/var/fider}"
MODE="${1:-replace}"
VERIFY_SLEEP_SECONDS="${VERIFY_SLEEP_SECONDS:-5}"

usage() {
    echo "Usage: $0 [replace|replace-prebuilt|up|update|update-prebuilt|tls-init|renew-certs|status|down]"
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

ENABLE_LETSENCRYPT="${ENABLE_LETSENCRYPT:-false}"
if [ "${ENABLE_LETSENCRYPT}" = "true" ] || [ "${ENABLE_LETSENCRYPT}" = "1" ]; then
    TLS_ENABLED=true
else
    TLS_ENABLED=false
fi

if [ "${TLS_ENABLED}" = "true" ]; then
    export DOMAIN="${DOMAIN:-uabspark.com}"
    export DOMAIN_WWW="${DOMAIN_WWW:-www.${DOMAIN}}"
    export LETSENCRYPT_EMAIL="${LETSENCRYPT_EMAIL:-admin@${DOMAIN}}"
    export WEB_BIND_HOST="${WEB_BIND_HOST:-127.0.0.1}"
    export WEB_PORT="${WEB_PORT:-8080}"
    export WEB_HTTP_PORT="${WEB_HTTP_PORT:-80}"
    export WEB_HTTPS_PORT="${WEB_HTTPS_PORT:-443}"
    if [ -z "${CLOUDFLARE_DNS_API_TOKEN:-}" ]; then
        echo "ERROR: CLOUDFLARE_DNS_API_TOKEN is required when ENABLE_LETSENCRYPT=true."
        exit 1
    fi
else
    # Preserve legacy behavior when TLS profile is disabled.
    export WEB_BIND_HOST="${WEB_BIND_HOST:-0.0.0.0}"
    export WEB_PORT="${WEB_PORT:-80}"
fi

# Prevent a common production misconfiguration copied from local dev.
if [ -n "${DATABASE_URL:-}" ] && [[ "${DATABASE_URL}" == *"@localhost"* ]]; then
    echo "WARNING: DATABASE_URL points to localhost; overriding to use compose db service."
    export DATABASE_URL="postgres://${POSTGRES_USER:-postgres}:${POSTGRES_PASSWORD:-postgres}@db/${POSTGRES_DB:-realworld}"
fi

stop_legacy_stack() {
    if [ -f "$LEGACY_DIR/docker-compose.yml" ]; then
        echo "Stopping legacy stack in $LEGACY_DIR ..."
        (cd "$LEGACY_DIR" && docker compose down) || true
    else
        echo "No legacy compose file found in $LEGACY_DIR (nothing to stop)."
    fi
}

deploy_stack_without_tls() {
    local build_flag="$1"
    docker compose up -d --remove-orphans "${build_flag}"
}

deploy_stack_with_tls() {
    local build_flag="$1"
    docker compose --profile tls up -d --remove-orphans "${build_flag}"
}

deploy_stack_with_tls_prebuilt() {
    # Keep app/database in no-build mode, but explicitly build caddy (build-only service).
    docker compose --profile tls up -d --remove-orphans --no-build db web
    docker compose --profile tls up -d --remove-orphans --no-deps --build caddy
}

echo ""
echo "[1/3] Validating compose configuration..."
if [ "${TLS_ENABLED}" = "true" ]; then
    docker compose --profile tls config >/dev/null
else
    docker compose config >/dev/null
fi
echo "Compose configuration is valid."

DEPLOY_RAN=false

echo ""
echo "[2/3] Deploying services..."
case "$MODE" in
    replace)
        stop_legacy_stack
        if [ "${TLS_ENABLED}" = "true" ]; then
            deploy_stack_with_tls "--build"
        else
            deploy_stack_without_tls "--build"
        fi
        DEPLOY_RAN=true
        ;;
    replace-prebuilt)
        stop_legacy_stack
        if [ "${TLS_ENABLED}" = "true" ]; then
            deploy_stack_with_tls_prebuilt
        else
            deploy_stack_without_tls "--no-build"
        fi
        DEPLOY_RAN=true
        ;;
    up)
        if [ "${TLS_ENABLED}" = "true" ]; then
            deploy_stack_with_tls "--build"
        else
            deploy_stack_without_tls "--build"
        fi
        DEPLOY_RAN=true
        ;;
    update)
        if [ "${TLS_ENABLED}" = "true" ]; then
            docker compose --profile tls up -d --no-deps --force-recreate --build web
        else
            docker compose up -d --no-deps --force-recreate --build web
        fi
        DEPLOY_RAN=true
        ;;
    update-prebuilt)
        if [ "${TLS_ENABLED}" = "true" ]; then
            docker compose --profile tls up -d --no-deps --force-recreate --no-build web
        else
            docker compose up -d --no-deps --force-recreate --no-build web
        fi
        DEPLOY_RAN=true
        ;;
    tls-init)
        if [ "${TLS_ENABLED}" != "true" ]; then
            echo "ERROR: tls-init requires ENABLE_LETSENCRYPT=true in environment or .env."
            exit 1
        fi
        docker compose --profile tls up -d --remove-orphans --no-deps --build caddy
        DEPLOY_RAN=true
        ;;
    renew-certs)
        echo "Caddy renews certificates automatically; no manual renewal action required."
        exit 0
        ;;
    status)
        docker compose --profile tls ps
        exit 0
        ;;
    down)
        docker compose --profile tls down
        exit 0
        ;;
    *)
        echo "ERROR: unknown mode '$MODE'"
        usage
        exit 1
        ;;
esac

if [ "${DEPLOY_RAN}" != "true" ]; then
    exit 0
fi

echo ""
echo "[3/3] Verifying deployment..."
sleep "${VERIFY_SLEEP_SECONDS}"
echo ""
echo "Container Status:"
docker compose --profile tls ps
echo ""
echo "Recent web logs:"
docker compose logs --tail=30 web || true
if [ "${TLS_ENABLED}" = "true" ]; then
    echo ""
    echo "Recent caddy logs:"
    docker compose --profile tls logs --tail=30 caddy || true
fi

echo ""
echo "========================================================================"
echo "Deployment complete"
echo "========================================================================"
if [ "${TLS_ENABLED}" = "true" ]; then
    echo "Expected URL: https://${DOMAIN}"
else
    echo "Expected URL: http://<floating-ip>:${WEB_PORT}"
fi
echo ""
echo "Useful commands:"
echo "  Status:  docker compose --profile tls ps"
echo "  Logs:    docker compose logs -f web"
echo "  Caddy:   docker compose --profile tls logs -f caddy"
echo "  Stop:    ./scripts/deploy.sh down"
echo ""
