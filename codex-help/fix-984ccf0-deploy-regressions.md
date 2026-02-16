# Fix Notes: deploy regressions from 984ccf0

## Scope
- `scripts/deploy.sh`
- `scripts/tests/test-deploy-invocations.sh`

## Changes
- Prevented TLS outages during `update` and `update-prebuilt` by using profiled compose calls when TLS is enabled.
- Removed `--remove-orphans` from web-only update flows to avoid accidental proxy/container removal.
- Fixed TLS `replace-prebuilt` on fresh hosts by splitting deployment:
  - `db` + `web` with `--no-build`
  - `caddy` with explicit `--build`
- Added script testability hooks:
  - `DEPLOYMENT_DIR` and `LEGACY_DIR` are now overridable via env.
  - `VERIFY_SLEEP_SECONDS` allows fast, non-sleep test runs.
- Added a shell-based invocation test harness covering TLS/non-TLS behavior for `update`, `update-prebuilt`, and `replace-prebuilt`.

## Validation Run
- `bash -n scripts/deploy.sh scripts/deploy-rc-cloud-scp.sh scripts/tests/test-deploy-invocations.sh`
- `shellcheck scripts/deploy.sh scripts/deploy-rc-cloud-scp.sh scripts/tests/test-deploy-invocations.sh` (only pre-existing informational SC2029 notes in ssh lines)
- `scripts/tests/test-deploy-invocations.sh`
