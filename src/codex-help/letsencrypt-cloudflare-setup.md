# Cloudflare + Caddy HTTPS Setup Notes

Date: 2026-02-13

## Goal
- Terminate HTTPS on origin with Caddy (Let's Encrypt ACME) for
  `uabspark.com` and `www.uabspark.com`.
- Keep Cloudflare in front and use SSL mode `Full (strict)`.

## Required Environment Values (`/var/bboard-rw/.env`)
- `ENABLE_LETSENCRYPT=true`
- `DOMAIN=uabspark.com`
- `DOMAIN_WWW=www.uabspark.com`
- `LETSENCRYPT_EMAIL=<admin-email>`
- `CLOUDFLARE_DNS_API_TOKEN=<dns-edit-token>`
- `WEB_BIND_HOST=127.0.0.1`
- `WEB_PORT=8080`
- `WEB_HTTP_PORT=80`
- `WEB_HTTPS_PORT=443`

## Deploy Flow
1. Ensure VPN is connected for SSH access to cloud.rc instance.
2. Sync changes: `./scripts/update-rc-cloud.sh ubuntu@138.26.48.197`
3. On server: `cd /var/bboard-rw && ./scripts/deploy.sh replace`
4. Verify HTTPS: `curl -I https://uabspark.com`

## Renewal
- Caddy renews certificates automatically.
- `./scripts/deploy.sh renew-certs` is informational only.
