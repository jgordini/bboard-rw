#!/bin/bash
# Update INITIAL_ADMIN_PASSWORD on the server from varlock.
# Secret is passed via stdin — never appears in process args or shell history.
#
# Usage:
#   ./push-admin-secret.sh
#   RC_HOST=ubuntu@138.26.48.197 RC_SSH_KEY=~/.ssh/cloud_key ./push-admin-secret.sh

set -euo pipefail

RC_HOST="${RC_HOST:-ubuntu@138.26.48.197}"
RC_SSH_KEY="${RC_SSH_KEY:-~/.ssh/cloud_key}"
REMOTE_ENV="/var/bboard-rw/.env"
SSH="ssh -i $RC_SSH_KEY"

varlock run --path ~/.varlock/.env -- bash -c '
  printf "%s" "$BBOARD_INITIAL_ADMIN_PASSWORD"
' | $SSH "$RC_HOST" "
  read -r pw
  sed -i \"s|INITIAL_ADMIN_PASSWORD=.*|INITIAL_ADMIN_PASSWORD=\\\"\$pw\\\"|\" $REMOTE_ENV
  echo '✅ INITIAL_ADMIN_PASSWORD updated on server'
"
