#!/bin/bash
# =============================================================================
# Update bboard-rw containers on UAB Cloud.rc
# =============================================================================
# Run this from your local machine (not on the RC cloud instance).
# Syncs project files to the instance and runs deploy.sh update.
#
# Usage:
#   ./update-rc-cloud.sh ubuntu@138.26.48.197
#   RC_HOST=ubuntu@138.26.48.197 RC_SSH_KEY=~/.ssh/cloud_key ./update-rc-cloud.sh
#
# Optional:
#   RC_SSH_KEY path to SSH private key (default: ~/.ssh/cloud_key; set to "" to use SSH agent)
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOYMENT_DIR="$(dirname "$SCRIPT_DIR")"
REMOTE_DIR="/var/bboard-rw"

RC_HOST="${RC_HOST:-$1}"
# Default to cloud_key if not set (use RC_SSH_KEY="" to disable)
RC_SSH_KEY="${RC_SSH_KEY:-$HOME/.ssh/cloud_key}"
if [ -z "$RC_HOST" ]; then
    echo "Usage: $0 <user@host>"
    echo "   or: RC_HOST=ubuntu@138.26.48.197 $0"
    echo ""
    echo "Examples:"
    echo "  $0 ubuntu@138.26.48.197"
    echo "  RC_HOST=ubuntu@138.26.48.197 RC_SSH_KEY=~/.ssh/cloud_key $0"
    exit 1
fi

RC_SSH_KEY="$(eval echo "$RC_SSH_KEY")"
if [ -n "$RC_SSH_KEY" ] && [ -f "$RC_SSH_KEY" ]; then
    RSYNC_RSH="ssh -i $RC_SSH_KEY"
    SSH_OPTS=(-i "$RC_SSH_KEY")
else
    RSYNC_RSH="ssh"
    SSH_OPTS=()
fi

echo "========================================================================"
echo "Updating bboard-rw on RC Cloud: $RC_HOST"
echo "========================================================================"
echo ""
echo "[1/4] Ensuring remote directory exists..."
ssh "${SSH_OPTS[@]}" "$RC_HOST" "if [ ! -d $REMOTE_DIR ]; then sudo mkdir -p $REMOTE_DIR && sudo chown -R \$(id -un):\$(id -gn) $REMOTE_DIR; fi"

echo ""
echo "[2/4] Syncing deployment files to $RC_HOST:$REMOTE_DIR ..."
rsync -avz -e "$RSYNC_RSH" --exclude='.env' --exclude='.git' --exclude='target' \
    "$DEPLOYMENT_DIR/" "$RC_HOST:$REMOTE_DIR/"

echo ""
echo "[3/4] Running deploy.sh update on remote..."
ssh "${SSH_OPTS[@]}" "$RC_HOST" "cd $REMOTE_DIR && ./scripts/deploy.sh update"

echo ""
echo "[4/4] Done. bboard-rw has been updated on RC cloud."
echo ""
