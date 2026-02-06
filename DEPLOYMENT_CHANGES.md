# Deployment Changes for `bboard-rw`

Date: 2026-02-06

## Goal

Replace the legacy app stack in `/var/fider` with `bboard-rw`, and run the new app on host port `80`.

## Files Changed

- `/Users/jeremy/repos/bboard-rw/docker-compose.yml`
- `/Users/jeremy/repos/bboard-rw/Dockerfile`
- `/Users/jeremy/repos/bboard-rw/scripts/deploy.sh`
- `/Users/jeremy/repos/bboard-rw/scripts/update-rc-cloud.sh`
- `/Users/jeremy/repos/bboard-rw/scripts/deploy-rc-cloud-scp.sh`
- `/Users/jeremy/repos/bboard-rw/scripts/setup-instance.sh`
- `/Users/jeremy/repos/bboard-rw/scripts/generate-saml-certs.sh`
- `/Users/jeremy/repos/bboard-rw/scripts/rc-cloud.env.example` (new)

## What Changed

### 1. Compose updates (`docker-compose.yml`)

- Added env-driven config for production overrides:
  - `WEB_PORT` (defaults to `8080`, set to `80` in cloud)
  - `POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`
  - `POSTGRES_DATA_DIR` for persistent volume mounting
  - `DATABASE_URL`, `JWT_SECRET`
  - Admin/mailer env defaults
- Added Postgres healthcheck.
- Added `depends_on` health condition for `web -> db`.
- Bound DB port to localhost only: `127.0.0.1:${DB_PORT:-5432}:5432`.

### 2. Build reliability updates (`Dockerfile`)

- Replaced in-container `cargo install cargo-leptos` source build with a prebuilt release binary download.
- Added architecture-aware install logic for `amd64` and `arm64`.
- Kept `CARGO_BUILD_JOBS=1` to reduce memory pressure on constrained instances.

### 3. New deployment control flow (`scripts/deploy.sh`)

- Replaced old BlazeBoard/Fider logic with `bboard-rw` logic.
- Switched deployment directory to `/var/bboard-rw`.
- Added deployment modes:
  - `replace`
  - `replace-prebuilt`
  - `up`
  - `update`
  - `update-prebuilt`
  - `status`
  - `down`
- `replace`/`replace-prebuilt` now stop legacy `/var/fider` stack before starting new stack.
- Default `WEB_PORT` is forced to `80` unless overridden.
- Auto-uses `/mnt/postgres-data/bboard-rw-db` when persistent storage is present.
- Ensures the Postgres data directory exists before startup.
- Detects and overrides bad `DATABASE_URL` values that point to `localhost`.

### 4. Remote update script (`scripts/update-rc-cloud.sh`)

- Switched remote path from `/var/fider` to `/var/bboard-rw`.
- Ensures remote directory exists and has ownership before syncing.
- Syncs project files and runs `./scripts/deploy.sh update`.
- Removed obsolete nginx-specific excludes.

### 5. SCP deployment script (`scripts/deploy-rc-cloud-scp.sh`)

- Fixed build path to current repo root.
- Updated image/tag to `realworld-leptos:0.1.0`.
- Updated remote path to `/var/bboard-rw`.
- Ensures remote directory exists and has ownership.
- Loads prebuilt image on remote and runs:
  - `./scripts/deploy.sh replace-prebuilt`
- Removes local tarball after deploy.

### 6. Instance setup script (`scripts/setup-instance.sh`)

- Updated app naming/paths to `bboard-rw`.
- Creates `/var/bboard-rw` instead of `/var/fider`.
- Keeps Docker + compose + persistent volume setup.
- Updated next-step text to use:
  - `/var/bboard-rw`
  - `scripts/rc-cloud.env.example`

### 7. Cert helper update (`scripts/generate-saml-certs.sh`)

- Marked as legacy helper for reverse-proxy/TLS use.
- Output path changed to `/var/bboard-rw/ssl`.
- Updated messaging (no direct SAML dependency assumed for this app).

### 8. New cloud env template (`scripts/rc-cloud.env.example`)

- Added deployment-ready env template for RC cloud:
  - `WEB_PORT=80`
  - Postgres credentials + data dir (`POSTGRES_DATA_DIR=/mnt/postgres-data/bboard-rw-db`)
  - `DATABASE_URL`
  - `JWT_SECRET`
  - Admin credentials
  - Optional mailer values

## Validation Performed

- `bash -n` on deployment scripts: passed.
- `docker compose config` for `docker-compose.yml`: passed.
- `shellcheck`: no blocking issues (only informational remote-expansion notes for SSH command strings).
- Live deploy to RC cloud completed on `2026-02-06`.
- Runtime checks:
  - `curl http://127.0.0.1:80` on host returned `200`.
  - `curl http://138.26.48.197` returned `200`.
- Service state after deploy:
  - `bboard-rw-web-1` running on `0.0.0.0:80->8080`.
  - `bboard-rw-db-1` healthy on `127.0.0.1:5432`.

## Cutover Command

From local machine:

```bash
cd /Users/jeremy/repos/bboard-rw
./scripts/deploy-rc-cloud-scp.sh ubuntu@138.26.48.197
```

This performs a full replacement deploy and stops legacy `/var/fider` if present.
