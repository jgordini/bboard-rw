# Deploy Ops Notes

## Access Prerequisite
- SSH to `ubuntu@138.26.48.197` requires UAB VPN.

## 502 Recovery (Caddy + Docker)
1. Check service status:
   - `ssh -i ~/.ssh/cloud_key ubuntu@138.26.48.197 'cd /var/bboard-rw && docker compose --profile tls ps -a'`
2. If `web` is down, restart it:
   - `ssh -i ~/.ssh/cloud_key ubuntu@138.26.48.197 'cd /var/bboard-rw && docker compose --profile tls up -d web'`
3. Verify origin and public URL:
   - `ssh -i ~/.ssh/cloud_key ubuntu@138.26.48.197 'curl -I http://127.0.0.1:8080 && curl -I https://uabspark.com'`

## Root Cause Observed On 2026-02-13
- Kernel OOM killer terminated `realworld-lepto` (`web`) at `20:09:39`.
- Host had `0B` swap configured.
- Caddy reported `502` due to upstream unavailability/resolution failures.

## Hardening Applied
- Set running `web` container restart policy to `unless-stopped`.
- Persisted restart policy in `docker-compose.yml` under `web`.
- Added persistent `4G` swap on host (`/swapfile`) to reduce OOM kills.
- Added kernel memory tunables:
  - `vm.swappiness=10`
  - `vm.vfs_cache_pressure=50`
