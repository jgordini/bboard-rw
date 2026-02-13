# Deployment Note: VPN Required for SSH

Date: 2026-02-13

## Important
- SSH access to `ubuntu@138.26.48.197` requires being connected to VPN.
- Without VPN, deployment scripts that use SSH will fail or time out on port 22.

## Impacted Commands
- `./scripts/update-rc-cloud.sh ubuntu@138.26.48.197`
- `./scripts/deploy-rc-cloud-scp.sh ubuntu@138.26.48.197`

## Quick Check Before Deploy
- Confirm VPN is active before running deploy/update scripts.
- Optional check:
  - `ssh -i ~/.ssh/cloud_key -o ConnectTimeout=10 ubuntu@138.26.48.197 'echo ok'`
